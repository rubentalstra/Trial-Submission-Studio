"""Input/output utilities for dataset parsing and column analysis.

This module provides functions for:
- Loading datasets from various formats (CSV, Excel, SAS)
- Extracting column hints for mapping heuristics
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Dict

import pandas as pd

try:  # pragma: no cover - optional dependency at runtime
    import pyreadstat
except ModuleNotFoundError:  # pragma: no cover
    pyreadstat = None


# =============================================================================
# Exceptions
# =============================================================================


class ParseError(RuntimeError):
    """Raised when an input file cannot be parsed."""


# =============================================================================
# Column Hints
# =============================================================================


@dataclass(frozen=True)
class ColumnHint:
    """Lightweight stats about a column used during mapping heuristics."""

    is_numeric: bool
    unique_ratio: float
    null_ratio: float


Hints = Dict[str, ColumnHint]


def build_column_hints(frame: pd.DataFrame) -> Hints:
    """Derive hints (numeric-ness, sparsity, uniqueness) for each column."""
    hints: Hints = {}
    row_count = len(frame)
    for column in frame.columns:
        series = frame[column]
        is_numeric = pd.api.types.is_numeric_dtype(series)
        non_null = int(series.notna().sum())
        unique_non_null = series.nunique(dropna=True)
        unique_ratio = float(unique_non_null / non_null) if non_null else 0.0
        null_ratio = float(1 - (non_null / row_count)) if row_count else 0.0
        hints[column] = ColumnHint(
            is_numeric=bool(is_numeric),
            unique_ratio=unique_ratio,
            null_ratio=null_ratio,
        )
    return hints


# =============================================================================
# File Readers
# =============================================================================


Reader = Callable[[Path], pd.DataFrame]


def _read_csv(path: Path) -> pd.DataFrame:
    """Read CSV file with intelligent header detection.

    Many mock datasets include a human-readable header row followed by a code row.
    Detect this pattern and, when present, use the second row as the header.
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
    """Read Excel file."""
    return pd.read_excel(path)


def _read_sas(path: Path) -> pd.DataFrame:
    """Read SAS7BDAT file."""
    if pyreadstat is None:
        raise ParseError("pyreadstat is required to read SAS files")
    frame, _ = pyreadstat.read_sas7bdat(str(path))
    return frame


_READERS: dict[str, Reader] = {
    ".csv": _read_csv,
    ".txt": _read_csv,
    ".tsv": _read_csv,
    ".xls": _read_excel,
    ".xlsx": _read_excel,
    ".sas7bdat": _read_sas,
}


def load_input_dataset(path: str | Path) -> pd.DataFrame:
    """Load a dataset from various file formats.

    Args:
        path: Path to the input file (CSV, Excel, or SAS7BDAT).

    Returns:
        DataFrame containing the loaded data.

    Raises:
        ParseError: If the file cannot be read or has no columns.
    """
    file_path = Path(path)
    if not file_path.exists():
        raise ParseError(f"Input file not found: {file_path}")
    if not file_path.is_file():
        raise ParseError(f"Input path is not a file: {file_path}")

    ext = file_path.suffix.lower()
    reader = _READERS.get(ext)
    if reader is None:
        supported = ", ".join(sorted(_READERS))
        raise ParseError(f"Unsupported format '{ext}'. Supported: {supported}")

    try:
        frame = reader(file_path)
    except Exception as exc:  # pragma: no cover - pass through real errors
        raise ParseError(f"Failed to parse {file_path}: {exc}") from exc

    if frame.shape[1] == 0:
        raise ParseError("Input data contains no columns")

    return frame
