# Multi-Agent Abstraction - Supporting Different Agent Architectures

## The Challenge

Different coding agents have completely different architectures:
- **Claude Code CLI** - Bidirectional control protocol with approval hooks
- **Cursor Agent** - Simple JSON streaming without control protocol
- **Codex** - Uses Agent Client Protocol (ACP) over stdio
- **Gemini CLI** - Different message format and tools
- **Amp** - Session-based with thread resumption
- **Others** - Each with unique quirks

How does Vibe Kanban support them all?

---

## The Solution: Abstraction Layer

**Location:** `crates/executors/src/executors/mod.rs`

### The StandardCodingAgentExecutor Trait

```rust
#[async_trait]
pub trait StandardCodingAgentExecutor {
    // Spawn initial agent process
    async fn spawn(
        &self,
        current_dir: &Path,
        prompt: &str,
        env: &ExecutionEnv,
    ) -> Result<SpawnedChild, ExecutorError>;

    // Spawn follow-up in existing session
    async fn spawn_follow_up(
        &self,
        current_dir: &Path,
        prompt: &str,
        session_id: &str,
        env: &ExecutionEnv,
    ) -> Result<SpawnedChild, ExecutorError>;

    // Convert agent-specific logs to normalized format
    fn normalize_logs(&self, msg_store: Arc<MsgStore>, current_dir: &Path);

    // Optional: Approval service integration
    fn use_approvals(&mut self, approvals: Arc<dyn ExecutorApprovalService>) {
        // Default: no-op
    }

    // MCP configuration
    fn default_mcp_config_path(&self) -> Option<PathBuf>;

    // Check if agent is installed/authenticated
    fn get_availability_info(&self) -> AvailabilityInfo;
}
```

**Every agent** implements this trait, allowing uniform orchestration despite vastly different implementations.

---

## Case Study: Cursor vs Claude Code

### Architecture Comparison

| Feature | Claude Code | Cursor Agent |
|---------|-------------|--------------|
| **Command** | `npx -y @anthropic-ai/claude-code` | `cursor-agent` |
| **Protocol** | Bidirectional control protocol | One-way JSON streaming |
| **Input** | stdin (multiple messages) | stdin (single prompt, then closed) |
| **Output** | stdout JSON with control requests | stdout JSON (logs only) |
| **Approvals** | Via `CanUseTool` control requests | Via `--force` flag |
| **Session Resume** | `--fork-session --resume <id>` | `--resume <id>` |
| **Hooks** | PreToolUse hooks configurable | Not supported |
| **Permission Modes** | Plan/Approval/Bypass (dynamic) | Force mode (static) |

---

## Cursor Implementation Deep Dive

**Location:** `crates/executors/src/executors/cursor.rs`

### 1. Spawning Cursor

```rust
async fn spawn(
    &self,
    current_dir: &Path,
    prompt: &str,
    env: &ExecutionEnv,
) -> Result<SpawnedChild, ExecutorError> {
    // Build command
    let mut command = Command::new("cursor-agent")
        .args(["-p", "--output-format=stream-json"])
        .args(if self.force { &["--force"] } else { &[] })
        .args(if let Some(model) = &self.model { &["--model", model] } else { &[] })
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(current_dir)
        .spawn()?;

    // Write prompt to stdin and CLOSE it
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(prompt.as_bytes()).await?;
        stdin.shutdown().await?;  // ← Close stdin!
    }

    Ok(child.into())
}
```

**Key Difference from Claude:**
- Claude keeps stdin open for bidirectional communication
- Cursor writes prompt once and closes stdin
- No protocol peer, no control requests

### 2. Log Normalization for Cursor

**Location:** `cursor.rs:148-485`

```rust
fn normalize_logs(&self, msg_store: Arc<MsgStore>, worktree_path: &Path) {
    // Process stderr for error messages
    tokio::spawn(async move {
        let mut stderr = msg_store.stderr_chunked_stream();
        while let Some(Ok(chunk)) = stderr.next().await {
            // Check for auth errors
            if content.contains("Authentication required") {
                // Emit setup error
                msg_store.push_patch(ConversationPatch::add_normalized_entry(
                    id,
                    NormalizedEntry {
                        entry_type: NormalizedEntryType::ErrorMessage {
                            error_type: NormalizedEntryError::SetupRequired,
                        },
                        content,
                        ...
                    }
                ));
            }
        }
    });

    // Process stdout JSON lines
    tokio::spawn(async move {
        let mut lines = msg_store.stdout_lines_stream();

        while let Some(Ok(line)) = lines.next().await {
            // Parse Cursor-specific JSON format
            let cursor_json: CursorJson = serde_json::from_str(&line)?;

            match cursor_json {
                CursorJson::System { model, .. } => {
                    // Emit "System initialized with model: ..."
                }

                CursorJson::Assistant { message, .. } => {
                    // Stream assistant response chunks
                    // Coalesce text chunks into single entry
                }

                CursorJson::Thinking { text, .. } => {
                    // Stream thinking content
                }

                CursorJson::ToolCall { subtype, call_id, tool_call, .. } => {
                    if subtype == "started" {
                        // Create ToolUse entry
                    } else if subtype == "completed" {
                        // Update entry with results
                    }
                }

                CursorJson::Result { .. } => {
                    // Final result metadata
                }
            }
        }
    });
}
```

