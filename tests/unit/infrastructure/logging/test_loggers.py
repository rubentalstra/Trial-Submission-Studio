"""Unit tests for logger implementations.

Tests verify that:
1. LoggerPort protocol is properly implemented
2. ConsoleLogger (moved from cli/logging_config.py) implements LoggerPort
3. NullLogger provides silent testing capability
"""

from io import StringIO
from pathlib import Path
import unittest

from rich.console import Console

from cdisc_transpiler.application.ports.services import LoggerPort
from cdisc_transpiler.infrastructure.logging import (
    ConsoleLogger,
    LogContext,
    LogLevel,
    NullLogger,
)


class TestLoggerPort(unittest.TestCase):
    """Test that logger implementations comply with LoggerPort protocol."""

    def test_console_logger_implements_loggerport(self):
        """ConsoleLogger should implement LoggerPort protocol."""
        logger = ConsoleLogger()
        self.assertIsInstance(logger, LoggerPort)

    def test_null_logger_implements_loggerport(self):
        """NullLogger should implement LoggerPort protocol."""
        logger = NullLogger()
        self.assertIsInstance(logger, LoggerPort)

    def test_loggerport_has_required_methods(self):
        """LoggerPort protocol should define all required methods."""
        required_methods = {"info", "success", "warning", "error", "debug"}
        protocol_methods = {
            name for name in dir(LoggerPort) if not name.startswith("_")
        }
        self.assertTrue(required_methods.issubset(protocol_methods))


class TestConsoleLogger(unittest.TestCase):
    """Test ConsoleLogger implementation (moved from cli/logging_config.py)."""

    def setUp(self):
        """Set up test fixtures."""
        self.buffer = StringIO()
        self.console = Console(file=self.buffer, force_terminal=True, width=80)
        self.logger = ConsoleLogger(console=self.console, verbosity=LogLevel.DEBUG)

    def test_initialization(self):
        """Logger should initialize with proper defaults."""
        logger = ConsoleLogger()
        self.assertEqual(logger.verbosity, 0)
        self.assertIsNone(logger._context)
        self.assertEqual(logger._stats["files_processed"], 0)

    def test_info_logging(self):
        """info() should output message to console."""
        self.logger.info("Test message")
        output = self.buffer.getvalue()
        self.assertIn("Test message", output)

    def test_success_logging(self):
        """success() should output message with success indicator."""
        self.logger.success("Operation complete")
        output = self.buffer.getvalue()
        self.assertIn("Operation complete", output)

    def test_warning_logging(self):
        """warning() should output message and increment warning count."""
        initial_warnings = self.logger._stats["warnings"]
        self.logger.warning("Warning message")
        output = self.buffer.getvalue()
        self.assertIn("Warning message", output)
        self.assertEqual(self.logger._stats["warnings"], initial_warnings + 1)

    def test_error_logging(self):
        """error() should output message and increment error count."""
        initial_errors = self.logger._stats["errors"]
        self.logger.error("Error message")
        output = self.buffer.getvalue()
        self.assertIn("Error message", output)
        self.assertEqual(self.logger._stats["errors"], initial_errors + 1)

    def test_debug_logging_with_verbosity(self):
        """debug() should only output when verbosity >= DEBUG."""
        # Debug logger (verbosity=2)
        self.logger.debug("Debug message")
        output = self.buffer.getvalue()
        self.assertIn("Debug message", output)

        # Normal logger (verbosity=0) - should not output
        self.buffer.truncate(0)
        self.buffer.seek(0)
        normal_logger = ConsoleLogger(console=self.console, verbosity=LogLevel.NORMAL)
        normal_logger.debug("Should not appear")
        output = self.buffer.getvalue()
        self.assertEqual(output.strip(), "")

    def test_context_management(self):
        """Logger should manage logging context."""
        self.logger.set_context(study_id="STUDY01", domain_code="DM")
        self.assertIsNotNone(self.logger._context)
        self.assertEqual(self.logger._context.study_id, "STUDY01")
        self.assertEqual(self.logger._context.domain_code, "DM")

        self.logger.clear_context()
        self.assertIsNone(self.logger._context)

    def test_stats_tracking(self):
        """Logger should track processing statistics."""
        initial_stats = self.logger.get_stats()
        self.assertEqual(initial_stats["files_processed"], 0)

        self.logger.log_file_loaded("test.csv", 100, 10)
        stats = self.logger.get_stats()
        self.assertEqual(stats["files_processed"], 1)

        self.logger.reset_stats()
        stats = self.logger.get_stats()
        self.assertEqual(stats["files_processed"], 0)

    def test_log_study_start(self):
        """log_study_start() should output study information."""
        self.logger.log_study_start(
            study_id="STUDY01",
            study_folder=Path("/data/study01"),
            output_format="xpt",
            supported_domains=["DM", "EX"],
        )
        output = self.buffer.getvalue()
        self.assertIn("STUDY01", output)

    def test_log_domain_start(self):
        """log_domain_start() should output domain processing header."""
        self.logger.log_domain_start(
            domain_code="DM",
            files=[(Path("dm.csv"), "standard")],
        )
        output = self.buffer.getvalue()
        self.assertIn("DM", output)
        self.assertIn("dm.csv", output)


class TestNullLogger(unittest.TestCase):
    """Test NullLogger for silent testing."""

    def test_null_logger_produces_no_output(self):
        """NullLogger should not produce any output."""
        logger = NullLogger()

        # Call all logging methods - should not raise errors
        logger.info("Info message")
        logger.success("Success message")
        logger.warning("Warning message")
        logger.error("Error message")
        logger.debug("Debug message")

        # NullLogger should complete without errors (no assertions needed for output)

    def test_null_logger_has_all_methods(self):
        """NullLogger should have all LoggerPort methods."""
        logger = NullLogger()
        self.assertTrue(hasattr(logger, "info"))
        self.assertTrue(hasattr(logger, "success"))
        self.assertTrue(hasattr(logger, "warning"))
        self.assertTrue(hasattr(logger, "error"))
        self.assertTrue(hasattr(logger, "debug"))


class TestLogContext(unittest.TestCase):
    """Test LogContext dataclass."""

    def test_log_context_creation(self):
        """LogContext should initialize with defaults."""
        context = LogContext()
        self.assertEqual(context.study_id, "")
        self.assertEqual(context.domain_code, "")
        self.assertIsNotNone(context.start_time)

    def test_log_context_elapsed_time(self):
        """LogContext should calculate elapsed time."""
        import time

        context = LogContext()
        time.sleep(0.01)  # Sleep for 10ms
        elapsed = context.elapsed_ms()
        self.assertGreater(elapsed, 5)  # Should be at least 5ms


if __name__ == "__main__":
    unittest.main()
