# Agent Instructions Directory

This directory contains mandatory rules and instructions for coding agents working on this project.

## Purpose

The `.agent/` folder provides:
- **Automated rule enforcement** - Agents read these files before performing tasks
- **Consistent behavior** - All agents follow the same protocols
- **Project-specific guidelines** - Tailored to this project's workflow
- **Traceability** - Clear documentation of what agents should do

## Structure

```
.agent/
├── README.md          # This file - explains the agent instructions system
└── spec-rules.md      # MANDATORY rules for spec implementation
```

## How Agents Should Use This Directory

### 1. Automatic Discovery
Coding agents should automatically check for `.agent/` folder in the project root and read relevant instruction files before performing tasks.

### 2. Task-Specific Rules
When working on specific tasks, agents should read the appropriate rules file:

| Task Type | Read This File |
|-----------|---------------|
| Implementing a spec | `.agent/spec-rules.md` |
| (Future) Code review | `.agent/code-review-rules.md` |
| (Future) Testing | `.agent/testing-rules.md` |

### 3. Priority Order
If multiple instruction sources exist, follow this priority:

1. `.agent/` rules (highest priority - project-specific)
2. Built-in agent capabilities
3. General best practices (lowest priority)

## Current Rules Files

### spec-rules.md
**Purpose**: Defines mandatory protocols for specification implementation

**Key Requirements**:
- Spec file naming conventions
- Implementation log creation and maintenance (append-only)
- Completion criteria (unit tests + integration tests)
- Error handling and logging protocols
- Step-by-step implementation workflow

**When to read**: Before implementing any spec file in the `specs/` folder

## Adding New Rule Files

As the project evolves, you can add new rule files for different agent tasks:

**Suggested future additions**:
- `code-review-rules.md` - Standards for reviewing pull requests
- `testing-rules.md` - Testing strategy and requirements
- `deployment-rules.md` - Deployment and release procedures
- `documentation-rules.md` - Documentation standards
- `code-style-rules.md` - Project-specific coding conventions

## For Human Developers

This folder is primarily for agent automation, but humans can benefit too:

- **Review rules** to understand what agents will do automatically
- **Update rules** when project processes change
- **Reference rules** when manually performing similar tasks
- **Verify compliance** by checking if agents followed the rules

## Rules File Format

All rules files should:
- Use clear, imperative language ("MUST", "NEVER", "ALWAYS")
- Include specific examples
- Provide checklists when applicable
- Explain the "why" behind critical rules
- Be concise but comprehensive

## Maintaining This Directory

**When to update**:
- Project workflow changes
- New automation requirements emerge
- Issues found in current rules
- New task types need agent support

**How to update**:
1. Modify or create the appropriate rules file
2. Update this README if adding new files
3. Test with agent to ensure rules are clear
4. Commit changes with descriptive message

## Questions?

For questions about agent rules or to propose new rules, consult the project lead or open a discussion.

---

**Note**: This directory uses a leading dot (`.agent/`) to indicate it's a configuration/metadata folder, similar to `.git/`, `.vscode/`, etc.
