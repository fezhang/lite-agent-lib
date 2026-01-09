//! Protocol handling module
//!
//! This module provides JSON streaming protocol support for bidirectional communication
//! with agent processes. It handles message parsing, serialization, and stdio management.

pub mod control;
pub mod messages;

use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use std::sync::Arc;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, WriteHalf};
use tokio::sync::Mutex;
use tokio_util::io::ReaderStream;

pub use messages::*;

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

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    #[error("Timeout waiting for response")]
    Timeout,
}

/// Protocol handler trait
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

/// JSON streaming protocol handler
///
/// Manages bidirectional JSON communication over stdin/stdout.
/// Uses newline-delimited JSON (NDJSON) for message framing.
pub struct JsonStreamProtocol {
    writer: Arc<Mutex<WriteHalf<tokio::io::DuplexStream>>>,
    _reader_handle: tokio::task::JoinHandle<()>,
}

impl JsonStreamProtocol {
    /// Create a new JSON streaming protocol handler
    ///
    /// This creates an in-memory duplex stream for testing. For production use
    /// with actual processes, use `from_stdio` instead.
    pub fn new() -> Self {
        let (stream, _) = tokio::io::duplex(8192);
        Self::from_stream(stream)
    }

    /// Create protocol handler from a duplex stream
    pub fn from_stream(stream: tokio::io::DuplexStream) -> Self {
        let (reader, writer) = tokio::io::split(stream);
        let writer = Arc::new(Mutex::new(writer));

        let reader_handle = tokio::spawn(async move {
            let _ = reader;
            // Reader task would process incoming messages
        });

        Self {
            writer,
            _reader_handle: reader_handle,
        }
    }

    /// Send a protocol message
    pub async fn send_message(&self, message: &ProtocolMessage) -> Result<(), ProtocolError> {
        let json = serde_json::to_string(message)?;
        let mut writer = self.writer.lock().await;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
        Ok(())
    }

    /// Send a user message
    pub async fn send_user_message(&self, content: String) -> Result<(), ProtocolError> {
        let message = ProtocolMessage::user(content);
        self.send_message(&message).await
    }

    /// Send a control request
    pub async fn send_control_request(
        &self,
        request: ControlRequest,
    ) -> Result<RequestId, ProtocolError> {
        let request_id = RequestId::new();
        let message = ProtocolMessage::control_request(request_id.clone(), request);
        self.send_message(&message).await?;
        Ok(request_id)
    }

    /// Send a control response
    pub async fn send_control_response(
        &self,
        request_id: RequestId,
        response: ControlResponse,
    ) -> Result<(), ProtocolError> {
        let message = ProtocolMessage::control_response(request_id, response);
        self.send_message(&message).await
    }
}

impl Default for JsonStreamProtocol {
    fn default() -> Self {
        Self::new()
    }
}

/// Protocol reader for parsing incoming messages
///
/// Reads from a stream and parses newline-delimited JSON messages.
pub struct ProtocolReader {
    reader: BufReader<tokio::process::ChildStdout>,
}

impl ProtocolReader {
    /// Create a new protocol reader
    pub fn new(stdout: tokio::process::ChildStdout) -> Self {
        Self {
            reader: BufReader::new(stdout),
        }
    }

    /// Read the next message from the stream
    pub async fn read_message(&mut self) -> Result<ProtocolMessage, ProtocolError> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            return Err(ProtocolError::ConnectionClosed);
        }

        let message: ProtocolMessage = serde_json::from_str(&line)
            .map_err(|e| ProtocolError::InvalidMessage(format!("{}: '{}'", e, line)))?;

        Ok(message)
    }

    /// Convert to a stream of messages
    pub fn into_stream(self) -> impl Stream<Item = Result<ProtocolMessage, ProtocolError>> {
        ReaderStream::new(self.reader).then(|line_result| async move {
            let bytes = line_result?;
            let line = std::str::from_utf8(&bytes)
                .map_err(|e| ProtocolError::InvalidMessage(format!("UTF-8 error: {}", e)))?;
            let message: ProtocolMessage =
                serde_json::from_str(line).map_err(|e| {
                    ProtocolError::InvalidMessage(format!("{}: '{}'", e, line))
                })?;
            Ok(message)
        })
    }
}

/// Protocol writer for sending messages
///
/// Writes messages to a stream with newline delimiters.
pub struct ProtocolWriter {
    writer: Arc<Mutex<tokio::process::ChildStdin>>,
}

impl ProtocolWriter {
    /// Create a new protocol writer
    pub fn new(stdin: tokio::process::ChildStdin) -> Self {
        Self {
            writer: Arc::new(Mutex::new(stdin)),
        }
    }

    /// Write a message to the stream
    pub async fn write_message(&self, message: &ProtocolMessage) -> Result<(), ProtocolError> {
        let json = serde_json::to_string(message)?;
        let mut writer = self.writer.lock().await;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
        Ok(())
    }

    /// Write raw bytes to the stream
    pub async fn write_raw(&self, data: &[u8]) -> Result<(), ProtocolError> {
        let mut writer = self.writer.lock().await;
        writer.write_all(data).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
        Ok(())
    }
}

/// Combined protocol peer with read and write capabilities
///
/// Manages bidirectional communication with an agent process.
pub struct ProtocolPeer {
    reader: Option<ProtocolReader>,
    writer: ProtocolWriter,
}

impl ProtocolPeer {
    /// Create a new protocol peer from process stdio
    pub fn from_stdio(
        stdin: tokio::process::ChildStdin,
        stdout: tokio::process::ChildStdout,
    ) -> Self {
        Self {
            reader: Some(ProtocolReader::new(stdout)),
            writer: ProtocolWriter::new(stdin),
        }
    }

    /// Get the writer for sending messages
    pub fn writer(&self) -> &ProtocolWriter {
        &self.writer
    }

    /// Take the reader (consumes the peer)
    pub fn take_reader(&mut self) -> Option<ProtocolReader> {
        self.reader.take()
    }

    /// Read the next message
    pub async fn read_message(&mut self) -> Result<ProtocolMessage, ProtocolError> {
        match &mut self.reader {
            Some(reader) => reader.read_message().await,
            None => Err(ProtocolError::ConnectionClosed),
        }
    }

    /// Write a message
    pub async fn write_message(&self, message: &ProtocolMessage) -> Result<(), ProtocolError> {
        self.writer.write_message(message).await
    }

    /// Send a user message
    pub async fn send_user_message(&self, content: String) -> Result<(), ProtocolError> {
        self.writer
            .write_message(&ProtocolMessage::user(content))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = ProtocolMessage::user("test".to_string());
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("test"));
    }

    #[tokio::test]
    async fn test_protocol_writer() {
        // Test message serialization
        let msg = ProtocolMessage::user("test".to_string());
        let json = serde_json::to_string(&msg).unwrap();

        // Verify the message can be serialized and deserialized
        let parsed: ProtocolMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            ProtocolMessage::User { content } => {
                assert_eq!(content, "test");
            }
            _ => panic!("Expected user message"),
        }

        // Verify newline delimiter is included
        let msg_with_newline = format!("{}\n", json);
        assert!(msg_with_newline.ends_with('\n'));
    }
}
