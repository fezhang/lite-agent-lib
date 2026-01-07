"""
Unit tests for lite_agent_cli.py

These tests verify command parsing, argument handling, and error conditions.
"""

import unittest
import sys
import subprocess
import argparse
from unittest.mock import patch, MagicMock, call
from io import StringIO

# Import the CLI module
import lite_agent_cli


class TestCargoAvailability(unittest.TestCase):
    """Test cargo availability checking."""

    @patch('subprocess.run')
    def test_cargo_available(self, mock_run):
        """Test that cargo is detected when available."""
        mock_run.return_value = MagicMock(returncode=0)
        self.assertTrue(lite_agent_cli.check_cargo_available())

    @patch('subprocess.run')
    def test_cargo_not_found(self, mock_run):
        """Test that cargo not found is handled correctly."""
        mock_run.side_effect = FileNotFoundError()
        self.assertFalse(lite_agent_cli.check_cargo_available())

    @patch('subprocess.run')
    def test_cargo_timeout(self, mock_run):
        """Test that cargo timeout is handled correctly."""
        mock_run.side_effect = subprocess.TimeoutExpired('cargo', 5)
        self.assertFalse(lite_agent_cli.check_cargo_available())


class TestPrintFunctions(unittest.TestCase):
    """Test output functions."""

    def test_print_error(self):
        """Test error message printing."""
        output = StringIO()
        with patch('sys.stderr', output):
            lite_agent_cli.print_error("Test error")
        self.assertIn("[lite-agent-cli] ERROR: Test error", output.getvalue())

    def test_print_info(self):
        """Test info message printing."""
        output = StringIO()
        with patch('sys.stdout', output):
            lite_agent_cli.print_info("Test info")
        self.assertIn("[lite-agent-cli] Test info", output.getvalue())


class TestRunCommand(unittest.TestCase):
    """Test command execution."""

    @patch('subprocess.run')
    @patch('sys.stdout', new_callable=StringIO)
    def test_run_command_success(self, mock_stdout, mock_run):
        """Test successful command execution."""
        mock_run.return_value = MagicMock(returncode=0)
        result = lite_agent_cli.run_command(['echo', 'test'], 'Test command')
        self.assertEqual(result, 0)
        mock_run.assert_called_once()

    @patch('subprocess.run')
    @patch('sys.stdout', new_callable=StringIO)
    def test_run_command_failure(self, mock_stdout, mock_run):
        """Test command execution with failure."""
        mock_run.return_value = MagicMock(returncode=1)
        result = lite_agent_cli.run_command(['false'], 'Test command')
        self.assertEqual(result, 1)

    @patch('subprocess.run')
    @patch('sys.stderr', new_callable=StringIO)
    def test_run_command_not_found(self, mock_stderr, mock_run):
        """Test command execution when command not found."""
        mock_run.side_effect = FileNotFoundError()
        result = lite_agent_cli.run_command(['nonexistent'], 'Test command')
        self.assertEqual(result, 2)
        output = mock_stderr.getvalue()
        self.assertIn("not found", output)

    @patch('subprocess.run')
    @patch('sys.stdout', new_callable=StringIO)
    def test_run_command_keyboard_interrupt(self, mock_stdout, mock_run):
        """Test command execution with keyboard interrupt."""
        mock_run.side_effect = KeyboardInterrupt()
        result = lite_agent_cli.run_command(['sleep', '10'], 'Test command')
        self.assertEqual(result, 130)


class TestCmdTest(unittest.TestCase):
    """Test the 'test' command."""

    @patch('lite_agent_cli.run_command')
    def test_cmd_test_basic(self, mock_run):
        """Test basic test command."""
        mock_run.return_value = 0
        args = MagicMock(verbose=False, extra=None)
        result = lite_agent_cli.cmd_test(args)
        self.assertEqual(result, 0)
        mock_run.assert_called_once_with(['cargo', 'test'], 'Unit tests')

    @patch('lite_agent_cli.run_command')
    def test_cmd_test_verbose(self, mock_run):
        """Test test command with verbose flag."""
        mock_run.return_value = 0
        args = MagicMock(verbose=True, extra=None)
        result = lite_agent_cli.cmd_test(args)
        self.assertEqual(result, 0)
        mock_run.assert_called_once_with(['cargo', 'test', '--verbose'], 'Unit tests')

    @patch('lite_agent_cli.run_command')
    def test_cmd_test_with_extra_args(self, mock_run):
        """Test test command with extra arguments."""
        mock_run.return_value = 0
        args = MagicMock(verbose=False, extra=['--', '--nocapture'])
        result = lite_agent_cli.cmd_test(args)
        self.assertEqual(result, 0)
        mock_run.assert_called_once_with(
            ['cargo', 'test', '--', '--nocapture'],
            'Unit tests'
        )


