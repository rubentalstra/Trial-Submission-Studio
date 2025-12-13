"""CSV utilities for metadata loading."""

from __future__ import annotations

import pandas as pd


def find_column(df: pd.DataFrame, options: list[str]) -> str | None:
    """Find a column by trying various name options (case-insensitive).

    Args:
        df: The DataFrame to search
        options: List of possible column names to try

    Returns:
        The actual column name if found, None otherwise
    """
    for opt in options:
        if opt in df.columns:
            return opt
        # Case-insensitive match
        for col in df.columns:
            if col.lower() == opt.lower():
                return col
    return None


def detect_header_row(df: pd.DataFrame) -> int:
    """Detect which row contains the actual header.

    Items.csv and CodeLists.csv often have a human-readable header row
    followed by a code row. This detects that pattern.

    Args:
        df: DataFrame with initial rows loaded (no header assumed)

    Returns:
        Row index to use as header (0 or 1)
    """
    if len(df) < 2:
        return 0

    # Check if first row has spaces (human-readable) and second row is codes
    first_row = df.iloc[0].astype(str)
    second_row = df.iloc[1].astype(str)

    first_has_spaces = first_row.str.contains(r"\s").any()
    second_is_codes = second_row.str.match(r"^[A-Za-z][A-Za-z0-9_]*$").all()

    if first_has_spaces and second_is_codes:
        return 1

    return 0


def normalize_column_names(df: pd.DataFrame) -> pd.DataFrame:
    """Normalize column names by stripping whitespace.

    Args:
        df: DataFrame to normalize

    Returns:
        DataFrame with normalized column names
    """
    df.columns = [str(c).strip() for c in df.columns]
    return df
