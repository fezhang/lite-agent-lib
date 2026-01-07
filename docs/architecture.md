# lite-agent-lib Architecture

## Overview

lite-agent-lib is a lightweight, async-first Rust library for managing different kinds of agents. It provides a generic abstraction layer over agent execution with support for protocol handling, log normalization, session management, and workspace isolation.

## Design Philosophy

### Generic Agent Support

Unlike domain-specific frameworks, lite-agent-lib supports **any type of agent**:
- Shell command agents
- LLM-based agents (Claude, GPT, etc.)
- Coding agents (Claude Code, Cursor, etc.)
- Data analysis agents
- Custom agent types

The core `AgentExecutor` trait is intentionally generic to support this flexibility.

### Async-First

All APIs are fully asynchronous using Tokio:
- Non-blocking I/O operations
- Concurrent agent execution
- Efficient resource utilization
- Stream-based log processing

### Explicit Dependencies

No hidden dependencies or magic:
- All dependencies declared in `Cargo.toml`
- No global state (except necessary locks)
- Explicit configuration via `AgentConfig`
- Clear ownership and lifecycle management

### Separation of Concerns

The library is organized into focused modules:
- `agent` - Core executor trait and types
- `protocol` - Communication protocol handling
- `logs` - Log normalization and storage
- `session` - Session and execution tracking
- `workspace` - Workspace isolation (git worktrees)

## Core Components

### 1. AgentExecutor Trait

The central abstraction for all agents:

```rust
#[async_trait]
pub trait AgentExecutor: Send + Sync {
    fn agent_type(&self) -> &str;
    async fn spawn(&self, config: &AgentConfig, input: &str) -> Result<SpawnedAgent, AgentError>;
    async fn spawn_follow_up(&self, config: &AgentConfig, input: &str, session_id: &str) -> Result<SpawnedAgent, AgentError>;
    fn normalize_logs(&self, raw_logs: Arc<LogStore>) -> BoxStream<'static, NormalizedEntry>;
    async fn check_availability(&self) -> AvailabilityStatus;
    fn capabilities(&self) -> Vec<AgentCapability>;
}
```

**Key Design Decisions:**
- Async methods for I/O-bound operations
- Generic over agent type (not coded to specific agents)
- Log normalization returns a stream (not blocking)
- Availability checking separate from spawning
- Capabilities declared upfront

### 2. Configuration System

`AgentConfig` provides flexible configuration:

```rust
pub struct AgentConfig {
    pub work_dir: PathBuf,
    pub env: HashMap<String, String>,
    pub workspace: Option<WorkspaceConfig>,
    pub timeout: Option<Duration>,
    pub custom: serde_json::Value,
}
```

**Features:**
- Builder pattern for ergonomic construction
- Optional workspace isolation
- Agent-specific custom config via JSON
- Environment variable injection

### 3. Process Management

`SpawnedAgent` encapsulates spawned processes:

```rust
pub struct SpawnedAgent {
    pub child: AsyncGroupChild,              // Process group handle
    pub stdin: Option<ChildStdin>,           // Stdin for input
    pub stdout: Option<ChildStdout>,         // Stdout for output
    pub stderr: Option<ChildStderr>,         // Stderr for errors
    pub exit_signal: Option<oneshot::Receiver<ExitResult>>,  // Exit notification
    pub interrupt_signal: Option<oneshot::Sender<()>>,       // Interrupt request
    pub log_store: Arc<LogStore>,            // Log storage
}
```

**Key Features:**
- Process group management (kill entire tree)
- Bidirectional signaling (exit, interrupt)
- Stdio handle management
- Associated log store

### 4. Log Normalization

Unified log format across all agents:

```rust
pub struct NormalizedEntry {
    pub timestamp: Option<String>,
    pub entry_type: EntryType,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub agent_type: String,
}

pub enum EntryType {
    Input, Output, Thinking, Action, System, Error, Progress
}
```

**Benefits:**
- Consistent rendering regardless of agent
- Structured metadata for rich display
- Stream-based processing (memory efficient)
- Agent-specific parsers via `normalize_logs()`

### 5. Session Management

Track agent sessions and execution history:

```rust
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    log_stores: Arc<RwLock<HashMap<String, Arc<LogStore>>>>,
}

pub struct SessionState {
    pub id: String,
    pub agent_type: String,
    pub created_at: DateTime<Utc>,
    pub executions: Vec<ExecutionInfo>,
    pub status: SessionStatus,
}
```

**Features:**
- In-memory state (no database dependency)
- Multiple executions per session
- Conversation continuity support
- Associated log storage

### 6. Workspace Isolation

Git worktree-based isolation for parallel agents:

```rust
pub struct WorkspaceManager {
    base_dir: PathBuf,
    locks: Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
}

pub enum IsolationType {
    None,                          // Direct execution
    GitWorktree { ... },          // Git worktree isolation
    TempDir,                      // Temporary directory
}
```

**Features:**
- Per-session isolation
- Global locks to prevent race conditions
- Automatic cleanup
- Optional (not all agents need isolation)

## Data Flow

### Agent Spawning Flow

```
User Code
    ↓
AgentExecutor::spawn(config, input)
    ↓
Create SpawnedAgent
    ├→ Spawn process (AsyncGroupChild)
    ├→ Setup stdio handles
    ├→ Create exit/interrupt channels
    └→ Create LogStore
    ↓
Return SpawnedAgent to user
    ↓
User monitors logs, waits for exit, or interrupts
```

