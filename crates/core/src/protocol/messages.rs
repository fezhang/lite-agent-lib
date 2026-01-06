//! Protocol message types
//!
//! Defines the message types used in the JSON streaming protocol for
//! bidirectional communication with agent processes.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique request identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(String);

impl RequestId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Protocol message sent between the orchestrator and agent
///
/// Messages are newline-delimited JSON (NDJSON).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProtocolMessage {
    /// User message/input
    User {
        content: String,
    },

    /// Control request (from orchestrator to agent)
    ControlRequest {
        request_id: String,
        #[serde(flatten)]
        request: ControlRequest,
    },

    /// Control response (from agent to orchestrator)
    ControlResponse {
        request_id: String,
        response: ControlResponse,
    },

    /// Log/data message
    Log {
        #[serde(flatten)]
        entry: LogEntry,
    },

    /// Result/final message
    Result {
        #[serde(flatten)]
        result: ResultMessage,
    },

    /// Error message
    Error {
        message: String,
        #[serde(flatten)]
        details: serde_json::Value,
    },
}

impl ProtocolMessage {
    /// Create a user message
    pub fn user(content: String) -> Self {
        Self::User { content }
    }

    /// Create a control request
    pub fn control_request(request_id: RequestId, request: ControlRequest) -> Self {
        Self::ControlRequest {
            request_id: request_id.0,
            request,
        }
    }

    /// Create a control response
    pub fn control_response(request_id: RequestId, response: ControlResponse) -> Self {
        Self::ControlResponse {
            request_id: request_id.0,
            response,
        }
    }

    /// Create a log message
    pub fn log(entry: LogEntry) -> Self {
        Self::Log { entry }
    }

    /// Create a result message
    pub fn result(result: ResultMessage) -> Self {
        Self::Result { result }
    }

    /// Create an error message
    pub fn error(message: String, details: serde_json::Value) -> Self {
        Self::Error { message, details }
    }
}

/// Control request types (orchestrator → agent)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "subtype", rename_all = "snake_case")]
pub enum ControlRequest {
    /// Initialize the protocol session
    Initialize {
        #[serde(skip_serializing_if = "Option::is_none")]
        hooks: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        config: Option<serde_json::Value>,
    },

    /// Set permission mode
    SetPermissionMode {
        mode: PermissionMode,
        #[serde(skip_serializing_if = "Option::is_none")]
        destination: Option<PermissionDestination>,
    },

    /// Interrupt execution
    Interrupt {
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },

    /// Request status
    Status {},

    /// Custom control request
    Custom {
        name: String,
        #[serde(flatten)]
        params: serde_json::Value,
    },
}

/// Control response types (agent → orchestrator)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "result_type", rename_all = "snake_case")]
pub enum ControlResponse {
    /// Success response
    Success {
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },

    /// Error response
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
    },

    /// Acknowledgment
    Ack {
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
}

/// Permission modes for agent execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    /// Ask for each tool/action
    Default,

    /// Auto-approve file edits only
    AcceptEdits,

    /// Plan mode - only approve ExitPlanMode tool
    Plan,

    /// Auto-approve everything
    BypassPermissions,
}

/// Destination for permission mode changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionDestination {
    /// Apply to current session only
    Session,

    /// Apply to current tool use only
    Tool,
}

/// Log entry types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "log_type", rename_all = "snake_case")]
pub enum LogEntry {
    /// Stdout output
    Stdout {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<String>,
    },

    /// Stderr output
    Stderr {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<String>,
    },

    /// Tool use
    ToolUse {
        tool_use_id: String,
        name: String,
        input: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<String>,
    },

    /// Tool result
    ToolResult {
        tool_use_id: String,
        output: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<String>,
    },

    /// Thinking/reasoning
    Thinking {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<String>,
    },

    /// Status update
    Status {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<String>,
    },

    /// Progress update
    Progress {
        percent: f32,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<String>,
    },

    /// Generic log entry
    Other {
        #[serde(flatten)]
        data: serde_json::Value,
    },
}

/// Result message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "result_type", content = "data", rename_all = "snake_case")]
pub enum ResultMessage {
    /// Successful completion
    Success {
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        return_code: Option<i32>,
    },

    /// Failed completion
    Failure {
        error: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        exit_code: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        details: Option<serde_json::Value>,
    },

    /// Interrupted completion
    Interrupted {
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
}

/// Tool approval request/response types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolApprovalRequest {
    pub tool_name: String,
    pub input: serde_json::Value,
    pub tool_use_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_suggestions: Option<Vec<PermissionUpdate>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolApprovalResponse {
    pub behavior: ApprovalBehavior,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_permissions: Option<Vec<PermissionUpdate>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interrupt: Option<bool>,
}

/// Approval behavior
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalBehavior {
    Allow,
    Deny,
    Ask,
}

/// Permission update
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct PermissionUpdate {
    #[serde(rename = "type")]
    pub update_type: PermissionUpdateType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<PermissionMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<PermissionDestination>,
}

/// Permission update type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionUpdateType {
    SetMode,
    ApproveTool,
    DenyTool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id() {
        let id = RequestId::new();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn test_message_serialization() {
        let msg = ProtocolMessage::user("test content".to_string());
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ProtocolMessage = serde_json::from_str(&json).unwrap();

        match parsed {
            ProtocolMessage::User { content } => {
                assert_eq!(content, "test content");
            }
            _ => panic!("Expected user message"),
        }
    }

    #[test]
    fn test_control_request_serialization() {
        let req = ControlRequest::Initialize {
            hooks: None,
            config: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("initialize"));
    }

    #[test]
    fn test_log_entry_serialization() {
        let entry = LogEntry::Stdout {
            content: "output".to_string(),
            timestamp: Some("2024-01-01T00:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("stdout"));
        assert!(json.contains("output"));

        let parsed: LogEntry = serde_json::from_str(&json).unwrap();
        match parsed {
            LogEntry::Stdout { content, .. } => {
                assert_eq!(content, "output");
            }
            _ => panic!("Expected stdout log entry"),
        }
    }

    #[test]
    fn test_permission_mode() {
        let mode = PermissionMode::BypassPermissions;
        let json = serde_json::to_string(&mode).unwrap();
        assert!(json.contains("bypassPermissions"));

        let parsed: PermissionMode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, PermissionMode::BypassPermissions);
    }
}
