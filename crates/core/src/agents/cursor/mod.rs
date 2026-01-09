//! Cursor agent implementation
//!
//! This module provides an agent executor for the Cursor CLI tool with
//! support for session continuation.

pub mod config;
pub mod types;

use async_trait::async_trait;
use command_group::AsyncCommandGroup;
use futures_util::stream::{self, BoxStream, StreamExt};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::Command as TokioCommand;

use crate::agent::{
    AgentCapability, AgentConfig, AgentError, AgentExecutor, AvailabilityStatus,
};
use crate::logs::{LogStore, NormalizedEntry};

pub use config::{CursorConfig, CursorConfigBuilder};
pub use types::CursorMessage;

/// Cursor agent executor
#[derive(Debug, Clone)]
pub struct CursorAgent {
    /// Configuration for the Cursor agent
    config: CursorConfig,
}

impl CursorAgent {
    /// Create a new Cursor agent with default configuration
    pub fn new() -> Self {
        Self {
            config: CursorConfig::default(),
        }
    }

    /// Create a new Cursor agent with a configuration builder
    pub fn builder() -> CursorConfigBuilder {
        CursorConfigBuilder::default()
    }

    /// Create a Cursor agent with a specific configuration
    pub fn with_config(config: CursorConfig) -> Self {
        Self { config }
    }

    /// Build command arguments from configuration
    fn build_args(&self) -> Vec<String> {
        let mut args = vec![
            "-p".to_string(),
            "--output-format=stream-json".to_string(),
        ];

        if self.config.force {
            args.push("--force".to_string());
        }

        if let Some(ref model) = self.config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        args
    }

    /// Find the cursor-agent executable
    fn find_cursor_cli(&self) -> Option<PathBuf> {
        // If custom path is specified, use it
        if let Some(ref custom) = self.config.custom_path {
            if custom.exists() {
                return Some(custom.clone());
            }
        }

        // Try to find cursor-agent in PATH
        let which_result = if cfg!(windows) {
            std::process::Command::new("where")
                .arg("cursor-agent")
                .output()
        } else {
            std::process::Command::new("which")
                .arg("cursor-agent")
                .output()
        };

        if let Ok(output) = which_result {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Some(PathBuf::from(path_str));
            }
        }

        None
    }
}

impl Default for CursorAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentExecutor for CursorAgent {
    fn agent_type(&self) -> &str {
        "cursor"
    }

    async fn spawn(
        &self,
        config: &AgentConfig,
        input: &str,
    ) -> Result<crate::agent::SpawnedAgent, AgentError> {
        // Find the cursor-agent executable
        let exe_path = self
            .find_cursor_cli()
            .ok_or_else(|| {
                AgentError::SpawnError(
                    "Cursor agent not found. Please install it from:\n\
                     https://cursor.sh".to_string()
                )
            })?;

        let args = self.build_args();

        let mut cmd = TokioCommand::new(&exe_path);
        cmd.args(&args);

        // Configure working directory
        cmd.current_dir(&config.work_dir);

        // Configure environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // Configure stdio
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Spawn the process with group management
        let mut child = cmd
            .group_spawn()
            .map_err(|e| AgentError::SpawnError(format!("Failed to spawn cursor-agent: {}", e)))?;

        // Write the input to stdin
        if let Some(mut stdin) = child.inner().stdin.take() {
            stdin
                .write_all(input.as_bytes())
                .await
                .map_err(|e| AgentError::SpawnError(format!("Failed to write to stdin: {}", e)))?;
            stdin
                .shutdown()
                .await
                .map_err(|e| AgentError::SpawnError(format!("Failed to close stdin: {}", e)))?;
        }

        // Create log store
        let log_store = Arc::new(LogStore::new());

        // Add a system message with information
        log_store
            .add_system(
                format!("Cursor agent started"),
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
        input: &str,
        session_id: &str,
    ) -> Result<crate::agent::SpawnedAgent, AgentError> {
        // Find the cursor-agent executable
        let exe_path = self
            .find_cursor_cli()
            .ok_or_else(|| {
                AgentError::SpawnError(
                    "Cursor agent not found. Please install it from:\n\
                     https://cursor.sh".to_string()
                )
            })?;

        let mut args = self.build_args();
        args.extend(vec![
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

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd
            .group_spawn()
            .map_err(|e| AgentError::SpawnError(format!("Failed to spawn cursor-agent: {}", e)))?;

        // Write the input to stdin
        if let Some(mut stdin) = child.inner().stdin.take() {
            stdin
                .write_all(input.as_bytes())
                .await
                .map_err(|e| AgentError::SpawnError(format!("Failed to write to stdin: {}", e)))?;
            stdin
                .shutdown()
                .await
                .map_err(|e| AgentError::SpawnError(format!("Failed to close stdin: {}", e)))?;
        }

        let log_store = Arc::new(LogStore::new());

        log_store
            .add_system(
                format!("Cursor agent resumed from session {}", session_id),
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
        match self.find_cursor_cli() {
            Some(path) => {
                if path.exists() {
                    AvailabilityStatus::Available
                } else {
                    AvailabilityStatus::NotFound {
                        reason: format!("Cursor agent path does not exist: {:?}", path),
                    }
                }
            }
            None => AvailabilityStatus::NotFound {
                reason: "Cursor agent not found. Please install it from https://cursor.sh".to_string(),
            },
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::SessionContinuation,
            AgentCapability::WorkspaceIsolation,
        ]
    }

    fn description(&self) -> Option<String> {
        Some(format!(
            "Cursor agent (force: {}, model: {:?})",
            self.config.force, self.config.model
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type() {
        let agent = CursorAgent::new();
        assert_eq!(agent.agent_type(), "cursor");
    }

    #[test]
    fn test_default_agent() {
        let agent = CursorAgent::new();
        assert!(!agent.config.force);
        assert!(agent.config.model.is_none());
    }

    #[test]
    fn test_builder() {
        let agent = CursorAgent::builder()
            .with_force()
            .with_model("sonnet-4.5")
            .build();

        assert!(agent.config.force);
        assert_eq!(agent.config.model.as_deref(), Some("sonnet-4.5"));
    }

    #[tokio::test]
    async fn test_capabilities() {
        let agent = CursorAgent::new();
        let caps = agent.capabilities();
        assert!(!caps.is_empty());
        assert!(caps.contains(&AgentCapability::SessionContinuation));
    }

    #[tokio::test]
    async fn test_check_availability() {
        let agent = CursorAgent::new();
        let status = agent.check_availability().await;
        // The result depends on what's installed, but it should not panic
        match &status {
            AvailabilityStatus::Available => {}
            AvailabilityStatus::NotFound { .. } => {}
            _ => {}
        }
    }
}
