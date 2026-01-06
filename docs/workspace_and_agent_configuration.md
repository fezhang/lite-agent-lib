# Workspace and Agent Configuration Guide

## Overview

This document explains the relationship between **Agents** and **Workspaces**, and details all configuration options for both.

## Conceptual Model

### Agent vs. Workspace

**Agent**: The actual process/program that does work (e.g., shell command, LLM, coding assistant)

**Workspace**: An isolated environment where the agent executes (optional)

```
┌─────────────────────────────────────────────┐
│           AgentConfig                       │
│  ┌────────────────────────────────────┐    │
│  │ work_dir: /path/to/project         │    │
│  │ env: { "API_KEY": "..." }          │    │
│  │ timeout: 60s                        │    │
│  │ custom: { ... }                     │    │
│  │                                     │    │
│  │ workspace: Some(WorkspaceConfig)   │◄───┼─── Optional
│  │   ┌──────────────────────────────┐ │    │
│  │   │ isolation_type: GitWorktree  │ │    │
│  │   │ work_dir: /path/to/project   │ │    │
│  │   │ base_branch: "main"          │ │    │
│  │   └──────────────────────────────┘ │    │
│  └────────────────────────────────────┘    │
└─────────────────────────────────────────────┘
         │
         │ AgentExecutor::spawn(config, input)
         ▼
┌─────────────────────────────────────────────┐
│         SpawnedAgent (Process)              │
│                                             │
│  Executes in:                               │
│  - No workspace: /path/to/project           │
│  - GitWorktree: /tmp/worktrees/session-123  │
│  - TempDir: /tmp/agent-temp-456             │
└─────────────────────────────────────────────┘
```

## Relationship: Agent ↔ Workspace

### 1. Agent Executes IN a Workspace

The workspace is the **environment** where the agent runs:

- **No Workspace** (`workspace: None`): Agent executes directly in `work_dir`
- **GitWorktree Workspace**: Agent executes in an isolated git worktree
- **TempDir Workspace**: Agent executes in a temporary directory

### 2. Multiple Agents Can Share Workspace Config

```rust
// Same workspace config for multiple agents
let workspace_config = WorkspaceConfig {
    work_dir: PathBuf::from("/path/to/repo"),
    isolation_type: IsolationType::GitWorktree {
        repo_path: PathBuf::from("/path/to/repo"),
        branch_prefix: "agent-task".to_string(),
    },
    base_branch: "main".to_string(),
};

// Agent 1: Shell agent
let config1 = AgentConfig::new(PathBuf::from("/path/to/repo"))
    .with_workspace(workspace_config.clone());

// Agent 2: Python agent
let config2 = AgentConfig::new(PathBuf::from("/path/to/repo"))
    .with_workspace(workspace_config.clone());
```

### 3. One Agent Per Workspace Instance

Each agent spawn creates its own workspace instance:

```rust
// Session 1: Creates worktree at /tmp/worktrees/session-001
agent.spawn(&config, "task 1").await?;

// Session 2: Creates worktree at /tmp/worktrees/session-002
agent.spawn(&config, "task 2").await?;

// Both run in parallel without conflicts
```

### 4. Workspace Lifecycle

```
┌──────────────────────────────────────────────────┐
│ Session Start                                    │
└──────────────────────────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│ WorkspaceManager::create_workspace()             │
│  - Creates git worktree (if IsolationType::Git)  │
│  - Returns WorkspacePath                         │
└──────────────────────────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│ Agent::spawn(config)                             │
│  - Spawns process in workspace path              │
│  - Agent executes isolated from others           │
└──────────────────────────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│ Agent completes                                  │
└──────────────────────────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────┐
│ WorkspaceManager::cleanup_workspace()            │
│  - Removes worktree                              │
│  - Cleans git metadata                           │
└──────────────────────────────────────────────────┘
```

## Complete Configuration Reference

### AgentConfig (All Parameters)

```rust
pub struct AgentConfig {
    /// Working directory for agent execution
    /// - If workspace is None: Agent runs here directly
    /// - If workspace is Some: This is the base directory
    pub work_dir: PathBuf,

    /// Environment variables passed to agent process
    /// Example: { "API_KEY": "sk-...", "DEBUG": "1" }
    pub env: HashMap<String, String>,

    /// Optional workspace isolation
    /// - None: No isolation, run directly in work_dir
    /// - Some(config): Create isolated workspace
    pub workspace: Option<WorkspaceConfig>,

    /// Maximum execution time
    /// - None: No timeout
    /// - Some(duration): Agent killed after timeout
    pub timeout: Option<Duration>,

    /// Agent-specific custom configuration
    /// Use for agent-specific settings not covered above
    /// Example: { "model": "gpt-4", "temperature": 0.7 }
    pub custom: serde_json::Value,
}
```

#### Building AgentConfig

