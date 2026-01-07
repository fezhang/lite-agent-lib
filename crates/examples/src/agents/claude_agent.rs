//! Claude Code CLI agent
//!
//! This agent launches the Claude Code CLI tool with streaming JSON support.
//! It enables ANTHROPIC debug mode and streams responses in real-time.
//!
//! NOTE: This is a simplified demonstration. The full implementation requires
//! enhancements to the SpawnedAgent design to support stdio handle extraction
//! from AsyncGroupChild. See docs/reference/protocol-communication.md for the
//! complete implementation pattern used in production.

use async_trait::async_trait;
use command_group::AsyncCommandGroup;
use futures_util::stream::{self, BoxStream, StreamExt};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command as TokioCommand;

use lite_agent_core::{
    AgentCapability, AgentConfig, AgentError, AgentExecutor, AvailabilityStatus, LogStore,
    NormalizedEntry, SpawnedAgent,
};

/// Claude Code CLI executor
///
/// This agent launches Claude Code CLI with streaming JSON protocol.
#[derive(Debug, Clone)]
pub struct ClaudeCodeAgent {
    /// Enable ANTHROPIC debug mode
    debug_mode: bool,
    /// Custom path to claude.exe (optional)
    custom_path: Option<PathBuf>,
}

impl ClaudeCodeAgent {
    /// Create a new Claude Code CLI agent
    pub fn new() -> Self {
        Self {
            debug_mode: false,
            custom_path: None,
        }
    }

    /// Enable ANTHROPIC debug mode
    pub fn with_debug_mode(mut self) -> Self {
        self.debug_mode = true;
        self
    }

    /// Set a custom path to claude.exe
    pub fn with_custom_path(mut self, path: PathBuf) -> Self {
        self.custom_path = Some(path);
        self
    }

    /// Find the Claude Code CLI executable
    /// Returns (executable_path, args) where executable_path is the full path to the exe
    fn find_claude_cli(&self) -> Option<(PathBuf, Vec<String>)> {
        // If custom path is specified, use it
        if let Some(ref custom) = self.custom_path {
            if custom.exists() {
                let args = if cfg!(windows) {
                    vec![
                        "--output-format".to_string(),
                        "stream-json".to_string(),
                        "--input-format".to_string(),
                        "stream-json".to_string(),
                    ]
                } else {
                    vec![
                        "--output-format".to_string(),
                        "stream-json".to_string(),
                        "--input-format".to_string(),
                        "stream-json".to_string(),
                    ]
                };
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
            let args = vec![
                "-y".to_string(),
                "@anthropic-ai/claude-code".to_string(),
                "--output-format".to_string(),
                "stream-json".to_string(),
                "--input-format".to_string(),
                "stream-json".to_string(),
            ];
            Some((PathBuf::from("npx"), args))
        } else {
            None
        }
    }
}

impl Default for ClaudeCodeAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentExecutor for ClaudeCodeAgent {
    fn agent_type(&self) -> &str {
        "claude-code"
    }

    async fn spawn(
        &self,
        config: &AgentConfig,
        _input: &str,
    ) -> Result<SpawnedAgent, AgentError> {
        // Find the Claude Code CLI executable
        let (exe_path, args) = self
            .find_claude_cli()
            .ok_or_else(|| {
                AgentError::SpawnError(
                    "Claude Code CLI not found. Please install it from:\n\
                     https://claude.ai/download\n\n\
                     Or ensure Node.js and npx are installed.".to_string()
                )
            })?;

        let mut cmd = TokioCommand::new(&exe_path);
        cmd.args(&args);

        // Configure working directory
        cmd.current_dir(&config.work_dir);

        // Configure environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // Enable ANTHROPIC debug mode if requested
        if self.debug_mode {
            cmd.env("ANTHROPIC_LOG", "debug");
            cmd.env("RUST_LOG", "debug");
        }

        // For this simplified demo, inherit stdio so the user can interact directly
        // In the full implementation, you'd use piped stdio and handle JSON streaming
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
        let spawned = SpawnedAgent {
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
            AgentCapability::BidirectionalControl,
            AgentCapability::WorkspaceIsolation,
        ]
    }

    fn description(&self) -> Option<String> {
        let method = if self.custom_path.is_some() {
            "custom path"
        } else if cfg!(windows) {
            "native Windows"
        } else {
            "npx"
        };

        Some(format!(
            "Claude Code CLI (debug mode: {}, via: {})",
            self.debug_mode, method
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type() {
        let agent = ClaudeCodeAgent::new();
        assert_eq!(agent.agent_type(), "claude-code");
    }

    #[test]
    fn test_description() {
        let agent = ClaudeCodeAgent::new();
        assert!(agent.description().is_some());
        let desc = agent.description().unwrap();
        assert!(desc.contains("claude-code"));
    }

    #[test]
    fn test_debug_mode() {
        let agent = ClaudeCodeAgent::new().with_debug_mode();
        assert!(agent.debug_mode);
    }

    #[test]
    fn test_custom_path() {
        let path = PathBuf::from("/path/to/claude");
        let agent = ClaudeCodeAgent::new().with_custom_path(path.clone());
        assert_eq!(agent.custom_path, Some(path));
    }

    #[tokio::test]
    async fn test_capabilities() {
        let agent = ClaudeCodeAgent::new();
        let caps = agent.capabilities();
        assert!(!caps.is_empty());
    }

    #[tokio::test]
    async fn test_check_availability() {
        let agent = ClaudeCodeAgent::new();
        let status = agent.check_availability().await;
        // The result depends on what's installed, but it should not panic
        match &status {
            AvailabilityStatus::Available => {}
            AvailabilityStatus::NotFound { .. } => {}
            _ => {}
        }
    }
}
