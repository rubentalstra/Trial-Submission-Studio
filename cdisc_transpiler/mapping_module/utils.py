"""Utility functions for mapping operations.

This module contains helper functions used by the mapping engines.
"""

from __future__ import annotations

import re


def normalize_text(text: str) -> str:
    """Normalize text for matching (removes non-alphanumeric chars, uppercases).

    Args:
        text: Text to normalize

    Returns:
        Normalized text (uppercase alphanumeric only)

    Example:
        >>> normalize_text("Subject ID")
        'SUBJECTID'
    """
    return re.sub(r"[^A-Z0-9]", "", text.upper())


def safe_column_name(column: str) -> str:
    """Make column name safe for SAS.

    Args:
        column: Source column name

    Returns:
        SAS-safe column name (quoted if necessary)

    Example:
        >>> safe_column_name("valid_name")
        'valid_name'
        >>> safe_column_name("invalid-name")
        '"invalid-name"n'
    """
    if re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", column):
        return column
    escaped = column.replace('"', '""')
    return f'"{escaped}"n'
