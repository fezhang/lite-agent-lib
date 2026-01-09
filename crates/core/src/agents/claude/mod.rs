//! Claude Code CLI agent implementation
//!
//! This module provides an agent executor for the Claude Code CLI tool with
//! support for bidirectional control protocol and session continuation.

pub mod config;
pub mod types;

use async_trait::async_trait;
use command_group::AsyncCommandGroup;
use futures_util::stream::{self, BoxStream, StreamExt};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command as TokioCommand;

use crate::agent::{
    AgentCapability, AgentConfig, AgentError, AgentExecutor, AvailabilityStatus,
};
use crate::logs::{LogStore, NormalizedEntry};
use crate::protocol::control::PermissionMode;

pub use config::{ClaudeConfig, ClaudeConfigBuilder};
pub use types::ClaudeMessage;

/// Claude Code CLI agent executor
#[derive(Debug, Clone)]
pub struct ClaudeAgent {
    /// Configuration for the Claude agent
    config: ClaudeConfig,
}

impl ClaudeAgent {
    /// Create a new Claude agent with default configuration
    pub fn new() -> Self {
        Self {
            config: ClaudeConfig::default(),
        }
    }

    /// Create a new Claude agent with a configuration builder
    pub fn builder() -> ClaudeConfigBuilder {
        ClaudeConfigBuilder::default()
    }

    /// Create a Claude agent with a specific configuration
    pub fn with_config(config: ClaudeConfig) -> Self {
        Self { config }
    }

    /// Find the Claude Code CLI executable
    /// Returns (executable_path, args) where executable_path is the full path to the exe
    fn find_claude_cli(&self) -> Option<(PathBuf, Vec<String>)> {
        // If custom path is specified, use it
        if let Some(ref custom) = self.config.custom_path {
            if custom.exists() {
                let args = vec![
                    "--output-format".to_string(),
                    "stream-json".to_string(),
                    "--input-format".to_string(),
                    "stream-json".to_string(),
                ];
                return Some((custom.clone(), args));
            }
        }

        // Try to find native claude.exe on Windows
        if cfg!(windows) {
            // Check common installation paths
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let paths = vec![
                    local_app_data.clone() + r"\AnthropicClaude\claude.exe",
                    local_app_data + r"\Programs\AnthropicClaude\claude.exe",
                ];

                for path in paths {
                    let pb = PathBuf::from(&path);
                    if pb.exists() {
                        let args = vec![
                            "--output-format".to_string(),
                            "stream-json".to_string(),
                            "--input-format".to_string(),
                            "stream-json".to_string(),
                        ];
                        return Some((pb, args));
                    }
                }
            }
        }

        // Fall back to npx for cross-platform support
        let npx_result = std::process::Command::new("npx")
            .arg("--version")
            .output();

        if npx_result.is_ok() {
            let base_cmd = self.config.base_command();
            let parts: Vec<&str> = base_cmd.split_whitespace().collect();
            let exe = PathBuf::from(parts[0]);
            let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).chain(vec![
                "--output-format".to_string(),
                "stream-json".to_string(),
                "--input-format".to_string(),
                "stream-json".to_string(),
            ]).collect();
            Some((exe, args))
        } else {
            None
        }
    }

    /// Build command arguments from configuration
    fn build_args(&self) -> Vec<String> {
        let mut args = vec![
            "-p".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--input-format".to_string(),
            "stream-json".to_string(),
            "--include-partial-messages".to_string(),
            "--verbose".to_string(),
            "--disallowedTools=AskUserQuestion".to_string(),
        ];

        // Enable permission prompt tool for plan/approval modes
        if self.config.plan_mode || self.config.approvals {
            args.push("--permission-prompt-tool=stdio".to_string());
            args.push(format!(
                "--permission-mode={}",
                PermissionMode::BypassPermissions
            ));
        }

        // Dangerously skip permissions if configured
        if self.config.dangerously_skip_permissions {
            args.push("--dangerously-skip-permissions".to_string());
        }

        // Set model if specified
        if let Some(ref model) = self.config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        args
    }
}

impl Default for ClaudeAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentExecutor for ClaudeAgent {
    fn agent_type(&self) -> &str {
        "claude-code"
    }

    async fn spawn(
        &self,
        config: &AgentConfig,
        _input: &str,
    ) -> Result<crate::agent::SpawnedAgent, AgentError> {
        // Find the Claude Code CLI executable
        let (exe_path, cli_args) = self
            .find_claude_cli()
            .ok_or_else(|| {
                AgentError::SpawnError(
                    "Claude Code CLI not found. Please install it from:\n\
                     https://claude.ai/download\n\n\
                     Or ensure Node.js and npx are installed.".to_string()
                )
            })?;

        let mut args = cli_args;
        args.extend(self.build_args());

        let mut cmd = TokioCommand::new(&exe_path);
        cmd.args(&args);

        // Configure working directory
        cmd.current_dir(&config.work_dir);

        // Configure environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // Enable ANTHROPIC debug mode if requested
        if self.config.debug_mode {
            cmd.env("ANTHROPIC_LOG", "debug");
            cmd.env("RUST_LOG", "debug");
        }

        // Disable API key if configured
        if self.config.disable_api_key {
            cmd.env_remove("ANTHROPIC_API_KEY");
        }

        // For now, inherit stdio for basic interaction
        // TODO: Implement full bidirectional protocol support
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        // Spawn the process with group management
        let child = cmd
            .group_spawn()
            .map_err(|e| AgentError::SpawnError(format!("Failed to spawn Claude Code CLI: {}", e)))?;

        // Create log store
        let log_store = Arc::new(LogStore::new());

        // Add a system message with information
        let exe_name = exe_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("claude");
        log_store
            .add_system(
                format!(
                    "Claude Code CLI started using: {}",
                    exe_name
                ),
                self.agent_type().to_string(),
            )
            .await;

        // Create spawned agent
        let spawned = crate::agent::SpawnedAgent {
            child,
            stdin: None,
            stdout: None,
            stderr: None,
            exit_signal: None,
            interrupt_signal: None,
            log_store,
        };

        Ok(spawned)
    }

