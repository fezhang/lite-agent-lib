//! Basic echo agent example
//!
//! Demonstrates the simplest way to use an agent with AgentRunner.

use lite_agent_core::{AgentConfig, AgentRunner};
use lite_agent_examples::EchoAgent;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create the echo agent
    let agent = EchoAgent::new();

    // Create a runner with the agent
    let runner = AgentRunner::new(agent);

    // Configure the agent
    let config = AgentConfig::new(PathBuf::from("."));

    // Run the agent with input
    println!("Running echo agent...");
    let result = runner.run("Hello, lite-agent-lib!", config).await?;

    println!("\n=== Result ===");
    println!("Success: {}", result.success);
    println!("Exit Result: {:?}", result.exit_result);
    println!("\n=== Logs ===");
    for log in &result.logs {
        println!("{:?}: {}", log.entry_type, log.content);
    }

    println!("\n=== Output ===");
    println!("{}", result.output);

    Ok(())
}
