# Architecture

## High-Level Structure

```
┌─────────────────────────────────────────────────────────┐
│ Frontend (React + TypeScript)                           │
│ - Pages: Projects, Kanban Board, Logs, Settings        │
│ - Real-time updates via SSE                             │
└────────────────────┬────────────────────────────────────┘
                     │ REST API + SSE
┌────────────────────▼────────────────────────────────────┐
│ Backend (Rust + Axum)                                   │
│ ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐ │
│ │   Routes    │  │   Services   │  │   Executors     │ │
│ │  (API layer)│→ │(Business logic)│→ │(Agent spawning)│ │
│ └─────────────┘  └──────────────┘  └─────────────────┘ │
│                          │                              │
└──────────────────────────┼──────────────────────────────┘
                           │
                  ┌────────▼─────────┐
                  │ SQLite Database  │
                  │ - Projects       │
                  │ - Tasks          │
                  │ - Workspaces     │
                  │ - Sessions       │
                  │ - ExecProcesses  │
                  └──────────────────┘
```

## Crate Organization (Backend)

### Core Crates
- **server** - Main server, Axum routes, API endpoints
- **db** - SQLite models and migrations
- **services** - Business logic layer
- **executors** - Agent executor implementations
- **deployment** - Service orchestration abstraction
- **local-deployment** - Local deployment implementation
- **remote** - Remote server support

### Supporting Crates
- **utils** - Shared utilities
- **review** - Code review functionality

## Frontend Structure

```
frontend/src/
├── pages/          # Main views (Projects, ProjectTasks, Logs, Settings)
├── components/     # Reusable UI components
├── contexts/       # React contexts for state
├── lib/            # API clients, utilities
└── hooks/          # Custom React hooks
```

## Data Flow

1. **Task Creation** → Creates DB entry → Creates git worktree
2. **Agent Execution** → Spawns process → Streams logs → Updates DB
3. **Real-time Updates** → SSE stream → Frontend updates UI
4. **Code Changes** → Git operations → Diff generation → Review workflow
