# Vibe-Kanban Reference Documentation

This folder contains reference documentation from the original vibe-kanban project that informed the design of lite-agent-lib.

## Source

These documents were originally created for the [vibe-kanban](https://github.com/vibe-teams/vibe-kanban) project, a multi-agent orchestration platform for coding agents. The lite-agent-lib project extracts and generalizes the core agent management patterns from vibe-kanban into a reusable library.

## Contents

- **architecture.md** - System architecture and design patterns from vibe-kanban
- **multi-agent-system.md** - Multi-agent orchestration concepts
- **multi-agent-abstraction.md** - Supporting heterogeneous agent architectures
- **agent-lifecycle.md** - Agent spawning, distribution, and termination patterns
- **protocol-communication.md** - Bidirectional control protocol design (Claude Code specific)

## How These Informed lite-agent-lib

The lite-agent-lib project learned from vibe-kanban's patterns:

1. **AgentExecutor Trait** - Based on vibe-kanban's `StandardCodingAgentExecutor` but generalized for any agent type
2. **Log Normalization** - Unified `NormalizedEntry` format inspired by vibe-kanban's log abstraction
3. **Workspace Isolation** - Git worktree patterns adapted from `WorktreeManager`
4. **Protocol Handling** - JSON streaming and bidirectional control patterns
5. **Session Management** - Conversation continuity and execution tracking

## Key Differences

lite-agent-lib differs from vibe-kanban in several ways:

- **Generic vs. Coding-Specific**: Supports any agent type, not just coding agents
- **Library vs. Application**: Reusable library, not a complete application
- **Simplified**: Focuses on core patterns, removes UI and project management
- **REST API**: Python integration via HTTP/SSE instead of native integration
- **No Database**: In-memory session state instead of SQLite

## License

These reference documents are provided for architectural understanding. The vibe-kanban project is licensed under Apache 2.0.

lite-agent-lib is an independent library inspired by these patterns, also licensed under Apache 2.0.
