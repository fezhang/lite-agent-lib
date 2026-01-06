//! Log normalization and storage module
//!
//! This module provides unified log format and storage for all agent types.
//! It supports streaming logs, raw log collection from process stdout/stderr,
//! and real-time broadcasting to multiple consumers.

use chrono::Utc;
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

pub mod collector;

pub use collector::LogCollector;

/// Normalized log entry - unified format for all agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedEntry {
    /// Timestamp (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Entry type
    pub entry_type: EntryType,
    /// Human-readable content
    pub content: String,
    /// Structured metadata (agent-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Source agent type
    pub agent_type: String,
}

impl NormalizedEntry {
    /// Create a new normalized entry
    pub fn new(entry_type: EntryType, content: String, agent_type: String) -> Self {
        Self {
            timestamp: Some(Utc::now().to_rfc3339()),
            entry_type,
            content,
            metadata: None,
            agent_type,
        }
    }

    /// Add metadata to the entry
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Entry type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EntryType {
    /// Agent input/request
    Input,
    /// Agent output/response
    Output,
    /// Agent thinking/processing
    Thinking {
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<String>,
    },
    /// Agent action/operation
    Action { action: ActionInfo },
    /// System message
    System,
    /// Error message
    Error { error_type: ErrorType },
    /// Progress update
    Progress {
        percent: f32,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
}

impl EntryType {
    /// Create an input entry type
    pub fn input() -> Self {
        Self::Input
    }

    /// Create an output entry type
    pub fn output() -> Self {
        Self::Output
    }

    /// Create a thinking entry type
    pub fn thinking(details: Option<String>) -> Self {
        Self::Thinking { details }
    }

    /// Create an action entry type
    pub fn action(action: ActionInfo) -> Self {
        Self::Action { action }
    }

    /// Create a system entry type
    pub fn system() -> Self {
        Self::System
    }

    /// Create an error entry type
    pub fn error(error_type: ErrorType) -> Self {
        Self::Error { error_type }
    }

    /// Create a progress entry type
    pub fn progress(percent: f32, message: Option<String>) -> Self {
        Self::Progress { percent, message }
    }
}

/// Action information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInfo {
    pub name: String,
    pub status: ActionStatus,
    pub details: serde_json::Value,
}

impl ActionInfo {
    /// Create a new action info
    pub fn new(name: String, status: ActionStatus, details: serde_json::Value) -> Self {
        Self {
            name,
            status,
            details,
        }
    }
}

/// Action status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Started,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Error type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    Timeout,
    Authentication,
    NotFound,
    PermissionDenied,
    Io(String),
    Other,
}

/// In-memory log store with streaming support
///
/// Stores normalized log entries and broadcasts them to subscribers.
#[derive(Debug, Clone)]
pub struct LogStore {
    entries: Arc<RwLock<Vec<NormalizedEntry>>>,
    broadcaster: Arc<broadcast::Sender<NormalizedEntry>>,
    capacity: usize,
}

impl LogStore {
    /// Create a new log store with default capacity (1000 entries)
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// Create a new log store with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self {
            entries: Arc::new(RwLock::new(Vec::with_capacity(capacity))),
            broadcaster: Arc::new(tx),
            capacity,
        }
    }

    /// Add an entry to the store
    pub async fn add_entry(&self, entry: NormalizedEntry) {
        // Store in memory
        let mut entries = self.entries.write().await;
        if entries.len() >= self.capacity {
            entries.remove(0); // Remove oldest entry
        }
        entries.push(entry.clone());

        // Broadcast to subscribers
        let _ = self.broadcaster.send(entry);
    }

    /// Get all entries from the store
    pub async fn get_entries(&self) -> Vec<NormalizedEntry> {
        self.entries.read().await.clone()
    }

    /// Get entries since a specific index
    pub async fn get_entries_since(&self, index: usize) -> Vec<NormalizedEntry> {
        let entries = self.entries.read().await;
        if index >= entries.len() {
            Vec::new()
        } else {
            entries[index..].to_vec()
        }
    }

    /// Get the current number of entries
    pub async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Check if the store is empty
    pub async fn is_empty(&self) -> bool {
        self.entries.read().await.is_empty()
    }

    /// Clear all entries
    pub async fn clear(&self) {
        self.entries.write().await.clear();
    }

    /// Subscribe to log entries as a stream
    ///
    /// Returns a stream that will receive all new log entries.
    pub fn subscribe(&self) -> LogStream {
        LogStream {
            receiver: self.broadcaster.subscribe(),
        }
    }

    /// Get a snapshot of current entries + a subscription for future entries
    pub async fn snapshot_and_subscribe_async(&self) -> (Vec<NormalizedEntry>, LogStream) {
        let entries = self.entries.read().await.clone();
        let stream = self.subscribe();
        (entries, stream)
    }

    /// Add raw stdout output
    pub async fn add_stdout(&self, content: String, agent_type: String) {
        let entry = NormalizedEntry::new(EntryType::output(), content, agent_type);
        self.add_entry(entry).await;
    }

    /// Add raw stderr output
    pub async fn add_stderr(&self, content: String, agent_type: String) {
        let entry = NormalizedEntry::new(
            EntryType::error(ErrorType::Io(content.clone())),
            content,
            agent_type,
        );
        self.add_entry(entry).await;
    }

    /// Add a system message
    pub async fn add_system(&self, content: String, agent_type: String) {
        let entry = NormalizedEntry::new(EntryType::system(), content, agent_type);
        self.add_entry(entry).await;
    }
}

