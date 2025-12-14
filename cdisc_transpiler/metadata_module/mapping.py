"""SDTM mapping utilities for inferring target variables and transformers.

This module uses the centralized SDTM_INFERENCE_PATTERNS from mapping_module.constants
to avoid duplicate pattern definitions across the codebase.
"""

from __future__ import annotations

import re
from typing import Any, Callable

import pandas as pd

from .models import SourceColumn, StudyMetadata
from ..mapping_module.constants import SDTM_INFERENCE_PATTERNS


def _normalize_column_name(name: str) -> str:
    """Normalize a column name for comparison.

    Args:
        name: The column name to normalize

    Returns:
        Normalized column name (uppercase, alphanumeric only)
    """
    return re.sub(r"[^A-Z0-9]", "", name.upper())


def infer_sdtm_target(
    source_column: str,
    domain_code: str,
    items: dict[str, SourceColumn] | None = None,
) -> str | None:
    """Infer the SDTM target variable for a source column.

    Uses centralized SDTM_INFERENCE_PATTERNS from mapping_module.constants.

    Args:
        source_column: The source column name
        domain_code: The target SDTM domain code
        items: Optional Items.csv metadata

    Returns:
        The inferred SDTM variable name, or None if no match
    """
    normalized = _normalize_column_name(source_column)
    domain_prefix = domain_code.upper()

    # Check if column already has domain prefix (e.g., AETERM, LBORRES)
    if normalized.startswith(domain_prefix):
        # Could be a valid SDTM variable already
        return source_column.upper()

    # Check domain suffix patterns (domain-specific variables like --TERM, --ORRES)
    for suffix, patterns in SDTM_INFERENCE_PATTERNS.get("_DOMAIN_SUFFIXES", {}).items():
        for pattern in patterns:
            if _normalize_column_name(pattern) == normalized:
                # Return domain-prefixed variable (e.g., AETERM, LBORRES)
                return domain_prefix + suffix

    # Check if Items.csv provides hints
    if items:
        item = items.get(normalized)
        if item:
            # If label contains SDTM variable name hints
            label_normalized = _normalize_column_name(item.label)
            # Check domain suffix patterns in label
            for suffix, patterns in SDTM_INFERENCE_PATTERNS.get("_DOMAIN_SUFFIXES", {}).items():
                for pattern in patterns:
                    if _normalize_column_name(pattern) in label_normalized:
                        return domain_prefix + suffix

    return None


def get_value_transformer(
    source_column: str,
    metadata: StudyMetadata,
    target_variable: str,
) -> Callable[[Any], Any] | None:
    """Get a transformation function for a source column.

    Args:
        source_column: The source column name
        metadata: The study metadata
        target_variable: The SDTM target variable

    Returns:
        A callable that transforms values, or None if no transformation needed
    """
    # Check if there's a code column with a codelist
    code_column = source_column + "CD"
    column_def = metadata.get_column(code_column)

    if column_def and column_def.format_name:
        codelist = metadata.get_codelist(column_def.format_name)
        if codelist:
            # Return a transformer that converts codes to text
            def transformer(value: Any) -> Any:
                if pd.isna(value):
                    return value
                result = codelist.get_text(value)
                return result if result is not None else value

            return transformer

    # Check the column itself for a codelist
    column_def = metadata.get_column(source_column)
    if column_def and column_def.format_name:
        codelist = metadata.get_codelist(column_def.format_name)
        if codelist:

            def transformer(value: Any) -> Any:
                if pd.isna(value):
                    return value
                result = codelist.get_text(value)
                return result if result is not None else value

            return transformer

    return None
