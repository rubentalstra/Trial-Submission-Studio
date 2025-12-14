"""Port interfaces for application services.

This module defines protocols (interfaces) that services depend on,
following the Ports & Adapters (Hexagonal) architecture pattern.
"""

from __future__ import annotations

from typing import Protocol, runtime_checkable


@runtime_checkable
class LoggerPort(Protocol):
    """Protocol for logging services.
    
    This interface allows services to be decoupled from specific logging
    implementations. Services depend on this protocol, not on concrete
    logger classes.
    
    Example:
        >>> def process_data(logger: LoggerPort):
        ...     logger.info("Processing started")
        ...     logger.success("Processing complete")
    """
    
    def info(self, message: str) -> None:
        """Log an informational message.
        
        Args:
            message: The message to log
        """
        ...
    
    def success(self, message: str) -> None:
        """Log a success message.
        
        Args:
            message: The message to log
        """
        ...
    
    def warning(self, message: str) -> None:
        """Log a warning message.
        
        Args:
            message: The message to log
        """
        ...
    
    def error(self, message: str) -> None:
        """Log an error message.
        
        Args:
            message: The message to log
        """
        ...
    
    def debug(self, message: str) -> None:
        """Log a debug message.
        
        Args:
            message: The message to log
        """
        ...
