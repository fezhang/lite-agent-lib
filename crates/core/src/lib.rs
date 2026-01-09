//! # lite-agent-core
//!
//! A lightweight, async-first Rust library for managing different kinds of agents.
//!
//! ## Features
//!
//! - Generic agent abstraction via `AgentExecutor` trait
//! - Built-in support for Claude Code CLI and Cursor agents
//! - Async protocol handling (JSON streaming, stdin/stdout)
//! - Bidirectional control protocol support
//! - Log normalization to unified format
//! - Session management for conversation continuity
//! - Workspace isolation using git worktrees
//!
//! ## Quick Start
//!
//! ### Using Built-in Agents
//!
//! ```ignore
//! use lite_agent_core::{AgentConfig, AgentRunner, agents::ClaudeAgent};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a Claude agent with custom configuration
//!     let agent = ClaudeAgent::builder()
//!         .with_plan_mode()
//!         .with_model("claude-sonnet-4")
//!         .build();
//!
//!     let runner = AgentRunner::new(agent);
//!     let config = AgentConfig::default();
//!     let result = runner.run("Help me refactor this code", config).await.unwrap();
//! }
//! ```
//!
//! ### Using Cursor Agent
//!
//! ```ignore
//! use lite_agent_core::{AgentConfig, AgentRunner, agents::CursorAgent};
//!
//! #[tokio::main]
//! async fn main() {
//!     let agent = CursorAgent::builder()
//!         .with_force()
//!         .with_model("sonnet-4.5")
//!         .build();
//!
//!     let runner = AgentRunner::new(agent);
//!     let config = AgentConfig::default();
//!     let result = runner.run("Fix the bug in main.rs", config).await.unwrap();
//! }
//! ```
//!
//! ### Custom Agent Implementation
//!
//! ```ignore
//! use lite_agent_core::{AgentExecutor, AgentConfig, AgentRunner};
//!
//! struct MyCustomAgent;
//!
//! #[async_trait::async_trait]
//! impl AgentExecutor for MyCustomAgent {
//!     fn agent_type(&self) -> &str {
//!         "my-custom-agent"
//!     }
//!
//!     async fn spawn(&self, config: &AgentConfig, input: &str)
//!         -> Result<SpawnedAgent, AgentError>
//!     {
//!         // Implementation here
//!     }
//!     // ... implement other required methods
//! }
//! ```

// Re-export public API
pub mod agent;
pub mod agents;
pub mod logs;
pub mod protocol;
pub mod session;
pub mod workspace;

// Convenience re-exports for common types
pub use agent::{
    AgentCapability, AgentConfig, AgentError, AgentExecutor, AgentResult, AgentRunner,
    AvailabilityStatus, ExitResult, RunResult, SpawnedAgent,
};

pub use logs::{ActionInfo, ActionStatus, EntryType, ErrorType, LogStore, NormalizedEntry};

pub use protocol::{
    JsonStreamProtocol, PermissionControl, PermissionMode, ProtocolError, ProtocolHandler,
    SessionContinuation, SessionHandle, ToolApproval,
};

pub use session::{ExecutionInfo, ExecutionStatus, SessionManager, SessionState, SessionStatus};

pub use workspace::{
    IsolationType, WorkspaceConfig, WorkspaceError, WorkspaceManager, WorkspacePath,
};

// Re-export built-in agents
pub use agents::{ClaudeAgent, ClaudeConfig, CursorAgent, CursorConfig};
