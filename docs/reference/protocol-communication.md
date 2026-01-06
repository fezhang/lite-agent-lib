# Claude Code Control Protocol - I/O Hijacking Deep Dive

## Overview

Vibe Kanban doesn't just spawn Claude Code CLI as a subprocess - it **hijacks the I/O streams** to implement a **bidirectional control protocol** that allows:
- Intercepting tool approval requests
- Managing permission modes dynamically
- Injecting hooks for custom behavior
- Controlling agent execution flow
- Streaming logs in real-time

This is the **core magic** that enables multi-agent orchestration.

---

## Protocol Architecture

### The I/O Setup

**Location:** `crates/executors/src/executors/claude.rs:233-315`

```rust
// Spawn Claude Code CLI with piped I/O
let mut child = Command::new("npx")
    .args(["-y", "@anthropic-ai/claude-code@2.0.75"])
    .args(["--output-format=stream-json"])    // ← JSON streaming on stdout
    .args(["--input-format=stream-json"])     // ← JSON messages on stdin
    .stdin(Stdio::piped())   // ← Hijack stdin  (SDK → CLI)
    .stdout(Stdio::piped())  // ← Hijack stdout (CLI → SDK)
    .stderr(Stdio::piped())  // ← Hijack stderr (errors/debug)
    .spawn()?;

// Extract the pipes
let child_stdin = child.stdin.take().unwrap();
let child_stdout = child.stdout.take().unwrap();
let child_stderr = child.stderr.take().unwrap();

// Create the protocol peer to manage bidirectional communication
let protocol_peer = ProtocolPeer::spawn(
    child_stdin,   // Write commands to Claude Code
    child_stdout,  // Read responses from Claude Code
    client,        // Handler for control requests
    interrupt_rx   // Channel for graceful shutdown
);
```

**Key Insight:** By using `--input-format=stream-json` and `--output-format=stream-json`, Claude Code CLI speaks a JSON-based protocol over stdin/stdout that Vibe Kanban can intercept and control.

---

## Protocol Peer Implementation

**Location:** `crates/executors/src/executors/claude/protocol.rs`

### Core Structure

```rust
pub struct ProtocolPeer {
    stdin: Arc<Mutex<ChildStdin>>,  // Shared stdin for sending messages
}

impl ProtocolPeer {
    pub fn spawn(
        stdin: ChildStdin,
        stdout: ChildStdout,
        client: Arc<ClaudeAgentClient>,
        interrupt_rx: oneshot::Receiver<()>,
    ) -> Self {
        let peer = Self {
            stdin: Arc::new(Mutex::new(stdin)),
        };

        // Spawn background task to read stdout
        tokio::spawn(async move {
            peer.read_loop(stdout, client, interrupt_rx).await
        });

        peer
    }
}
```

### The Read Loop - Message Dispatcher

**Location:** `protocol.rs:46-101`

```rust
async fn read_loop(
    &self,
    stdout: ChildStdout,
    client: Arc<ClaudeAgentClient>,
    interrupt_rx: oneshot::Receiver<()>,
) -> Result<(), ExecutorError> {
    let mut reader = BufReader::new(stdout);
    let mut buffer = String::new();

    loop {
        buffer.clear();
        tokio::select! {
            // Read line from Claude Code stdout
            line_result = reader.read_line(&mut buffer) => {
                let line = buffer.trim();

                // Parse JSON message
                match serde_json::from_str::<CLIMessage>(line) {
                    // Control request FROM Claude Code (asking for permission)
                    Ok(CLIMessage::ControlRequest { request_id, request }) => {
                        self.handle_control_request(&client, request_id, request).await;
                    }

                    // Control response TO Claude Code (our response was received)
                    Ok(CLIMessage::ControlResponse { .. }) => {}

                    // Result message (final execution result)
                    Ok(CLIMessage::Result(_)) => {
                        client.on_non_control(line).await?;
                        break;  // End of execution
                    }

                    // All other messages (logs, thinking, tool use, etc.)
                    _ => {
                        client.on_non_control(line).await?;
                    }
                }
            }

            // Interrupt signal (user clicked "Stop")
            _ = &mut interrupt_rx => {
                self.interrupt().await?;
            }
        }
    }
    Ok(())
}
```

