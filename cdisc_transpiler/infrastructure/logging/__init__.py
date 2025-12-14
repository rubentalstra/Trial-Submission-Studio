"""Logging infrastructure.

This module provides logging adapters and implementations.
"""

from .console_logger import ConsoleLogger, LogContext, LogLevel, SDTMLogger
from .null_logger import NullLogger

__all__ = [
    "ConsoleLogger",
    "SDTMLogger",  # Alias for backward compatibility
    "LogContext",
    "LogLevel",
    "NullLogger",
]