```rust
use std::time::Duration;
use std::collections::HashMap;

// Method 1: Default
let config = AgentConfig::default();

// Method 2: Minimal
let config = AgentConfig::new(PathBuf::from("/tmp/work"));

// Method 3: Builder pattern (recommended)
let config = AgentConfig::new(PathBuf::from("/tmp/work"))
    .add_env("API_KEY", "secret-key")
    .add_env("DEBUG", "1")
    .with_timeout(Duration::from_secs(300))
    .with_custom(serde_json::json!({
        "model": "gpt-4",
        "temperature": 0.7
    }));

// Method 4: With workspace
let config = AgentConfig::new(PathBuf::from("/path/to/repo"))
    .with_workspace(WorkspaceConfig {
        work_dir: PathBuf::from("/path/to/repo"),
        isolation_type: IsolationType::GitWorktree {
            repo_path: PathBuf::from("/path/to/repo"),
            branch_prefix: "agent".to_string(),
        },
        base_branch: "main".to_string(),
    })
    .with_timeout(Duration::from_secs(600));

// Method 5: Bulk env vars
let mut env = HashMap::new();
env.insert("VAR1".to_string(), "value1".to_string());
env.insert("VAR2".to_string(), "value2".to_string());

let config = AgentConfig::new(PathBuf::from("/tmp"))
    .with_env(env);
```

### WorkspaceConfig (All Parameters)

```rust
pub struct WorkspaceConfig {
    /// Base working directory
    /// This is the original directory (before isolation)
    pub work_dir: PathBuf,

    /// Type of isolation to use
    pub isolation_type: IsolationType,

    /// Base branch for git worktree isolation
    /// Only used when isolation_type is GitWorktree
    pub base_branch: String,
}

pub enum IsolationType {
    /// No isolation - use work_dir directly
    /// Agent runs in the original directory
    None,

    /// Git worktree isolation
    /// Creates isolated git worktree + branch
    GitWorktree {
        /// Path to the git repository
        repo_path: PathBuf,

        /// Prefix for branch names
        /// Actual branch: "{branch_prefix}-{session_id}"
        /// Example: "agent-task-550e8400..."
        branch_prefix: String,
    },

    /// Temporary directory isolation
    /// Creates temp directory for agent execution
    TempDir,
}
```

#### Building WorkspaceConfig

```rust
// Option 1: No isolation
let workspace = WorkspaceConfig {
    work_dir: PathBuf::from("/path/to/project"),
    isolation_type: IsolationType::None,
    base_branch: String::new(), // Not used
};

// Option 2: Git worktree isolation
let workspace = WorkspaceConfig {
    work_dir: PathBuf::from("/path/to/repo"),
    isolation_type: IsolationType::GitWorktree {
        repo_path: PathBuf::from("/path/to/repo"),
        branch_prefix: "agent-task".to_string(),
    },
    base_branch: "main".to_string(),
};

// Option 3: Temp directory isolation
let workspace = WorkspaceConfig {
    work_dir: PathBuf::from("/path/to/project"),
    isolation_type: IsolationType::TempDir,
    base_branch: String::new(), // Not used
};
```

## Isolation Types Explained

### 1. No Isolation (`IsolationType::None`)

**Use When:**
- Single agent at a time
- Agent doesn't modify files
- Read-only operations
- Simplest deployment

**Behavior:**
- Agent runs directly in `work_dir`
- No overhead
- No isolation

**Example:**
```rust
let config = AgentConfig::new(PathBuf::from("/data"))
    .with_workspace(WorkspaceConfig {
        work_dir: PathBuf::from("/data"),
        isolation_type: IsolationType::None,
        base_branch: String::new(),
    });

// Agent runs in: /data
```

### 2. Git Worktree Isolation (`IsolationType::GitWorktree`)

**Use When:**
- Multiple agents working on same repo
- Parallel agent execution needed
- Want to preserve git history
- Need branch-based isolation

**Behavior:**
- Creates git worktree: `/tmp/worktrees/{session_id}/`
- Creates new branch: `{branch_prefix}-{session_id}`
- Based on `base_branch` (usually "main")
- Full git history available
- Changes isolated to branch

**Example:**
```rust
let config = AgentConfig::new(PathBuf::from("/repo"))
    .with_workspace(WorkspaceConfig {
        work_dir: PathBuf::from("/repo"),
        isolation_type: IsolationType::GitWorktree {
            repo_path: PathBuf::from("/repo"),
            branch_prefix: "agent-feature".to_string(),
        },
        base_branch: "main".to_string(),
    });

// Session ID: "abc123"
// Agent runs in: /tmp/worktrees/abc123/
// Branch: agent-feature-abc123
// Based on: main
```

**Directory Structure:**
```
Original Repo:
/repo/.git/worktrees/abc123/    ← Git metadata
/repo/.git/refs/heads/agent-feature-abc123  ← Branch

Worktree Location:
/tmp/worktrees/abc123/          ← Agent executes here
/tmp/worktrees/abc123/.git      ← Points to main repo
```

