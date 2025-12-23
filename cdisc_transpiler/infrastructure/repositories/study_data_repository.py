"""Study Data Repository implementation.

This module provides access to study source data files and metadata
through a clean repository interface.
"""

from __future__ import annotations

from pathlib import Path

import pandas as pd

from ...application.ports.repositories import StudyDataRepositoryPort
from ...domain.entities.study_metadata import StudyMetadata
from ..io.csv_reader import CSVReader, CSVReadOptions
from ..io.exceptions import DataParseError, DataSourceNotFoundError
from .study_metadata_loader import load_study_metadata as _load_metadata


class StudyDataRepository:
    """Repository for study data file access.

    This implementation provides a unified interface for reading study
    datasets (CSV, Excel, SAS) and loading study metadata (Items.csv,
    CodeLists.csv).

    Example:
        >>> repo = StudyDataRepository()
        >>> df = repo.read_dataset("DM.csv")
        >>> print(f"Loaded {len(df)} subjects")
        >>> metadata = repo.load_study_metadata(Path("study001"))
        >>> print(f"Loaded {len(metadata.items or {})} columns")
    """

    def __init__(self, csv_reader: CSVReader | None = None):
        """Initialize the repository.

        Args:
            csv_reader: Optional CSV reader instance. Creates one if None.
        """
        self._csv_reader = csv_reader or CSVReader()

    def read_dataset(self, file_path: str | Path) -> pd.DataFrame:
        """Read a study dataset file into a DataFrame.

        Supports CSV, TSV, TXT, Excel (xls/xlsx), and SAS7BDAT formats.

        Args:
            file_path: Path to the dataset file (relative or absolute)

        Returns:
            DataFrame containing the dataset

        Raises:
            DataSourceNotFoundError: If file does not exist
            DataParseError: If file cannot be parsed
        """
        path = Path(file_path)

        if not path.exists():
            raise DataSourceNotFoundError(f"File not found: {path}")

        if not path.is_file():
            raise DataSourceNotFoundError(f"Not a file: {path}")

        ext = path.suffix.lower()

        # Route to appropriate reader based on extension
        if ext in (".csv", ".tsv", ".txt"):
            return self._read_csv(path)
        if ext in (".xls", ".xlsx"):
            return self._read_excel(path)
        if ext == ".sas7bdat":
            return self._read_sas(path)
        supported = ".csv, .tsv, .txt, .xls, .xlsx, .sas7bdat"
        raise DataParseError(f"Unsupported format '{ext}'. Supported: {supported}")

    def load_study_metadata(self, study_folder: Path) -> StudyMetadata:
        """Load study metadata from Items.csv and CodeLists.csv.

        Args:
            study_folder: Path to study folder containing metadata files

        Returns:
            StudyMetadata object with source columns and codelists.
            Returns empty metadata if files not found (graceful degradation).
        """
        if not study_folder.exists():
            return StudyMetadata(source_path=study_folder)

        # Delegate to existing metadata loader
        return _load_metadata(study_folder)

    def list_data_files(self, folder: Path, pattern: str = "*.csv") -> list[Path]:
        """List data files in a folder matching a pattern.

        Args:
            folder: Path to folder to search
            pattern: Glob pattern to match files (default: "*.csv")

        Returns:
            List of Path objects for matching files
        """
        if not folder.exists() or not folder.is_dir():
            return []

        return sorted(folder.glob(pattern))

    def _read_csv(self, path: Path) -> pd.DataFrame:
        """Read CSV file using the CSVReader.

        Args:
            path: Path to CSV file

        Returns:
            DataFrame with CSV data
        """
        options = CSVReadOptions(
            normalize_headers=True,
            strict_na_handling=True,
            detect_header_row=True,
        )
        return self._csv_reader.read(path, options)

    def _read_excel(self, path: Path) -> pd.DataFrame:
        """Read Excel file.

        Args:
            path: Path to Excel file

        Returns:
            DataFrame with Excel data
        """
        try:
            return pd.read_excel(path)
        except Exception as e:
            raise DataParseError(f"Failed to read Excel file {path}: {e}") from e

    def _read_sas(self, path: Path) -> pd.DataFrame:
        """Read SAS7BDAT file.

        Args:
            path: Path to SAS file

        Returns:
            DataFrame with SAS data

        Raises:
            DataParseError: If pyreadstat is not installed
        """
        try:
            import pyreadstat
        except ModuleNotFoundError as e:
            raise DataParseError(
                "pyreadstat is required to read SAS files (optional dependency). "
                "Install with: pip install pyreadstat"
            ) from e

        try:
            frame, _ = pyreadstat.read_sas7bdat(str(path))
            return frame
        except Exception as e:
            raise DataParseError(f"Failed to read SAS file {path}: {e}") from e


# Verify protocol compliance at runtime (duck typing)
def _verify_protocol_compliance() -> None:
    """Verify StudyDataRepository implements StudyDataRepositoryPort."""
    repo: StudyDataRepositoryPort = StudyDataRepository()
    assert isinstance(repo, StudyDataRepositoryPort)


_verify_protocol_compliance()
