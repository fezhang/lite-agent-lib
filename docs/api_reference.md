# lite-agent-lib API Reference

## Core Traits

### AgentExecutor

The central trait that all agents must implement.

```rust
#[async_trait]
pub trait AgentExecutor: Send + Sync {
    /// Unique identifier for this agent type (e.g., "shell", "llm")
    fn agent_type(&self) -> &str;

    /// Spawn a new agent session
    async fn spawn(
        &self,
        config: &AgentConfig,
        input: &str,
    ) -> Result<SpawnedAgent, AgentError>;

    /// Continue an existing session (optional)
    async fn spawn_follow_up(
        &self,
        config: &AgentConfig,
        input: &str,
        session_id: &str,
    ) -> Result<SpawnedAgent, AgentError>;

    /// Normalize agent logs to unified format
    fn normalize_logs(
        &self,
        raw_logs: Arc<LogStore>,
    ) -> BoxStream<'static, NormalizedEntry>;

    /// Check if agent is available
    async fn check_availability(&self) -> AvailabilityStatus;

    /// Declare agent capabilities
    fn capabilities(&self) -> Vec<AgentCapability>;

    /// Optional description
    fn description(&self) -> Option<String>;
}
```

## Configuration Types

### AgentConfig

Configuration for agent execution.

```rust
pub struct AgentConfig {
    pub work_dir: PathBuf,
    pub env: HashMap<String, String>,
    pub workspace: Option<WorkspaceConfig>,
    pub timeout: Option<Duration>,
    pub custom: serde_json::Value,
}

impl AgentConfig {
    pub fn new(work_dir: PathBuf) -> Self;
    pub fn with_env(self, env: HashMap<String, String>) -> Self;
    pub fn add_env(self, key: impl Into<String>, value: impl Into<String>) -> Self;
    pub fn with_workspace(self, workspace: WorkspaceConfig) -> Self;
    pub fn with_timeout(self, timeout: Duration) -> Self;
    pub fn with_custom(self, custom: serde_json::Value) -> Self;
}
```

**Example:**
```rust
let config = AgentConfig::new(PathBuf::from("/tmp"))
    .add_env("API_KEY", "secret")
    .with_timeout(Duration::from_secs(30));
```

### WorkspaceConfig

Workspace isolation configuration.

```rust
pub struct WorkspaceConfig {
    pub work_dir: PathBuf,
    pub isolation_type: IsolationType,
    pub base_branch: String,
}

pub enum IsolationType {
    None,
    GitWorktree { repo_path: PathBuf, branch_prefix: String },
    TempDir,
}
```

## Spawned Agent

### SpawnedAgent

Result of spawning an agent.

```rust
pub struct SpawnedAgent {
    pub child: AsyncGroupChild,
    pub stdin: Option<ChildStdin>,
    pub stdout: Option<ChildStdout>,
    pub stderr: Option<ChildStderr>,
    pub exit_signal: Option<oneshot::Receiver<ExitResult>>,
    pub interrupt_signal: Option<oneshot::Sender<()>>,
    pub log_store: Arc<LogStore>,
}

impl SpawnedAgent {
    pub async fn wait(self) -> std::io::Result<std::process::ExitStatus>;
    pub async fn kill(self) -> std::io::Result<()>;
}
```

### ExitResult

Agent exit status.

```rust
pub enum ExitResult {
    Success,
    Failure(i32),
    Interrupted,
}
```

## Log Types

### NormalizedEntry

Unified log entry format.

```rust
pub struct NormalizedEntry {
    pub timestamp: Option<String>,
    pub entry_type: EntryType,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub agent_type: String,
}
```

### EntryType

Log entry classification.

```rust
pub enum EntryType {
    Input,
    Output,
    Thinking { details: Option<String> },
    Action { action: ActionInfo },
    System,
    Error { error_type: ErrorType },
    Progress { percent: f32, message: Option<String> },
}
```

### LogStore

In-memory log storage.

```rust
pub struct LogStore;

impl LogStore {
    pub fn new() -> Self;
    pub async fn add_entry(&self, entry: NormalizedEntry);
    pub async fn get_entries(&self) -> Vec<NormalizedEntry>;
}
```

## Session Management

### SessionManager

Manage agent sessions.

```rust
pub struct SessionManager;

impl SessionManager {
    pub fn new() -> Self;
    // Additional methods to be implemented in Phase 3
}
```

### SessionState

Session state tracking.

```rust
pub struct SessionState {
    pub id: String,
    pub agent_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub executions: Vec<ExecutionInfo>,
    pub status: SessionStatus,
}

pub enum SessionStatus {
    Active,
    Idle,
    Completed,
    Failed,
}
```

### ExecutionInfo

Individual execution information.

```rust
pub struct ExecutionInfo {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ExecutionStatus,
    pub exit_code: Option<i32>,
    pub input: String,
}

pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}
```

## Workspace Management

### WorkspaceManager

