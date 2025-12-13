"""Metadata loading and parsing for automatic SDTM mapping.

This module provides functionality to:
- Load Items.csv (source column definitions) and CodeLists.csv (value mappings)
- Parse and validate metadata structures
- Provide automatic mapping suggestions from source data to SDTM variables
- Apply codelist transformations to convert coded values to their text equivalents
"""

from __future__ import annotations

from .models import (
    CodeList,
    CodeListValue,
    SourceColumn,
    StudyMetadata,
)
from .loaders import (
    MetadataLoadError,
    discover_metadata_files,
    load_codelists_csv,
    load_items_csv,
    load_study_metadata,
)
from .mapping import (
    get_value_transformer,
    infer_sdtm_target,
)

__all__ = [
    # Models
    "CodeList",
    "CodeListValue",
    "SourceColumn",
    "StudyMetadata",
    # Loaders
    "MetadataLoadError",
    "discover_metadata_files",
    "load_codelists_csv",
    "load_items_csv",
    "load_study_metadata",
    # Mapping
    "get_value_transformer",
    "infer_sdtm_target",
]
