"""File reading utilities for various data formats.

This module provides functions for loading datasets from
CSV, Excel, and SAS formats with intelligent header detection.
"""

from __future__ import annotations

from pathlib import Path
from typing import Callable

import pandas as pd

try:  # pragma: no cover - optional dependency at runtime
    import pyreadstat
except ModuleNotFoundError:  # pragma: no cover
    pyreadstat = None


class ParseError(RuntimeError):
    """Raised when an input file cannot be parsed."""


# Type alias for reader functions
Reader = Callable[[Path], pd.DataFrame]


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
    if pyreadstat is None:
        raise ParseError("pyreadstat is required to read SAS files")
    frame, _ = pyreadstat.read_sas7bdat(str(path))
    return frame


# Mapping of file extensions to reader functions
READERS: dict[str, Reader] = {
    ".csv": _read_csv,
    ".txt": _read_csv,
    ".tsv": _read_csv,
    ".xls": _read_excel,
    ".xlsx": _read_excel,
    ".sas7bdat": _read_sas,
}


def load_input_dataset(path: str | Path) -> pd.DataFrame:
    """Load a dataset from various file formats.

    Supports CSV, TSV, TXT, Excel (xls/xlsx), and SAS7BDAT formats.

    Args:
        path: Path to the input file

    Returns:
        DataFrame containing the loaded data

    Raises:
        ParseError: If the file cannot be read or has no columns
    """
    file_path = Path(path)
    if not file_path.exists():
        raise ParseError(f"Input file not found: {file_path}")
    if not file_path.is_file():
        raise ParseError(f"Input path is not a file: {file_path}")

    ext = file_path.suffix.lower()
    reader = READERS.get(ext)
    if reader is None:
        supported = ", ".join(sorted(READERS))
        raise ParseError(f"Unsupported format '{ext}'. Supported: {supported}")

    try:
        frame = reader(file_path)
    except Exception as exc:  # pragma: no cover - pass through real errors
        raise ParseError(f"Failed to parse {file_path}: {exc}") from exc

    if frame.shape[1] == 0:
        raise ParseError("Input data contains no columns")

    return frame
