# Agent Lifecycle - How Vibe Kanban Orchestrates Claude Code CLI

## Overview

Vibe Kanban orchestrates multiple coding agents (including Claude Code CLI) through a sophisticated lifecycle management system. Here's how it works:

---

## 1. How Claude Code CLI is Invoked to Start Work

### The Command
Located in `crates/executors/src/executors/claude.rs:42-48`

```rust
fn base_command(claude_code_router: bool) -> &'static str {
    if claude_code_router {
        "npx -y @musistudio/claude-code-router@1.0.66 code"
    } else {
        "npx -y @anthropic-ai/claude-code@2.0.75"
    }
}
```

### Invocation Flow

**Step 1: Task Creation → Workspace Creation**
- User creates a task in a project
- System creates an isolated **git worktree** for this task attempt
- Each worktree is a separate workspace with its own branch

**Step 2: Session Creation**
- A **Session** is created linking:
  - Workspace (git worktree)
  - Executor (e.g., ClaudeCode with specific profile)
  - Task

**Step 3: Execution Process Spawn**
Location: `crates/executors/src/actions/coding_agent_initial.rs:36-58`

```rust
async fn spawn(
    &self,
    current_dir: &Path,
    approvals: Arc<dyn ExecutorApprovalService>,
    env: &ExecutionEnv,
) -> Result<SpawnedChild, ExecutorError> {
    // 1. Resolve working directory
    let effective_dir = match &self.working_dir {
        Some(rel_path) => current_dir.join(rel_path),
        None => current_dir.to_path_buf(),
    };

    // 2. Get executor config from cached profile
    let mut agent = ExecutorConfigs::get_cached()
        .get_coding_agent(&executor_profile_id)?;

    // 3. Set up approval service
    agent.use_approvals(approvals.clone());

    // 4. Spawn the agent process
    agent.spawn(&effective_dir, &self.prompt, env).await
}
```

**Step 4: Process Spawning** (`claude.rs:233-315`)

```rust
async fn spawn_internal(...) -> Result<SpawnedChild, ExecutorError> {
    // Build command with all flags
    let mut command = Command::new("npx")
        .args(["-y", "@anthropic-ai/claude-code@2.0.75"])
        .args(["-p"])  // Prompt flag
        .args(["--verbose"])
        .args(["--output-format=stream-json"])
        .args(["--input-format=stream-json"])
        .args(["--include-partial-messages"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(worktree_path)
        .env(...) // Apply environment variables
        .group_spawn()?;  // Spawn as process group

    // Create protocol peer for bidirectional communication
    let protocol_peer = ProtocolPeer::spawn(
        child_stdin,
        child_stdout,
        client,
        interrupt_rx
    );

    // Initialize control protocol
    protocol_peer.initialize(hooks).await?;
    protocol_peer.set_permission_mode(permission_mode).await?;

    // Send user message (the actual task prompt)
    protocol_peer.send_user_message(prompt).await?;

    Ok(SpawnedChild { child, exit_signal, interrupt_sender })
}
```

**Key Details:**
- **Process Group**: Uses `command_group` to spawn as a process group (enables killing all child processes)
- **Bidirectional Communication**: Uses stdin/stdout with JSON streaming format
- **Control Protocol**: Vibe Kanban communicates with Claude Code via a protocol layer
- **Isolated Environment**: Runs in isolated git worktree with specific working directory

---

## 2. How Jobs are Distributed

### Architecture for Distribution

**Parallel Execution Model:**
- Each **Task** can have multiple **Workspaces** (different attempts/branches)
- Each **Workspace** can have multiple **Sessions** (sequential)
- Each **Session** can have multiple **ExecutionProcesses** (setup, agent, dev server)

### Distribution Mechanisms

#### A. **Multi-Workspace Execution** (Different Tasks in Parallel)
```
Project
├── Task 1 → Workspace 1 (branch: task-1-abc123) → Claude Code Agent
├── Task 2 → Workspace 2 (branch: task-2-def456) → Gemini CLI Agent
└── Task 3 → Workspace 3 (branch: task-3-ghi789) → Codex Agent
```
- Each task gets isolated git worktree
- No conflicts because each works on separate branch
- Managed by `WorkspaceManager` and `WorktreeManager`

#### B. **Sequential Execution within Session**
Location: `crates/services/src/services/container.rs:464-496`

For a single workspace:
1. **Setup Scripts** run first (in parallel across repos if multiple)
2. **Coding Agent** runs when setup completes
3. **Dev Server** can run alongside (optional)
4. **Cleanup Scripts** run on completion

```rust
// Setup actions chained together
fn setup_actions_for_repos(repos: &[ProjectRepoWithName]) -> Option<ExecutorAction> {
    let mut root_action = ExecutorAction::new(
        ExecutorActionType::ScriptRequest(ScriptRequest {
            script: first.setup_script.clone().unwrap(),
            working_dir: Some(first.repo_name.clone()),
        }),
        None,
    );

    // Chain additional repo setup scripts
    for repo in iter {
        root_action = root_action.append_action(ExecutorAction::new(...));
    }

    Some(root_action)
}
```

#### C. **Follow-up Requests (Session Resumption)**
Location: `claude.rs:178-193`

