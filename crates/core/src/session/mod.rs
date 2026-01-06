//! Session management module
//!
//! This module provides session tracking and lifecycle management for agent executions.
//! It maintains in-memory state for sessions and their associated log stores.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::agent::AgentError;
use crate::logs::LogStore;

/// Session manager
///
/// Manages agent sessions and their associated log stores.
/// Provides CRUD operations for sessions and execution tracking.
pub struct SessionManager {
    _sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    _log_stores: Arc<RwLock<HashMap<String, Arc<LogStore>>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            _sessions: Arc::new(RwLock::new(HashMap::new())),
            _log_stores: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session
    ///
    /// # Arguments
    ///
    /// * `agent_type` - Type of agent (e.g., "shell", "llm")
    /// * `initial_input` - Initial input for the session
    ///
    /// # Returns
    ///
    /// The created session ID
    pub async fn create_session(
        &self,
        agent_type: String,
        initial_input: String,
    ) -> Result<String, AgentError> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Create initial execution info
        let execution_id = Uuid::new_v4().to_string();
        let execution = ExecutionInfo {
            id: execution_id.clone(),
            started_at: now,
            completed_at: None,
            status: ExecutionStatus::Running,
            exit_code: None,
            input: initial_input,
        };

        // Create session state
        let session = SessionState {
            id: session_id.clone(),
            agent_type,
            created_at: now,
            updated_at: now,
            executions: vec![execution],
            status: SessionStatus::Active,
        };

        // Create log store for this session
        let log_store = Arc::new(LogStore::new());

        // Store session and log store
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session);
        }

        {
            let mut log_stores = self.log_stores.write().await;
            log_stores.insert(session_id.clone(), log_store);
        }

        Ok(session_id)
    }

    /// Get a session by ID
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session ID to retrieve
    ///
    /// # Returns
    ///
    /// The session state if found
    pub async fn get_session(&self, session_id: &str) -> Option<SessionState> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    /// Get all sessions
    ///
    /// # Returns
    ///
    /// A vector of all session states
    pub async fn get_all_sessions(&self) -> Vec<SessionState> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Update session status
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session ID to update
    /// * `status` - New status
    pub async fn update_session_status(
        &self,
        session_id: &str,
        status: SessionStatus,
    ) -> Result<(), AgentError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| AgentError::SessionNotFound(session_id.to_string()))?;

        session.status = status;
        session.updated_at = Utc::now();

        Ok(())
    }

    /// Add a new execution to a session
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session ID to add execution to
    /// * `input` - Input for this execution
    ///
    /// # Returns
    ///
    /// The execution ID
    pub async fn add_execution(
        &self,
        session_id: &str,
        input: String,
    ) -> Result<String, AgentError> {
        let execution_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let execution = ExecutionInfo {
            id: execution_id.clone(),
            started_at: now,
            completed_at: None,
            status: ExecutionStatus::Running,
            exit_code: None,
            input,
        };

        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| AgentError::SessionNotFound(session_id.to_string()))?;

        session.executions.push(execution);
        session.updated_at = Utc::now();
        session.status = SessionStatus::Active;

        Ok(execution_id)
    }

    /// Update execution status
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session ID
    /// * `execution_id` - Execution ID to update
    /// * `status` - New execution status
    /// * `exit_code` - Optional exit code
    pub async fn update_execution(
        &self,
        session_id: &str,
        execution_id: &str,
        status: ExecutionStatus,
        exit_code: Option<i32>,
    ) -> Result<(), AgentError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| AgentError::SessionNotFound(session_id.to_string()))?;

        let execution = session
            .executions
            .iter_mut()
            .find(|e| e.id == execution_id)
            .ok_or_else(|| AgentError::Custom(format!("Execution not found: {}", execution_id)))?;

        execution.status = status.clone();
        execution.exit_code = exit_code;

        if matches!(
            status,
            ExecutionStatus::Completed | ExecutionStatus::Failed | ExecutionStatus::Cancelled
        ) {
            execution.completed_at = Some(Utc::now());
        }

        session.updated_at = Utc::now();

        Ok(())
    }

    /// Get log store for a session
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session ID
    ///
    /// # Returns
    ///
    /// The log store if found
    pub async fn get_log_store(&self, session_id: &str) -> Option<Arc<LogStore>> {
        let log_stores = self.log_stores.read().await;
        log_stores.get(session_id).cloned()
    }

    /// Delete a session
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session ID to delete
    pub async fn delete_session(&self, session_id: &str) -> Result<(), AgentError> {
        // Remove session
        {
            let mut sessions = self.sessions.write().await;
            sessions
                .remove(session_id)
                .ok_or_else(|| AgentError::SessionNotFound(session_id.to_string()))?;
        }

        // Remove log store
        {
            let mut log_stores = self.log_stores.write().await;
            log_stores.remove(session_id);
        }

        Ok(())
    }

    /// Clean up old sessions
    ///
    /// # Arguments
    ///
    /// * `older_than` - Duration after which sessions are considered old
    ///
    /// # Returns
    ///
    /// Number of sessions cleaned up
    pub async fn cleanup_old_sessions(&self, older_than: chrono::Duration) -> usize {
        let cutoff = Utc::now() - older_than;
        let mut to_remove = Vec::new();

        // Find old sessions
        {
            let sessions = self.sessions.read().await;
            for (id, session) in sessions.iter() {
                if session.updated_at < cutoff {
                    to_remove.push(id.clone());
                }
            }
        }

        let count = to_remove.len();

        // Remove old sessions
        for id in to_remove {
            let _ = self.delete_session(&id).await;
        }

        count
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub id: String,
    pub agent_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub executions: Vec<ExecutionInfo>,
    pub status: SessionStatus,
}