impl Default for LogStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Log stream receiver
///
/// A stream of log entries from a LogStore.
#[derive(Debug)]
pub struct LogStream {
    receiver: broadcast::Receiver<NormalizedEntry>,
}

impl LogStream {
    /// Create a new log stream
    pub fn new(receiver: broadcast::Receiver<NormalizedEntry>) -> Self {
        Self { receiver }
    }
}

impl futures_util::Stream for LogStream {
    type Item = NormalizedEntry;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        // Use try_recv to avoid blocking
        match self.receiver.try_recv() {
            Ok(entry) => std::task::Poll::Ready(Some(entry)),
            Err(broadcast::error::TryRecvError::Empty) => {
                // Schedule wakeup
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            }
            Err(broadcast::error::TryRecvError::Closed) => std::task::Poll::Ready(None),
            Err(_) => std::task::Poll::Ready(None), // Lagged - channel overflow
        }
    }
}

/// Log entry stream type alias
pub type LogEntryStream = Pin<Box<dyn futures_util::Stream<Item = NormalizedEntry> + Send>>;

/// Trait for converting raw logs to normalized format
pub trait LogNormalizer: Send + Sync {
    /// Normalize raw logs into a stream of normalized entries
    fn normalize(&self, raw_logs: Arc<LogStore>) -> LogEntryStream;
}

/// Simple text log normalizer
///
/// Treats each line as an output entry.
pub struct TextLogNormalizer {
    agent_type: String,
}

impl TextLogNormalizer {
    /// Create a new text log normalizer
    pub fn new(agent_type: String) -> Self {
        Self { agent_type }
    }
}

impl LogNormalizer for TextLogNormalizer {
    fn normalize(&self, raw_logs: Arc<LogStore>) -> LogEntryStream {
        let agent_type = self.agent_type.clone();

        // Get entries upfront to avoid async in stream
        let entries = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::try_current().unwrap().block_on(raw_logs.get_entries())
        });

        // Create a simple iterator stream
        Box::pin(futures_util::stream::iter(entries))
    }
}

/// JSON log normalizer
///
/// Parses JSON log messages and extracts structured information.
pub struct JsonLogNormalizer {
    agent_type: String,
}

impl JsonLogNormalizer {
    /// Create a new JSON log normalizer
    pub fn new(agent_type: String) -> Self {
        Self { agent_type }
    }

