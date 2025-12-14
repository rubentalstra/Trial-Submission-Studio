"""Logging infrastructure.

This module provides logging adapters and implementations.
"""

from .console_logger import ConsoleLogger
from .null_logger import NullLogger

__all__ = ["ConsoleLogger", "NullLogger"]
