"""Unified CSV reader for consistent file I/O.

This module provides a single source of truth for CSV reading operations,
replacing multiple inconsistent implementations throughout the codebase.

Key Features:
- Consistent dtype and NA handling
- Optional header normalization
- Clear error messages
- Intelligent header row detection
- Configurable behavior without code changes
"""

from dataclasses import dataclass
from pathlib import Path
from typing import Any

import pandas as pd

from .exceptions import DataParseError, DataSourceNotFoundError


@dataclass(slots=True)
class CSVReadOptions:
    """Configuration options for CSV reading.

    Attributes:
        normalize_headers: Strip whitespace from column names
        strict_na_handling: Use strict NA handling (keep_default_na=False, na_values=[""])
        dtype: Data type for all columns (str) or per-column dict
        encoding: File encoding (default: utf-8)
        detect_header_row: Intelligently detect if second row is the real header
    """

    normalize_headers: bool = True
    strict_na_handling: bool = True
    # pandas accepts dtype as either a scalar dtype, a dtype type (e.g. `str`),
    # or a per-column mapping. The pandas typing stubs are restrictive here.
    dtype: Any = str
    encoding: str = "utf-8"
    detect_header_row: bool = True


class CSVReader:
    """Unified CSV reader with consistent behavior.

    This class provides a single interface for reading CSV files with
    consistent configuration and error handling. It replaces multiple
    implementations that had slightly different behaviors.

    Example:
        >>> reader = CSVReader()
        >>> df = reader.read(Path("data.csv"))
        >>> # Custom options
        >>> options = CSVReadOptions(normalize_headers=False, strict_na_handling=False)
        >>> df = reader.read(Path("data.csv"), options=options)
    """

    def read(
        self,
        path: Path,
        options: CSVReadOptions | None = None,
    ) -> pd.DataFrame:
        """Read CSV file with consistent behavior.

        Args:
            path: Path to CSV file
            options: Optional configuration (uses defaults if None)

        Returns:
            DataFrame with loaded data

        Raises:
            DataSourceNotFoundError: If file doesn't exist
            DataParseError: If file cannot be parsed
        """
        if options is None:
            options = CSVReadOptions()

        # Validate file exists
        if not path.exists():
            raise DataSourceNotFoundError(f"File not found: {path}")

        if not path.is_file():
            raise DataSourceNotFoundError(f"Not a file: {path}")

        try:
            # Detect header row if enabled
            header_row = 0
            if options.detect_header_row:
                header_row = self._detect_header_row(path)

            # Read CSV with consistent options
            df = pd.read_csv(
                path,
                header=header_row,
                dtype=options.dtype,
                keep_default_na=not options.strict_na_handling,
                na_values=[""] if options.strict_na_handling else None,
                encoding=options.encoding,
            )

        except FileNotFoundError as e:
            raise DataSourceNotFoundError(f"File not found: {path}") from e
        except pd.errors.ParserError as e:
            raise DataParseError(f"Failed to parse CSV {path}: {e}") from e
        except pd.errors.EmptyDataError as e:
            raise DataParseError(f"CSV file is empty: {path}") from e
        except UnicodeDecodeError as e:
            raise DataParseError(
                f"Encoding error reading {path}. Try a different encoding: {e}"
            ) from e
        except Exception as e:
            raise DataParseError(f"Unexpected error reading {path}: {e}") from e

        # Validate result
        if df.shape[1] == 0:
            raise DataParseError(f"CSV file has no columns: {path}")

        # Normalize headers if requested
        if options.normalize_headers:
            df = self._normalize_headers(df)

        return df

    def _detect_header_row(self, path: Path) -> int:
        """Detect which row contains the actual header.

        Many mock datasets include a human-readable header row followed by
        a code row. This detects that pattern and uses the second row.

        Args:
            path: Path to CSV file

        Returns:
            Row index to use as header (0 or 1)
        """
        try:
            # Read first 2 rows without assuming header
            sample = pd.read_csv(path, nrows=2, header=None)

            if sample.empty or len(sample) < 2:
                return 0

            # Check if first row has spaces (human-readable) and second is codes
            first_row = sample.iloc[0].astype(str)
            second_row = sample.iloc[1].astype(str)

            # First row should have spaces in most columns (human-readable text)
            first_has_spaces = first_row.str.contains(r"\s").mean() > 0.5
            # Second row should match code pattern in most columns
            # Treat CamelCase "code" rows as well as all-caps codes as identifiers
            second_is_codes = second_row.str.match(r"^[A-Z][A-Za-z0-9_]*$").mean() > 0.5

            # Both conditions must be true to use row 1 as header
            if first_has_spaces and second_is_codes:
                return 1

        except Exception:
            # If detection fails, fall back to row 0
            return 0

        return 0

    def _normalize_headers(self, df: pd.DataFrame) -> pd.DataFrame:
        """Normalize column names by stripping whitespace.

        Args:
            df: DataFrame to normalize

        Returns:
            DataFrame with normalized column names
        """
        df.columns = [str(col).strip() for col in df.columns]
        return df