    /// Parse a JSON log entry
    fn parse_json_entry(&self, json: &str) -> Option<NormalizedEntry> {
        let value: serde_json::Value = serde_json::from_str(json).ok()?;

        // Extract common fields
        let content = value.get("content")
            .or_else(|| value.get("message"))
            .or_else(|| value.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let entry_type = value.get("type")
            .and_then(|v| v.as_str())
            .and_then(|t| self.parse_entry_type(t, &value))
            .unwrap_or(EntryType::output());

        Some(NormalizedEntry {
            timestamp: value.get("timestamp").and_then(|v| v.as_str()).map(|s| s.to_string()),
            entry_type,
            content,
            metadata: Some(value.clone()),
            agent_type: self.agent_type.clone(),
        })
    }

    fn parse_entry_type(&self, type_str: &str, value: &serde_json::Value) -> Option<EntryType> {
        match type_str {
            "input" => Some(EntryType::Input),
            "output" | "stdout" => Some(EntryType::Output),
            "thinking" => Some(EntryType::thinking(
                value.get("details").and_then(|v| v.as_str()).map(|s| s.to_string()),
            )),
            "action" => {
                let action = ActionInfo {
                    name: value.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                    status: value.get("status")
                        .and_then(|v| v.as_str())
                        .and_then(|s| match s {
                            "started" => Some(ActionStatus::Started),
                            "in_progress" => Some(ActionStatus::InProgress),
                            "completed" => Some(ActionStatus::Completed),
                            "failed" => Some(ActionStatus::Failed),
                            "cancelled" => Some(ActionStatus::Cancelled),
                            _ => None,
                        })
                        .unwrap_or(ActionStatus::InProgress),
                    details: value.get("details").cloned().unwrap_or(serde_json::Value::Null),
                };
                Some(EntryType::action(action))
            }
            "error" | "stderr" => {
                let error_type = match value.get("error_type").and_then(|v| v.as_str()) {
                    Some("timeout") => ErrorType::Timeout,
                    Some("authentication") => ErrorType::Authentication,
                    Some("not_found") => ErrorType::NotFound,
                    Some("permission_denied") => ErrorType::PermissionDenied,
                    _ => ErrorType::Other,
                };
                Some(EntryType::error(error_type))
            }
            "progress" => {
                let percent = value.get("percent").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let message = value.get("message").and_then(|v| v.as_str()).map(|s| s.to_string());
                Some(EntryType::progress(percent, message))
            }
            "system" => Some(EntryType::System),
            _ => None,
        }
    }
}

impl LogNormalizer for JsonLogNormalizer {
    fn normalize(&self, raw_logs: Arc<LogStore>) -> LogEntryStream {
        let agent_type = self.agent_type.clone();

        // Get entries upfront to avoid async in stream
        let entries = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::try_current().unwrap().block_on(raw_logs.get_entries())
        });

        // Parse each entry
        let normalized: Vec<NormalizedEntry> = entries
            .into_iter()
            .filter_map(|entry| {
                if let Some(parsed) = self.parse_json_entry(&entry.content) {
                    Some(parsed)
                } else {
                    // Fall back to plain text
                    Some(NormalizedEntry {
                        timestamp: entry.timestamp.clone(),
                        entry_type: EntryType::Output,
                        content: entry.content.clone(),
                        metadata: None,
                        agent_type: agent_type.clone(),
                    })
                }
            })
            .collect();

        // Create a simple iterator stream
        Box::pin(futures_util::stream::iter(normalized))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_store() {
        let store = LogStore::new();

        let entry = NormalizedEntry::new(
            EntryType::output(),
            "test content".to_string(),
            "test-agent".to_string(),
        );

        store.add_entry(entry.clone()).await;

        let entries = store.get_entries().await;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "test content");
    }

    #[tokio::test]
    async fn test_log_stream() {
        let store = LogStore::new();
        let mut stream = store.subscribe();

        // Add entry
        let entry = NormalizedEntry::new(
            EntryType::output(),
            "test".to_string(),
            "test-agent".to_string(),
        );
        store.add_entry(entry).await;

        // Receive from stream
        let received = stream.next().await.unwrap();
        assert_eq!(received.content, "test");
    }

    #[tokio::test]
    async fn test_log_store_capacity() {
        let store = LogStore::with_capacity(3);

        // Add 5 entries
        for i in 0..5 {
            let entry = NormalizedEntry::new(
                EntryType::output(),
                format!("entry {}", i),
                "test-agent".to_string(),
            );
            store.add_entry(entry).await;
        }

        let entries = store.get_entries().await;
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].content, "entry 2"); // First two entries removed
    }

    #[test]
    fn test_normalized_entry() {
        let entry = NormalizedEntry::new(
            EntryType::thinking(Some("thinking details".to_string())),
            "content".to_string(),
            "test-agent".to_string(),
        );

        assert!(entry.timestamp.is_some());
        assert_eq!(entry.content, "content");
        assert_eq!(entry.agent_type, "test-agent");
    }

    #[test]
    fn test_entry_type() {
        let input_type = EntryType::Input;
        let output_type = EntryType::Output;
        let error_type = EntryType::Error {
            error_type: ErrorType::Timeout,
        };

        let json = serde_json::to_string(&input_type).unwrap();
        assert!(json.contains("input"));

        let json = serde_json::to_string(&error_type).unwrap();
        assert!(json.contains("error"));
    }
}
