"""Port interfaces for application services.

This module defines protocols (interfaces) that services depend on,
following the Ports & Adapters (Hexagonal) architecture pattern.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Iterable, Protocol, runtime_checkable

import pandas as pd

if TYPE_CHECKING:
    from ...infrastructure.io.models import OutputRequest, OutputResult
    from ...mapping_module import MappingConfig
    from ...xml_module.define_module import StudyDataset


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


@runtime_checkable
class XPTWriterPort(Protocol):
    """Protocol for XPT (SAS Transport) file writing.
    
    This interface abstracts XPT file generation, allowing different
    implementations without coupling the application to specific writing logic.
    
    Example:
        >>> writer = XPTWriter()
        >>> writer.write(dataframe, "DM", Path("output/dm.xpt"))
    """
    
    def write(self, dataframe: pd.DataFrame, domain_code: str, output_path: Path) -> None:
        """Write a DataFrame to an XPT file.
        
        Args:
            dataframe: Data to write
            domain_code: SDTM domain code (e.g., "DM", "AE")
            output_path: Path where XPT file should be written
            
        Raises:
            Exception: If writing fails
            
        Example:
            >>> df = pd.DataFrame({"STUDYID": ["001"], "USUBJID": ["001-001"]})
            >>> writer.write(df, "DM", Path("dm.xpt"))
        """
        ...


@runtime_checkable
class DatasetXMLWriterPort(Protocol):
    """Protocol for Dataset-XML file writing.
    
    This interface abstracts Dataset-XML generation, allowing different
    implementations without coupling the application to specific writing logic.
    
    Example:
        >>> writer = DatasetXMLWriter()
        >>> writer.write(dataframe, "DM", config, Path("output/dm.xml"))
    """
    
    def write(
        self,
        dataframe: pd.DataFrame,
        domain_code: str,
        config: MappingConfig,
        output_path: Path,
    ) -> None:
        """Write a DataFrame to a Dataset-XML file.
        
        Args:
            dataframe: Data to write
            domain_code: SDTM domain code (e.g., "DM", "AE")
            config: Mapping configuration with column metadata
            output_path: Path where XML file should be written
            
        Raises:
            Exception: If writing fails
            
        Example:
            >>> df = pd.DataFrame({"STUDYID": ["001"], "USUBJID": ["001-001"]})
            >>> writer.write(df, "DM", config, Path("dm.xml"))
        """
        ...


@runtime_checkable
class SASWriterPort(Protocol):
    """Protocol for SAS program generation and writing.
    
    This interface abstracts SAS program generation, allowing different
    implementations without coupling the application to specific writing logic.
    
    Example:
        >>> writer = SASWriter()
        >>> writer.write("DM", config, Path("output/dm.sas"), "work.dm", "sdtm.dm")
    """
    
    def write(
        self,
        domain_code: str,
        config: MappingConfig,
        output_path: Path,
        input_dataset: str | None = None,
        output_dataset: str | None = None,
    ) -> None:
        """Generate and write a SAS program.
        
        Args:
            domain_code: SDTM domain code (e.g., "DM", "AE")
            config: Mapping configuration with column metadata
            output_path: Path where SAS file should be written
            input_dataset: Input dataset name (e.g., "work.dm"), optional
            output_dataset: Output dataset name (e.g., "sdtm.dm"), optional
            
        Raises:
            Exception: If generation or writing fails
            
        Example:
            >>> writer.write("DM", config, Path("dm.sas"), "raw.demo", "final.dm")
        """
        ...


@runtime_checkable
class DefineXmlGeneratorPort(Protocol):
    """Protocol for Define-XML generation services.
    
    This interface abstracts Define-XML 2.1 generation, allowing different
    implementations without coupling the application to specific generation logic.
    
    Example:
        >>> generator = DefineXmlGenerator()
        >>> generator.generate(datasets, Path("define.xml"), sdtm_version="3.4", context="Submission")
    """
    
    def generate(
        self,
        datasets: Iterable[StudyDataset],
        output_path: Path,
        *,
        sdtm_version: str,
        context: str,
    ) -> None:
        """Generate a Define-XML 2.1 file for the given study datasets.
        
        Args:
            datasets: Iterable of StudyDataset objects containing domain metadata
            output_path: Path where Define-XML file should be written
            sdtm_version: SDTM-IG version (e.g., "3.4")
            context: Define-XML context - 'Submission' or 'Other'
            
        Raises:
            Exception: If generation or writing fails
            
        Example:
            >>> datasets = [StudyDataset(...), StudyDataset(...)]
            >>> generator.generate(datasets, Path("define.xml"), sdtm_version="3.4", context="Submission")
        """
        ...
