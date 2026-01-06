use thiserror::Error;

/// Errors that can occur during agent operations
#[derive(Debug, Error)]
pub enum AgentError {
    /// Agent spawn failed
    #[error("Agent spawn failed: {0}")]
    SpawnError(String),

    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    /// Agent not available (not installed or not authenticated)
    #[error("Agent not available: {0}")]
    AgentNotAvailable(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(#[from] crate::protocol::ProtocolError),

    /// Workspace error
    #[error("Workspace error: {0}")]
    Workspace(#[from] crate::workspace::WorkspaceError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Timeout error
    #[error("Timeout error")]
    Timeout,

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Custom error for agent-specific issues
    #[error("Custom error: {0}")]
    Custom(String),
}

/// Result type alias for agent operations
pub type AgentResult<T> = Result<T, AgentError>;
