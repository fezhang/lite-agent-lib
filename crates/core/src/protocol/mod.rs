//! Protocol handling module (stub - to be implemented in Phase 2)

use async_trait::async_trait;
use thiserror::Error;

/// Protocol errors
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Connection closed")]
    ConnectionClosed,
}

/// Protocol handler trait (stub)
#[async_trait]
pub trait ProtocolHandler: Send + Sync {
    /// Send data to agent
    async fn send(&self, data: &[u8]) -> Result<(), ProtocolError>;

    /// Receive data from agent
    async fn receive(&self) -> Result<Vec<u8>, ProtocolError>;

    /// Initialize protocol (handshake)
    async fn initialize(&self) -> Result<(), ProtocolError>;

    /// Graceful shutdown
    async fn shutdown(&self) -> Result<(), ProtocolError>;
}

/// JSON streaming protocol (stub)
pub struct JsonStreamProtocol {
    // To be implemented in Phase 2
}
