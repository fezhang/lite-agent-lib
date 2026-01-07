//! Claude Code CLI example with streaming responses
//!
//! This example demonstrates:
//! - Taking user input from stdin
//! - Launching Claude Code CLI with ANTHROPIC debug mode enabled
//! - Streaming responses in real-time until the job is done
//!
//! Usage:
//!   cargo run --bin claude_code_cli
//!
//! Then enter your message when prompted.

use futures_util::StreamExt;
use lite_agent_core::{AgentConfig, AgentRunner, AgentExecutor};
use lite_agent_examples::ClaudeCodeAgent;
use std::path::PathBuf;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for debug output
    tracing_subscriber::fmt::init();

    println!("=== Claude Code CLI Example ===");
    println!("This example launches Claude Code CLI with debug mode enabled.");
    println!("You will see streamed responses in real-time.");
    println!();

    // Get user input
    println!("Enter your message for Claude Code (or press Enter for a default message):");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    let input = input.trim();
    let input = if input.is_empty() {
        "Hello! Please explain what you can do."
    } else {
        input
    };

    println!();
    println!("Input: {}", input);
    println!("--- Starting Claude Code CLI with debug mode ---");
    println!();

    // Create the Claude Code CLI agent with debug mode enabled
    let agent = ClaudeCodeAgent::new().with_debug_mode();

    // Create the runner
    let runner = AgentRunner::new(agent);

    // Configure the agent
    let config = AgentConfig::new(PathBuf::from("."))
        .with_timeout(Duration::from_secs(300)) // 5 minutes timeout
        .add_env("ANTHROPIC_LOG", "debug")
        .add_env("RUST_LOG", "debug");

    println!("Agent description: {:?}", runner.executor().description());
    println!("Checking availability...");

    let availability = runner.executor().check_availability().await;
    match availability {
        lite_agent_core::AvailabilityStatus::Available => {
            println!("Claude Code CLI is available!")
        }
        lite_agent_core::AvailabilityStatus::NotFound { reason } => {
            eprintln!("Claude Code CLI not found: {}", reason);
            eprintln!("Please ensure Node.js and npx are installed.");
            return Err(format!("Claude Code CLI not available: {}", reason).into());
        }
        lite_agent_core::AvailabilityStatus::RequiresSetup { instructions } => {
            eprintln!("Claude Code CLI requires setup: {}", instructions);
            return Err(format!("Claude Code CLI requires setup: {}", instructions).into());
        }
        _ => {
            eprintln!("Claude Code CLI is not available for an unknown reason.");
            return Err("Claude Code CLI not available".into());
        }
    }

    println!();
    println!("=== Running agent with streaming ===");
    println!("(You will see Claude's responses streamed in real-time)");
    println!();

    // Run the agent with streaming
    let (spawned, mut log_stream) = runner.run_streamed(input, &config).await?;

    // Process logs in real-time as they arrive
    tokio::spawn(async move {
        while let Some(entry) = log_stream.next().await {
            match entry.entry_type {
                lite_agent_core::EntryType::Output => {
                    // Print Claude's responses
                    print!("{}", entry.content);
                    // Use stdout().flush() to ensure immediate output
                    use std::io::Write;
                    std::io::stdout().flush().ok();
                }
                lite_agent_core::EntryType::Error { .. } => {
                    // Print debug/error logs to stderr
                    eprintln!("[DEBUG] {}", entry.content);
                }
                _ => {
                    // Other entry types
                    println!("[{:?}] {}", entry.entry_type, entry.content);
                }
            }
        }
    });

    // Wait for the agent to complete
    println!();
    println!("--- Waiting for Claude Code CLI to complete ---");
    println!();

    let exit_result = spawned.wait().await?;

    println!();
    println!("=== Execution completed ===");
    println!("Exit result: {:?}", exit_result);

    // Check if the process exited successfully
    if exit_result.success() {
        println!("Claude Code CLI exited successfully!");
    } else {
        println!("Claude Code CLI exited with an error.");
        if let Some(code) = exit_result.code() {
            println!("Exit code: {}", code);
        }
    }

    println!();
    println!("=== Example finished ===");

    Ok(())
}
