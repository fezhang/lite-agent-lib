# Spec: Cursor and Claude Agent Support

## Critical Questions

**Q1: Why do we need separate config types for each agent?**
A: Each agent has unique configuration options (Claude has debug/plan/approvals, Cursor has force/model). Separate configs prevent type confusion and enable builder patterns tailored to each agent's capabilities.

**Q2: What is the bidirectional control protocol?**
A: Claude Code supports a control protocol where the agent can request permission changes and the host can interrupt or modify permissions in real-time. This requires specialized protocol handling beyond simple JSON streaming.

**Q3: How do we handle different JSON message formats?**
A: Both agents emit stream-json but with different message structures. We'll use strongly-typed serde structs for each agent's protocol and normalize them to a common `NormalizedEntry` format for consistent downstream processing.

**Q4: What testing is required for this feature?**
A: Unit tests for JSON parsing, integration tests with real agent CLIs (when available), mock implementations for protocol testing, and property-based testing for log normalization.

## What

Add support for Cursor and Claude Code agents to the lite-agent-lib library, providing:

1. **Agent Configurations** - Builder pattern configs for Claude and Cursor agents
2. **High-Level Protocol Abstractions** - Traits for permission control and tool approval
3. **Agent Executors** - ClaudeAgent and CursorAgent implementing AgentExecutor trait
4. **Bidirectional Control Protocol** - Support for Claude's permission control protocol
5. **Session Continuation** - Resume previous agent sessions via --resume / --fork-session
6. **Log Normalization** - Convert agent-specific JSON to normalized format

## Why

**Current Limitations:**
- lite-agent-lib lacks easy-to-use agent configurations for popular AI coding agents
- No support for bidirectional control protocols (Claude's permission system)
- Session resumption requires manual CLI flag management
- Different JSON formats require custom parsing for each agent

**Benefits:**
- **Developer Experience**: Simple builder API for configuring agents
- **Protocol Support**: First-class support for Claude's advanced permission control
- **Session Management**: Transparent session continuation for long-running workflows
- **Type Safety**: Strongly-typed message structures prevent runtime errors
- **Extensibility**: Pattern can be extended to future agents (Aider, Copilot, etc.)

## How

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

### Key Components

#### 1. Agent Configurations

**ClaudeConfig:**
```rust
pub struct ClaudeConfig {
    pub debug_mode: bool,           // Enable ANTHROPIC debug logging
    pub plan_mode: bool,            // Enable plan mode with auto-approve
    pub approvals: bool,            // Enable tool approvals
    pub model: Option<String>,      // Model selection (e.g., "claude-sonnet-4")
    pub use_router: bool,           // Use claude-code-router
    pub permission_mode: PermissionMode,  // Default/Plan/Bypass
    pub disable_api_key: bool,      // Remove ANTHROPIC_API_KEY from env
}
```

**CursorConfig:**
```rust
pub struct CursorConfig {
    pub force: bool,                // Auto-approve commands
    pub model: Option<String>,      // Model selection (e.g., "sonnet-4.5", "gpt-5")
}
```

#### 2. High-Level Protocol Traits

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

#### 3. Agent Executors

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

#### 4. JSON Message Types

Strongly-typed serde structures for parsing:
- Claude's protocol messages (system, assistant, tool_use, tool_result, etc.)
- Cursor's protocol messages (system, assistant, tool_call, etc.)
- Control protocol messages (permission requests, hooks)

### Implementation Approach

1. **Phase 1: Foundation**
   - Create agents module structure
   - Define configuration types with builders
   - Add high-level protocol traits (PermissionControl, ToolApproval)

2. **Phase 2: Claude Agent**
   - Implement ClaudeAgent executor with AgentExecutor trait
   - Add Claude-specific JSON message types
   - Implement bidirectional control protocol support
   - Add log normalization for Claude messages
   - Implement permission mode control
   - Add session continuation support

3. **Phase 3: Cursor Agent**
   - Implement CursorAgent executor with AgentExecutor trait
   - Add Cursor-specific JSON message types
   - Implement log normalization for Cursor messages
   - Add MCP configuration support
   - Add session continuation support

4. **Phase 4: Integration**
   - Update public API exports in lib.rs
   - Create usage examples
   - Add documentation to existing types
   - Write integration tests

## Verification

### Unit Tests

**Configuration Builders:**
- Test ClaudeConfig builder with all combinations of flags
- Test CursorConfig builder with all combinations of flags
- Verify default values are correct
- Test validation logic (if any)

**JSON Parsing:**
- Test parsing of Claude JSON message types (all variants)
- Test parsing of Cursor JSON message types (all variants)
- Test handling of malformed JSON (error cases)
- Test edge cases (null fields, unexpected types)

**Log Normalization:**
- Test normalization of Claude messages to NormalizedEntry
- Test normalization of Cursor messages to NormalizedEntry
- Test preservation of all critical fields
- Test handling of unknown message types

**Protocol Traits:**
- Test PermissionControl trait implementation (mock agent)
- Test ToolApproval trait implementation (mock agent)
- Test error handling for unsupported operations

### Integration Tests

**Claude Agent Integration:**
- Test spawning Claude agent with real CLI (if available)
- Test session continuation: create session, resume, verify state
- Test permission mode changes during execution
- Test tool approval flow
- Verify log output matches expected normalized format

**Cursor Agent Integration:**
- Test spawning Cursor agent with real CLI (if available)
- Test session continuation: create session, resume, verify state
- Test MCP server configuration
- Verify log output matches expected normalized format

**Error Handling:**
- Test agent spawn failure (CLI not found)
- Test agent crash during execution
- Test invalid JSON in stream output
- Test permission denial scenarios

### Acceptance Criteria

1. ✅ **Configuration API**: Both ClaudeConfig and CursorConfig builders work correctly with all options
2. ✅ **Agent Executors**: Both ClaudeAgent and CursorAgent implement AgentExecutor trait
3. ✅ **JSON Parsing**: All known message types parse correctly for both agents
4. ✅ **Log Normalization**: Both agents produce valid NormalizedEntry output
5. ✅ **Session Continuation**: Both agents support resuming previous sessions
6. ✅ **Protocol Support**: ClaudeAgent implements PermissionControl and ToolApproval traits
7. ✅ **Unit Tests**: All unit tests pass (parsing, normalization, configs, traits)
8. ✅ **Integration Tests**: All integration tests pass (agent spawn, session resume, protocols)
9. ✅ **Documentation**: Public API is documented with examples
10. ✅ **Backward Compatibility**: Existing API remains unchanged (new agents module only)

### Success Metrics

- **Coverage**: Unit test coverage >90% for new code
- **Parsing**: Zero parsing failures on valid JSON from real agents
- **Normalization**: All message types successfully normalized
- **Session Resume**: 100% success rate for session continuation in tests
- **API Safety**: Zero breaking changes to existing public API
