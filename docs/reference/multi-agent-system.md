# Multi-Agent System

## Supported Agents

Defined in `crates/executors/src/executors/mod.rs`:

```rust
enum CodingAgent {
    ClaudeCode,
    Amp,
    Gemini,
    Codex,
    Opencode,
    CursorAgent,
    QwenCode,
    Copilot,
    Droid
}
```

## Executor Pattern

Each agent implements `StandardCodingAgentExecutor` trait:

- **spawn()** - Start new agent session
- **spawn_follow_up()** - Continue existing session
- **normalize_logs()** - Parse agent-specific output
- **get_availability_info()** - Check if agent is installed/authenticated

## Key Components

### 1. Agent Profiles & Variants
- **Location:** `crates/executors/src/profile.rs`
- **Config:** `default_profiles.json`
- Supports multiple variants per agent (e.g., ClaudeCode has "PLAN", "ROUTER" variants)
- Per-agent configuration options

### 2. Workspace Isolation
- Each task attempt gets its own **git worktree**
- Enables parallel execution without conflicts
- Managed by `worktree_manager.rs`

### 3. Session Management
- **Sessions** track which agent/executor is running
- **ExecutionProcesses** track individual process runs
- **Queued Messages** allow follow-up prompts

### 4. Container Service
- **Location:** `crates/services/src/services/container.rs`
- Orchestrates:
  - Creating isolated workspaces
  - Spawning agent processes with proper environment
  - Tracking execution status and logs
  - Managing process lifecycle
  - Handling parallel setup scripts

### 5. MCP Configuration
- **Location:** `crates/executors/src/mcp_config.rs`
- Centralizes MCP server configs across all agents
- Each agent has specific MCP config paths and JSON structures
- Supports agent-specific preconfigured MCP servers

## Execution Flow

```
Task Request
    │
    ▼
Create Workspace (git worktree)
    │
    ▼
Select Agent + Profile
    │
    ▼
Spawn Agent Process
    │
    ▼
Stream Logs (SSE)
    │
    ▼
Track Changes (git diff)
    │
    ▼
Complete/Review
```

## Agent-Specific Implementations

Each agent executor in `crates/executors/src/executors/`:
- **claude.rs** - Claude Code with protocol client
- **codex.rs** - OpenAI Codex
- **cursor.rs**, **amp.rs**, **gemini.rs**, etc.

All implement standardized interface for unified management.
