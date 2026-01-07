# Spec ID: 000001 - Global CLI Python Script

**Status**: Draft
**Type**: Feature
**Created**: 2026-01-06
**Author**: @zhang

## Update Log
(Updates will be added here as spec evolves)

---

## Critical Questions

1. **What problem are we solving?**
   - Developers need a unified interface to run tests and examples without remembering multiple cargo commands
   - Current workflow requires different cargo commands for different tasks (e.g., `cargo test`, `cargo run --example X`)
   - No single entry point for common development tasks

2. **Who is affected by this?**
   - Developers working on lite-agent-lib
   - Contributors who need to verify their changes
   - Users who want to try example implementations

3. **What are the constraints?**
   - Must be a Python script (as requested)
   - Must work cross-platform (Windows, Linux, macOS)
   - Should be extensible for future commands
   - Must use existing cargo commands under the hood

4. **What are the risks?**
   - Python dependency required (though Python is commonly available)
   - Maintenance overhead of keeping CLI in sync with cargo project structure
   - Potential confusion if CLI behavior diverges from direct cargo commands

## What

A unified command-line interface Python script (`lite_agent_cli.py`) that provides convenient access to common development tasks.

### Specific Functionality

**Initial commands:**
- `python lite_agent_cli.py test` - Run all unit tests
- `python lite_agent_cli.py sample <name>` - Run a specific example by name
- `python lite_agent_cli.py --help` - Display usage information

### Scope (Included)
- Wrapper around existing `cargo test` command
- Wrapper around existing `cargo run --example <name>` command
- Basic argument parsing and validation
- Helpful error messages for invalid commands
- Extensible command structure for future additions

### Scope (Excluded)
- Modifying cargo test behavior or configuration
- Running integration tests differently from unit tests
- New testing or example execution logic
- Installation or dependency management
- Build optimization

### Expected Outcomes
- Single script location: project root as `lite_agent_cli.py`
- Consistent command interface: `python lite_agent_cli.py <command> [args]`
- Clear error messages for invalid usage
- Easy to add new commands in the future

## Why

### Business/Technical Rationale
- Reduces cognitive load: one interface instead of multiple cargo commands
- Lowers barrier to entry for new contributors
- Provides a foundation for future tooling (e.g., benchmarks, code generation, CI helpers)
- Python chosen for cross-platform compatibility and ease of extension

### User Impact
- Faster development workflow with shorter, memorable commands
- Less documentation needed (single CLI reference vs. multiple cargo commands)
- Better discoverability of available examples through `--help` or `list` command

### Priority
- Medium: Improves developer experience but not blocking

### Connection to Project Goals
- Supports project goal of being "lightweight" and "easy to use"
- Aligns with the reference implementation nature of the library
- Enables better testing and verification workflows

## How

### Approach

Create a Python script with a command-dispatch pattern using argparse for CLI parsing. The script will:
1. Parse command-line arguments
2. Validate commands and arguments
3. Execute corresponding cargo commands as subprocesses
4. Stream output back to console
5. Return appropriate exit codes

### Architecture

```
lite_agent_cli.py
├── Command Registry
│   ├── test: cargo test
│   ├── sample <name>: cargo run --example <name>
│   └── (future commands can be added here)
├── Argument Parser (argparse)
├── Subprocess Executor
└── Error Handler
```

### Components Affected
- **New file**: `lite_agent_cli.py` (project root)
- **No changes to existing code**: This is purely additive

### Dependencies
- Python 3.6+ (standard library only)
- `argparse` (standard library)
- `subprocess` (standard library)
- Cargo (must be available in PATH)

### Implementation Details

**Command: test**
```bash
python lite_agent_cli.py test
# Executes: cargo test
```

**Command: sample**
```bash
python lite_agent_cli.py sample basic_echo
# Executes: cargo run --example basic_echo
```

**Error handling:**
- Invalid command: Show help and list available commands
- Missing sample name: Show error and list available samples
- Cargo not found: Clear error message
- Test/sample failure: Propagate cargo exit code

### Alternatives Considered

**Alternative 1: Shell script** - Not chosen because:
- Cross-platform compatibility is harder (batch vs. bash)
- Python provides better argument parsing
- More extensible for future complex commands

**Alternative 2: Makefile** - Not chosen because:
- Make not universally available on Windows
- Python is more flexible for programmatic logic
- Make adds another dependency with different syntax

**Alternative 3: Cargo alias** - Not chosen because:
- Less discoverable than `--help`
- Harder to provide custom error messages
- Limited extensibility for complex future commands

## Verification

### Unit Tests
- [ ] Test command parsing with valid commands
- [ ] Test command parsing with invalid commands
- [ ] Test argument parsing for sample command
- [ ] Test help message generation
- [ ] Test error handling for missing cargo
- [ ] Test subprocess execution with mock cargo commands
- [ ] Test exit code propagation

### Integration Tests
- [ ] Execute `python lite_agent_cli.py test` and verify all tests run
- [ ] Execute `python lite_agent_cli.py sample basic_echo` and verify it runs
- [ ] Execute `python lite_agent_cli.py sample nonexistent` and verify proper error
- [ ] Execute `python lite_agent_cli.py invalid` and verify help is shown
- [ ] Verify cargo exit codes are properly propagated
- [ ] Test on Windows, Linux, and macOS if possible

**No integration tests needed for:**
- Python argparse functionality (standard library, well-tested)
- subprocess module (standard library, well-tested)

### Acceptance Criteria
- [ ] Script runs from project root without installation
- [ ] `python lite_agent_cli.py test` executes all unit tests successfully
- [ ] `python lite_agent_cli.py sample <name>` runs any valid example
- [ ] Invalid commands show helpful error messages
- [ ] Exit codes match cargo exit codes
- [ ] Help command displays usage information
- [ ] Script works on Windows (Git Bash or PowerShell)
- [ ] All unit tests pass

### Manual Testing Steps
- [ ] Run `python lite_agent_cli.py --help` and verify output
- [ ] Run `python lite_agent_cli.py test` and verify tests execute
- [ ] Run `python lite_agent_cli.py sample basic_echo` and verify example runs
- [ ] Run `python lite_agent_cli.py` (no args) and verify help is shown
- [ ] Run `python lite_agent_cli.py invalid_command` and verify error message
- [ ] Intentionally break a test and verify exit code is non-zero

## Additional Notes

### Future Extensions
The command structure should allow easy addition of:
- `python lite_agent_cli.py build` - Build all crates
- `python lite_agent_cli.py lint` - Run clippy
- `python lite_agent_cli.py docs` - Generate documentation
- `python lite_agent_cli.py clean` - Clean build artifacts
- `python lite_agent_cli.py bench` - Run benchmarks

### Naming Considerations
- Script name uses underscores: `lite_agent_cli.py` (Python convention)
- Commands use hyphens or underscores? (Recommend: match cargo convention, use hyphens if we add multi-word commands)

### Exit Codes
- Success: 0
- Command execution failure: Propagate cargo exit code
- Invalid CLI usage: 1 (with helpful message)
- Missing dependencies: 2 (with helpful message)

### Output Formatting
- Standard cargo output should pass through unchanged
- CLI-specific messages should be clearly distinguishable (e.g., `[lite-agent-cli]` prefix)

---

## Implementation Notes

When implementing this spec, create: `spec_20260106_000001_feature_--global-cli-python-script.implementation.md`

Track all progress, decisions, and test results in the implementation log following the rules in `.agent/spec-rules.md`.
