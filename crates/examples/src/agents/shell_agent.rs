//! Shell agent - Execute shell commands
//!
//! This agent executes shell commands and returns their output.
//! It supports both Windows (cmd.exe) and Unix (sh/bash) systems.

use async_trait::async_trait;
use command_group::CommandGroup;
use futures_util::stream::{self, BoxStream, StreamExt};
use std::process::Stdio;
use std::sync::Arc;

use lite_agent_core::{
    AgentCapability, AgentConfig, AgentError, AgentExecutor, AvailabilityStatus, EntryType,
    LogStore, NormalizedEntry, SpawnedAgent,
};

/// Shell command executor
///
/// This agent executes shell commands in the system's default shell.
#[derive(Debug, Clone)]
pub struct ShellAgent;

impl ShellAgent {
    /// Create a new shell agent
    pub fn new() -> Self {
        Self
    }

    /// Get the default shell command for the current platform
    fn get_shell_command() -> &'static str {
        if cfg!(windows) {
            "cmd.exe"
        } else {
            "sh"
        }
    }

    /// Get shell arguments for the current platform
    fn get_shell_args(command: &str) -> Vec<String> {
        if cfg!(windows) {
            vec!["/C".to_string(), command.to_string()]
        } else {
            vec!["-c".to_string(), command.to_string()]
        }
    }
}

impl Default for ShellAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentExecutor for ShellAgent {
    fn agent_type(&self) -> &str {
        "shell"
    }

    async fn spawn(
        &self,
        config: &AgentConfig,
        input: &str,
    ) -> Result<SpawnedAgent, AgentError> {
        let shell = Self::get_shell_command();
        let args = Self::get_shell_args(input);

        let mut cmd = std::process::Command::new(shell);
        cmd.args(&args);

        // Configure working directory
        cmd.current_dir(&config.work_dir);

        // Configure environment variables
        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        // Set up stdio
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Spawn the process
        let child = cmd
            .group_spawn()
            .map_err(|e| AgentError::SpawnError(format!("Failed to spawn shell: {}", e)))?;

        // Get stdio handles
        let stdin = child.stdin();
        let stdout = child.stdout();
        let stderr = child.stderr();

        // Create log store
        let log_store = Arc::new(LogStore::new());

        // Create spawned agent
        let spawned = SpawnedAgent {
            child,
            stdin,
            stdout,
            stderr,
            exit_signal: None,
            interrupt_signal: None,
            log_store,
        };

        Ok(spawned)
    }

    fn normalize_logs(
        &self,
        _raw_logs: Arc<LogStore>,
    ) -> BoxStream<'static, NormalizedEntry> {
        let agent_type = self.agent_type().to_string();

        // For shell agent, we create output entries
        let entry = NormalizedEntry {
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            entry_type: EntryType::Output,
            content: "Shell command executed".to_string(),
            metadata: None,
            agent_type,
        };

        stream::iter(vec![entry]).boxed()
    }

    async fn check_availability(&self) -> AvailabilityStatus {
        let shell = Self::get_shell_command();

        // Try to execute a simple command
        let result = if cfg!(windows) {
            std::process::Command::new(shell)
                .args(["/C", "echo", "test"])
                .output()
        } else {
            std::process::Command::new(shell)
                .args(["-c", "echo", "test"])
                .output()
        };

        match result {
            Ok(_) => AvailabilityStatus::Available,
            Err(e) => AvailabilityStatus::NotFound {
                reason: format!("Shell '{}' not available: {}", shell, e),
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
        Some(format!(
            "Execute shell commands using {}",
            Self::get_shell_command()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type() {
        let agent = ShellAgent::new();
        assert_eq!(agent.agent_type(), "shell");
    }

    #[test]
    fn test_description() {
        let agent = ShellAgent::new();
        assert!(agent.description().is_some());
    }

    #[tokio::test]
    async fn test_capabilities() {
        let agent = ShellAgent::new();
        let caps = agent.capabilities();
        assert!(!caps.is_empty());
    }

    #[tokio::test]
    async fn test_availability() {
        let agent = ShellAgent::new();
        let status = agent.check_availability().await;
        // Shell should be available on most systems
        assert!(status.is_available());
    }
}
