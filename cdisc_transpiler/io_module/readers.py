"""File reading utilities for various data formats.

This module provides functions for loading datasets from
CSV, Excel, and SAS formats with intelligent header detection.

NOTE: This module is a compatibility wrapper. The actual implementation
has been moved to `cdisc_transpiler.infrastructure.repositories.study_data_repository`.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Callable

import pandas as pd

if TYPE_CHECKING:
    from ..infrastructure.repositories.study_data_repository import StudyDataRepository


class ParseError(RuntimeError):
    """Raised when an input file cannot be parsed."""


# Type alias for reader functions
Reader = Callable[[Path], pd.DataFrame]


def load_input_dataset(path: str | Path) -> pd.DataFrame:
    """Load a dataset from various file formats.

    Supports CSV, TSV, TXT, Excel (xls/xlsx), and SAS7BDAT formats.

    NOTE: This function is a compatibility wrapper. New code should use
    `StudyDataRepository.read_dataset()` directly.

    Args:
        path: Path to the input file

    Returns:
        DataFrame containing the loaded data

    Raises:
        ParseError: If the file cannot be read or has no columns
    """
    # Lazy import to avoid circular import issues
    from ..infrastructure.io.exceptions import DataParseError, DataSourceNotFoundError
    from ..infrastructure.repositories.study_data_repository import StudyDataRepository

    repo = StudyDataRepository()

    try:
        return repo.read_dataset(path)
    except DataSourceNotFoundError as exc:
        raise ParseError(str(exc)) from exc
    except DataParseError as exc:
        raise ParseError(str(exc)) from exc


# Keep these for backwards compatibility with internal code
# that may import them directly
def _read_csv(path: Path) -> pd.DataFrame:
    """Read CSV file with intelligent header detection.

    Many mock datasets include a human-readable header row followed by a code row.
    Detect this pattern and, when present, use the second row as the header.

    Args:
        path: Path to CSV file

    Returns:
        DataFrame with data from the CSV file
    """
    sample = pd.read_csv(path, nrows=2, header=None)
    header_row = 0
    if not sample.empty and len(sample) >= 2:
        first = sample.iloc[0].astype(str)
        second = sample.iloc[1].astype(str)
        first_has_spaces = first.str.contains(r"\s").mean() > 0.5
        # Treat CamelCase "code" rows as well as all-caps codes as identifiers
        second_is_codes = second.str.match(r"^[A-Za-z][A-Za-z0-9_]*$").mean() > 0.3
        if first_has_spaces and second_is_codes:
            header_row = 1
    return pd.read_csv(path, header=header_row)


def _read_excel(path: Path) -> pd.DataFrame:
    """Read Excel file.

    Args:
        path: Path to Excel file (.xls or .xlsx)

    Returns:
        DataFrame with data from the Excel file
    """
    return pd.read_excel(path)


def _read_sas(path: Path) -> pd.DataFrame:
    """Read SAS7BDAT file.

    Args:
        path: Path to SAS file

    Returns:
        DataFrame with data from the SAS file

    Raises:
        ParseError: If pyreadstat is not installed
    """
    try:
        import pyreadstat  # type: ignore[import-untyped]
    except ModuleNotFoundError:
        raise ParseError("pyreadstat is required to read SAS files")
    frame, _ = pyreadstat.read_sas7bdat(str(path))
    return frame


# Mapping of file extensions to reader functions (for backwards compatibility)
READERS: dict[str, Reader] = {
    ".csv": _read_csv,
    ".txt": _read_csv,
    ".tsv": _read_csv,
    ".xls": _read_excel,
    ".xlsx": _read_excel,
    ".sas7bdat": _read_sas,
}