class TestCmdSample(unittest.TestCase):
    """Test the 'sample' command."""

    @patch('lite_agent_cli.run_command')
    @patch('sys.stderr', new_callable=StringIO)
    @patch('sys.stdout', new_callable=StringIO)
    def test_cmd_sample_missing_name(self, mock_stdout, mock_stderr, mock_run):
        """Test sample command without name."""
        args = argparse.Namespace(name=None, extra=None)
        result = lite_agent_cli.cmd_sample(args)
        self.assertEqual(result, 1)
        stderr_output = mock_stderr.getvalue()
        stdout_output = mock_stdout.getvalue()
        self.assertIn("Sample name is required", stderr_output)
        self.assertIn("agent_runner", stdout_output)  # Should list available samples
        mock_run.assert_not_called()

    @patch('lite_agent_cli.run_command')
    @patch('sys.stderr', new_callable=StringIO)
    def test_cmd_sample_invalid_name(self, mock_stderr, mock_run):
        """Test sample command with invalid name."""
        args = argparse.Namespace(name='nonexistent', extra=None)
        result = lite_agent_cli.cmd_sample(args)
        self.assertEqual(result, 1)
        output = mock_stderr.getvalue()
        self.assertIn("not found", output)
        mock_run.assert_not_called()

    @patch('lite_agent_cli.run_command')
    def test_cmd_sample_valid(self, mock_run):
        """Test sample command with valid name."""
        mock_run.return_value = 0
        args = argparse.Namespace(name='basic_echo', extra=None)
        result = lite_agent_cli.cmd_sample(args)
        self.assertEqual(result, 0)
        mock_run.assert_called_once_with(
            ['cargo', 'run', '--example', 'basic_echo'],
            'Example: basic_echo'
        )

    @patch('lite_agent_cli.run_command')
    def test_cmd_sample_with_extra_args(self, mock_run):
        """Test sample command with extra arguments."""
        mock_run.return_value = 0
        args = argparse.Namespace(name='basic_shell', extra=['--', '--help'])
        result = lite_agent_cli.cmd_sample(args)
        self.assertEqual(result, 0)
        mock_run.assert_called_once_with(
            ['cargo', 'run', '--example', 'basic_shell', '--', '--help'],
            'Example: basic_shell'
        )


class TestMainFunction(unittest.TestCase):
    """Test the main function."""

    @patch('sys.argv', ['lite_agent_cli.py'])
    @patch('lite_agent_cli.check_cargo_available', return_value=True)
    @patch('sys.stdout', new_callable=StringIO)
    def test_main_no_command(self, mock_stdout, mock_cargo):
        """Test main function with no command (shows help)."""
        result = lite_agent_cli.main()
        self.assertEqual(result, 1)
        # Help should be printed
        output = mock_stdout.getvalue()
        self.assertIn("usage:", output.lower())

    @patch('sys.argv', ['lite_agent_cli.py', '--help'])
    @patch('lite_agent_cli.check_cargo_available', return_value=True)
    @patch('sys.stdout', new_callable=StringIO)
    def test_main_help_flag(self, mock_stdout, mock_cargo):
        """Test main function with --help flag."""
        # With our try-except, --help returns 0 instead of raising SystemExit
        result = lite_agent_cli.main()
        self.assertEqual(result, 0)
        output = mock_stdout.getvalue()
        self.assertIn("Unified CLI", output)

    @patch('sys.argv', ['lite_agent_cli.py', 'invalid_command'])
    @patch('lite_agent_cli.check_cargo_available', return_value=True)
    @patch('sys.stderr', new_callable=StringIO)
    def test_main_invalid_command(self, mock_stderr, mock_cargo):
        """Test main function with invalid command."""
        result = lite_agent_cli.main()
        # argparse exits with code 2 for invalid commands
        self.assertEqual(result, 2)
        # Error message should be printed to stderr
        output = mock_stderr.getvalue()
        self.assertIn("invalid choice", output.lower())

    @patch('sys.argv', ['lite_agent_cli.py', 'test'])
    @patch('lite_agent_cli.check_cargo_available', return_value=True)
    @patch('lite_agent_cli.cmd_test', return_value=0)
    def test_main_test_command(self, mock_test, mock_cargo):
        """Test main function with test command."""
        result = lite_agent_cli.main()
        self.assertEqual(result, 0)
        mock_test.assert_called_once()

    @patch('sys.argv', ['lite_agent_cli.py', 'sample', 'basic_echo'])
    @patch('lite_agent_cli.check_cargo_available', return_value=True)
    @patch('lite_agent_cli.cmd_sample', return_value=0)
    def test_main_sample_command(self, mock_sample, mock_cargo):
        """Test main function with sample command."""
        result = lite_agent_cli.main()
        self.assertEqual(result, 0)
        mock_sample.assert_called_once()

    @patch('sys.argv', ['lite_agent_cli.py', 'test'])
    @patch('lite_agent_cli.check_cargo_available', return_value=False)
    @patch('sys.stderr', new_callable=StringIO)
    def test_main_cargo_not_available(self, mock_stderr, mock_cargo):
        """Test main function when cargo is not available."""
        result = lite_agent_cli.main()
        self.assertEqual(result, 2)
        output = mock_stderr.getvalue()
        self.assertIn("not installed", output)


if __name__ == '__main__':
    # Import subprocess here for TimeoutExpired test
    import subprocess
    unittest.main()
