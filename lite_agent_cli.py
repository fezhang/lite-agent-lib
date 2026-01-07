#!/usr/bin/env python3
"""
lite-agent-cli - Unified command-line interface for lite-agent-lib development tasks.

This script provides convenient access to common development operations like running
tests and examples without needing to remember multiple cargo commands.
"""

import argparse
import subprocess
import sys
from typing import List, Optional


# Available examples in the project
AVAILABLE_EXAMPLES = [
    "agent_runner",
    "basic_echo",
    "basic_shell",
    "custom_agent",
]


def print_error(message: str) -> None:
    """Print an error message with CLI prefix."""
    print(f"[lite-agent-cli] ERROR: {message}", file=sys.stderr)


def print_info(message: str) -> None:
    """Print an info message with CLI prefix."""
    print(f"[lite-agent-cli] {message}")


def check_cargo_available() -> bool:
    """Check if cargo is available in PATH."""
    try:
        result = subprocess.run(
            ["cargo", "--version"],
            capture_output=True,
            text=True,
            timeout=5
        )
        return result.returncode == 0
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return False


def run_command(args: List[str], description: str) -> int:
    """
    Run a command and stream its output.

    Args:
        args: Command and arguments to execute
        description: Human-readable description of what's being executed

    Returns:
        Exit code from the command
    """
    print_info(f"Running: {description}")
    print_info(f"Executing: {' '.join(args)}\n")

    try:
        # Run the command and stream output directly to console
        result = subprocess.run(args, stdout=None, stderr=None)
        return result.returncode
    except FileNotFoundError:
        print_error("Command not found. Make sure cargo is installed and in PATH.")
        return 2
    except KeyboardInterrupt:
        print_info("\nOperation cancelled by user.")
        return 130
    except Exception as e:
        print_error(f"Unexpected error: {e}")
        return 1


def cmd_test(args: argparse.Namespace) -> int:
    """Run all unit tests using cargo test."""
    cargo_args = ["cargo", "test"]

    # Add verbose flag if requested
    if args.verbose:
        cargo_args.append("--verbose")

    # Add any extra arguments directly to cargo
    if args.extra:
        cargo_args.extend(args.extra)

    return run_command(cargo_args, "Unit tests")


def cmd_sample(args: argparse.Namespace) -> int:
    """Run a specific example using cargo run --example."""
    if not args.name:
        print_error("Sample name is required.")
        print_info(f"Available samples: {', '.join(AVAILABLE_EXAMPLES)}")
        print_info("Usage: python lite_agent_cli.py sample <name>")
        return 1

    # Check if the example exists
    if args.name not in AVAILABLE_EXAMPLES:
        print_error(f"Sample '{args.name}' not found.")
        print_info(f"Available samples: {', '.join(AVAILABLE_EXAMPLES)}")
        return 1

    cargo_args = ["cargo", "run", "--example", args.name]

    # Add any extra arguments directly to cargo
    if args.extra:
        cargo_args.extend(args.extra)

    return run_command(cargo_args, f"Example: {args.name}")


def main() -> int:
    """Main entry point for the CLI."""
    parser = argparse.ArgumentParser(
        prog="lite_agent_cli.py",
        description="Unified CLI for lite-agent-lib development tasks",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python lite_agent_cli.py test              Run all unit tests
  python lite_agent_cli.py sample basic_echo Run the basic_echo example
  python lite_agent_cli.py test --verbose    Run tests with verbose output

Available samples:
  """ + ", ".join(AVAILABLE_EXAMPLES)
    )

    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Enable verbose output for commands"
    )

    subparsers = parser.add_subparsers(
        dest="command",
        title="Available commands",
        metavar="COMMAND",
        help="Command to execute"
    )

    # 'test' command
    test_parser = subparsers.add_parser(
        "test",
        help="Run all unit tests",
        description="Run all unit tests using cargo test",
        add_help=False
    )
    test_parser.add_argument(
        "extra",
        nargs=argparse.REMAINDER,
        help="Extra arguments to pass to cargo test"
    )

    # 'sample' command
    sample_parser = subparsers.add_parser(
        "sample",
        help="Run a specific example",
        description="Run a specific example using cargo run --example",
        add_help=False
    )
    sample_parser.add_argument(
        "name",
        nargs="?",
        help="Name of the example to run"
    )
    sample_parser.add_argument(
        "extra",
        nargs=argparse.REMAINDER,
        help="Extra arguments to pass to the example"
    )

    # Parse arguments
    try:
        args = parser.parse_args()
    except SystemExit as e:
        # argparse calls sys.exit on error, convert to return code
        return e.code if e.code is not None else 2

    # Check if cargo is available
    if not check_cargo_available():
        print_error("Cargo is not installed or not in PATH.")
        print_info("Please install Rust and Cargo from https://rustup.rs/")
        return 2

    # Dispatch to appropriate command handler
    if args.command == "test":
        return cmd_test(args)
    elif args.command == "sample":
        return cmd_sample(args)
    else:
        # No command specified or invalid command
        parser.print_help()
        return 1


if __name__ == "__main__":
    sys.exit(main())
