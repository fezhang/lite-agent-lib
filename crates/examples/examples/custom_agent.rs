//! Custom agent implementation example
//!
//! Demonstrates how to implement your own agent by using the AgentExecutor trait.

use async_trait::async_trait;
use command_group::CommandGroup;
use futures_util::stream::{self, BoxStream, StreamExt};
use lite_agent_core::{
    AgentCapability, AgentConfig, AgentError, AgentExecutor, AgentRunner, AvailabilityStatus,
    EntryType, LogStore, NormalizedEntry, SpawnedAgent,
};
use std::process::Stdio;
use std::sync::Arc;

/// Custom greeting agent
///
/// This agent demonstrates how to implement the AgentExecutor trait.
#[derive(Debug, Clone)]
struct GreetingAgent {
    greeting: String,
}

impl GreetingAgent {
    fn new(greeting: String) -> Self {
        Self { greeting }
    }
}

impl Default for GreetingAgent {
    fn default() -> Self {
        Self::new("Hello".to_string())
    }
}

#[async_trait]
impl AgentExecutor for GreetingAgent {
    fn agent_type(&self) -> &str {
        "greeting"
    }

    async fn spawn(
        &self,
        config: &AgentConfig,
        input: &str,
    ) -> Result<SpawnedAgent, AgentError> {
        // Create a custom command that greets the user
        let message = format!("{} {}!", self.greeting, input);

        let mut cmd = if cfg!(windows) {
            std::process::Command::new("cmd")
                .args(["/c", "echo", &message])
        } else {
            std::process::Command::new("echo")
                .arg(&message)
        };

        cmd.current_dir(&config.work_dir);

        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd.group_spawn().map_err(|e| {
            AgentError::SpawnError(format!("Failed to spawn greeting agent: {}", e))
        })?;

        let spawned = SpawnedAgent {
            child,
            stdin: child.stdin(),
            stdout: child.stdout(),
            stderr: child.stderr(),
            exit_signal: None,
            interrupt_signal: None,
            log_store: Arc::new(LogStore::new()),
        };

        Ok(spawned)
    }

    fn normalize_logs(&self, _raw_logs: Arc<LogStore>) -> BoxStream<'static, NormalizedEntry> {
        let entry = NormalizedEntry {
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            entry_type: EntryType::Output,
            content: format!("Greeting agent completed with greeting: {}", self.greeting),
            metadata: None,
            agent_type: self.agent_type().to_string(),
        };

        stream::iter(vec![entry]).boxed()
    }

    async fn check_availability(&self) -> AvailabilityStatus {
        AvailabilityStatus::Available
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::BidirectionalControl,
            AgentCapability::WorkspaceIsolation,
        ]
    }

    fn description(&self) -> Option<String> {
        Some(format!("A greeting agent that says '{}'", self.greeting))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("=== Custom Agent Example ===\n");

    // Create custom agent
    let agent = GreetingAgent::new("Welcome".to_string());

    println!("Agent Type: {}", agent.agent_type());
    println!("Description: {:?}", agent.description());
    println!("Capabilities: {:?}", agent.capabilities());

    // Create runner
    let runner = AgentRunner::new(agent);

    // Run the agent
    let config = AgentConfig::new(std::path::PathBuf::from("."));
    let result = runner.run("User", &config).await?;

    println!("\n=== Result ===");
    println!("Success: {}", result.success);
    println!("Logs:");
    for log in &result.logs {
        println!("  - {:?}", log);
    }

    Ok(())
}