**Flow:**
1. **Read line-by-line** from Claude Code stdout
2. **Parse JSON** into typed message enum
3. **Route messages**:
   - Control requests → Handle and respond
   - Other messages → Forward to logs (MsgStore)
4. **Interruptible** via channel (for graceful shutdown)

---

## Message Types

**Location:** `crates/executors/src/executors/claude/types.rs`

### 1. CLI Messages (FROM Claude Code)

```rust
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum CLIMessage {
    // Claude Code asking "Can I use this tool?"
    ControlRequest {
        request_id: String,
        request: ControlRequestType,
    },

    // Claude Code acknowledging our control request
    ControlResponse {
        response: ControlResponseType,
    },

    // Final execution result
    Result(serde_json::Value),

    // All other messages (logs, thinking, tool_use, etc.)
    #[serde(untagged)]
    Other(serde_json::Value),
}
```

### 2. Control Requests (FROM Claude Code)

```rust
#[derive(Debug, Deserialize)]
#[serde(tag = "subtype")]
pub enum ControlRequestType {
    // "Can I use this tool?"
    CanUseTool {
        tool_name: String,
        input: Value,
        permission_suggestions: Option<Vec<PermissionUpdate>>,
        tool_use_id: Option<String>,  // Links to tool_use message
    },

    // "Execute this hook callback"
    HookCallback {
        callback_id: String,
        input: Value,
        tool_use_id: Option<String>,
    },
}
```

### 3. SDK Control Requests (TO Claude Code)

```rust
#[derive(Debug, Serialize)]
pub struct SDKControlRequest {
    type: String,  // Always "control_request"
    request_id: String,
    request: SDKControlRequestType,
}

#[derive(Debug, Serialize)]
#[serde(tag = "subtype")]
pub enum SDKControlRequestType {
    // Initialize the protocol with hooks
    Initialize {
        hooks: Option<Value>,
    },

    // Change permission mode mid-execution
    SetPermissionMode {
        mode: PermissionMode,
    },

    // Interrupt execution gracefully
    Interrupt {},
}
```

### 4. Permission Modes

```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    Default,            // Ask for each tool
    AcceptEdits,        // Auto-approve file edits
    Plan,               // Plan mode (only approve ExitPlanMode)
    BypassPermissions,  // Auto-approve everything
}
```

---

## Control Flow Examples

### Example 1: Initialization Sequence

```
┌─────────────┐                                    ┌──────────────┐
│ Vibe Kanban │                                    │  Claude Code │
└──────┬──────┘                                    └──────┬───────┘
       │                                                  │
       │  1. Spawn process with --input/output-format=stream-json
       │─────────────────────────────────────────────────>│
       │                                                  │
       │  2. Initialize { hooks: { PreToolUse: [...] } }│
       │─────────────────────────────────────────────────>│
       │                                                  │
       │  3. SetPermissionMode { mode: Plan }            │
       │─────────────────────────────────────────────────>│
       │                                                  │
       │  4. User { message: "Add auth to the app" }     │
       │─────────────────────────────────────────────────>│
       │                                                  │
       │                          5. Start processing... │
       │                                                  │
```

### Example 2: Tool Approval Flow

```
┌─────────────┐                                    ┌──────────────┐
│ Vibe Kanban │                                    │  Claude Code │
└──────┬──────┘                                    └──────┬───────┘
       │                                                  │
       │                Claude wants to use Bash tool    │
       │  ControlRequest {                               │
       │    type: "can_use_tool",                        │
       │    tool_name: "Bash",                           │
       │    input: { command: "npm install" },           │
       │    tool_use_id: "toolu_abc123"                  │
       │  }                                               │
       │<─────────────────────────────────────────────────│
       │                                                  │
       │  Route to ApprovalService...                    │
       │  Show modal to user: "Allow Bash?"              │
       │  User clicks "Approve"                          │
       │                                                  │
       │  ControlResponse {                              │
       │    type: "success",                             │
       │    response: {                                  │
       │      behavior: "allow",                         │
       │      updatedInput: { command: "npm install" }   │
       │    }                                             │
       │  }                                               │
       │─────────────────────────────────────────────────>│
       │                                                  │
       │                        Tool executes: npm install│
       │  { type: "tool_result", ... }                   │
       │<─────────────────────────────────────────────────│
       │                                                  │
```

