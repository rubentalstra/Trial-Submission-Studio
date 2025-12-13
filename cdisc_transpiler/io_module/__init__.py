"""Input/output utilities for dataset parsing and column analysis.

This module provides a clean, modular interface for:
- Loading datasets from various formats (CSV, Excel, SAS)
- Extracting column hints for mapping heuristics

The module is organized into:
- models: Data classes (ColumnHint, Hints type)
- readers: File reading utilities (load_input_dataset)
- hints: Column analysis (build_column_hints)
"""

from __future__ import annotations

from .hints import build_column_hints
from .models import ColumnHint, Hints
from .readers import ParseError, load_input_dataset

__all__ = [
    # Models
    "ColumnHint",
    "Hints",
    # Readers
    "ParseError",
    "load_input_dataset",
    # Hints
    "build_column_hints",
]
