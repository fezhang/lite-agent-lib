//! Built-in agent implementations
//!
//! This module provides ready-to-use implementations of popular coding agents
//! including Claude Code CLI and Cursor.

pub mod claude;
pub mod cursor;

// Re-export commonly used types
pub use claude::{ClaudeAgent, ClaudeConfig, ClaudeConfigBuilder};
pub use cursor::{CursorAgent, CursorConfig, CursorConfigBuilder};
