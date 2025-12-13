"""Data models for input/output utilities.

This module contains dataclasses and type definitions used for
dataset parsing and column analysis.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Dict


@dataclass(frozen=True)
class ColumnHint:
    """Lightweight stats about a column used during mapping heuristics.

    Attributes:
        is_numeric: True if the column contains numeric data
        unique_ratio: Ratio of unique values to non-null values (0.0 to 1.0)
        null_ratio: Ratio of null values to total rows (0.0 to 1.0)
    """

    is_numeric: bool
    unique_ratio: float
    null_ratio: float


# Type alias for column hints dictionary
Hints = Dict[str, ColumnHint]