Manage workspace isolation.

```rust
pub struct WorkspaceManager;

impl WorkspaceManager {
    pub fn new(base_dir: PathBuf) -> Self;
    // Additional methods to be implemented in Phase 3
}
```

### WorkspacePath

Workspace location.

```rust
pub enum WorkspacePath {
    Direct(PathBuf),
    Worktree(PathBuf),
    Temp(PathBuf),
}

impl WorkspacePath {
    pub fn path(&self) -> &PathBuf;
}
```

## Error Types

### AgentError

Main error type for agent operations.

```rust
pub enum AgentError {
    SpawnError(String),
    SessionNotFound(String),
    AgentNotAvailable(String),
    Protocol(ProtocolError),
    Workspace(WorkspaceError),
    Io(std::io::Error),
    Timeout,
    Serialization(serde_json::Error),
    Custom(String),
}

pub type AgentResult<T> = Result<T, AgentError>;
```

### WorkspaceError

Workspace-specific errors.

```rust
pub enum WorkspaceError {
    Git(git2::Error),
    Io(std::io::Error),
    Workspace(String),
    InvalidPath(String),
}
```

### ProtocolError

Protocol-specific errors.

```rust
pub enum ProtocolError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
    Protocol(String),
    ConnectionClosed,
}
```

## Agent Capabilities

### AgentCapability

Agent capability flags.

```rust
pub enum AgentCapability {
    SessionContinuation,
    BidirectionalControl,
    WorkspaceIsolation,
    RequiresSetup,
    Custom(String),
}
```

### AvailabilityStatus

Agent availability status.

```rust
pub enum AvailabilityStatus {
    Available,
    InstalledNotAuthenticated,
    NotFound { reason: String },
    RequiresSetup { instructions: String },
}

impl AvailabilityStatus {
    pub fn is_available(&self) -> bool;
}
```

## Example: Implementing a Custom Agent

```rust
use lite_agent_core::*;
use async_trait::async_trait;

struct MyAgent;

#[async_trait]
impl AgentExecutor for MyAgent {
    fn agent_type(&self) -> &str {
        "my-agent"
    }

    async fn spawn(
        &self,
        config: &AgentConfig,
        input: &str,
    ) -> Result<SpawnedAgent, AgentError> {
        // Implementation here
        todo!()
    }

    fn normalize_logs(
        &self,
        raw_logs: Arc<LogStore>,
    ) -> BoxStream<'static, NormalizedEntry> {
        // Convert raw logs to normalized format
        futures_util::stream::empty().boxed()
    }
}
```

## Example: Using an Agent

```rust
use lite_agent_core::*;

#[tokio::main]
async fn main() -> Result<(), AgentError> {
    let agent = MyAgent;

    // Check availability
    if !agent.check_availability().await.is_available() {
        return Err(AgentError::AgentNotAvailable("Agent not ready".into()));
    }

    // Configure
    let config = AgentConfig::new(PathBuf::from("/tmp"))
        .add_env("KEY", "value")
        .with_timeout(Duration::from_secs(60));

    // Spawn
    let spawned = agent.spawn(&config, "Hello, agent!").await?;

    // Wait for completion
    let status = spawned.wait().await?;
    println!("Exit status: {:?}", status);

    Ok(())
}
```

## REST API (Phase 5)

The REST API server will provide HTTP endpoints for Python and other clients.

### Endpoints

**Agent Operations:**
- `POST /api/agents/spawn` - Spawn new agent
- `POST /api/agents/{session_id}/continue` - Continue session
- `GET /api/agents/{session_id}/status` - Get session status

**Log Streaming:**
- `GET /api/logs/{session_id}/stream` - SSE log stream
- `GET /api/logs/{session_id}/history` - Get log history

### Request/Response Examples

**Spawn Request:**
```json
{
  "agent_type": "shell",
  "input": "echo hello",
  "config": {
    "work_dir": "/tmp",
    "env": {"VAR": "value"},
    "timeout": 30000
  }
}
```

**Spawn Response:**
```json
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "execution_id": "660e8400-e29b-41d4-a716-446655440001"
}
```

**Log Entry (SSE):**
```json
{
  "timestamp": "2026-01-06T12:00:00Z",
  "entry_type": {"type": "output"},
  "content": "hello",
  "metadata": null,
  "agent_type": "shell"
}
```

## Status

This API reference will be updated as the library is implemented across Phases 2-6.

**Current Status:**
- ✅ Phase 1: Core Foundation (AgentExecutor, AgentConfig, SpawnedAgent, Errors)
- ✅ Phase 2: Protocol & Logs
- ✅ Phase 3: Session & Workspace
- ✅ Phase 4: High-Level API
- ✅ Phase 5: REST API Server
- ⏳ Phase 6: Python Client

## REST API

For REST API documentation, see [REST API Server](rest_api.md). The REST API provides HTTP endpoints with SSE log streaming for cross-language integration.
