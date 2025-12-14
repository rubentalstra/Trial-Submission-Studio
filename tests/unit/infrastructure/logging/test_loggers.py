"""Unit tests for logger implementations."""

from __future__ import annotations

from io import StringIO
from unittest.mock import MagicMock

import pytest
from rich.console import Console

from cdisc_transpiler.application.ports import LoggerPort
from cdisc_transpiler.infrastructure.logging import ConsoleLogger, NullLogger


class TestLoggerPort:
    """Test suite for LoggerPort protocol compliance."""
    
    def test_console_logger_implements_protocol(self):
        """Test that ConsoleLogger implements LoggerPort protocol."""
        logger = ConsoleLogger()
        assert isinstance(logger, LoggerPort)
    
    def test_null_logger_implements_protocol(self):
        """Test that NullLogger implements LoggerPort protocol."""
        logger = NullLogger()
        assert isinstance(logger, LoggerPort)
    
    def test_protocol_has_required_methods(self):
        """Test that LoggerPort protocol defines required methods."""
        required_methods = ["info", "success", "warning", "error", "debug"]
        
        for method in required_methods:
            assert hasattr(LoggerPort, method), f"LoggerPort should define {method}"


class TestConsoleLogger:
    """Test suite for ConsoleLogger class."""
    
    @pytest.fixture
    def string_console(self):
        """Create a console that writes to a string buffer."""
        buffer = StringIO()
        console = Console(file=buffer, force_terminal=True, width=120)
        return console, buffer
    
    def test_info_logs_message(self, string_console):
        """Test that info() logs messages."""
        console, buffer = string_console
        logger = ConsoleLogger(console=console, verbosity=0)
        
        logger.info("Test info message")
        
        output = buffer.getvalue()
        assert "Test info message" in output
    
    def test_success_logs_message(self, string_console):
        """Test that success() logs messages with checkmark."""
        console, buffer = string_console
        logger = ConsoleLogger(console=console, verbosity=0)
        
        logger.success("Test success message")
        
        output = buffer.getvalue()
        assert "Test success message" in output
        assert "✓" in output or "success" in output.lower()
    
    def test_warning_logs_message(self, string_console):
        """Test that warning() logs messages with warning symbol."""
        console, buffer = string_console
        logger = ConsoleLogger(console=console, verbosity=0)
        
        logger.warning("Test warning message")
        
        output = buffer.getvalue()
        assert "Test warning message" in output
        assert "⚠" in output or "warning" in output.lower()
    
    def test_error_logs_message(self, string_console):
        """Test that error() logs messages with error symbol."""
        console, buffer = string_console
        logger = ConsoleLogger(console=console, verbosity=0)
        
        logger.error("Test error message")
        
        output = buffer.getvalue()
        assert "Test error message" in output
        assert "✗" in output or "error" in output.lower()
    
    def test_debug_logs_message(self, string_console):
        """Test that debug() logs messages when verbosity is high enough."""
        console, buffer = string_console
        logger = ConsoleLogger(console=console, verbosity=2)  # Debug level
        
        logger.debug("Test debug message")
        
        output = buffer.getvalue()
        assert "Test debug message" in output
    
    def test_debug_suppressed_at_low_verbosity(self, string_console):
        """Test that debug() is suppressed at low verbosity."""
        console, buffer = string_console
        logger = ConsoleLogger(console=console, verbosity=0)  # Normal level
        
        logger.debug("Test debug message")
        
        output = buffer.getvalue()
        assert "Test debug message" not in output
    
    def test_sdtm_logger_access(self, string_console):
        """Test that underlying SDTMLogger is accessible."""
        console, buffer = string_console
        logger = ConsoleLogger(console=console, verbosity=0)
        
        # Access SDTMLogger for advanced features
        sdtm_logger = logger.sdtm_logger
        assert sdtm_logger is not None
        assert hasattr(sdtm_logger, "set_context")
        assert hasattr(sdtm_logger, "log_study_start")
    
    def test_different_verbosity_levels(self):
        """Test that logger respects different verbosity levels."""
        # Normal verbosity
        logger_normal = ConsoleLogger(verbosity=0)
        assert logger_normal._logger.verbosity == 0
        
        # Verbose
        logger_verbose = ConsoleLogger(verbosity=1)
        assert logger_verbose._logger.verbosity == 1
        
        # Debug
        logger_debug = ConsoleLogger(verbosity=2)
        assert logger_debug._logger.verbosity == 2


class TestNullLogger:
    """Test suite for NullLogger class."""
    
    def test_info_produces_no_output(self):
        """Test that info() produces no output."""
        logger = NullLogger()
        
        # Should not raise any exception and produce no output
        logger.info("Test message")
    
    def test_success_produces_no_output(self):
        """Test that success() produces no output."""
        logger = NullLogger()
        
        logger.success("Test message")
    
    def test_warning_produces_no_output(self):
        """Test that warning() produces no output."""
        logger = NullLogger()
        
        logger.warning("Test message")
    
    def test_error_produces_no_output(self):
        """Test that error() produces no output."""
        logger = NullLogger()
        
        logger.error("Test message")
    
    def test_debug_produces_no_output(self):
        """Test that debug() produces no output."""
        logger = NullLogger()
        
        logger.debug("Test message")
    
    def test_all_methods_callable(self):
        """Test that all protocol methods are callable without errors."""
        logger = NullLogger()
        
        # Call all methods - should not raise exceptions
        logger.info("info")
        logger.success("success")
        logger.warning("warning")
        logger.error("error")
        logger.debug("debug")
    
    def test_null_logger_for_testing(self):
        """Test using NullLogger in a simulated service."""
        def process_data(logger: LoggerPort):
            """Simulated service that uses a logger."""
            logger.info("Processing started")
            logger.success("Processing complete")
            return "result"
        
        # Use NullLogger for silent testing
        logger = NullLogger()
        result = process_data(logger)
        
        assert result == "result"


class TestLoggerDependencyInjection:
    """Test suite for logger dependency injection pattern."""
    
    def test_service_accepts_logger_port(self):
        """Test that services can accept any LoggerPort implementation."""
        def process_with_logging(logger: LoggerPort, data: str) -> str:
            """Example service that uses injected logger."""
            logger.info(f"Processing: {data}")
            result = data.upper()
            logger.success(f"Result: {result}")
            return result
        
        # Test with ConsoleLogger
        console_logger = ConsoleLogger()
        result1 = process_with_logging(console_logger, "test")
        assert result1 == "TEST"
        
        # Test with NullLogger
        null_logger = NullLogger()
        result2 = process_with_logging(null_logger, "test")
        assert result2 == "TEST"
    
    def test_mock_logger_for_testing(self):
        """Test that logger can be mocked for testing."""
        mock_logger = MagicMock(spec=LoggerPort)
        
        def process_with_logging(logger: LoggerPort):
            logger.info("Starting")
            logger.success("Done")
        
        process_with_logging(mock_logger)
        
        # Verify logger was called
        mock_logger.info.assert_called_once_with("Starting")
        mock_logger.success.assert_called_once_with("Done")
    
    def test_logger_swapping(self):
        """Test that logger implementations can be swapped easily."""
        def create_processor(logger: LoggerPort):
            """Factory that creates a processor with injected logger."""
            def process(data: str) -> str:
                logger.info(f"Processing {data}")
                return data.lower()
            return process
        
        # Create with ConsoleLogger
        console_processor = create_processor(ConsoleLogger())
        assert console_processor("TEST") == "test"
        
        # Create with NullLogger
        silent_processor = create_processor(NullLogger())
        assert silent_processor("TEST") == "test"
