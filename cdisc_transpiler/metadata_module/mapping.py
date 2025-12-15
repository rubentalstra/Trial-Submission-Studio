"""SDTM mapping utilities for inferring target variables and transformers.

This module uses dynamic pattern generation from domain metadata
to provide flexible and accurate variable mapping.
"""

from __future__ import annotations

import re
from typing import Any, Callable

import pandas as pd

from ..domain.entities.study_metadata import SourceColumn, StudyMetadata
from ..domains_module import get_domain
from ..mapping_module.pattern_builder import build_variable_patterns


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

    Uses dynamic patterns generated from domain metadata.

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

    # Get domain and build dynamic patterns
    try:
        domain = get_domain(domain_code)
        variable_patterns = build_variable_patterns(domain)

        # Check patterns for each variable
        for target_var, patterns in variable_patterns.items():
            for pattern in patterns:
                if normalized == pattern:
                    return target_var

        # Check if Items.csv provides hints
        if items:
            item = items.get(normalized)
            if item:
                # If label contains hints, check against patterns
                label_normalized = _normalize_column_name(item.label)
                for target_var, patterns in variable_patterns.items():
                    for pattern in patterns:
                        if pattern in label_normalized:
                            return target_var
    except KeyError:
        # Domain not found in registry - this is expected for invalid domain codes
        # or domains not yet loaded in the CSV metadata
        pass

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