### Example 3: Plan Mode → Execution Mode Transition

```
┌─────────────┐                                    ┌──────────────┐
│ Vibe Kanban │                                    │  Claude Code │
└──────┬──────┘                                    └──────┬───────┘
       │                                                  │
       │  Initialize with Plan mode                      │
       │  SetPermissionMode { mode: Plan }               │
       │─────────────────────────────────────────────────>│
       │                                                  │
       │                  All tools auto-approved        │
       │                  (due to AUTO_APPROVE hook)     │
       │                                                  │
       │                  Claude finishes planning...    │
       │  ControlRequest {                               │
       │    type: "can_use_tool",                        │
       │    tool_name: "ExitPlanMode",                   │
       │    input: { plan: "1. Setup DB\n2. Add API" }   │
       │  }                                               │
       │<─────────────────────────────────────────────────│
       │                                                  │
       │  Show plan to user for approval                 │
       │  User clicks "Approve Plan"                     │
       │                                                  │
       │  ControlResponse {                              │
       │    type: "success",                             │
       │    response: {                                  │
       │      behavior: "allow",                         │
       │      updatedPermissions: [{                     │
       │        type: "setMode",                         │
       │        mode: "bypassPermissions",  ← Switch!    │
       │        destination: "session"                   │
       │      }]                                          │
       │    }                                             │
       │  }                                               │
       │─────────────────────────────────────────────────>│
       │                                                  │
       │              Now in execution mode,             │
       │              all tools auto-approved            │
       │                                                  │
```

---

## Implementation Details

### 1. Sending Messages to Claude Code

**Location:** `protocol.rs:182-194`

```rust
async fn send_json<T: serde::Serialize>(&self, message: &T) -> Result<(), ExecutorError> {
    let json = serde_json::to_string(message)?;
    let mut stdin = self.stdin.lock().await;  // Thread-safe access
    stdin.write_all(json.as_bytes()).await?;
    stdin.write_all(b"\n").await?;            // Newline-delimited JSON
    stdin.flush().await?;                     // Ensure sent immediately
    Ok(())
}

pub async fn send_user_message(&self, content: String) -> Result<(), ExecutorError> {
    let message = Message::new_user(content);
    self.send_json(&message).await
}

pub async fn initialize(&self, hooks: Option<Value>) -> Result<(), ExecutorError> {
    self.send_json(&SDKControlRequest::new(
        SDKControlRequestType::Initialize { hooks }
    )).await
}
```

### 2. Handling Control Requests from Claude Code

**Location:** `protocol.rs:103-159`

```rust
async fn handle_control_request(
    &self,
    client: &Arc<ClaudeAgentClient>,
    request_id: String,
    request: ControlRequestType,
) {
    match request {
        ControlRequestType::CanUseTool {
            tool_name,
            input,
            permission_suggestions,
            tool_use_id,
        } => {
            // Delegate to client (which calls ApprovalService)
            let result = client
                .on_can_use_tool(tool_name, input, permission_suggestions, tool_use_id)
                .await;

            match result {
                Ok(permission_result) => {
                    // Send allow/deny response
                    self.send_hook_response(request_id, serde_json::to_value(permission_result)?).await?;
                }
                Err(e) => {
                    // Send error response
                    self.send_error(request_id, e.to_string()).await?;
                }
            }
        }

        ControlRequestType::HookCallback { callback_id, input, tool_use_id } => {
            // Handle hook callbacks (PreToolUse, etc.)
            let hook_output = client
                .on_hook_callback(callback_id, input, tool_use_id)
                .await?;

            self.send_hook_response(request_id, hook_output).await?;
        }
    }
}
```

### 3. The Approval Bridge

**Location:** `client.rs:114-141`