### 3. Cursor JSON Message Types

**Location:** `cursor.rs:513-591`

```rust
#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum CursorJson {
    #[serde(rename = "system")]
    System {
        subtype: Option<String>,
        session_id: Option<String>,
        model: Option<String>,
        permission_mode: Option<String>,
    },

    #[serde(rename = "user")]
    User {
        message: CursorMessage,
        session_id: Option<String>,
    },

    #[serde(rename = "assistant")]
    Assistant {
        message: CursorMessage,
        session_id: Option<String>,
    },

    #[serde(rename = "thinking")]
    Thinking {
        text: Option<String>,
        session_id: Option<String>,
    },

    #[serde(rename = "tool_call")]
    ToolCall {
        subtype: Option<String>,  // "started" | "completed"
        call_id: Option<String>,
        tool_call: CursorToolCall,
        session_id: Option<String>,
    },

    #[serde(rename = "result")]
    Result {
        subtype: Option<String>,
        is_error: Option<bool>,
        duration_ms: Option<u64>,
        session_id: Option<String>,
    },
}
```

**Contrast with Claude:**
- No `ControlRequest` or `ControlResponse` messages
- Tool calls are passive notifications, not requests
- No approval flow

### 4. Cursor Tool Calls

**Location:** `cursor.rs:620-694`

```rust
#[derive(Deserialize, Serialize)]
pub enum CursorToolCall {
    #[serde(rename = "shellToolCall")]
    Shell { args: CursorShellArgs, result: Option<Value> },

    #[serde(rename = "editToolCall")]
    Edit { args: CursorEditArgs, result: Option<CursorEditResult> },

    #[serde(rename = "readToolCall")]
    Read { args: CursorReadArgs, result: Option<Value> },

    #[serde(rename = "writeToolCall")]
    Write { args: CursorWriteArgs, result: Option<Value> },

    #[serde(rename = "globToolCall")]
    Glob { args: CursorGlobArgs, result: Option<Value> },

    #[serde(rename = "grepToolCall")]
    Grep { args: CursorGrepArgs, result: Option<Value> },

    #[serde(rename = "mcpToolCall")]
    Mcp { args: CursorMcpArgs, result: Option<Value> },

    // ... more tool variants
}
```

**Different tool naming:**
- Claude: `Bash`, `Edit`, `Read` (consistent tool names)
- Cursor: `shellToolCall`, `editToolCall`, `readToolCall` (suffixed with "ToolCall")

### 5. Converting to Unified Format

**Location:** `cursor.rs:716-916`

```rust
impl CursorToolCall {
    pub fn to_action_and_content(&self, worktree_path: &str) -> (ActionType, String) {
        match self {
            CursorToolCall::Shell { args, .. } => {
                (
                    ActionType::CommandRun {
                        command: args.command.clone(),
                        result: None,
                    },
                    args.command.clone(),
                )
            }

            CursorToolCall::Edit { args, result, .. } => {
                let path = make_path_relative(&args.path, worktree_path);
                let mut changes = vec![];

                // Extract diffs from different edit formats
                if let Some(apply_patch) = &args.apply_patch {
                    changes.push(FileChange::Edit {
                        unified_diff: normalize_unified_diff(&path, &apply_patch.patch_content),
                        has_line_numbers: false,
                    });
                }

                if let Some(str_replace) = &args.str_replace {
                    changes.push(FileChange::Edit {
                        unified_diff: create_unified_diff(
                            &path,
                            &str_replace.old_text,
                            &str_replace.new_text,
                        ),
                        has_line_numbers: false,
                    });
                }

                // ... handle multi_str_replace, etc.

                (
                    ActionType::FileEdit { path: path.clone(), changes },
                    path,
                )
            }

            // ... other tools converted to ActionType
        }
    }
}
```

**Normalization:** Cursor-specific tool formats are converted to Vibe Kanban's unified `ActionType` enum, allowing the UI to display all agents' actions consistently.

