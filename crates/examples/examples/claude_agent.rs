//! Claude Code CLI agent example
//!
//! This example demonstrates how to use the built-in Claude agent
//! with various configuration options.

use lite_agent_core::{AgentConfig, AgentExecutor, AgentRunner};
use lite_agent_core::agents::{ClaudeAgent, ClaudeConfigBuilder};
use lite_agent_core::protocol::control::PermissionMode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Example 1: Basic Claude agent with default configuration
    println!("=== Example 1: Basic Claude Agent ===");
    let agent = ClaudeAgent::new();
    println!("Agent type: {}", agent.agent_type());
    println!("Capabilities: {:?}", agent.capabilities());
    println!("Description: {:?}\n", agent.description());

    // Example 2: Claude agent with custom configuration
    println!("=== Example 2: Claude Agent with Config ===");
    let config = ClaudeConfigBuilder::default()
        .with_plan_mode()
        .with_model("claude-sonnet-4")
        .build();

    let agent = ClaudeAgent::with_config(config);
    println!("Plan mode enabled: {:?}", agent.config.plan_mode);
    println!("Model: {:?}\n", agent.config.model);

    // Example 3: Claude agent with debug mode and custom permissions
    println!("=== Example 3: Claude Agent with Debug + Permissions ===");
    let agent = ClaudeAgent::builder()
        .with_debug_mode()
        .with_permission_mode(PermissionMode::Plan)
        .with_disable_api_key()
        .build();

    println!("Debug mode: {}", agent.config.debug_mode);
    println!("Permission mode: {:?}", agent.config.permission_mode);
    println!("API key disabled: {}\n", agent.config.disable_api_key);

    // Example 4: Check availability before running
    println!("=== Example 4: Check Agent Availability ===");
    let agent = ClaudeAgent::new();
    let status = agent.check_availability().await;
    println!("Availability status: {:?}\n", status);

    // Example 5: Run agent (commented out - requires actual CLI)
    // println!("=== Example 5: Run Claude Agent ===");
    // let agent = ClaudeAgent::new();
    // let runner = AgentRunner::new(agent);
    //
    // let config = AgentConfig::default();
    // let result = runner.run("Help me understand this codebase", config).await?;
    // println!("Result: {:?}\n", result);

    // Example 6: Session continuation
    println!("=== Example 6: Session Continuation Support ===");
    let agent = ClaudeAgent::new();
    let caps = agent.capabilities();
    let supports_continuation = caps.iter().any(|c| matches!(c, crate::agent::AgentCapability::SessionContinuation));
    println!("Supports session continuation: {}\n", supports_continuation);

    println!("All examples completed successfully!");
    Ok(())
}