```rust
async fn spawn_follow_up(
    &self,
    current_dir: &Path,
    prompt: &str,
    session_id: &str,  // Resume existing session!
    env: &ExecutionEnv,
) -> Result<SpawnedChild, ExecutorError> {
    let command_parts = command_builder.build_follow_up(&[
        "--fork-session",
        "--resume",
        session_id,  // Claude Code resumes with conversation history
    ])?;

    self.spawn_internal(current_dir, prompt, command_parts, env).await
}
```

**Queued Messages:**
- User can queue follow-up prompts while agent is running
- System spawns new execution process with `--resume` flag
- Agent continues with full conversation context

#### D. **Process Tracking**
- **ExecutionProcess** table tracks all running processes
- **MsgStore** (in-memory) streams logs in real-time via SSE
- **ExecutionProcessStatus**: Running → Completed/Failed/Killed

---

## 3. How Agents are Killed When Job is Done

### Automatic Termination

**Exit Monitor** (`local-deployment/src/container.rs:343-400`)

```rust
pub fn spawn_exit_monitor(&self, exec_id: &Uuid, exit_signal: Option<ExecutorExitSignal>) {
    tokio::spawn(async move {
        tokio::select! {
            // Option 1: Executor signals completion (graceful)
            exit_result = &mut exit_signal_future => {
                // Kill process group
                kill_process_group(&mut child).await?;

                // Determine status based on result
                status = match exit_result {
                    Ok(ExecutorExitResult::Success) => Completed,
                    Ok(ExecutorExitResult::Failure) => Failed,
                };
            }

            // Option 2: Process exits naturally
            exit_status_result = &mut process_exit_rx => {
                status = if exit_status.success() {
                    Completed
                } else {
                    Failed
                };
            }
        }

        // Cleanup: commit changes, update DB, finalize task
    });
}
```

### Kill Process Implementation
Location: `local-deployment/src/command.rs:11-31`

```rust
pub async fn kill_process_group(child: &mut AsyncGroupChild) -> Result<(), ContainerError> {
    // On Unix: Send signals to entire process group
    let pgid = getpgid(Some(Pid::from_raw(pid as i32)))?;

    // Escalating signals with 2-second delays
    for sig in [SIGINT, SIGTERM, SIGKILL] {
        killpg(pgid, sig)?;
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Check if process exited
        if child.inner().try_wait()? {
            break;
        }
    }

    // On Windows: Use job_object to kill process tree
}
```

**Graceful Shutdown Sequence:**
1. **SIGINT** (2 sec wait) - Interrupt, allows cleanup
2. **SIGTERM** (2 sec wait) - Terminate, asks nicely
3. **SIGKILL** - Force kill if still running

### Manual Termination

**User-Initiated Kill:**
- User clicks "Stop" in UI
- API endpoint triggers `stop_execution(execution_process_id)`
- System attempts graceful shutdown with interrupt sender first
- Falls back to process group kill if needed
- Updates status to `ExecutionProcessStatus::Killed`

### Post-Termination Cleanup

```rust
// 1. Commit changes to git (if auto-commit enabled)
self.git().commit(&worktree_path, &commit_message)?;

// 2. Update execution process status in DB
ExecutionProcess::update_completion(
    &pool,
    exec_id,
    status,  // Completed/Failed/Killed
    exit_code,
).await?;

// 3. Finalize task if no more actions
if self.should_finalize(&ctx) {
    Task::update_status(&pool, task.id, TaskStatus::InReview).await?;
}

// 4. Send notification
self.notification_service().notify(&title, &message).await;

// 5. Clean up in-memory stores
self.remove_child_from_store(&exec_id).await;
self.msg_stores.remove(&exec_id).await;
```

---

## Key Insights

### Why This Architecture Works for Multi-Agent Orchestration

1. **Isolation**: Git worktrees ensure no conflicts between parallel agents
2. **Resumability**: Session IDs enable conversation continuation
3. **Observability**: Real-time log streaming via MsgStore + SSE
4. **Reliability**: Process groups ensure complete cleanup
5. **Flexibility**: ExecutorAction chains support complex workflows

### Data Flow Summary

```
User Request
    ↓
Task Created
    ↓
Workspace Created (git worktree)
    ↓
Session Created (links workspace + executor)
    ↓
ExecutionProcess Spawned
    ↓
    ├─→ Setup Scripts (parallel across repos)
    ├─→ Claude Code CLI Agent (npx spawn with protocol)
    │       ├─→ Logs streamed via MsgStore
    │       ├─→ Diffs tracked in real-time
    │       └─→ Approvals handled via protocol
    └─→ Dev Server (optional, runs alongside)
    ↓
Exit Monitor Detects Completion
    ↓
    ├─→ Graceful shutdown (SIGINT → SIGTERM → SIGKILL)
    ├─→ Commit changes to git
    ├─→ Update DB status
    └─→ Finalize task → InReview
```

### Configuration Points

- **Executor Profiles**: Define which agent variant to use (e.g., ClaudeCode PLAN mode)
- **Environment Variables**: Custom env per executor
- **Working Directory**: Agent runs in specific repo subdirectory
- **Approval Modes**: Plan mode, full approval, or bypass
- **Session Resumption**: Enables multi-turn conversations