---

## The Abstraction in Action

### Unified Spawning

```rust
// User selects Cursor or Claude Code from UI
let executor_profile_id = ExecutorProfileId {
    executor: BaseCodingAgent::CursorAgent,  // or ClaudeCode
    variant: "default".to_string(),
};

// Get the executor (different types, same trait)
let mut agent = ExecutorConfigs::get_cached()
    .get_coding_agent(&executor_profile_id)?;

// Set up approvals (if supported)
agent.use_approvals(approvals);

// Spawn (implementation differs, interface same)
let child = agent.spawn(&current_dir, &prompt, &env).await?;
```

### Unified Log Processing

```rust
// Regardless of agent type, logs go to MsgStore
let msg_store = Arc::new(MsgStore::new());

// Agent-specific normalization
executor.normalize_logs(msg_store.clone(), &current_dir);

// Unified output: NormalizedEntry stream
let mut stream = msg_store.normalized_stream();
while let Some(entry) = stream.next().await {
    match entry.entry_type {
        NormalizedEntryType::AssistantMessage => { /* render */ }
        NormalizedEntryType::ToolUse { tool_name, action_type, status } => { /* render */ }
        NormalizedEntryType::Thinking => { /* render */ }
        // ... same UI for all agents
    }
}
```

---

## Approval Handling Differences

### Claude Code (Full Control)

```rust
// User approval modal
approval_service.request_tool_approval(&tool_name, input, &tool_use_id).await
    ↓
// Claude pauses, waits for response
ControlRequest { type: "can_use_tool", tool_name: "Bash", ... }
    ↓
// User clicks Approve/Deny
ControlResponse { behavior: "allow"/"deny" }
    ↓
// Claude executes or skips tool
```

### Cursor (Force Mode)

```rust
// No runtime approval system
// Instead, use --force flag at spawn time

if approvals_enabled {
    // Don't use --force, Cursor will use default permissions
    Command::new("cursor-agent").args(["-p", "--output-format=stream-json"])
} else {
    // Auto-approve everything
    Command::new("cursor-agent").args(["-p", "--output-format=stream-json", "--force"])
}

// Tools execute immediately, no pause
```

---

## Session Resumption

### Claude Code

```rust
// Initial spawn
agent.spawn(&dir, "Add authentication", env).await?;
// → Outputs: { session_id: "session_abc123", ... }

// Follow-up
agent.spawn_follow_up(&dir, "Also add logging", "session_abc123", env).await?;
// → Command: npx claude-code --fork-session --resume session_abc123
// → Claude continues with full conversation history
```

### Cursor

```rust
// Initial spawn
agent.spawn(&dir, "Add authentication", env).await?;
// → Outputs: { session_id: "session_xyz", ... }

// Follow-up
agent.spawn_follow_up(&dir, "Also add logging", "session_xyz", env).await?;
// → Command: cursor-agent --resume session_xyz -p
// → Cursor continues with session context
```

**Similar interface, different flags!**

---

## MCP Configuration Differences

### Claude Code

```rust
fn default_mcp_config_path(&self) -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".claude.json"))
}
```

### Cursor

```rust
fn default_mcp_config_path(&self) -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".cursor").join("mcp.json"))
}
```

### Codex

```rust
fn default_mcp_config_path(&self) -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".codex").join("mcp.json"))
}
```

**Different paths, same concept** - Vibe Kanban knows where each agent stores its MCP config.

---

## Agent Registry

**Location:** `crates/executors/src/executors/mod.rs:90-103`

```rust
pub enum CodingAgent {
    ClaudeCode,
    Amp,
    Gemini,
    Codex,
    Opencode,
    #[serde(alias = "CURSOR")]
    CursorAgent,
    QwenCode,
    Copilot,
    Droid,
}
```

All agents are registered in this enum, enabling:
- Type-safe executor selection
- Serialization to database
- Frontend dropdown selection
- Profile management

---

## Benefits of This Abstraction

### 1. **Unified UI**
- Same log viewer for all agents
- Same approval modal (when supported)
- Same task/workspace management
- Same diff viewer

### 2. **Consistent Behavior**
- All agents use git worktrees
- All agents get isolated environments
- All agents support session resumption (if capable)
- All agents stream logs in real-time

### 3. **Easy to Add New Agents**

To support a new agent:
1. Implement `StandardCodingAgentExecutor` trait
2. Parse agent-specific JSON format
3. Convert to `NormalizedEntry` stream
4. Add to `CodingAgent` enum
5. Done!

Example template:

