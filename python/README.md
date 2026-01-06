# Lite Agent Python Client

Python client for [lite-agent-lib](../README.md) - A lightweight, async-first library for managing different kinds of agents.

## Features

- **Async-First**: Built on `aiohttp` for async/await operations
- **Type Safety**: Pydantic models for request/response validation
- **SSE Streaming**: Real-time log streaming via Server-Sent Events
- **Context Manager Support**: Automatic session management
- **Comprehensive Error Handling**: Detailed error messages and exceptions
- **Session Management**: Create, continue, and delete sessions
- **Full API Coverage**: All endpoints supported

## Installation

```bash
# From PyPI (when published)
pip install lite-agent-client

# Or from source
cd python
pip install -e .
```

### Development Installation

```bash
cd python
pip install -e ".[dev]"
```

This installs additional development tools:
- `pytest` - Testing framework
- `pytest-asyncio` - Async test support
- `pytest-cov` - Coverage reporting
- `black` - Code formatting
- `ruff` - Linting
- `mypy` - Type checking

## Quick Start

```python
import asyncio
from lite_agent_client import LiteAgentClient, SpawnRequest

async def main():
    # Use as context manager (recommended)
    async with LiteAgentClient("http://localhost:3000") as client:
        # Spawn an agent
        response = await client.spawn_agent(SpawnRequest(
            agent_type="shell",
            input="echo 'Hello, world!'"
        ))

        # Stream logs
        async for log in client.stream_logs(response.session_id):
            print(f"[{log.level}] {log.content}")

asyncio.run(main())
```

## Usage Examples

### Basic Agent Spawn

```python
from lite_agent_client import LiteAgentClient, SpawnRequest

async with LiteAgentClient("http://localhost:3000") as client:
    request = SpawnRequest(
        agent_type="echo",
        input="Hello, echo agent!"
    )

    response = await client.spawn_agent(request)
    print(f"Session ID: {response.session_id}")
    print(f"Execution ID: {response.execution_id}")
```

### Streaming Logs

```python
from lite_agent_client import LiteAgentClient, LogEntryType

async with LiteAgentClient("http://localhost:3000") as client:
    response = await client.spawn_agent(SpawnRequest(
        agent_type="shell",
        input="ls -la"
    ))

    async for log in client.stream_logs(response.session_id):
        if log.level == LogEntryType.STDOUT:
            print(log.content, end="")
        elif log.level == LogEntryType.STDERR:
            print(f"[stderr] {log.content}", file=sys.stderr)
```

### Convenience Method: Spawn and Stream

```python
async with LiteAgentClient("http://localhost:3000") as client:
    request = SpawnRequest(agent_type="echo", input="test")

    # Combine spawn and stream in one operation
    async for log in client.spawn_and_stream(request):
        print(f"[{log.level}] {log.content}")
```

### Custom Configuration

```python
from pathlib import Path
from lite_agent_client import LiteAgentClient, SpawnRequest, AgentConfigOptions

async with LiteAgentClient("http://localhost:3000") as client:
    config = AgentConfigOptions(
        work_dir=Path("/tmp"),
        env={"DEBUG": "1", "VERBOSE": "true"},
        timeout_secs=60,
        custom={"retry_count": 3}
    )

    request = SpawnRequest(
        agent_type="shell",
        input="python script.py",
        config=config
    )

    response = await client.spawn_agent(request)
```

### Session Management

```python
async with LiteAgentClient("http://localhost:3000") as client:
    # Create a new session
    response = await client.spawn_agent(SpawnRequest(
        agent_type="shell",
        input="echo 'First command'"
    ))
    session_id = response.session_id

    # Continue the session
    await client.spawn_agent(SpawnRequest(
        agent_type="shell",
        input="echo 'Second command'",
        session_id=session_id
    ))

    # Get session status
    status = await client.get_session_status(session_id)
    print(f"Status: {status.status}")
    print(f"Executions: {status.execution_count}")

    # List all sessions
    all_sessions = await client.list_sessions()
    for session in all_sessions.sessions:
        print(f"{session.session_id}: {session.status}")

    # Delete session when done
    await client.delete_session(session_id)
```

