//! Cursor agent example
//!
//! This example demonstrates how to use the built-in Cursor agent
//! with various configuration options.

use lite_agent_core::{AgentConfig, AgentExecutor, AgentRunner};
use lite_agent_core::agents::{CursorAgent, CursorConfigBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Example 1: Basic Cursor agent with default configuration
    println!("=== Example 1: Basic Cursor Agent ===");
    let agent = CursorAgent::new();
    println!("Agent type: {}", agent.agent_type());
    println!("Capabilities: {:?}", agent.capabilities());
    println!("Description: {:?}\n", agent.description());

    // Example 2: Cursor agent with custom configuration
    println!("=== Example 2: Cursor Agent with Config ===");
    let config = CursorConfigBuilder::default()
        .with_force()
        .with_model("sonnet-4.5")
        .build();

    let agent = CursorAgent::with_config(config);
    println!("Force mode: {}", agent.config.force);
    println!("Model: {:?}\n", agent.config.model);

    // Example 3: Check availability before running
    println!("=== Example 3: Check Agent Availability ===");
    let agent = CursorAgent::new();
    let status = agent.check_availability().await;
    println!("Availability status: {:?}\n", status);

    // Example 4: Run agent (commented out - requires actual CLI)
    // println!("=== Example 4: Run Cursor Agent ===");
    // let agent = CursorAgent::builder()
    //     .with_model("gpt-5")
    //     .with_force()
    //     .build();
    //
    // let runner = AgentRunner::new(agent);
    // let config = AgentConfig::default();
    // let result = runner.run("Fix the bug in main.rs", config).await?;
    // println!("Result: {:?}\n", result);

    // Example 5: Different model configurations
    println!("=== Example 5: Model Configurations ===");
    let models = vec!["auto", "sonnet-4.5", "gpt-5", "opus-4.1"];

    for model in models {
        let agent = CursorAgent::builder()
            .with_model(model)
            .build();
        println!("Model {:?}: Description = {:?}", model, agent.description());
    }
    println!();

    // Example 6: Session continuation
    println!("=== Example 6: Session Continuation Support ===");
    let agent = CursorAgent::new();
    let caps = agent.capabilities();
    let supports_continuation = caps.iter().any(|c| matches!(c, crate::agent::AgentCapability::SessionContinuation));
    println!("Supports session continuation: {}\n", supports_continuation);

    println!("All examples completed successfully!");
    Ok(())
}
