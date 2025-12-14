"""Utility functions for Dataset-XML generation.

This module re-exports shared utilities from the parent xml module and
provides Dataset-XML specific helper functions.
"""

import pandas as pd
from ..utils import tag, attr
from .constants import SHARED_VARIABLE_OIDS


def generate_item_oid(variable_name: str, domain_code: str) -> str:
    """Generate ItemOID following CDISC standard conventions.

    Per CDISC Dataset-XML 1.0 standard:
    - Shared variables (STUDYID, USUBJID) use IT.{VARIABLE} without domain prefix
    - Domain-specific variables use IT.{DOMAIN}.{VARIABLE}

    Args:
        variable_name: Variable name
        domain_code: Domain code

    Returns:
        ItemOID string
    """
    name = variable_name.upper()
    if name in SHARED_VARIABLE_OIDS:
        return f"IT.{name}"
    return f"IT.{domain_code.upper()}.{variable_name}"


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
        if bool(pd.isna(value)):
            return ""
    except Exception:
        pass

    # Convert to string
    if isinstance(value, (int, float)):
        # Keep numeric precision
        if isinstance(value, float):
            # Remove trailing zeros for floats
            return f"{value:g}"
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
