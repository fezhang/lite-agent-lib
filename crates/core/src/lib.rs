//! # lite-agent-core
//!
//! A lightweight, async-first Rust library for managing different kinds of agents.
//!
//! ## Features
//!
//! - Generic agent abstraction via `AgentExecutor` trait
//! - Async protocol handling (JSON streaming, stdin/stdout)
//! - Log normalization to unified format
//! - Session management for conversation continuity
//! - Workspace isolation using git worktrees
//!
//! ## Quick Start
//!
//! ```ignore
//! use lite_agent_core::{AgentExecutor, AgentConfig, AgentRunner};
//!
//! #[tokio::main]
//! async fn main() {
//!     let executor = MyCustomAgent::new();
//!     let runner = AgentRunner::new(executor);
//!
//!     let config = AgentConfig::default();
//!     let result = runner.run("input", config).await.unwrap();
//! }
//! ```

// Re-export public API
pub mod agent;
pub mod logs;
pub mod protocol;
pub mod session;
pub mod workspace;

// Convenience re-exports for common types
pub use agent::{
    AgentCapability, AgentConfig, AgentError, AgentExecutor, AgentResult, AvailabilityStatus,
    ExitResult, SpawnedAgent,
};

pub use logs::{ActionInfo, ActionStatus, EntryType, ErrorType, LogStore, NormalizedEntry};

pub use protocol::{JsonStreamProtocol, ProtocolError, ProtocolHandler};

pub use session::{ExecutionInfo, ExecutionStatus, SessionManager, SessionState, SessionStatus};

pub use workspace::{
    IsolationType, WorkspaceConfig, WorkspaceError, WorkspaceManager, WorkspacePath,
};
