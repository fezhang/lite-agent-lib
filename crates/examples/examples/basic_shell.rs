//! Basic shell agent example
//!
//! Demonstrates executing shell commands with AgentRunner.

use lite_agent_core::{AgentConfig, AgentRunner};
use lite_agent_examples::ShellAgent;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create the shell agent
    let agent = ShellAgent::new();

    // Create a runner with the agent
    let runner = AgentRunner::new(agent);

    // Configure the agent
    let config = AgentConfig::new(PathBuf::from("."));

    // Run a shell command
    println!("Running shell agent...");

    // Try different commands based on platform
    let command = if cfg!(windows) {
        "dir"
    } else {
        "ls -la"
    };

    println!("Executing command: {}", command);
    let result = runner.run(command, &config).await?;

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
