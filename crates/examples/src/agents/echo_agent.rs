//! Echo agent - A simple example agent that echoes input back
//!
//! This agent demonstrates the basic structure of implementing the `AgentExecutor` trait.
//! It spawns a simple process that echoes its input back to stdout.

use async_trait::async_trait;
use futures_util::stream::{self, BoxStream, StreamExt};
use std::process::Stdio;
use std::sync::Arc;

use lite_agent_core::{
    AgentCapability, AgentConfig, AgentError, AgentExecutor, AvailabilityStatus, LogStore,
    NormalizedEntry, SpawnedAgent,
};

use command_group::AsyncCommandGroup;
use tokio::process::Command as TokioCommand;

/// Simple echo agent that repeats input back
///
/// This is a minimal example agent for demonstration purposes.
#[derive(Debug, Clone)]
pub struct EchoAgent;

impl EchoAgent {
    /// Create a new echo agent
    pub fn new() -> Self {
        Self
    }
}

impl Default for EchoAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentExecutor for EchoAgent {
    fn agent_type(&self) -> &str {
        "echo"
    }

    async fn spawn(
        &self,
        config: &AgentConfig,
        input: &str,
    ) -> Result<SpawnedAgent, AgentError> {
        // Use the system's echo command
        let mut cmd = if cfg!(windows) {
            TokioCommand::new("cmd")
        } else {
            TokioCommand::new("echo")
        };

        // Add arguments
        if cfg!(windows) {
            cmd.args(["/c", "echo", input]);
        } else {
            cmd.arg(input);
        }

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

        // Spawn the process group
        let child = cmd
            .group_spawn()
            .map_err(|e| AgentError::SpawnError(format!("Failed to spawn echo agent: {}", e)))?;

        // Create log store
        let log_store = Arc::new(LogStore::new());

        // Create spawned agent (stdio handles are managed by the command itself)
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
        _raw_logs: Arc<LogStore>,
    ) -> BoxStream<'static, NormalizedEntry> {
        // For echo agent, we just return a simple output entry
        let agent_type = self.agent_type().to_string();
        let entry = NormalizedEntry {
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            entry_type: lite_agent_core::EntryType::Output,
            content: "Echo agent executed".to_string(),
            metadata: None,
            agent_type,
        };

        stream::iter(vec![entry]).boxed()
    }

    async fn check_availability(&self) -> AvailabilityStatus {
        // Check if echo command is available
        let result = if cfg!(windows) {
            std::process::Command::new("cmd")
                .args(["/c", "echo", "test"])
                .output()
        } else {
            std::process::Command::new("echo")
                .arg("test")
                .output()
        };

        match result {
            Ok(_) => AvailabilityStatus::Available,
            Err(_) => AvailabilityStatus::NotFound {
                reason: "echo command not found".to_string(),
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
        Some("A simple echo agent that repeats input back".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type() {
        let agent = EchoAgent::new();
        assert_eq!(agent.agent_type(), "echo");
    }

    #[test]
    fn test_description() {
        let agent = EchoAgent::new();
        assert!(agent.description().is_some());
    }

    #[tokio::test]
    async fn test_capabilities() {
        let agent = EchoAgent::new();
        let caps = agent.capabilities();
        assert!(!caps.is_empty());
    }
}
