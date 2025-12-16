"""Utility functions for Dataset-XML generation.

This module re-exports shared utilities from the parent xml module and
provides Dataset-XML specific helper functions.
"""

from typing import Any, cast

import pandas as pd

from ..xml_utils import attr, tag


def generate_item_oid(variable_name: str, dataset_name: str) -> str:
    """Generate ItemOID following CDISC standard conventions.

    Per CDISC Dataset-XML 1.0 (and the CDISC MSG sample submission package fixtures
    used in this repo), ItemOIDs follow the pattern:
    - IT.{DOMAIN}.{VARIABLE}

    Args:
        variable_name: Variable name
        domain_code: Domain code

    Returns:
        ItemOID string
    """
    return f"IT.{dataset_name.upper()}.{variable_name.upper()}"


def is_null(value: object) -> bool:
    """Check if a value is null/NaN/empty.

    Args:
        value: Value to check

    Returns:
        True if value is null/NaN/empty
    """

    if value is None:
        return True
    if isinstance(value, float) and pd.isna(value):
        return True
    if isinstance(value, str) and value.strip() == "":
        return True
    return False


def format_value(value: object, column_name: str) -> str:
    """Format a value for Dataset-XML output.

    Args:
        value: Value to format
        column_name: Column name (for context)

    Returns:
        Formatted string value
    """

    if isinstance(value, (pd.Series, pd.DataFrame)):
        try:
            value = value.iloc[0]  # type: ignore[index]
        except Exception:
            return ""
    try:
        if bool(pd.isna(cast(Any, value))):
            return ""
    except Exception:
        pass

    # Convert to string
    if isinstance(value, (int, float)):
        # Keep numeric precision.
        # `g` defaults to 6 significant digits, which can round values and
        # diverge from the CDISC MSG sample submission package fixtures.
        if isinstance(value, float):
            return format(value, ".15g")
        return str(value)

    return str(value).strip()


def escape_xml(value: str) -> str:
    """Escape XML special characters in a string.

    Args:
        value: String value to escape

    Returns:
        XML-escaped string
    """
    return (
        value.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&apos;")
    )


__all__ = [
    "tag",
    "attr",
    "generate_item_oid",
    "is_null",
    "format_value",
    "escape_xml",
]
