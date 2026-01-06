# REST API Server Documentation

The `lite-agent-server` crate provides a REST API server with Server-Sent Events (SSE) log streaming for `lite-agent-lib`.

## Overview

The REST API server exposes HTTP endpoints for:
- Spawning and managing agent sessions
- Streaming logs in real-time via SSE
- Listing available agents
- Querying session status

## API Endpoints

### Base URL
All endpoints are prefixed with `/api`.

### Agent Operations

#### POST `/api/agents/spawn`
Spawn a new agent session or continue an existing one.

**Request Body:**
```json
{
  "agent_type": "shell",
  "input": "echo hello world",
  "session_id": null,
  "config": {
    "work_dir": "/path/to/dir",
    "timeout_secs": 30,
    "env": {
      "KEY": "value"
    }
  }
}
```

**Response (201 Created):**
```json
{
  "session_id": "uuid-v4",
  "execution_id": "uuid-v4",
  "agent_type": "shell",
  "status": "started"
}
```

#### GET `/api/agents`
List all available agents.

**Response (200 OK):**
```json
{
  "agents": [
    {
      "agent_type": "shell",
      "description": "Shell command executor",
      "capabilities": ["BidirectionalControl", "WorkspaceIsolation"],
      "availability": "Available"
    }
  ],
  "total": 1
}
```

### Session Operations

#### GET `/api/sessions`
List all sessions.

**Response (200 OK):**
```json
{
  "sessions": [
    {
      "session_id": "uuid-v4",
      "agent_type": "shell",
      "status": "active",
      "execution_count": 1,
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 1
}
```

#### GET `/api/sessions/{session_id}`
Get session status.

**Response (200 OK):**
```json
{
  "session_id": "uuid-v4",
  "agent_type": "shell",
  "status": "active",
  "execution_count": 1,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### DELETE `/api/sessions/{session_id}`
Delete a session.

**Response (204 No Content)**

### Log Streaming

#### GET `/api/logs/{session_id}/stream`
Stream logs for a session via Server-Sent Events (SSE).

**Response:**
```
Content-Type: text/event-stream

data: {"type":"stream_started","session_id":"uuid-v4"}

data: {"timestamp":"2024-01-01T00:00:00Z","entry_type":{"type":"output"},"content":"hello world","agent_type":"shell"}

data: {"type":"stream_ended"}
```

## Usage Example

### Starting the Server

```rust
use lite_agent_server::{ServerState, create_router};
use lite_agent_core::{SessionManager, WorkspaceManager};
use lite_agent_examples::{EchoAgent, ShellAgent};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Create server state
    let session_manager = Arc::new(SessionManager::new());
    let workspace_manager = Arc::new(WorkspaceManager::new(".".into()));
    let state = ServerState::new(session_manager, workspace_manager);

    // Register agents
    state.register_agent(Arc::new(EchoAgent::new())).await;
    state.register_agent(Arc::new(ShellAgent::new())).await;

    // Create router
    let app = create_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### Making Requests

#### Spawn Agent

```bash
curl -X POST http://localhost:3000/api/agents/spawn \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "shell",
    "input": "echo hello world"
  }'
```

#### Stream Logs

```bash
curl -N http://localhost:3000/api/logs/{session_id}/stream
```

#### List Sessions

```bash
curl http://localhost:3000/api/sessions
```

## Architecture

The REST API server is built with:
- **Axum**: Web framework for async HTTP
- **Server-Sent Events (SSE)**: Real-time log streaming
- **Session Manager**: Manages agent session state
- **Agent Registry**: Dynamic agent registration

### Request Flow

```
HTTP Request
    ↓
Axum Router
    ↓
Handler Function
    ↓
SessionManager / AgentRegistry
    ↓
AgentExecutor (agent-specific logic)
    ↓
Response (JSON or SSE stream)
```

### SSE Streaming

Log entries are streamed in real-time using Server-Sent Events:

1. Client connects to `/api/logs/{session_id}/stream`
2. Server subscribes to the session's log store
3. New log entries are broadcast as SSE events
4. Stream closes when session ends

## Error Handling

All endpoints return consistent error responses:

```json
{
  "error": "Error message",
  "details": "Optional additional details"
}
```

Common HTTP status codes:
- `200 OK`: Successful GET request
- `201 Created`: Agent spawned successfully
- `204 No Content`: Successful DELETE
- `400 Bad Request`: Invalid request body
- `404 Not Found`: Session or agent not found
- `500 Internal Server Error`: Server error

## Testing

The server includes comprehensive unit tests:

```bash
cargo test -p lite-agent-server
```

All 9 tests pass, covering:
- API serialization/deserialization
- Agent registry
- Session management
- Router creation
- Handler functions
- Health checks

## Security Considerations

For production use, consider adding:
- Authentication middleware
- Rate limiting
- CORS configuration
- Request validation
- TLS/HTTPS support

## Performance

- Async request handling with Tokio
- Efficient SSE streaming without buffering
- In-memory session management (fast access)
- Minimal allocations during streaming

## Future Enhancements

Potential improvements:
- WebSocket support as alternative to SSE
- Request batching for multiple agent spawns
- Persistent session storage (Redis, database)
- Metrics and observability endpoints
- Webhook support for session completion
