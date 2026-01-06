//! AgentRunner feature demonstration
//!
//! Demonstrates advanced features of AgentRunner including workspace isolation,
//! configuration options, and error handling.

use lite_agent_core::{AgentConfig, AgentRunner};
use lite_agent_examples::ShellAgent;
use std::path::PathBuf;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create the shell agent
    let agent = ShellAgent::new();

    // Example 1: Basic configuration
    println!("=== Example 1: Basic Configuration ===");
    let runner = AgentRunner::new(agent);

    let config = AgentConfig::new(PathBuf::from("."));
    let result = runner.run("echo 'Hello from AgentRunner!'", config).await?;

    println!("Success: {}", result.success);
    println!("Output: {}", result.output);

    // Example 2: Configuration with environment variables
    println!("\n=== Example 2: With Environment Variables ===");

    let config = AgentConfig::new(PathBuf::from("."))
        .add_env("MY_VAR", "my_value")
        .add_env("ANOTHER_VAR", "another_value");

    let command = if cfg!(windows) {
        "echo %MY_VAR%"
    } else {
        "echo $MY_VAR"
    };

    let result = runner.run(command, config).await?;
    println!("Output: {}", result.output);

    // Example 3: Configuration with timeout
    println!("\n=== Example 3: With Timeout ===");

    let config = AgentConfig::new(PathBuf::from("."))
        .with_timeout(Duration::from_secs(5));

    let result = runner.run("echo 'With timeout'", config).await?;
    println!("Success: {}", result.success);

    // Example 4: Error handling
    println!("\n=== Example 4: Error Handling ===");

    let config = AgentConfig::new(PathBuf::from("."));
    let result = runner.run("false", config).await?; // Command that fails

    println!("Command failed (as expected)");
    println!("Success: {}", result.success);
    println!("Exit Result: {:?}", result.exit_result);

    // Example 5: Using workspace isolation
    println!("\n=== Example 5: Workspace Isolation ===");

    let temp_dir = tempfile::tempdir()?;
    let runner = AgentRunner::with_workspace(agent, temp_dir.path().to_path_buf());

    let config = AgentConfig::new(PathBuf::from("."));
    let result = runner.run("echo 'Running in isolated workspace'", config).await?;

    println!("Success: {}", result.success);
    println!("Workspace isolation: enabled");

    println!("\n=== All Examples Completed ===");

    Ok(())
}
