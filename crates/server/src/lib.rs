//! lite-agent-server
//!
//! REST API server for lite-agent-lib with SSE log streaming support.

pub mod api;
pub mod handlers;
pub mod routes;

pub use api::{AgentRegistry, ServerState};
pub use routes::create_router;

pub use lite_agent_core;
