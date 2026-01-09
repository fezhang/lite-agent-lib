//! Claude Code CLI protocol types
//!
//! This module contains strongly-typed serde structures for Claude's JSON protocol.

use serde::{Deserialize, Serialize};

/// Top-level message types from Claude Code CLI stdout
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClaudeMessage {
    /// System message with initialization info
    System {
        subtype: Option<String>,
        #[serde(default, rename = "apiKeySource")]
        api_key_source: Option<String>,
        #[serde(default)]
        cwd: Option<String>,
        #[serde(default)]
        tools: Option<Vec<serde_json::Value>>,
        #[serde(default)]
        model: Option<String>,
        #[serde(default)]
        session_id: Option<String>,
    },
    /// Assistant message (response from Claude)
    Assistant {
        message: ClaudeChatMessage,
        #[serde(default)]
        session_id: Option<String>,
    },
    /// User message (echoed back or from continuation)
    User {
        message: ClaudeChatMessage,
        #[serde(default)]
        session_id: Option<String>,
    },
    /// Tool use by Claude
    ToolUse {
        tool_name: String,
        #[serde(flatten)]
        tool_data: ClaudeToolData,
        #[serde(default)]
        session_id: Option<String>,
    },
    /// Result from tool execution
    ToolResult {
        result: serde_json::Value,
        #[serde(default)]
        is_error: Option<bool>,
        #[serde(default)]
        session_id: Option<String>,
    },
    /// Streaming event (for real-time updates)
    StreamEvent {
        event: ClaudeStreamEvent,
        #[serde(default)]
        session_id: Option<String>,
    },
    /// Final result message
    Result {
        #[serde(default)]
        subtype: Option<String>,
        #[serde(default, alias = "isError")]
        is_error: Option<bool>,
        #[serde(default, alias = "durationMs")]
        duration_ms: Option<u64>,
        #[serde(default)]
        result: Option<serde_json::Value>,
        #[serde(default)]
        error: Option<String>,
        #[serde(default, alias = "numTurns")]
        num_turns: Option<u32>,
        #[serde(default, alias = "sessionId")]
        session_id: Option<String>,
    },
    /// Approval response for tool use
    #[serde(rename = "approval_response")]
    ApprovalResponse {
        #[serde(rename = "callId")]
        call_id: String,
        tool_name: String,
        approval_status: ApprovalStatus,
    },
}

/// Claude chat message structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClaudeChatMessage {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub message_type: Option<String>,
    pub role: String,
    pub model: Option<String>,
    pub content: Vec<ClaudeContentItem>,
    pub stop_reason: Option<String>,
}

/// Content items in Claude messages
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ClaudeContentItem {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "thinking")]
    Thinking { thinking: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        #[serde(flatten)]
        tool_data: ClaudeToolData,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: serde_json::Value,
        is_error: Option<bool>,
    },
}

/// Streaming events from Claude
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClaudeStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: ClaudeChatMessage },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: usize,
        content_block: ClaudeContentItem,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: usize,
        delta: ClaudeContentDelta,
    },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },
    #[serde(rename = "message_delta")]
    MessageDelta {
        #[serde(default)]
        delta: Option<ClaudeMessageDelta>,
        #[serde(default)]
        usage: Option<ClaudeUsage>,
    },
    #[serde(rename = "message_stop")]
    MessageStop,
}

/// Content delta for streaming
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ClaudeContentDelta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "thinking_delta")]
    ThinkingDelta { thinking: String },
}

/// Message delta for streaming
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ClaudeMessageDelta {
    #[serde(default)]
    pub stop_reason: Option<String>,
    #[serde(default)]
    pub stop_sequence: Option<String>,
}

/// Token usage information
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ClaudeUsage {
    #[serde(default)]
    pub input_tokens: Option<u64>,
    #[serde(default)]
    pub output_tokens: Option<u64>,
    #[serde(default, rename = "cache_creation_input_tokens")]
    pub cache_creation_input_tokens: Option<u64>,
    #[serde(default, rename = "cache_read_input_tokens")]
    pub cache_read_input_tokens: Option<u64>,
    #[serde(default)]
    pub service_tier: Option<String>,
}

/// Tool data structures for Claude
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "name", content = "input")]
pub enum ClaudeToolData {
    #[serde(rename = "Read", alias = "read")]
    Read {
        #[serde(alias = "path")]
        file_path: String,
    },
    #[serde(rename = "Edit", alias = "edit_file")]
    Edit {
        #[serde(alias = "path")]
        file_path: String,
        #[serde(alias = "old_str")]
        old_string: Option<String>,
        #[serde(alias = "new_str")]
        new_string: Option<String>,
    },
    #[serde(rename = "Write", alias = "create_file", alias = "write_file")]
    Write {
        #[serde(alias = "path")]
        file_path: String,
        content: String,
    },
    #[serde(rename = "Bash", alias = "bash")]
    Bash {
        #[serde(alias = "cmd", alias = "command_line")]
        command: String,
        #[serde(default)]
        description: Option<String>,
    },
    #[serde(rename = "Grep", alias = "grep")]
    Grep {
        pattern: String,
        #[serde(default)]
        output_mode: Option<String>,
        #[serde(default)]
        path: Option<String>,
    },
    #[serde(rename = "Glob", alias = "glob")]
    Glob {
        #[serde(alias = "filePattern")]
        pattern: String,
        #[serde(default)]
        path: Option<String>,
        #[serde(default)]
        limit: Option<u32>,
    },
    #[serde(rename = "Task", alias = "task")]
    Task {
        subagent_type: Option<String>,
        description: Option<String>,
        prompt: Option<String>,
    },
    #[serde(rename = "ExitPlanMode")]
    ExitPlanMode { plan: String },
    #[serde(untagged)]
    Unknown {
        #[serde(flatten)]
        data: std::collections::HashMap<String, serde_json::Value>,
    },
}

impl ClaudeToolData {
    pub fn get_name(&self) -> &str {
        match self {
            ClaudeToolData::Read { .. } => "Read",
            ClaudeToolData::Edit { .. } => "Edit",
            ClaudeToolData::Write { .. } => "Write",
            ClaudeToolData::Bash { .. } => "Bash",
            ClaudeToolData::Grep { .. } => "Grep",
            ClaudeToolData::Glob { .. } => "Glob",
            ClaudeToolData::Task { .. } => "Task",
            ClaudeToolData::ExitPlanMode { .. } => "ExitPlanMode",
            ClaudeToolData::Unknown { data } => data
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown"),
        }
    }
}

/// Approval status for tool use
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Denied {
        reason: Option<String>,
    },
    TimedOut,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_message_deserialization() {
        let json = r#"{"type":"system","subtype":"init","model":"claude-sonnet-4"}"#;
        let msg: ClaudeMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClaudeMessage::System { model, .. } => {
                assert_eq!(model.as_deref(), Some("claude-sonnet-4"));
            }
            _ => panic!("Expected system message"),
        }
    }

    #[test]
    fn test_tool_data_get_name() {
        let tool = ClaudeToolData::Read {
            file_path: "/path/to/file".to_string(),
        };
        assert_eq!(tool.get_name(), "Read");
    }

    #[test]
    fn test_approval_status() {
        let json = r#"{"status":"denied","reason":"Not allowed"}"#;
        let status: ApprovalStatus = serde_json::from_str(json).unwrap();
        match status {
            ApprovalStatus::Denied { reason } => {
                assert_eq!(reason.as_deref(), Some("Not allowed"));
            }
            _ => panic!("Expected denied status"),
        }
    }
}