```rust
pub async fn on_can_use_tool(
    &self,
    tool_name: String,
    input: Value,
    _permission_suggestions: Option<Vec<PermissionUpdate>>,
    tool_use_id: Option<String>,
) -> Result<PermissionResult, ExecutorError> {
    if self.auto_approve {
        // Bypass mode: approve everything
        Ok(PermissionResult::Allow {
            updated_input: input,
            updated_permissions: None,
        })
    } else if let Some(tool_use_id) = tool_use_id {
        // Request approval from Vibe Kanban UI
        let approval_service = self.approvals.as_ref().unwrap();
        let status = approval_service
            .request_tool_approval(&tool_name, input.clone(), &tool_use_id)
            .await?;

        // Convert ApprovalStatus → PermissionResult
        match status {
            ApprovalStatus::Approved => {
                // Special handling for ExitPlanMode
                if tool_name == "ExitPlanMode" {
                    Ok(PermissionResult::Allow {
                        updated_input: input,
                        updated_permissions: Some(vec![PermissionUpdate {
                            update_type: PermissionUpdateType::SetMode,
                            mode: Some(PermissionMode::BypassPermissions),
                            destination: PermissionUpdateDestination::Session,
                        }]),
                    })
                } else {
                    Ok(PermissionResult::Allow {
                        updated_input: input,
                        updated_permissions: None,
                    })
                }
            }
            ApprovalStatus::Denied { reason } => {
                Ok(PermissionResult::Deny {
                    message: reason.unwrap_or("Denied by user".to_string()),
                    interrupt: Some(false),
                })
            }
            _ => { /* ... */ }
        }
    }
}
```

### 4. Hook Configuration

**Location:** `claude.rs:131-158`

Vibe Kanban configures **PreToolUse hooks** to intercept tool calls:

```rust
pub fn get_hooks(&self) -> Option<Value> {
    if self.plan.unwrap_or(false) {
        // Plan mode: Auto-approve all tools except ExitPlanMode
        Some(serde_json::json!({
            "PreToolUse": [
                {
                    "matcher": "^ExitPlanMode$",  // Exact match
                    "hookCallbackIds": ["tool_approval"],
                },
                {
                    "matcher": "^(?!ExitPlanMode$).*",  // Everything else
                    "hookCallbackIds": ["AUTO_APPROVE_CALLBACK_ID"],
                }
            ]
        }))
    } else if self.approvals.unwrap_or(false) {
        // Approval mode: Ask for approval on destructive tools
        Some(serde_json::json!({
            "PreToolUse": [
                {
                    // Approve read-only tools automatically
                    "matcher": "^(?!(Glob|Grep|NotebookRead|Read|Task|TodoWrite)$).*",
                    "hookCallbackIds": ["tool_approval"],
                }
            ]
        }))
    } else {
        None  // Bypass mode: no hooks needed
    }
}
```

**Hook Flow:**
1. Claude Code wants to use a tool
2. **Before executing**, it checks hooks matching the tool name
3. If hook matches, it sends `HookCallback` request
4. Vibe Kanban responds with:
   - `"allow"` → Execute immediately
   - `"deny"` → Block execution
   - `"ask"` → Trigger `CanUseTool` request for approval

---

## Log Streaming & Message Store

### Forwarding Non-Control Messages

**Location:** `client.rs:182-186`

```rust
pub async fn on_non_control(&self, line: &str) -> Result<(), ExecutorError> {
    // Forward all non-control messages to stdout (MsgStore)
    self.log_writer.log_raw(line).await
}
```

**LogWriter** pipes to **MsgStore**, which:
1. Stores messages in memory
2. Broadcasts via **Server-Sent Events (SSE)** to frontend
3. Allows real-time log viewing in UI

### Message Store Architecture

```
Claude Code stdout
       ↓
  ProtocolPeer.read_loop()
       ↓
  ClaudeAgentClient.on_non_control()
       ↓
  LogWriter.log_raw()
       ↓
  MsgStore.push_stdout()
       ↓
  SSE Stream → Frontend UI
```

---

## Key Protocol Features

### 1. **Dynamic Permission Mode Changes**