### List Available Agents

```python
async with LiteAgentClient("http://localhost:3000") as client:
    agents_response = await client.list_agents()

    print(f"Available agents: {agents_response.total}")
    for agent in agents_response.agents:
        print(f"  - {agent.agent_type}")
        if agent.description:
            print(f"    {agent.description}")
        if agent.capabilities:
            print(f"    Capabilities: {', '.join(agent.capabilities)}")
```

### Health Check

```python
async with LiteAgentClient("http://localhost:3000") as client:
    health = await client.health_check()
    print(f"Server status: {health.status}")
    print(f"Timestamp: {health.timestamp}")
```

### Error Handling

```python
from lite_agent_client import LiteAgentClientError

async with LiteAgentClient("http://localhost:3000") as client:
    try:
        await client.get_session_status("nonexistent-session")
    except LiteAgentClientError as e:
        print(f"Error: {e.message}")
        if e.details:
            print(f"Details: {e.details}")
```

## API Reference

### LiteAgentClient

Main client class for interacting with the lite-agent server.

#### Constructor

```python
LiteAgentClient(
    base_url: str,
    timeout: float = 300.0,
    verify_ssl: bool = True
)
```

- `base_url`: Base URL of the lite-agent server (e.g., "http://localhost:3000")
- `timeout`: Request timeout in seconds (default: 300)
- `verify_ssl`: Whether to verify SSL certificates (default: True)

#### Methods

##### spawn_agent

```python
async def spawn_agent(request: SpawnRequest) -> SpawnResponse
```

Spawn a new agent or continue an existing session.

##### get_session_status

```python
async def get_session_status(session_id: str) -> SessionStatusResponse
```

Get status of a specific session.

##### list_sessions

```python
async def list_sessions() -> ListSessionsResponse
```

List all sessions.

##### delete_session

```python
async def delete_session(session_id: str) -> None
```

Delete a session.

##### list_agents

```python
async def list_agents() -> ListAgentsResponse
```

List available agent types.

##### stream_logs

```python
async def stream_logs(session_id: str) -> AsyncIterator[LogEntry]
```

Stream logs for a session via Server-Sent Events (SSE).

##### health_check

```python
async def health_check() -> HealthResponse
```

Check server health.

##### spawn_and_stream

```python
async def spawn_and_stream(request: SpawnRequest) -> AsyncIterator[LogEntry]
```

Convenience method combining spawn_agent() and stream_logs().

## Data Models

### SpawnRequest

Request to spawn an agent.

- `agent_type` (str): Type of agent to spawn
- `input` (str): Input/prompt for the agent
- `session_id` (Optional[str]): Optional session ID to continue
- `config` (AgentConfigOptions): Agent configuration

### AgentConfigOptions

Configuration for agent execution.

- `work_dir` (Optional[Path]): Working directory
- `env` (Dict[str, str]): Environment variables
- `timeout_secs` (Optional[int]): Timeout in seconds
- `custom` (Optional[Dict[str, Any]]): Custom agent-specific config

### LogEntry

A log entry from an agent.

- `timestamp` (str): Timestamp of the log entry
- `level` (str): Log level/type (stdout, stderr, info, warning, error, debug)
- `content` (str): Log content

## Development

### Running Tests

```bash
cd python
pytest
```

With coverage:

```bash
pytest --cov=_lite_agent_client --cov-report=html
```

### Code Formatting

```bash
# Format code
black lite_agent_client tests examples

# Check linting
ruff check lite_agent_client tests examples

# Type checking
mypy lite_agent_client
```

### Examples

Run examples from the `examples/` directory:

```bash
cd python

# Basic echo agent
python examples/basic_echo.py

# Basic shell agent
python examples/basic_shell.py

# Advanced features
python examples/custom_agent.py
```

**Note**: Make sure the lite-agent server is running before executing examples.

## Requirements

- Python 3.8+
- aiohttp >= 3.8.0
- pydantic >= 2.0.0
- typing-extensions >= 4.5.0

## License

Apache 2.0

## Contributing

Contributions welcome! Please open an issue or PR at the [main repository](https://github.com/your-repo/lite-agent-lib).