### 3. Temporary Directory Isolation (`IsolationType::TempDir`)

**Use When:**
- Don't need git integration
- Temporary work only
- Fastest cleanup
- No history needed

**Behavior:**
- Creates temp directory
- No git operations
- Cleaned up after execution

**Example:**
```rust
let config = AgentConfig::new(PathBuf::from("/work"))
    .with_workspace(WorkspaceConfig {
        work_dir: PathBuf::from("/work"),
        isolation_type: IsolationType::TempDir,
        base_branch: String::new(),
    });

// Agent runs in: /tmp/agent-temp-{random}/
```

## Use Cases and Patterns

### Use Case 1: Single Agent, No Conflicts

```rust
// Simple shell command execution
let config = AgentConfig::new(PathBuf::from("/data"));
// No workspace needed

agent.spawn(&config, "ls -la").await?;
```

### Use Case 2: Parallel Agents on Same Repo

```rust
// Configure workspace for isolation
let workspace = WorkspaceConfig {
    work_dir: PathBuf::from("/repo"),
    isolation_type: IsolationType::GitWorktree {
        repo_path: PathBuf::from("/repo"),
        branch_prefix: "task".to_string(),
    },
    base_branch: "main".to_string(),
};

// Spawn multiple agents in parallel
let config1 = AgentConfig::new(PathBuf::from("/repo"))
    .with_workspace(workspace.clone());
let config2 = AgentConfig::new(PathBuf::from("/repo"))
    .with_workspace(workspace.clone());

// Both run simultaneously without conflicts
let agent1 = agent.spawn(&config1, "implement feature A").await?;
let agent2 = agent.spawn(&config2, "implement feature B").await?;
```

### Use Case 3: Agent with Environment and Timeout

```rust
let config = AgentConfig::new(PathBuf::from("/work"))
    .add_env("API_KEY", "sk-...")
    .add_env("DEBUG", "1")
    .with_timeout(Duration::from_secs(300))
    .with_custom(serde_json::json!({
        "model": "gpt-4",
        "temperature": 0.7
    }));

agent.spawn(&config, "analyze data").await?;
```

### Use Case 4: Session Continuation

```rust
// First spawn
let config = AgentConfig::new(PathBuf::from("/work"));
let spawned = agent.spawn(&config, "start task").await?;
let session_id = "session-123";

// Continue later (same workspace)
agent.spawn_follow_up(&config, "continue task", session_id).await?;
```

## Configuration Decision Tree

```
Do you need isolation?
│
├─ NO → Use AgentConfig::new() without workspace
│        - Simplest
│        - Direct execution in work_dir
│
└─ YES → Do you need git integration?
    │
    ├─ YES → Use IsolationType::GitWorktree
    │        - Parallel agents
    │        - Branch isolation
    │        - Git history preserved
    │
    └─ NO → Use IsolationType::TempDir
             - Fast cleanup
             - No git overhead
             - Temporary work
```

## Advanced Topics

### Workspace Manager Internals

The `WorkspaceManager` handles workspace lifecycle:

```rust
pub struct WorkspaceManager {
    base_dir: PathBuf,  // Where worktrees are created
    locks: Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
}

impl WorkspaceManager {
    // Create workspace (called by SessionManager)
    pub async fn create_workspace(
        &self,
        session_id: &str,
        config: WorkspaceConfig,
    ) -> Result<WorkspacePath, WorkspaceError>;

    // Cleanup workspace
    pub async fn cleanup_workspace(
        &self,
        path: &WorkspacePath
    ) -> Result<(), WorkspaceError>;
}
```

### Locking Strategy

To prevent race conditions, the `WorkspaceManager` uses per-path locks:

```rust
// Global lock map: path → mutex
locks: Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>

// When creating workspace:
let lock = get_or_create_lock(path);
let _guard = lock.lock().await;  // Acquire lock
// Create worktree
// Lock released on drop
```

This ensures:
- No concurrent worktree creation at same path
- No concurrent cleanup of same worktree
- Safe parallel workspace operations

## Summary

### Agent Configuration
- **work_dir**: Where agent executes (base directory)
- **env**: Environment variables
- **workspace**: Optional isolation config
- **timeout**: Max execution time
- **custom**: Agent-specific settings

### Workspace Configuration
- **work_dir**: Base directory
- **isolation_type**: None | GitWorktree | TempDir
- **base_branch**: Git base branch (for worktrees)

### Relationship
- Agent executes **IN** a workspace
- Workspace provides **isolation**
- One workspace instance per agent spawn
- Multiple agents can use same workspace config (different instances)

### When to Use Workspace
- ✅ Parallel agents on same repo → GitWorktree
- ✅ Temporary isolated work → TempDir
- ❌ Single agent, no conflicts → No workspace
- ❌ Read-only operations → No workspace
