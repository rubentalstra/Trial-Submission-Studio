"""Backward compatibility module for CLI logging configuration.

This module maintains backward compatibility by re-exporting the logger
components from their new location in infrastructure/logging.

DEPRECATED: Import directly from cdisc_transpiler.infrastructure.logging instead.

For new code, use:
    from cdisc_transpiler.infrastructure.logging import ConsoleLogger, LogLevel, LogContext
"""

from __future__ import annotations

from rich.console import Console

# Re-export from new location for backward compatibility
from ..infrastructure.logging.console_logger import (
    ConsoleLogger,
    LogContext,
    LogLevel,
    SDTMLogger,
)

__all__ = [
    "SDTMLogger",
    "ConsoleLogger",
    "LogLevel",
    "LogContext",
    "get_logger",
    "set_logger",
    "create_logger",
]


# Global logger instance (can be replaced in CLI)
_logger: ConsoleLogger | None = None


def get_logger() -> ConsoleLogger:
    """Get the global logger instance.

    Returns:
        The global ConsoleLogger instance (SDTMLogger alias)
    """
    global _logger
    if _logger is None:
        _logger = ConsoleLogger()
    return _logger


def set_logger(logger: ConsoleLogger) -> None:
    """Set the global logger instance.

    Args:
        logger: Logger instance to use globally
    """
    global _logger
    _logger = logger


def create_logger(console: Console | None = None, verbosity: int = 0) -> ConsoleLogger:
    """Create and set a new logger instance.

    Args:
        console: Rich console for output
        verbosity: Verbosity level

    Returns:
        The new logger instance
    """
    logger = ConsoleLogger(console, verbosity)
    set_logger(logger)
    return logger
