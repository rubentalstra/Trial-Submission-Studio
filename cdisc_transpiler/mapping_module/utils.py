"""Utility functions for mapping and column name operations.

NOTE: This module is a compatibility wrapper. The actual implementation
has been moved to `cdisc_transpiler.domain.services.mapping.utils`.
"""

from __future__ import annotations

# Re-export from domain services for backwards compatibility
from ..domain.services.mapping.utils import (
    normalize_text,
    safe_column_name,
    unquote_column_name,
)

__all__ = ["normalize_text", "safe_column_name", "unquote_column_name"]