### Log Normalization Flow

```
Agent Process (stdout/stderr)
    ↓
Raw bytes collected in LogStore
    ↓
AgentExecutor::normalize_logs(raw_logs)
    ↓
Agent-specific parsing logic
    ↓
Stream<NormalizedEntry>
    ↓
Consumer (UI, storage, forwarding)
```

### Workspace Isolation Flow

```
SessionManager::create_session()
    ↓
WorkspaceManager::create_workspace(config)
    ↓
    [IsolationType::GitWorktree]
    ├→ Acquire lock for workspace path
    ├→ Create git worktree + branch
    ├→ Return WorkspacePath::Worktree(path)
    ↓
AgentExecutor::spawn(config with workspace)
    ↓
Agent executes in isolated worktree
    ↓
WorkspaceManager::cleanup_workspace()
    ├→ Acquire lock
    ├→ Remove worktree
    └→ Cleanup git metadata
```

## Error Handling Strategy

### Error Types

```rust
pub enum AgentError {
    SpawnError(String),
    SessionNotFound(String),
    AgentNotAvailable(String),
    Protocol(ProtocolError),
    Workspace(WorkspaceError),
    Io(std::io::Error),
    Timeout,
    Custom(String),
}
```

**Principles:**
- Use `thiserror` for domain errors
- Transparent wrapping via `#[from]`
- Return `Result<T, AgentError>` for all fallible ops
- Specific error types for common cases
- Custom variant for agent-specific errors

## Concurrency Model

### Thread Safety

- `AgentExecutor` trait requires `Send + Sync`
- Shared state uses `Arc<RwLock<T>>` or `Arc<Mutex<T>>`
- Tokio async runtime for concurrency
- Stream-based processing for logs

### Locking Strategy

**WorkspaceManager:**
- Global lock map per workspace path
- Prevents concurrent worktree creation/deletion
- Uses `tokio::sync::Mutex` for async locks
- Locks released via RAII

**SessionManager:**
- Read-write locks for session state
- Multiple concurrent reads, exclusive writes
- Log stores accessed via `Arc` (cheap clones)

## REST API Architecture (Phase 5)

The REST API server provides HTTP/SSE interface:

```
HTTP Client
    ↓
Axum Routes
    ├→ POST /api/agents/spawn
    ├→ POST /api/agents/{id}/continue
    ├→ GET  /api/agents/{id}/status
    └→ GET  /api/logs/{id}/stream (SSE)
    ↓
AppState (contains SessionManager, WorkspaceManager)
    ↓
lite-agent-core library
```

**Features:**
- Server-Sent Events (SSE) for log streaming
- CORS support for web clients
- JSON request/response
- Stateful session management

## Design Patterns

### 1. Trait-Based Polymorphism

Different agents implement the same trait, enabling:
- Uniform API
- Runtime agent selection
- Testability (mock agents)

### 2. Builder Pattern

`AgentConfig` uses builder pattern:
```rust
let config = AgentConfig::new(work_dir)
    .add_env("KEY", "value")
    .with_timeout(Duration::from_secs(30))
    .with_workspace(workspace_config);
```

### 3. Stream-Based Processing

Logs processed as streams, not batches:
- Memory efficient
- Real-time rendering
- Backpressure support

### 4. Shared Ownership (Arc)

Shared state uses `Arc`:
- Multiple readers
- Thread-safe
- Cheap clones

### 5. Channel-Based Signaling

Process lifecycle uses channels:
- `oneshot::Receiver<ExitResult>` for exit
- `oneshot::Sender<()>` for interrupt
- Type-safe communication

## Future Extensions

Potential areas for extension:

1. **Persistence Layer** - Optional database backend for session storage
2. **Distributed Execution** - Agent execution across multiple machines
3. **Observability** - Tracing, metrics, structured logging
4. **Plugin System** - Dynamic agent loading
5. **Resource Limits** - CPU, memory, disk quotas
6. **Advanced Protocols** - gRPC, custom binary protocols
7. **Caching** - Result caching for deterministic agents
8. **Scheduling** - Task queuing and prioritization

## Performance Considerations

### Memory

- In-memory log storage (bounded by session lifetime)
- Stream processing (constant memory per stream)
- Arc-based sharing (minimal cloning)

### Latency

- Async I/O (no blocking)
- Stream-based logs (incremental processing)
- Minimal serialization overhead

### Throughput

- Concurrent agent execution (Tokio)
- Lock-free where possible
- Efficient worktree operations

## Testing Strategy

### Unit Tests

- Mock `AgentExecutor` implementations
- Test configuration builders
- Verify error handling
- Test serialization

### Integration Tests

- End-to-end agent spawning
- Workspace isolation
- Session management
- Protocol handling

### Property-Based Tests (Future)

- Random input generation
- Invariant checking
- Fuzz testing

## Conclusion

lite-agent-lib provides a clean, generic abstraction for agent management with minimal dependencies and maximum flexibility. By incorporating proven patterns from production multi-agent systems and simplifying/generalizing them, it offers a solid foundation for building multi-agent systems in Rust.

The library prioritizes:
- **Simplicity**: Easy to understand and use
- **Flexibility**: Support any agent type
- **Performance**: Async-first, efficient resource usage
- **Reliability**: Type-safe, explicit error handling
- **Composability**: Building blocks for custom solutions

Whether you're building a simple shell agent wrapper or a complex multi-agent orchestration system, lite-agent-lib provides the core abstractions you need without forcing architectural decisions.
