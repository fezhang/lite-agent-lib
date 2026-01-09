# Agent Spec Implementation Plan

## Overview
Add support for Cursor and Claude Code agents to the lite-agent-lib library, providing:
- Easy-to-use agent configurations
- High-level protocol abstractions
- Bidirectional control protocol support (for Claude)
- Session continuation support
- Log normalization for both agents

## Architecture

### Module Structure
```
crates/core/src/
├── agents/
│   ├── mod.rs                    # Public exports
│   ├── claude/
│   │   ├── mod.rs                # Claude agent executor
│   │   ├── config.rs             # Claude-specific configuration
│   │   ├── protocol.rs           # Bidirectional control protocol
│   │   └── types.rs              # Claude JSON message types
│   └── cursor/
│       ├── mod.rs                # Cursor agent executor
│       ├── config.rs             # Cursor-specific configuration
│       └── types.rs              # Cursor JSON message types
├── protocol/
│   ├── mod.rs                    # Existing protocol module
│   └── control.rs                # High-level control protocol traits
```

## Key Components

### 1. Agent Configurations
Each agent will have its own configuration builder:

**ClaudeConfig:**
- `debug_mode: bool` - Enable ANTHROPIC debug logging
- `plan_mode: bool` - Enable plan mode with auto-approve
- `approvals: bool` - Enable tool approvals
- `model: Option<String>` - Model selection (e.g., "claude-sonnet-4")
- `use_router: bool` - Use claude-code-router
- `permission_mode: PermissionMode` - Default/Plan/Bypass
- `disable_api_key: bool` - Remove ANTHROPIC_API_KEY from env

**CursorConfig:**
- `force: bool` - Auto-approve commands
- `model: Option<String>` - Model selection (e.g., "sonnet-4.5", "gpt-5")

### 2. High-Level Protocol Traits

```rust
/// Permission control for agents that support it
pub trait PermissionControl {
    fn set_permission_mode(&self, mode: PermissionMode) -> impl Future<Output = Result<(), AgentError>>;
    fn interrupt(&self) -> impl Future<Output = Result<(), AgentError>>;
}

/// Tool approval handling
pub trait ToolApproval {
    fn approve_tool(&self, tool_name: &str, input: Value) -> impl Future<Output = Result<ApprovalResponse, AgentError>>;
}
```

### 3. Agent Executors

Both agents will implement the existing `AgentExecutor` trait with:

**ClaudeAgent:**
- Spawns claude-code CLI with stream-json format
- Implements bidirectional control protocol
- Supports session continuation via --fork-session --resume
- Normalizes Claude JSON logs to NormalizedEntry
- Permission control and tool approval hooks

**CursorAgent:**
- Spawns cursor-agent CLI with stream-json format
- Supports session continuation via --resume
- Normalizes Cursor JSON logs to NormalizedEntry
- Handles MCP server trust configuration

### 4. JSON Message Types

Strongly-typed serde structures for parsing:
- Claude's protocol messages (system, assistant, tool_use, tool_result, etc.)
- Cursor's protocol messages (system, assistant, tool_call, etc.)
- Control protocol messages (permission requests, hooks)

## Implementation Phases

### Phase 1: Foundation (Current)
- [x] Review existing codebase structure
- [ ] Create agents module structure
- [ ] Define configuration types
- [ ] Add high-level protocol traits

### Phase 2: Claude Agent
- [ ] Implement ClaudeAgent executor
- [ ] Add bidirectional control protocol support
- [ ] Implement log normalization
- [ ] Add permission mode control
- [ ] Session continuation support

### Phase 3: Cursor Agent
- [ ] Implement CursorAgent executor
- [ ] Add log normalization
- [ ] MCP configuration support
- [ ] Session continuation support

### Phase 4: Integration
- [ ] Update public API exports
- [ ] Create usage examples
- [ ] Add documentation
- [ ] Integration tests

## Usage Examples

### Basic Claude Usage
```rust
use lite_agent_core::{AgentExecutor, AgentRunner};
use lite_agent_core::agents::ClaudeAgent;

#[tokio::main]
async fn main() {
    let agent = ClaudeAgent::builder()
        .with_plan_mode(true)
        .with_model("claude-sonnet-4")
        .build();

    let runner = AgentRunner::new(agent);
    let result = runner.run("Help me refactor this code", config).await?;
}
```

### Basic Cursor Usage
```rust
use lite_agent_core::agents::CursorAgent;

let agent = CursorAgent::builder()
    .with_model("sonnet-4.5")
    .with_force(true)
    .build();

let runner = AgentRunner::new(agent);
let result = runner.run("Fix the bug in main.rs", config).await?;
}
```

### With Permission Control
```rust
use lite_agent_core::agents::{ClaudeAgent, PermissionMode};
use lite_agent_core::protocol::PermissionControl;

let agent = ClaudeAgent::builder()
    .with_permission_mode(PermissionMode::Plan)
    .build();

// Spawn agent and get control handle
let spawned = agent.spawn(&config, "Plan the refactoring").await?;

// Later change permission mode
spawned.set_permission_mode(PermissionMode::BypassPermissions).await?;
```

## Design Decisions

1. **Separate Config Types**: Each agent has its own config type to avoid type confusion and enable builder pattern

2. **Protocol Abstractions**: High-level traits for common features (permissions, approval) allow agents to share interfaces while maintaining flexibility

3. **Typed JSON**: Strongly-typed serde structs prevent runtime parsing errors and provide better IDE support

4. **Backward Compatible**: New agents module doesn't break existing API

5. **Extensible**: Easy to add new agents (Aider, Copilot, etc.) following the same pattern

## Testing Strategy

- Unit tests for JSON parsing
- Integration tests with real agent CLIs (when available)
- Mock implementations for protocol testing
- Property-based testing for log normalization

## Future Extensions

- Add Aider agent support
- Add GitHub Copilot agent support
- Plugin system for custom agents
- Agent composition (multi-agent workflows)
- Unified configuration format (YAML/TOML)