/// Execution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInfo {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ExecutionStatus,
    pub exit_code: Option<i32>,
    pub input: String,
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Idle,
    Completed,
    Failed,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let manager = SessionManager::new();

        let session_id = manager
            .create_session("test-agent".to_string(), "test input".to_string())
            .await
            .unwrap();

        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.agent_type, "test-agent");
        assert_eq!(session.executions.len(), 1);
        assert_eq!(session.executions[0].input, "test input");
        assert!(matches!(session.status, SessionStatus::Active));
    }

    #[tokio::test]
    async fn test_get_session_not_found() {
        let manager = SessionManager::new();

        let session = manager.get_session("non-existent").await;
        assert!(session.is_none());
    }

    #[tokio::test]
    async fn test_update_session_status() {
        let manager = SessionManager::new();

        let session_id = manager
            .create_session("test-agent".to_string(), "test input".to_string())
            .await
            .unwrap();

        manager
            .update_session_status(&session_id, SessionStatus::Completed)
            .await
            .unwrap();

        let session = manager.get_session(&session_id).await.unwrap();
        assert!(matches!(session.status, SessionStatus::Completed));
    }

    #[tokio::test]
    async fn test_add_execution() {
        let manager = SessionManager::new();

        let session_id = manager
            .create_session("test-agent".to_string(), "first input".to_string())
            .await
            .unwrap();

        let execution_id = manager
            .add_execution(&session_id, "second input".to_string())
            .await
            .unwrap();

        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.executions.len(), 2);
        assert_eq!(session.executions[1].input, "second input");
        assert_eq!(session.executions[1].id, execution_id);
    }

    #[tokio::test]
    async fn test_update_execution() {
        let manager = SessionManager::new();

        let session_id = manager
            .create_session("test-agent".to_string(), "test input".to_string())
            .await
            .unwrap();

        let session = manager.get_session(&session_id).await.unwrap();
        let execution_id = &session.executions[0].id;

        manager
            .update_execution(
                &session_id,
                execution_id,
                ExecutionStatus::Completed,
                Some(0),
            )
            .await
            .unwrap();

        let session = manager.get_session(&session_id).await.unwrap();
        assert!(matches!(
            session.executions[0].status,
            ExecutionStatus::Completed
        ));
        assert_eq!(session.executions[0].exit_code, Some(0));
        assert!(session.executions[0].completed_at.is_some());
    }

    #[tokio::test]
    async fn test_get_log_store() {
        let manager = SessionManager::new();

        let session_id = manager
            .create_session("test-agent".to_string(), "test input".to_string())
            .await
            .unwrap();

        let log_store = manager.get_log_store(&session_id).await;
        assert!(log_store.is_some());

        // Add an entry to verify it works
        let log_store = log_store.unwrap();
        log_store
            .add_system("test message".to_string(), "test-agent".to_string())
            .await;

        let entries = log_store.get_entries().await;
        assert_eq!(entries.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_session() {
        let manager = SessionManager::new();

        let session_id = manager
            .create_session("test-agent".to_string(), "test input".to_string())
            .await
            .unwrap();

        // Verify session exists
        assert!(manager.get_session(&session_id).await.is_some());
        assert!(manager.get_log_store(&session_id).await.is_some());

        // Delete session
        manager.delete_session(&session_id).await.unwrap();

        // Verify session is gone
        assert!(manager.get_session(&session_id).await.is_none());
        assert!(manager.get_log_store(&session_id).await.is_none());
    }

    #[tokio::test]
    async fn test_get_all_sessions() {
        let manager = SessionManager::new();

        let id1 = manager
            .create_session("agent1".to_string(), "input1".to_string())
            .await
            .unwrap();
        let id2 = manager
            .create_session("agent2".to_string(), "input2".to_string())
            .await
            .unwrap();

        let sessions = manager.get_all_sessions().await;
        assert_eq!(sessions.len(), 2);

        let ids: Vec<_> = sessions.iter().map(|s| s.id.clone()).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[tokio::test]
    async fn test_cleanup_old_sessions() {
        let manager = SessionManager::new();

        // Create a session
        let session_id = manager
            .create_session("test-agent".to_string(), "test input".to_string())
            .await
            .unwrap();

        // Manually update the timestamp to make it old
        {
            let mut sessions = manager.sessions.write().await;
            if let Some(session) = sessions.get_mut(&session_id) {
                session.updated_at = Utc::now() - chrono::Duration::hours(25);
            }
        }

        // Cleanup sessions older than 24 hours
        let cleaned = manager
            .cleanup_old_sessions(chrono::Duration::hours(24))
            .await;

        assert_eq!(cleaned, 1);
        assert!(manager.get_session(&session_id).await.is_none());
    }

    #[tokio::test]
    async fn test_session_serialization() {
        let manager = SessionManager::new();

        let session_id = manager
            .create_session("test-agent".to_string(), "test input".to_string())
            .await
            .unwrap();

        let session = manager.get_session(&session_id).await.unwrap();

        // Test serialization
        let json = serde_json::to_string(&session).unwrap();
        assert!(json.contains("test-agent"));
        assert!(json.contains("test input"));

        // Test deserialization
        let deserialized: SessionState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, session.id);
        assert_eq!(deserialized.agent_type, session.agent_type);
    }
}
