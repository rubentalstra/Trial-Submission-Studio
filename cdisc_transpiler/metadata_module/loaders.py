"""Metadata loaders for Items.csv and CodeLists.csv files.

NOTE: This module is a compatibility wrapper. The actual implementation
has been moved to `cdisc_transpiler.infrastructure.repositories.study_metadata_loader`.
"""

from __future__ import annotations


# Re-export from infrastructure for backwards compatibility
from ..infrastructure.repositories.study_metadata_loader import (
    MetadataLoadError,
    load_items_csv,
    load_codelists_csv,
    discover_metadata_files,
    load_study_metadata,
    # Also export helper functions for backwards compatibility
    detect_header_row,
    normalize_column_names,
    find_column,
)

# Keep these for backwards compatibility with existing imports
from ..domain.entities.study_metadata import (
    CodeList,
    CodeListValue,
    SourceColumn,
    StudyMetadata,
)

__all__ = [
    "MetadataLoadError",
    "load_items_csv",
    "load_codelists_csv",
    "discover_metadata_files",
    "load_study_metadata",
    # Helper functions
    "detect_header_row",
    "normalize_column_names",
    "find_column",
    # Re-export domain entities for convenience
    "CodeList",
    "CodeListValue",
    "SourceColumn",
    "StudyMetadata",
]