```rust
// Start in Plan mode
protocol_peer.set_permission_mode(PermissionMode::Plan).await?;

// After plan approval, switch to execution mode
protocol_peer.set_permission_mode(PermissionMode::BypassPermissions).await?;
```

### 2. **Graceful Interruption**

```rust
// Send interrupt request
protocol_peer.interrupt().await?;

// Claude Code receives interrupt and can:
// - Save state
// - Clean up resources
// - Exit gracefully
```

### 3. **Session Resumption**

```rust
// Initial spawn
agent.spawn(&dir, "Add authentication", env).await?;
// Session ID: "session_abc123"

// Follow-up (preserves conversation history)
agent.spawn_follow_up(&dir, "Also add logging", "session_abc123", env).await?;
```

### 4. **Approval Logging**

Approval responses are **logged to MsgStore** so they appear in the UI:

```rust
self.log_writer.log_raw(&serde_json::to_string(&ClaudeJson::ApprovalResponse {
    call_id: tool_use_id,
    tool_name,
    approval_status,  // Approved, Denied, TimedOut
})?).await?;
```

---

## Protocol Message Examples

### Initialize Request (SDK → CLI)

```json
{
  "type": "control_request",
  "request_id": "req_123",
  "request": {
    "subtype": "initialize",
    "hooks": {
      "PreToolUse": [
        {
          "matcher": "^ExitPlanMode$",
          "hookCallbackIds": ["tool_approval"]
        }
      ]
    }
  }
}
```

### CanUseTool Request (CLI → SDK)

```json
{
  "type": "control_request",
  "request_id": "req_456",
  "request": {
    "subtype": "can_use_tool",
    "tool_name": "Bash",
    "input": {
      "command": "npm install express"
    },
    "tool_use_id": "toolu_789"
  }
}
```

### Allow Response (SDK → CLI)

```json
{
  "type": "control_response",
  "response": {
    "subtype": "success",
    "request_id": "req_456",
    "response": {
      "behavior": "allow",
      "updatedInput": {
        "command": "npm install express"
      }
    }
  }
}
```

### Deny Response (SDK → CLI)

```json
{
  "type": "control_response",
  "response": {
    "subtype": "success",
    "request_id": "req_456",
    "response": {
      "behavior": "deny",
      "message": "User denied this tool use request",
      "interrupt": false
    }
  }
}
```

---

## Summary: Why This Architecture is Powerful

### 1. **Full Control Over Agent Behavior**
- Intercept every tool call before execution
- Dynamically change permission modes
- Inject custom approval logic

### 2. **Multi-Tenant Orchestration**
- Each agent runs in isolation (separate process + git worktree)
- Protocol peer manages independent communication channels
- No cross-contamination between agents

### 3. **Real-Time Observability**
- All messages flow through MsgStore
- SSE streams to frontend for live updates
- Complete audit trail of agent actions

### 4. **Graceful Lifecycle Management**
- Interrupt agents mid-execution
- Resume sessions with conversation history
- Clean shutdown via SIGINT/SIGTERM/SIGKILL cascade

### 5. **Permission Mode Flexibility**
- **Plan Mode**: Review before execution
- **Approval Mode**: Ask for destructive tools
- **Bypass Mode**: Fully autonomous
- **Dynamic Switching**: Change mid-execution

---

## The "Hijacking" Explained

**What gets hijacked:**
- ✅ **stdin** - Vibe Kanban sends control requests, user messages
- ✅ **stdout** - Vibe Kanban receives logs, tool outputs, control requests
- ✅ **stderr** - Error logs captured separately

**What gets controlled:**
- ✅ Tool execution (approve/deny)
- ✅ Permission modes (plan/approval/bypass)
- ✅ Agent interruption (graceful shutdown)
- ✅ Session continuation (resume with history)

**What remains unchanged:**
- ✅ Claude's reasoning and responses
- ✅ Tool implementations
- ✅ MCP server connections
- ✅ Model behavior

**Result:** Vibe Kanban acts as a **middleware orchestrator** between the user and Claude Code, enabling multi-agent coordination without modifying Claude Code itself.