```rust
pub struct NewAgent {
    pub model: Option<String>,
    pub cmd: CmdOverrides,
}

#[async_trait]
impl StandardCodingAgentExecutor for NewAgent {
    async fn spawn(&self, current_dir: &Path, prompt: &str, env: &ExecutionEnv)
        -> Result<SpawnedChild, ExecutorError>
    {
        // 1. Build command
        let mut command = Command::new("new-agent")
            .args(["--json-output"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .current_dir(current_dir)
            .spawn()?;

        // 2. Send initial prompt
        // 3. Return spawned child
    }

    fn normalize_logs(&self, msg_store: Arc<MsgStore>, current_dir: &Path) {
        tokio::spawn(async move {
            let mut lines = msg_store.stdout_lines_stream();

            while let Some(Ok(line)) = lines.next().await {
                // Parse agent-specific JSON
                let agent_json: NewAgentJson = serde_json::from_str(&line)?;

                // Convert to NormalizedEntry
                let entry = match agent_json {
                    NewAgentJson::Message { text, .. } => NormalizedEntry {
                        entry_type: NormalizedEntryType::AssistantMessage,
                        content: text,
                        ...
                    },
                    // ... handle other message types
                };

                msg_store.push_patch(ConversationPatch::add_normalized_entry(id, entry));
            }
        });
    }

    fn get_availability_info(&self) -> AvailabilityInfo {
        // Check if agent binary exists
        if resolve_executable_path("new-agent").is_some() {
            AvailabilityInfo::InstallationFound
        } else {
            AvailabilityInfo::NotFound
        }
    }
}
```

### 4. **Feature Parity**
Despite different agent capabilities, users get:
- Session resumption
- Real-time log streaming
- Tool action tracking
- MCP server support
- Git integration
- Parallel execution

---

## Capability Detection

**Location:** `crates/executors/src/executors/mod.rs:38-45`

```rust
pub enum BaseAgentCapability {
    SessionFork,     // Can resume sessions
    SetupHelper,     // Needs setup before use
}

pub enum AvailabilityInfo {
    NotFound,                           // Agent not installed
    InstallationFound,                  // Agent binary exists
    LoginDetected { last_auth_timestamp: i64 },  // Agent authenticated
}
```

Vibe Kanban checks agent capabilities:
- If `SessionFork` supported → Enable follow-up prompts
- If `SetupHelper` needed → Show setup instructions
- If `NotFound` → Prompt user to install agent

---

## Summary: The Abstraction Hierarchy

```
┌─────────────────────────────────────────────────────┐
│         Frontend (React + TypeScript)               │
│  - Task Management                                  │
│  - Log Viewer (unified for all agents)              │
│  - Approval Modals                                  │
└─────────────────┬───────────────────────────────────┘
                  │ REST API
┌─────────────────▼───────────────────────────────────┐
│         ContainerService (Orchestration)            │
│  - Workspace Management                             │
│  - Process Spawning                                 │
│  - Log Streaming                                    │
└─────────────────┬───────────────────────────────────┘
                  │ StandardCodingAgentExecutor trait
┌─────────────────▼───────────────────────────────────┐
│         Agent Implementations                       │
├─────────────────┬───────────────────────────────────┤
│  Claude Code    │  Cursor Agent  │  Codex  │  Amp   │
│  (Control       │  (JSON Stream) │  (ACP)  │ (Thrd) │
│   Protocol)     │                │         │        │
├─────────────────┼────────────────┼─────────┼────────┤
│  - spawn()      │  - spawn()     │ - spawn()│ etc.  │
│  - normalize()  │  - normalize() │ - ...    │       │
└─────────────────┴────────────────┴─────────┴────────┘
                  │
┌─────────────────▼───────────────────────────────────┐
│         Unified Output (MsgStore)                   │
│  - NormalizedEntry stream                           │
│  - SSE to frontend                                  │
└─────────────────────────────────────────────────────┘
```

**Result:** Different agents, same user experience, unified orchestration!

---

## Key Takeaway

Vibe Kanban doesn't force all agents to work the same way. Instead:
- **Each agent keeps its unique protocol** (control, streaming, ACP, etc.)
- **Abstraction layer standardizes the interface** (spawn, normalize_logs)
- **Unified log format** (NormalizedEntry) enables consistent UI
- **Agent-specific features** (approvals, hooks) are optional traits
- **Users get the same orchestration experience** regardless of agent choice

This design allows Vibe Kanban to support:
- Agents with bidirectional control (Claude Code)
- Agents with simple streaming (Cursor)
- Agents with custom protocols (Codex, Amp)
- Future agents with unknown architectures (easy to add!)
