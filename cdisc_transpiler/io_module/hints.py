"""Column hint analysis utilities.

This module provides functions for analyzing DataFrame columns
to derive hints used in mapping heuristics.
"""

from __future__ import annotations

import pandas as pd

from .models import ColumnHint, Hints


def build_column_hints(frame: pd.DataFrame) -> Hints:
    """Derive hints (numeric-ness, sparsity, uniqueness) for each column.

    Analyzes each column in the DataFrame to produce lightweight statistics
    that help inform mapping heuristics.

    Args:
        frame: DataFrame to analyze

    Returns:
        Dictionary mapping column names to ColumnHint objects
    """
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
