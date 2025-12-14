"""Console logger implementation using Rich.

This module provides a Rich-based console logger that implements
the LoggerPort protocol.
"""

from __future__ import annotations

from rich.console import Console

from ...application.ports.services import LoggerPort
from ...cli.logging_config import SDTMLogger


class ConsoleLogger(LoggerPort):
    """Console logger using Rich formatting.
    
    This is an adapter that wraps the existing SDTMLogger to implement
    the LoggerPort protocol, enabling dependency injection.
    
    Example:
        >>> logger = ConsoleLogger(verbosity=1)
        >>> logger.info("Processing started")
        >>> logger.success("Done!")
    """
    
    def __init__(self, console: Console | None = None, verbosity: int = 0):
        """Initialize the console logger.
        
        Args:
            console: Rich console for output (creates new if None)
            verbosity: Verbosity level (0=normal, 1=verbose, 2=debug)
        """
        self._logger = SDTMLogger(console=console, verbosity=verbosity)
    
    def info(self, message: str) -> None:
        """Log an informational message.
        
        Args:
            message: The message to log
        """
        self._logger.info(message)
    
    def success(self, message: str) -> None:
        """Log a success message.
        
        Args:
            message: The message to log
        """
        self._logger.success(message)
    
    def warning(self, message: str) -> None:
        """Log a warning message.
        
        Args:
            message: The message to log
        """
        self._logger.warning(message)
    
    def error(self, message: str) -> None:
        """Log an error message.
        
        Args:
            message: The message to log
        """
        self._logger.error(message)
    
    def debug(self, message: str) -> None:
        """Log a debug message.
        
        Args:
            message: The message to log
        """
        self._logger.debug(message)
    
    @property
    def sdtm_logger(self) -> SDTMLogger:
        """Access the underlying SDTMLogger for advanced features.
        
        Returns:
            The wrapped SDTMLogger instance
            
        Note:
            This property allows access to SDTMLogger-specific methods
            like set_context(), log_study_start(), etc. for backward
            compatibility with existing code.
        """
        return self._logger
