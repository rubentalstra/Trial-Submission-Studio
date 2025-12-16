"""Port interfaces for application services.

This module defines protocols (interfaces) that services depend on,
following the Ports & Adapters (Hexagonal) architecture pattern.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Iterable, Protocol, runtime_checkable

import pandas as pd

if TYPE_CHECKING:
    from ..models import DefineDatasetDTO, OutputRequest, OutputResult
    from ...mapping_module import MappingConfig


@runtime_checkable
class OutputPreparationPort(Protocol):
    """Protocol for preparing output directories and required placeholder files.

    The application layer must not perform direct filesystem I/O. This port
    abstracts creation of output folders (xpt/dataset-xml/sas) and optional
    Define-XML prerequisites such as an ACRF placeholder PDF.
    """

    def prepare(
        self,
        *,
        output_dir: Path,
        output_formats: set[str],
        generate_sas: bool,
        generate_define_xml: bool,
    ) -> None:
        """Prepare the output directory structure.

        Implementations may create directories and files as needed.
        """
        raise NotImplementedError

    def ensure_dir(self, path: Path) -> None:
        """Ensure a directory exists at path."""
        raise NotImplementedError


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
        raise NotImplementedError

    def success(self, message: str) -> None:
        """Log a success message.

        Args:
            message: The message to log
        """
        raise NotImplementedError

    def warning(self, message: str) -> None:
        """Log a warning message.

        Args:
            message: The message to log
        """
        raise NotImplementedError

    def error(self, message: str) -> None:
        """Log an error message.

        Args:
            message: The message to log
        """
        raise NotImplementedError

    def debug(self, message: str) -> None:
        """Log a debug message.

        Args:
            message: The message to log
        """
        raise NotImplementedError

    def verbose(self, message: str) -> None:
        """Log a verbose message.

        Args:
            message: The message to log
        """
        raise NotImplementedError

    def log_study_start(
        self,
        study_id: str,
        study_folder: Path,
        output_format: str,
        supported_domains: list[str],
    ) -> None:
        """Log the start of study processing."""
        raise NotImplementedError

    def log_metadata_loaded(
        self,
        *,
        items_count: int | None,
        codelists_count: int | None,
    ) -> None:
        """Log study metadata loading results."""
        raise NotImplementedError

    def log_processing_summary(
        self,
        *,
        study_id: str,
        domain_count: int,
        file_count: int,
        output_format: str,
        generate_define: bool,
        generate_sas: bool,
    ) -> None:
        """Log a summary of processing configuration and inputs."""
        raise NotImplementedError

    def log_final_stats(self) -> None:
        """Log final overall processing statistics."""
        raise NotImplementedError

    def log_domain_start(
        self, domain_code: str, files_for_domain: list[tuple[Path, str]]
    ) -> None:
        """Log the start of processing a specific domain."""
        raise NotImplementedError

    def log_synthesis_start(self, domain_code: str, reason: str) -> None:
        """Log the start of synthesis for a domain."""
        raise NotImplementedError

    def log_synthesis_complete(self, domain_code: str, records: int) -> None:
        """Log successful completion of synthesis for a domain."""
        raise NotImplementedError


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
        raise NotImplementedError


@runtime_checkable
class XPTWriterPort(Protocol):
    """Protocol for XPT (SAS Transport) file writing.

    This interface abstracts XPT file generation, allowing different
    implementations without coupling the application to specific writing logic.

    Example:
        >>> writer = XPTWriter()
        >>> writer.write(dataframe, "DM", Path("output/dm.xpt"))
    """

    def write(
        self,
        dataframe: pd.DataFrame,
        domain_code: str,
        output_path: Path,
        *,
        file_label: str | None = None,
        table_name: str | None = None,
    ) -> None:
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
        raise NotImplementedError


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
        raise NotImplementedError


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
        raise NotImplementedError


@runtime_checkable
class DefineXmlGeneratorPort(Protocol):
    """Protocol for Define-XML generation services.

    This interface abstracts Define-XML 2.1 generation, allowing different
    implementations without coupling the application to specific generation logic.

    The port accepts application-layer DTOs (DefineDatasetDTO) which the
    infrastructure adapter converts to infrastructure-specific models.

    Example:
        >>> generator = DefineXmlGenerator()
        >>> generator.generate(datasets, Path("define.xml"), sdtm_version="3.4", context="Submission")
    """

    def generate(
        self,
        datasets: Iterable[DefineDatasetDTO],
        output_path: Path,
        *,
        sdtm_version: str,
        context: str,
    ) -> None:
        """Generate a Define-XML 2.1 file for the given study datasets.

        Args:
            datasets: Iterable of DefineDatasetDTO objects containing domain metadata
            output_path: Path where Define-XML file should be written
            sdtm_version: SDTM-IG version (e.g., "3.4")
            context: Define-XML context - 'Submission' or 'Other'

        Raises:
            Exception: If generation or writing fails

        Example:
            >>> datasets = [DefineDatasetDTO(...), DefineDatasetDTO(...)]
            >>> generator.generate(datasets, Path("define.xml"), sdtm_version="3.4", context="Submission")
        """
        raise NotImplementedError
