"""Mapping services for SDTM variable suggestion.

This package provides the core business logic for mapping source columns
to SDTM target variables using fuzzy matching, pattern recognition, and
metadata-aware mapping.
"""

from .engine import MappingEngine
from .metadata_mapper import MetadataAwareMapper
from .pattern_builder import build_variable_patterns, get_domain_suffix_patterns
from .utils import normalize_text, safe_column_name, unquote_column_name

__all__ = [
    # Engines
    "MappingEngine",
    "MetadataAwareMapper",
    # Pattern building
    "build_variable_patterns",
    "get_domain_suffix_patterns",
    # Utilities
    "normalize_text",
    "safe_column_name",
    "unquote_column_name",
]
