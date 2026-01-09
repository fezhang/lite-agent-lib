# lite-agent-lib Examples

This directory contains examples demonstrating how to use the `lite-agent-core` library with various coding agents.

## Built-in Agent Examples

### Claude Code CLI (`claude_agent.rs`)

Demonstrates usage of the Claude Code CLI agent with various configurations:

```bash
cargo run --example claude_agent
```

**What it demonstrates:**
- Creating Claude agents with default and custom configurations
- Using the builder pattern for configuration
- Enabling plan mode for automatic approval
- Setting custom models
- Debug mode and permission management
- Checking agent availability before execution
- Session continuation support

**Key configurations:**
- Plan mode: Agent plans without executing
- Approvals: Require user approval for tools
- Debug mode: Enable ANTHROPIC debug logging
- Permission modes: Default, Plan, BypassPermissions
- Model selection: claude-sonnet-4, claude-opus-4, etc.

### Cursor Agent (`cursor_agent.rs`)

Demonstrates usage of the Cursor agent with various configurations:

```bash
cargo run --example cursor_agent
```

**What it demonstrates:**
- Creating Cursor agents with default and custom configurations
- Force mode for auto-approving commands
- Model selection (auto, sonnet-4.5, gpt-5, etc.)
- Checking agent availability
- Session continuation support

**Key configurations:**
- Force mode: Auto-approve all commands
- Model selection: auto, sonnet-4.5, gpt-5, opus-4.1, grok, composer-1

## Custom Agent Examples

### Echo Agent (`basic_echo.rs`)

A simple agent that echoes input back. Good for understanding the basic `AgentExecutor` trait implementation.

### Shell Agent (`basic_shell.rs`)

Demonstrates running shell commands as an agent.

### Custom Agent (`custom_agent.rs`)

Shows how to implement a custom agent from scratch.

### Agent Runner (`agent_runner.rs`)

Comprehensive example showing how to use `AgentRunner` to manage agent execution.

## Running the Examples

### Prerequisites

1. **For Claude agent examples:**
   - Install Claude Code CLI: https://claude.ai/download
   - Or ensure Node.js and npx are installed
   - Authenticate: `claude login`

2. **For Cursor agent examples:**
   - Install Cursor: https://cursor.sh
   - Or install cursor-agent CLI
   - Authenticate: `cursor-agent login`

### Running Examples

```bash
# Run Claude agent examples
cargo run --example claude_agent

# Run Cursor agent examples
cargo run --example cursor_agent

# Run basic echo example
cargo run --example basic_echo

# Run custom agent example
cargo run --example custom_agent
```

## Next Steps

1. **Explore the source code**: Each example is heavily commented to explain concepts
2. **Check the documentation**: Run `cargo doc --open` to see full API docs
3. **Build your own agent**: Use the custom agent examples as a template
4. **Integrate into your project**: Copy the patterns shown in the examples

## Example: Using Claude in Your Project

```rust
use lite_agent_core::{AgentConfig, AgentRunner, agents::ClaudeAgent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Claude agent with plan mode
    let agent = ClaudeAgent::builder()
        .with_plan_mode()
        .with_model("claude-sonnet-4")
        .build();

    // Create a runner
    let runner = AgentRunner::new(agent);

    // Configure the execution
    let config = AgentConfig::default();

    // Run the agent
    let result = runner.run(
        "Help me understand this codebase and suggest improvements",
        config
    ).await?;

    Ok(())
}
```

## Example: Using Cursor in Your Project

```rust
use lite_agent_core::{AgentConfig, AgentRunner, agents::CursorAgent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Cursor agent with force mode
    let agent = CursorAgent::builder()
        .with_force()
        .with_model("sonnet-4.5")
        .build();

    // Create a runner
    let runner = AgentRunner::new(agent);

    // Configure the execution
    let config = AgentConfig::default();

    // Run the agent
    let result = runner.run(
        "Refactor this function to be more readable",
        config
    ).await?;

    Ok(())
}
```

## Troubleshooting

### "Agent not found" errors

Make sure the respective CLI tool is installed and in your PATH:
- Claude: `claude --version` or `npx --version`
- Cursor: `cursor-agent --version`

### Authentication errors

Make sure you're authenticated:
- Claude: Run `claude login`
- Cursor: Run `cursor-agent login` or set `CURSOR_API_KEY`

### Permission errors

Some agents require permissions to read/write files. Make sure you're running in an appropriate directory with the right permissions.
