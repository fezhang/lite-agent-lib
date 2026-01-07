//! lite-agent-examples
//!
//! Example agent implementations demonstrating the `AgentExecutor` trait.

pub mod agents;

pub use agents::claude_agent::ClaudeCodeAgent;
pub use agents::echo_agent::EchoAgent;
pub use agents::shell_agent::ShellAgent;
