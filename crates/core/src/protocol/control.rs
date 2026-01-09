//! High-level control protocol abstractions
//!
//! This module defines traits and types for interacting with agent control protocols,
//! particularly bidirectional control protocols like Claude Code's.

use crate::agent::AgentError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::future::Future;

/// Permission modes for agents that support permission control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    /// Default permission mode (ask for permissions)
    Default,
    /// Accept all edits without prompting
    AcceptEdits,
    /// Plan mode - agent plans but doesn't execute
    Plan,
    /// Bypass all permissions
    BypassPermissions,
}

impl PermissionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::AcceptEdits => "acceptEdits",
            Self::Plan => "plan",
            Self::BypassPermissions => "bypassPermissions",
        }
    }
}

impl std::fmt::Display for PermissionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Result of a tool approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "behavior", rename_all = "camelCase")]
pub enum ApprovalResponse {
    /// Allow the tool use with optional input modification
    Allow {
        /// Modified input for the tool
        updated_input: serde_json::Value,
        /// Optional permission updates to apply
        updated_permissions: Option<Vec<PermissionUpdate>>,
    },
    /// Deny the tool use
    Deny {
        /// Reason for denial
        message: String,
        /// Whether to interrupt the agent
        interrupt: Option<bool>,
    },
}

/// Permission update operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionUpdate {
    #[serde(rename = "type")]
    pub update_type: PermissionUpdateType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<PermissionMode>,
    pub destination: PermissionUpdateDestination,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionUpdateType {
    SetMode,
    AddRules,
    RemoveRules,
    ClearRules,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionUpdateDestination {
    Session,
    UserSettings,
    ProjectSettings,
    LocalSettings,
}

/// Trait for agents that support permission control
#[async_trait]
pub trait PermissionControl: Send + Sync {
    /// Set the permission mode for the agent
    async fn set_permission_mode(&self, mode: PermissionMode) -> Result<(), AgentError>;

    /// Interrupt the current agent execution
    async fn interrupt(&self) -> Result<(), AgentError>;
}

/// Trait for agents that support tool approval handling
#[async_trait]
pub trait ToolApproval: Send + Sync {
    /// Handle a tool approval request from the agent
    async fn handle_tool_approval(
        &self,
        tool_name: String,
        input: serde_json::Value,
        permission_suggestions: Option<Vec<PermissionUpdate>>,
        tool_use_id: Option<String>,
    ) -> Result<ApprovalResponse, AgentError>;

    /// Handle a hook callback from the agent
    async fn handle_hook_callback(
        &self,
        callback_id: String,
        input: serde_json::Value,
        tool_use_id: Option<String>,
    ) -> Result<serde_json::Value, AgentError>;
}

/// Trait for agents that support session continuation
#[async_trait]
pub trait SessionContinuation: Send + Sync {
    /// Continue from a previous session
    async fn continue_session(
        &self,
        session_id: String,
        prompt: String,
    ) -> Result<Box<dyn SessionHandle>, AgentError>;
}

/// Handle for an active agent session that can be controlled
#[async_trait]
pub trait SessionHandle: Send + Sync {
    /// Send a follow-up message
    async fn send_message(&self, message: String) -> Result<(), AgentError>;

    /// Get the session ID
    fn session_id(&self) -> &str;

    /// Check if the session is still active
    fn is_active(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_mode_display() {
        assert_eq!(PermissionMode::Default.to_string(), "default");
        assert_eq!(PermissionMode::Plan.to_string(), "plan");
        assert_eq!(PermissionMode::BypassPermissions.to_string(), "bypassPermissions");
    }

    #[test]
    fn test_approval_response_serialization() {
        let response = ApprovalResponse::Allow {
            updated_input: serde_json::json!({"test": "value"}),
            updated_permissions: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"behavior\":\"allow\""));
    }
}
