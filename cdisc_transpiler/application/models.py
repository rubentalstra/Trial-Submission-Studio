"""Data models for application layer use cases.

This module contains request and response DTOs (Data Transfer Objects) used
by application layer use cases to maintain clear boundaries and enable testing.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import pandas as pd


@dataclass
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
        sdtm_version: SDTM-IG version for Define-XML (e.g., "3.4")
        define_context: Define-XML context ("Submission" or "Other")
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
    output_formats: set[str] = field(default_factory=lambda: {"xpt", "xml"})
    generate_define_xml: bool = True
    generate_sas: bool = True
    sdtm_version: str = "3.4"
    define_context: str = "Submission"
    streaming: bool = False
    chunk_size: int = 1000
    min_confidence: float = 0.5
    verbose: int = 0


@dataclass
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
        split_datasets: List of split datasets for large domains
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
    config: Any = None  # MappingConfig, but avoiding circular import
    xpt_path: Path | None = None
    xml_path: Path | None = None
    sas_path: Path | None = None
    supplementals: list[DomainProcessingResult] = field(default_factory=list)
    split_datasets: list[tuple[str, pd.DataFrame, Path]] = field(default_factory=list)
    error: str | None = None
    synthesized: bool = False
    synthesis_reason: str | None = None


@dataclass
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
    processed_domains: set[str] = field(default_factory=set)
    domain_results: list[DomainProcessingResult] = field(default_factory=list)
    errors: list[tuple[str, str]] = field(default_factory=list)
    define_xml_path: Path | None = None
    define_xml_error: str | None = None
    output_dir: Path | None = None
    total_records: int = 0
    
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


@dataclass
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
    output_formats: set[str] = field(default_factory=lambda: {"xpt", "xml"})
    output_dirs: dict[str, Path | None] = field(default_factory=dict)
    min_confidence: float = 0.5
    streaming: bool = False
    chunk_size: int = 1000
    generate_sas: bool = True
    verbose: int = 0
    metadata: Any = None  # StudyMetadata, avoiding circular import
    reference_starts: dict[str, str] | None = None
    common_column_counts: dict[str, int] | None = None
    total_input_files: int | None = None


@dataclass
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
        split_datasets: List of (name, dataframe, path) for split datasets
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
    config: Any = None  # MappingConfig, avoiding circular import
    xpt_path: Path | None = None
    xml_path: Path | None = None
    sas_path: Path | None = None
    supplementals: list[ProcessDomainResponse] = field(default_factory=list)
    split_datasets: list[tuple[str, pd.DataFrame, Path]] = field(default_factory=list)
    error: str | None = None
    warnings: list[str] = field(default_factory=list)
    
    def to_dict(self) -> dict:
        """Convert to dictionary format for compatibility with existing code.
        
        Returns:
            Dictionary with keys expected by legacy code
        """
        result = {
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
            "split_datasets": self.split_datasets,
        }
        return result