    /// Continue from a previous session
    async fn spawn_follow_up(
        &self,
        config: &AgentConfig,
        _input: &str,
        session_id: &str,
    ) -> Result<crate::agent::SpawnedAgent, AgentError> {
        // Find the Claude Code CLI executable
        let (exe_path, cli_args) = self
            .find_claude_cli()
            .ok_or_else(|| {
                AgentError::SpawnError(
                    "Claude Code CLI not found. Please install it from:\n\
                     https://claude.ai/download\n\n\
                     Or ensure Node.js and npx are installed.".to_string()
                )
            })?;

        let mut args = cli_args;
        args.extend(self.build_args());
        args.extend(vec![
            "--fork-session".to_string(),
            "--resume".to_string(),
            session_id.to_string(),
        ]);

        let mut cmd = TokioCommand::new(&exe_path);
        cmd.args(&args);
        cmd.current_dir(&config.work_dir);

        // Configure environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        if self.config.debug_mode {
            cmd.env("ANTHROPIC_LOG", "debug");
            cmd.env("RUST_LOG", "debug");
        }

        if self.config.disable_api_key {
            cmd.env_remove("ANTHROPIC_API_KEY");
        }

        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        let child = cmd
            .group_spawn()
            .map_err(|e| AgentError::SpawnError(format!("Failed to spawn Claude Code CLI: {}", e)))?;

        let log_store = Arc::new(LogStore::new());

        let exe_name = exe_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("claude");
        log_store
            .add_system(
                format!(
                    "Claude Code CLI resumed from session {} using: {}",
                    session_id, exe_name
                ),
                self.agent_type().to_string(),
            )
            .await;

        let spawned = crate::agent::SpawnedAgent {
            child,
            stdin: None,
            stdout: None,
            stderr: None,
            exit_signal: None,
            interrupt_signal: None,
            log_store,
        };

        Ok(spawned)
    }

    fn normalize_logs(
        &self,
        raw_logs: Arc<LogStore>,
    ) -> BoxStream<'static, NormalizedEntry> {
        // Get entries upfront - we need to block here since we're in a sync context
        let entries = std::thread::spawn(move || {
            // Use tokio runtime to get the entries
            let rt = tokio::runtime::Handle::try_current();
            match rt {
                Ok(handle) => handle.block_on(raw_logs.get_entries()),
                Err(_) => {
                    // Create a new runtime if needed
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(raw_logs.get_entries())
                }
            }
        })
        .join()
        .unwrap_or_default();

        stream::iter(entries).boxed()
    }

    async fn check_availability(&self) -> AvailabilityStatus {
        match self.find_claude_cli() {
            Some((path, _)) => {
                if path.exists() || path.to_str() == Some("npx") {
                    AvailabilityStatus::Available
                } else {
                    AvailabilityStatus::NotFound {
                        reason: format!("Claude Code CLI path does not exist: {:?}", path),
                    }
                }
            }
            None => AvailabilityStatus::NotFound {
                reason: "Claude Code CLI not found. Please install it from https://claude.ai/download or ensure Node.js/npx is installed.".to_string(),
            },
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::SessionContinuation,
            AgentCapability::BidirectionalControl,
            AgentCapability::WorkspaceIsolation,
        ]
    }

    fn description(&self) -> Option<String> {
        let method = if self.config.custom_path.is_some() {
            "custom path"
        } else if self.config.use_router {
            "claude-code-router"
        } else if cfg!(windows) {
            "native Windows"
        } else {
            "npx"
        };

        Some(format!(
            "Claude Code CLI (debug: {}, plan: {}, via: {})",
            self.config.debug_mode, self.config.plan_mode, method
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type() {
        let agent = ClaudeAgent::new();
        assert_eq!(agent.agent_type(), "claude-code");
    }

    #[test]
    fn test_default_agent() {
        let agent = ClaudeAgent::new();
        assert!(!agent.config.debug_mode);
        assert!(!agent.config.plan_mode);
    }

    #[test]
    fn test_builder() {
        let agent = ClaudeAgent::builder()
            .with_debug_mode()
            .with_plan_mode()
            .with_model("claude-sonnet-4")
            .build();

        assert!(agent.config.debug_mode);
        assert!(agent.config.plan_mode);
        assert_eq!(agent.config.model.as_deref(), Some("claude-sonnet-4"));
    }

    #[tokio::test]
    async fn test_capabilities() {
        let agent = ClaudeAgent::new();
        let caps = agent.capabilities();
        assert!(!caps.is_empty());
        assert!(caps.contains(&AgentCapability::SessionContinuation));
    }

    #[tokio::test]
    async fn test_check_availability() {
        let agent = ClaudeAgent::new();
        let status = agent.check_availability().await;
        // The result depends on what's installed, but it should not panic
        match &status {
            AvailabilityStatus::Available => {}
            AvailabilityStatus::NotFound { .. } => {}
            _ => {}
        }
    }
}
