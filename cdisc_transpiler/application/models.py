"""Data models for application layer use cases.

This module contains request and response DTOs (Data Transfer Objects) used
by application layer use cases to maintain clear boundaries and enable testing.

Includes:
- Study/Domain processing request/response DTOs
- Output generation DTOs (OutputDirs, OutputRequest, OutputResult)
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING

import pandas as pd

from ..constants import Defaults, SDTMVersions

if TYPE_CHECKING:
    from ..domain.entities.mapping import MappingConfig
    from ..domain.entities.study_metadata import StudyMetadata
    from ..domain.services.sdtm_conformance_checker import ConformanceReport


def _default_output_formats() -> set[str]:
    return {"xpt", "xml"}


def _empty_str_list() -> list[str]:
    return []


def _empty_str_set() -> set[str]:
    return set()


def _empty_output_dirs() -> dict[str, Path | None]:
    return {}


def _empty_domain_results() -> list[DomainProcessingResult]:
    return []


def _empty_domain_responses() -> list[ProcessDomainResponse]:
    return []


def _empty_error_list() -> list[tuple[str, str]]:
    return []


# ============================================================================
# Output Generation DTOs
# ============================================================================


@dataclass(slots=True)
class OutputDirs:
    """Output directory configuration.

    This DTO specifies which output directories should be used for each
    output format. Setting a directory to None skips generation for that format.

    Attributes:
        xpt_dir: Directory for XPT files (None to skip)
        xml_dir: Directory for Dataset-XML files (None to skip)
        sas_dir: Directory for SAS programs (None to skip)

    Example:
        >>> dirs = OutputDirs(
        ...     xpt_dir=Path("output/xpt"),
        ...     xml_dir=Path("output/dataset-xml"),
        ... )
    """

    xpt_dir: Path | None = None
    xml_dir: Path | None = None
    sas_dir: Path | None = None


@dataclass(slots=True)
class OutputRequest:
    """Request for file generation.

    This DTO encapsulates all inputs needed for output file generation,
    providing a clean interface between use cases and file generators.

    Attributes:
        dataframe: DataFrame to write
        domain_code: SDTM domain code (e.g., "DM", "AE")
        config: Mapping configuration for the domain
        output_dirs: Directory configuration
        formats: Set of formats to generate ({"xpt", "xml", "sas"})
        base_filename: Base filename (defaults to lowercase domain code)
        input_dataset: Input dataset name for SAS (e.g., "work.dm")
        output_dataset: Output dataset name for SAS (e.g., "sdtm.dm")

    Example:
        >>> request = OutputRequest(
        ...     dataframe=df,
        ...     domain_code="DM",
        ...     config=config,
        ...     output_dirs=OutputDirs(xpt_dir=Path("output/xpt")),
        ...     formats={"xpt", "xml"},
        ... )
    """

    dataframe: pd.DataFrame
    domain_code: str
    config: MappingConfig
    output_dirs: OutputDirs
    formats: set[str]
    base_filename: str | None = None
    input_dataset: str | None = None
    output_dataset: str | None = None


@dataclass(slots=True)
class OutputResult:
    """Result of file generation.

    This DTO captures the outputs from file generation, including paths
    to generated files and any errors encountered.

    Attributes:
        xpt_path: Path to generated XPT file (None if not generated)
        xml_path: Path to generated Dataset-XML file (None if not generated)
        sas_path: Path to generated SAS program (None if not generated)
        errors: List of error messages encountered

    Example:
        >>> result = OutputResult(
        ...     xpt_path=Path("output/xpt/dm.xpt"),
        ...     xml_path=Path("output/dataset-xml/dm.xml"),
        ... )
        >>> if result.success:
        ...     print("Generation successful")
    """

    xpt_path: Path | None = None
    xml_path: Path | None = None
    sas_path: Path | None = None
    errors: list[str] = field(default_factory=_empty_str_list)

    @property
    def success(self) -> bool:
        """Check if generation was successful (no errors)."""
        return len(self.errors) == 0


# ============================================================================
# Define-XML DTOs
# ============================================================================


@dataclass(slots=True)
class DefineDatasetDTO:
    """Application-layer DTO for Define-XML dataset metadata.

    This DTO represents dataset metadata needed for Define-XML generation,
    providing a clean boundary between the application layer and the
    infrastructure layer that generates the actual Define-XML files.

    The infrastructure adapter converts these DTOs into infrastructure-specific
    models (e.g., `StudyDataset` in the infrastructure Define-XML package).

    Attributes:
        domain_code: SDTM domain code (e.g., "DM", "AE", "LB")
        dataframe: The dataset DataFrame
        config: Mapping configuration with column metadata
        label: Dataset label (optional)
        structure: Dataset structure description
        archive_location: Relative path to the dataset file in the archive

    Example:
        >>> dto = DefineDatasetDTO(
        ...     domain_code="DM",
        ...     dataframe=dm_df,
        ...     config=dm_config,
        ...     archive_location=Path("xpt/dm.xpt"),
        ... )
    """

    domain_code: str
    dataframe: pd.DataFrame
    config: MappingConfig
    label: str | None = None
    structure: str = "One record per subject per domain-specific entity"
    archive_location: Path | None = None


# ============================================================================
# Study and Domain Processing DTOs
# ============================================================================


@dataclass(slots=True)
class ProcessStudyRequest:
    """Request to process a study folder.

    This DTO encapsulates all inputs needed for study processing, providing
    a clean interface between the CLI layer and the application layer.

    Attributes:
        study_folder: Path to the study folder containing CSV files
        study_id: Study identifier (derived from folder if not provided)
        output_dir: Output directory for generated files
        output_formats: Set of formats to generate ({"xpt", "xml"})
        generate_define_xml: Whether to generate Define-XML file
        generate_sas: Whether to generate SAS programs
        sdtm_version: SDTM-IG version for Define-XML (e.g., SDTMVersions.DEFAULT_VERSION)
        define_context: Define-XML context (e.g., SDTMVersions.DEFINE_CONTEXT_SUBMISSION)
        streaming: Use streaming mode for large datasets
        chunk_size: Chunk size for streaming mode
        min_confidence: Minimum confidence for fuzzy matches (0.0-1.0)
        verbose: Verbosity level (0, 1, 2, ...)

    Example:
        >>> request = ProcessStudyRequest(
        ...     study_folder=Path("study001"),
        ...     study_id="STUDY001",
        ...     output_dir=Path("output"),
        ...     output_formats={"xpt", "xml"},
        ...     generate_define_xml=True,
        ...     generate_sas=True,
        ... )
    """

    study_folder: Path
    study_id: str
    output_dir: Path
    output_formats: set[str] = field(default_factory=_default_output_formats)
    generate_define_xml: bool = True
    generate_sas: bool = True
    sdtm_version: str = SDTMVersions.DEFAULT_VERSION
    define_context: str = SDTMVersions.DEFINE_CONTEXT_SUBMISSION
    streaming: bool = False
    chunk_size: int = Defaults.CHUNK_SIZE
    min_confidence: float = Defaults.MIN_CONFIDENCE
    verbose: int = 0

    # Conformance behavior
    write_conformance_report_json: bool = True
    fail_on_conformance_errors: bool = False

    # Optional study-level defaults used for required variables when source data
    # doesn't provide them (e.g., DM.COUNTRY).
    default_country: str | None = None


@dataclass(slots=True)
class DomainProcessingResult:
    """Result of processing a single domain.

    This DTO captures the output of domain processing for a single domain,
    including the generated dataframe, files, and any supplemental domains.

    Attributes:
        domain_code: SDTM domain code (e.g., "DM", "AE")
        success: Whether processing succeeded
        records: Number of records in the domain
        domain_dataframe: The processed domain DataFrame
        config: Mapping configuration used
        xpt_path: Path to generated XPT file (if any)
        xml_path: Path to generated Dataset-XML file (if any)
        sas_path: Path to generated SAS program (if any)
        supplementals: List of supplemental domain results (e.g., SUPPAE)
        error: Error message if processing failed
        synthesized: Whether this domain was synthesized (not from source data)
        synthesis_reason: Reason for synthesis (if synthesized)

    Example:
        >>> result = DomainProcessingResult(
        ...     domain_code="DM",
        ...     success=True,
        ...     records=100,
        ...     domain_dataframe=dm_df,
        ...     xpt_path=Path("output/xpt/dm.xpt"),
        ... )
    """

    domain_code: str
    success: bool = True
    records: int = 0
    domain_dataframe: pd.DataFrame | None = None
    config: MappingConfig | None = None
    xpt_path: Path | None = None
    xml_path: Path | None = None
    sas_path: Path | None = None
    supplementals: list[DomainProcessingResult] = field(
        default_factory=_empty_domain_results
    )
    error: str | None = None
    synthesized: bool = False
    synthesis_reason: str | None = None

    # Optional machine-readable conformance report (domain-layer type), when strict checks ran
    conformance_report: ConformanceReport | None = None


@dataclass(slots=True)
class ProcessStudyResponse:
    """Response from study processing.

    This DTO encapsulates all outputs from study processing, providing
    a clean interface for the CLI layer to format and display results.

    Attributes:
        success: Whether overall processing succeeded
        study_id: Study identifier
        processed_domains: Set of domain codes that were processed
        domain_results: List of domain processing results
        errors: List of (domain_code, error_message) tuples
        define_xml_path: Path to generated Define-XML file (if any)
        define_xml_error: Error message if Define-XML generation failed
        output_dir: Output directory where files were generated
        total_records: Total number of records across all domains

    Example:
        >>> response = ProcessStudyResponse(
        ...     success=True,
        ...     study_id="STUDY001",
        ...     processed_domains={"DM", "AE", "LB"},
        ...     domain_results=[dm_result, ae_result, lb_result],
        ...     define_xml_path=Path("output/define.xml"),
        ... )
    """

    success: bool = True
    study_id: str = ""
    processed_domains: set[str] = field(default_factory=_empty_str_set)
    domain_results: list[DomainProcessingResult] = field(
        default_factory=_empty_domain_results
    )
    errors: list[tuple[str, str]] = field(default_factory=_empty_error_list)
    define_xml_path: Path | None = None
    define_xml_error: str | None = None
    output_dir: Path | None = None
    total_records: int = 0

    # Optional machine-readable conformance report artifact
    conformance_report_path: Path | None = None
    conformance_report_error: str | None = None

    @property
    def has_errors(self) -> bool:
        """Check if any errors occurred during processing."""
        return len(self.errors) > 0 or self.define_xml_error is not None

    @property
    def successful_domains(self) -> list[str]:
        """Get list of successfully processed domain codes."""
        return [r.domain_code for r in self.domain_results if r.success]

    @property
    def failed_domains(self) -> list[str]:
        """Get list of failed domain codes."""
        return [code for code, _ in self.errors]


@dataclass(slots=True)
class ProcessDomainRequest:
    """Request to process a single SDTM domain.

    This DTO encapsulates all inputs needed for domain processing, providing
    a clean interface for domain-level operations.

    Attributes:
        files_for_domain: List of (file_path, variant_name) tuples to process
        domain_code: SDTM domain code (e.g., "DM", "AE", "LB")
        study_id: Study identifier
        output_formats: Set of formats to generate ({"xpt", "xml"})
        output_dirs: Dictionary with "xpt", "xml", "sas" directory paths
        min_confidence: Minimum confidence for fuzzy matches (0.0-1.0)
        streaming: Use streaming mode for large datasets
        chunk_size: Chunk size for streaming mode
        generate_sas: Whether to generate SAS programs
        verbose: Verbosity level (0, 1, 2, ...)
        metadata: Study metadata (Items.csv, CodeLists.csv)
        reference_starts: Reference start dates by subject ID
        common_column_counts: Common column frequency counts for heuristics
        total_input_files: Total number of input files (for heuristics)

    Example:
        >>> request = ProcessDomainRequest(
        ...     files_for_domain=[(Path("DM.csv"), "DM")],
        ...     domain_code="DM",
        ...     study_id="STUDY001",
        ...     output_formats={"xpt", "xml"},
        ...     output_dirs={"xpt": Path("output/xpt")},
        ... )
    """

    files_for_domain: list[tuple[Path, str]]
    domain_code: str
    study_id: str
    output_formats: set[str] = field(default_factory=_default_output_formats)
    output_dirs: dict[str, Path | None] = field(default_factory=_empty_output_dirs)
    min_confidence: float = 0.5
    streaming: bool = False
    chunk_size: int = 1000
    generate_sas: bool = True
    verbose: int = 0
    metadata: StudyMetadata | None = None
    reference_starts: dict[str, str] | None = None
    common_column_counts: dict[str, int] | None = None
    total_input_files: int | None = None

    # Conformance behavior
    fail_on_conformance_errors: bool = False

    # Optional study-level defaults propagated into domain mapping/processing.
    default_country: str | None = None


@dataclass(slots=True)
class ProcessDomainResponse:
    """Response from domain processing.

    This DTO encapsulates all outputs from domain processing, providing
    a clean interface for returning results.

    Attributes:
        success: Whether processing succeeded
        domain_code: SDTM domain code
        records: Number of records in processed domain
        domain_dataframe: The processed domain DataFrame
        config: Mapping configuration used
        xpt_path: Path to generated XPT file (if any)
        xml_path: Path to generated Dataset-XML file (if any)
        sas_path: Path to generated SAS program (if any)
        supplementals: List of supplemental domain responses (e.g., SUPPAE)
        error: Error message if processing failed
        warnings: List of warning messages

    Example:
        >>> response = ProcessDomainResponse(
        ...     success=True,
        ...     domain_code="DM",
        ...     records=100,
        ...     domain_dataframe=dm_df,
        ...     xpt_path=Path("output/xpt/dm.xpt"),
        ... )
    """

    success: bool = True
    domain_code: str = ""
    records: int = 0
    domain_dataframe: pd.DataFrame | None = None
    config: MappingConfig | None = None
    xpt_path: Path | None = None
    xml_path: Path | None = None
    sas_path: Path | None = None
    supplementals: list[ProcessDomainResponse] = field(
        default_factory=_empty_domain_responses
    )
    error: str | None = None
    warnings: list[str] = field(default_factory=_empty_str_list)

    # Optional machine-readable conformance report (domain-layer type)
    conformance_report: ConformanceReport | None = None

    def to_dict(self) -> dict[str, object]:
        """Convert to a plain dictionary representation.

        Returns:
            Dictionary with keys expected by existing callers
        """
        result: dict[str, object] = {
            "domain_code": self.domain_code,
            "records": self.records,
            "domain_dataframe": self.domain_dataframe,
            "config": self.config,
            "xpt_path": self.xpt_path,
            "xml_path": self.xml_path,
            "sas_path": self.sas_path,
            "supplementals": [
                {
                    "domain_code": supp.domain_code,
                    "records": supp.records,
                    "domain_dataframe": supp.domain_dataframe,
                    "config": supp.config,
                    "xpt_path": supp.xpt_path,
                    "xml_path": supp.xml_path,
                    "sas_path": supp.sas_path,
                }
                for supp in self.supplementals
            ],
            "conformance_report": self.conformance_report,
        }
        return result
