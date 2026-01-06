//! Agent executor abstraction module
//!
//! This module provides the core `AgentExecutor` trait that all agents must implement,
//! along with related types for agent configuration, spawning, and lifecycle management.

use async_trait::async_trait;
use futures_util::stream::BoxStream;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod config;
pub mod error;
pub mod spawned;

pub use config::AgentConfig;
pub use error::{AgentError, AgentResult};
pub use spawned::{ExitResult, SpawnedAgent};

use crate::logs::{LogStore, NormalizedEntry};

/// Agent capability flags
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentCapability {
    /// Agent supports continuing from previous session (follow-up)
    SessionContinuation,

    /// Agent supports bidirectional control protocol
    BidirectionalControl,

    /// Agent can be isolated in workspace
    WorkspaceIsolation,

    /// Agent requires setup/installation
    RequiresSetup,

    /// Custom capability
    Custom(String),
}

/// Agent availability status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AvailabilityStatus {
    /// Agent is available and ready to use
    Available,

    /// Agent is installed but may need authentication
    InstalledNotAuthenticated,

    /// Agent is not found/installed
    NotFound { reason: String },

    /// Agent requires setup before use
    RequiresSetup { instructions: String },
}

impl AvailabilityStatus {
    /// Check if agent is available for use
    pub fn is_available(&self) -> bool {
        matches!(self, AvailabilityStatus::Available)
    }
}

/// Core trait for agent executors
///
/// This trait defines the interface that all agents must implement.
/// It's generic to support any type of agent (shell, LLM, coding, etc.).
#[async_trait]
pub trait AgentExecutor: Send + Sync {
    /// Unique identifier for this agent type
    ///
    /// Examples: "shell", "python", "llm", "claude-code"
    fn agent_type(&self) -> &str;

    /// Spawn a new agent session with initial input
    ///
    /// This creates a new agent process/session and returns a `SpawnedAgent`
    /// handle that can be used to interact with the agent.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for agent execution
    /// * `input` - Initial input/prompt for the agent
    ///
    /// # Returns
    ///
    /// A `SpawnedAgent` containing the process handle and communication channels
    async fn spawn(
        &self,
        config: &AgentConfig,
        input: &str,
    ) -> Result<SpawnedAgent, AgentError>;

    /// Continue an existing session with follow-up input
    ///
    /// This allows conversation continuity by continuing from a previous session.
    /// Not all agents support this capability.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for agent execution
    /// * `input` - Follow-up input/prompt
    /// * `session_id` - ID of the session to continue
    ///
    /// # Returns
    ///
    /// A new `SpawnedAgent` that continues the previous session
    async fn spawn_follow_up(
        &self,
        _config: &AgentConfig,
        _input: &str,
        _session_id: &str,
    ) -> Result<SpawnedAgent, AgentError> {
        // Default implementation: session continuation not supported
        Err(AgentError::Custom(format!(
            "Agent '{}' does not support session continuation",
            self.agent_type()
        )))
    }

    /// Normalize raw logs from this agent into unified format
    ///
    /// This method converts agent-specific log formats into the standardized
    /// `NormalizedEntry` format for consistent rendering and storage.
    ///
    /// # Arguments
    ///
    /// * `raw_logs` - Raw log store from the agent
    ///
    /// # Returns
    ///
    /// A stream of normalized log entries
    fn normalize_logs(
        &self,
        raw_logs: Arc<LogStore>,
    ) -> BoxStream<'static, NormalizedEntry>;

    /// Check if agent is available/installed
    ///
    /// This allows checking agent availability before attempting to spawn.
    async fn check_availability(&self) -> AvailabilityStatus {
        // Default implementation: assume available
        AvailabilityStatus::Available
    }

    /// Get agent capabilities
    ///
    /// Returns a list of capabilities that this agent supports.
    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![]
    }

    /// Get agent description (optional)
    fn description(&self) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests;
