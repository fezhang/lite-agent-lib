//! Cursor agent protocol types
//!
//! This module contains strongly-typed serde structures for Cursor's JSON protocol.

use serde::{Deserialize, Serialize};

/// Top-level message types from Cursor agent stdout
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum CursorMessage {
    /// System initialization message
    #[serde(rename = "system")]
    System {
        #[serde(default)]
        subtype: Option<String>,
        #[serde(default, rename = "apiKeySource")]
        api_key_source: Option<String>,
        #[serde(default)]
        cwd: Option<String>,
        #[serde(default)]
        session_id: Option<String>,
        #[serde(default)]
        model: Option<String>,
        #[serde(default, rename = "permissionMode")]
        permission_mode: Option<String>,
    },
    /// User message
    #[serde(rename = "user")]
    User {
        message: CursorChatMessage,
        #[serde(default)]
        session_id: Option<String>,
    },
    /// Assistant message
    #[serde(rename = "assistant")]
    Assistant {
        message: CursorChatMessage,
        #[serde(default)]
        session_id: Option<String>,
    },
    /// Thinking message
    #[serde(rename = "thinking")]
    Thinking {
        #[serde(default)]
        subtype: Option<String>,
        #[serde(default)]
        text: Option<String>,
        #[serde(default)]
        session_id: Option<String>,
    },
    /// Tool call
    #[serde(rename = "tool_call")]
    ToolCall {
        #[serde(default)]
        subtype: Option<String>, // "started" | "completed"
        #[serde(default)]
        call_id: Option<String>,
        tool_call: CursorToolCall,
        #[serde(default)]
        session_id: Option<String>,
    },
    /// Result message
    #[serde(rename = "result")]
    Result {
        #[serde(default)]
        subtype: Option<String>,
        #[serde(default)]
        is_error: Option<bool>,
        #[serde(default)]
        duration_ms: Option<u64>,
        #[serde(default)]
        result: Option<serde_json::Value>,
        #[serde(default)]
        session_id: Option<String>,
    },
}

impl CursorMessage {
    /// Extract session ID if present
    pub fn extract_session_id(&self) -> Option<String> {
        match self {
            CursorMessage::System { .. } => None,
            CursorMessage::User { session_id, .. } => session_id.clone(),
            CursorMessage::Assistant { session_id, .. } => session_id.clone(),
            CursorMessage::Thinking { session_id, .. } => session_id.clone(),
            CursorMessage::ToolCall { session_id, .. } => session_id.clone(),
            CursorMessage::Result { session_id, .. } => session_id.clone(),
        }
    }
}

/// Cursor chat message structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CursorChatMessage {
    pub role: String,
    pub content: Vec<CursorContentItem>,
}

impl CursorChatMessage {
    /// Concatenate all text content items
    pub fn concat_text(&self) -> Option<String> {
        let mut out = String::new();
        for CursorContentItem::Text { text } in &self.content {
            out.push_str(text);
        }
        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }
}

/// Content items in Cursor messages
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum CursorContentItem {
    #[serde(rename = "text")]
    Text { text: String },
}

/// Tool call structures for Cursor
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum CursorToolCall {
    #[serde(rename = "shellToolCall")]
    Shell {
        args: CursorShellArgs,
        #[serde(default)]
        result: Option<serde_json::Value>,
    },
    #[serde(rename = "lsToolCall")]
    LS {
        args: CursorLsArgs,
        #[serde(default)]
        result: Option<serde_json::Value>,
    },
    #[serde(rename = "globToolCall")]
    Glob {
        args: CursorGlobArgs,
        #[serde(default)]
        result: Option<serde_json::Value>,
    },
    #[serde(rename = "grepToolCall")]
    Grep {
        args: CursorGrepArgs,
        #[serde(default)]
        result: Option<serde_json::Value>,
    },
    #[serde(rename = "writeToolCall")]
    Write {
        args: CursorWriteArgs,
        #[serde(default)]
        result: Option<serde_json::Value>,
    },
    #[serde(rename = "readToolCall")]
    Read {
        args: CursorReadArgs,
        #[serde(default)]
        result: Option<serde_json::Value>,
    },
    #[serde(rename = "editToolCall")]
    Edit {
        args: CursorEditArgs,
        #[serde(default)]
        result: Option<serde_json::Value>,
    },
    #[serde(rename = "deleteToolCall")]
    Delete {
        args: CursorDeleteArgs,
        #[serde(default)]
        result: Option<serde_json::Value>,
    },
    #[serde(untagged)]
    Unknown {
        #[serde(flatten)]
        data: std::collections::HashMap<String, serde_json::Value>,
    },
}

impl CursorToolCall {
    pub fn get_name(&self) -> &str {
        match self {
            CursorToolCall::Shell { .. } => "shell",
            CursorToolCall::LS { .. } => "ls",
            CursorToolCall::Glob { .. } => "glob",
            CursorToolCall::Grep { .. } => "grep",
            CursorToolCall::Write { .. } => "write",
            CursorToolCall::Read { .. } => "read",
            CursorToolCall::Edit { .. } => "edit",
            CursorToolCall::Delete { .. } => "delete",
            CursorToolCall::Unknown { data } => data
                .keys()
                .next()
                .map(|s| s.as_str())
                .unwrap_or("unknown"),
        }
    }
}

/// Tool argument structures
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CursorShellArgs {
    pub command: String,
    #[serde(default, alias = "working_directory")]
    pub working_directory: Option<String>,
    #[serde(default)]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CursorLsArgs {
    pub path: String,
    #[serde(default)]
    pub ignore: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CursorGlobArgs {
    #[serde(default, alias = "globPattern")]
    pub glob_pattern: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CursorGrepArgs {
    pub pattern: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub output_mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CursorWriteArgs {
    pub path: String,
    #[serde(default, alias = "fileText")]
    pub contents: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CursorReadArgs {
    pub path: String,
    #[serde(default)]
    pub offset: Option<u64>,
    #[serde(default)]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CursorEditArgs {
    pub path: String,
    #[serde(default)]
    pub old_text: Option<String>,
    #[serde(default)]
    pub new_text: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CursorDeleteArgs {
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_message_deserialization() {
        let json = r#"{"type":"system","subtype":"init","model":"gpt-4"}"#;
        let msg: CursorMessage = serde_json::from_str(json).unwrap();
        match msg {
            CursorMessage::System { model, .. } => {
                assert_eq!(model.as_deref(), Some("gpt-4"));
            }
            _ => panic!("Expected system message"),
        }
    }

    #[test]
    fn test_chat_message_concat_text() {
        let msg = CursorChatMessage {
            role: "assistant".to_string(),
            content: vec![
                CursorContentItem::Text {
                    text: "Hello ".to_string(),
                },
                CursorContentItem::Text {
                    text: "world".to_string(),
                },
            ],
        };

        assert_eq!(msg.concat_text().as_deref(), Some("Hello world"));
    }

    #[test]
    fn test_tool_call_get_name() {
        let tool = CursorToolCall::Shell {
            args: CursorShellArgs {
                command: "ls".to_string(),
                working_directory: None,
                timeout: None,
            },
            result: None,
        };

        assert_eq!(tool.get_name(), "shell");
    }

    #[test]
    fn test_session_id_extraction() {
        let msg = CursorMessage::Assistant {
            message: CursorChatMessage {
                role: "assistant".to_string(),
                content: vec![],
            },
            session_id: Some("test-session".to_string()),
        };

        assert_eq!(msg.extract_session_id().as_deref(), Some("test-session"));
    }
}
