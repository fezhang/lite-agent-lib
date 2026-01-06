# lite-agent-lib

A lightweight, async-first Rust library for managing different kinds of agents with support for protocol handling, log normalization, session management, and workspace isolation.

## Features

- **Generic Agent Abstraction**: Support any agent type (shell, LLM, coding agents, etc.)
- **Async-First Design**: All APIs fully asynchronous using Tokio
- **Protocol Handling**: Bidirectional stdin/stdout communication with JSON streaming
- **Log Normalization**: Unified log format across different agent types
- **Session Management**: Track sessions and support conversation continuity
- **Workspace Isolation**: Git worktree-based isolation for parallel agent execution
- **REST API Server**: HTTP/SSE interface for Python and other language clients
- **No Hidden Dependencies**: All dependencies explicit and documented

## Architecture

The library is organized into several crates:

- **`lite-agent-core`**: Core library with agent abstractions, protocols, logs, sessions, and workspace management
- **`lite-agent-server`**: REST API server with SSE log streaming
- **`lite-agent-examples`**: Example agent implementations (echo, shell)

## Quick Start

### Rust

```rust
use lite_agent_core::{AgentExecutor, AgentConfig, AgentRunner};

#[tokio::main]
async fn main() {
    // Implement your custom agent
    let executor = MyCustomAgent::new();
    let runner = AgentRunner::new(executor);

    // Run agent
    let config = AgentConfig::default();
    let result = runner.run("your input", config).await.unwrap();
    println!("Result: {}", result);
}
```

### Python

```python
from lite_agent_client import LiteAgentClient, SpawnRequest

async def main():
    client = LiteAgentClient("http://localhost:3000")

    # Spawn agent
    response = await client.spawn_agent(SpawnRequest(
        agent_type="shell",
        input="echo hello",
        config={}
    ))

    # Stream logs
    async for log in client.stream_logs(response.session_id):
        print(f"[{log.entry_type}] {log.content}")
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
lite-agent-core = "0.1.0"
```

## Documentation

- [Architecture Overview](docs/architecture.md) - Design philosophy and core components
- [API Reference](docs/api_reference.md) - Complete API documentation
- [Workspace & Agent Configuration](docs/workspace_and_agent_configuration.md) - Understanding workspaces and configuration
- [Reference Documentation](docs/reference/) - Production design patterns that informed this library
- [Examples](examples/) - Usage examples in Rust and Python

## Development Status

🚧 **Under Active Development** - This library is currently being built following a 6-phase implementation plan.

Current status:
- ✅ Phase 1: Core Foundation - In Progress
- ⏳ Phase 2: Protocol & Logs
- ⏳ Phase 3: Session & Workspace
- ⏳ Phase 4: High-Level API & Examples
- ⏳ Phase 5: REST API Server
- ⏳ Phase 6: Python Client

## License

Apache 2.0

## Contributing

Contributions welcome! Please open an issue or PR.
