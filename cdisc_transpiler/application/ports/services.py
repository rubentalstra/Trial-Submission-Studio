"""Port interfaces for application services.

This module defines protocols (interfaces) that services depend on,
following the Ports & Adapters (Hexagonal) architecture pattern.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Protocol, runtime_checkable

if TYPE_CHECKING:
    from ...infrastructure.io.models import OutputRequest, OutputResult


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
    
    def verbose(self, message: str) -> None:
        """Log a verbose message.
        
        Args:
            message: The message to log
        """
        ...


@runtime_checkable
class FileGeneratorPort(Protocol):
    """Protocol for file generation services.
    
    This interface abstracts file generation operations, allowing different
    implementations for different output formats (XPT, XML, SAS) without
    coupling the application to specific generation logic.
    
    Implementations handle:
    - XPT (SAS Transport) file generation
    - Dataset-XML file generation
    - SAS program generation
    
    Example:
        >>> def save_domain(generator: FileGeneratorPort, df: pd.DataFrame):
        ...     request = OutputRequest(
        ...         dataframe=df,
        ...         domain_code="DM",
        ...         config=config,
        ...         output_dirs=dirs,
        ...         formats={"xpt", "xml"}
        ...     )
        ...     result = generator.generate(request)
        ...     if result.success:
        ...         print(f"Generated XPT: {result.xpt_path}")
    """
    
    def generate(self, request: OutputRequest) -> OutputResult:
        """Generate output files based on the request.
        
        Args:
            request: OutputRequest containing DataFrame, domain, and configuration
            
        Returns:
            OutputResult with paths to generated files and any errors
            
        Example:
            >>> result = generator.generate(request)
            >>> if result.success:
            ...     print(f"XPT: {result.xpt_path}")
            ...     print(f"XML: {result.xml_path}")
            ... else:
            ...     print(f"Errors: {result.errors}")
        """
        ...
