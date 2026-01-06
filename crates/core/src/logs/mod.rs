//! Log normalization module (stub - to be implemented in Phase 2)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Normalized log entry - unified format for all agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedEntry {
    /// Timestamp (ISO 8601)
    pub timestamp: Option<String>,
    /// Entry type
    pub entry_type: EntryType,
    /// Human-readable content
    pub content: String,
    /// Structured metadata (agent-specific)
    pub metadata: Option<serde_json::Value>,
    /// Source agent type
    pub agent_type: String,
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
    Thinking { details: Option<String> },
    /// Agent action/operation
    Action { action: ActionInfo },
    /// System message
    System,
    /// Error message
    Error { error_type: ErrorType },
    /// Progress update
    Progress { percent: f32, message: Option<String> },
}

/// Action information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInfo {
    pub name: String,
    pub status: ActionStatus,
    pub details: serde_json::Value,
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
    Other,
}

/// In-memory log store (stub)
#[derive(Debug)]
pub struct LogStore {
    entries: Arc<RwLock<Vec<NormalizedEntry>>>,
}

impl LogStore {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_entry(&self, entry: NormalizedEntry) {
        self.entries.write().await.push(entry);
    }

    pub async fn get_entries(&self) -> Vec<NormalizedEntry> {
        self.entries.read().await.clone()
    }
}

impl Default for LogStore {
    fn default() -> Self {
        Self::new()
    }
}
