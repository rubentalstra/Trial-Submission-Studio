"""Port interfaces for data access repositories.

This module defines protocols (interfaces) for data access following the
Ports & Adapters (Hexagonal) architecture pattern. These interfaces allow
the application layer to remain independent of specific data storage
implementations.

All repository ports use Protocol for duck typing, so implementations don't
need to explicitly inherit from these interfaces.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Protocol, runtime_checkable

import pandas as pd

from ...terminology_module.models import ControlledTerminology
from ...domain.entities.study_metadata import StudyMetadata

if TYPE_CHECKING:
    from ...domain.entities.sdtm_domain import SDTMDomain


@runtime_checkable
class CTRepositoryPort(Protocol):
    """Protocol for Controlled Terminology repository access.

    This interface provides access to CDISC controlled terminology data,
    allowing the application to query and retrieve CT codelists without
    depending on specific file formats or storage mechanisms.

    Implementations might load from:
    - CSV files (current implementation)
    - Database tables
    - Web services / APIs
    - Cached in-memory storage

    Example:
        >>> def validate_domain(ct_repo: CTRepositoryPort):
        ...     ct = ct_repo.get_by_code("C66767")
        ...     if ct and "MALE" in ct.submission_values:
        ...         print("Valid gender value")
    """

    def get_by_code(self, codelist_code: str) -> ControlledTerminology | None:
        """Retrieve controlled terminology by codelist code.

        Args:
            codelist_code: NCI codelist code (e.g., "C66767" for SEX)

        Returns:
            ControlledTerminology object if found, None otherwise

        Example:
            >>> ct = ct_repo.get_by_code("C66767")
            >>> if ct:
            ...     print(f"Codelist: {ct.codelist_name}")
            ...     print(f"Values: {ct.submission_values}")
        """
        ...

    def get_by_name(self, codelist_name: str) -> ControlledTerminology | None:
        """Retrieve controlled terminology by codelist name.

        Args:
            codelist_name: Human-readable codelist name (e.g., "SEX")

        Returns:
            ControlledTerminology object if found, None otherwise

        Example:
            >>> ct = ct_repo.get_by_name("SEX")
            >>> if ct:
            ...     normalized = ct.normalize("male")  # Returns "MALE"
        """
        ...

    def list_all_codes(self) -> list[str]:
        """List all available codelist codes.

        Returns:
            List of all NCI codelist codes available in the repository

        Example:
            >>> codes = ct_repo.list_all_codes()
            >>> print(f"Available codelists: {len(codes)}")
        """
        ...


@runtime_checkable
class SDTMSpecRepositoryPort(Protocol):
    """Protocol for SDTM specification repository access.

    This interface provides access to SDTM Implementation Guide specifications,
    including domain definitions, variable metadata, and dataset structures.

    Implementations might load from:
    - CSV files (current implementation)
    - Database tables
    - CDISC Library API
    - Local cache with versioning

    Example:
        >>> def get_domain_spec(spec_repo: SDTMSpecRepositoryPort):
        ...     variables = spec_repo.get_domain_variables("DM")
        ...     for var in variables:
        ...         print(f"{var['Variable Name']}: {var['Label']}")
    """

    def get_domain_variables(self, domain_code: str) -> list[dict[str, str]]:
        """Retrieve variable specifications for a domain.

        Args:
            domain_code: SDTM domain code (e.g., "DM", "AE", "LB")

        Returns:
            List of variable specification dictionaries containing metadata
            such as Variable Name, Label, Type, Length, Role, etc.

        Example:
            >>> variables = spec_repo.get_domain_variables("DM")
            >>> for var in variables:
            ...     if var["Role"] == "Identifier":
            ...         print(f"Key variable: {var['Variable Name']}")
        """
        ...


@runtime_checkable
class DomainDefinitionPort(Protocol):
    """Protocol to retrieve SDTM domain definitions as domain entities."""

    def get_domain(self, code: str) -> "SDTMDomain":
        """Return the SDTM domain definition for a domain code."""
        ...

    def get_dataset_attributes(self, domain_code: str) -> dict[str, str] | None:
        """Retrieve dataset-level attributes for a domain.

        Args:
            domain_code: SDTM domain code (e.g., "DM", "AE", "LB")

        Returns:
            Dictionary with dataset attributes like class, label, and structure,
            or None if domain not found

        Example:
            >>> attrs = spec_repo.get_dataset_attributes("DM")
            >>> if attrs:
            ...     print(f"Class: {attrs['class']}")
            ...     print(f"Label: {attrs['label']}")
        """
        ...

    def list_available_domains(self) -> list[str]:
        """List all available SDTM domains in the specification.

        Returns:
            List of domain codes available in the SDTM specification

        Example:
            >>> domains = spec_repo.list_available_domains()
            >>> if "DM" in domains:
            ...     print("Demographics domain available")
        """
        ...


@runtime_checkable
class StudyDataRepositoryPort(Protocol):
    """Protocol for study data file access.

    This interface provides access to study source data files, abstracting
    away the specific file formats and storage locations.

    Implementations might read from:
    - CSV files (current implementation)
    - Excel workbooks
    - SAS datasets
    - Database tables
    - Cloud storage (S3, Azure Blob)

    Example:
        >>> def load_demographics(data_repo: StudyDataRepositoryPort):
        ...     dm_df = data_repo.read_dataset("DM.csv")
        ...     print(f"Loaded {len(dm_df)} subjects")
    """

    def read_dataset(self, file_path: str | Path) -> pd.DataFrame:
        """Read a study dataset file into a DataFrame.

        Args:
            file_path: Path to the dataset file (relative or absolute)

        Returns:
            DataFrame containing the dataset

        Raises:
            DataSourceNotFoundError: If file does not exist
            DataParseError: If file cannot be parsed

        Example:
            >>> df = data_repo.read_dataset("DM.csv")
            >>> print(df.columns.tolist())
            ['STUDYID', 'DOMAIN', 'USUBJID', 'SUBJID', ...]
        """
        ...

    def load_study_metadata(self, study_folder: Path) -> StudyMetadata:
        """Load study metadata from Items.csv and CodeLists.csv.

        Args:
            study_folder: Path to study folder containing metadata files

        Returns:
            StudyMetadata object with source columns and codelists

        Raises:
            MetadataLoadError: If metadata files cannot be loaded

        Example:
            >>> metadata = data_repo.load_study_metadata(Path("study001"))
            >>> for col_id, column in metadata.columns.items():
            ...     print(f"{col_id}: {column.label}")
        """
        ...

    def list_data_files(self, folder: Path, pattern: str = "*.csv") -> list[Path]:
        """List data files in a folder matching a pattern.

        Args:
            folder: Path to folder to search
            pattern: Glob pattern to match files (default: "*.csv")

        Returns:
            List of Path objects for matching files

        Example:
            >>> files = data_repo.list_data_files(Path("study001"))
            >>> domain_files = [f for f in files if f.stem.upper() in ["DM", "AE", "LB"]]
        """
        ...
