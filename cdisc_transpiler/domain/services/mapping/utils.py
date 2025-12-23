"""Utility functions for mapping and column name operations.

This module contains helper functions used by the mapping engines and
column name handling throughout the CDISC Transpiler.

SDTM Reference:
    Column names follow SAS naming conventions which restrict names to
    8 characters of alphanumeric and underscore characters. The quoting
    mechanism allows special characters in source column names.
"""

import re

# Pattern for SAS name literal syntax: "column name"n
_SAS_NAME_LITERAL_RE = re.compile(r'^(?P<quoted>"(?:[^"]|"")*")n$', re.IGNORECASE)


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
    """Make column name safe for SAS using name literal syntax.

    SAS column names must start with a letter or underscore and contain
    only alphanumeric characters and underscores. Names with special
    characters are quoted using the SAS name literal syntax: "name"n

    Args:
        column: Source column name

    Returns:
        SAS-safe column name (quoted if necessary)

    Example:
        >>> safe_column_name("USUBJID")
        'USUBJID'
        >>> safe_column_name("Subject ID")
        '"Subject ID"n'
    """
    if re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", column):
        return column
    escaped = column.replace('"', '""')
    return f'"{escaped}"n'


def unquote_column_name(name: str | None) -> str:
    """Remove SAS name literal quoting from a column name.

    Reverses the safe_column_name() operation by extracting the original
    column name from SAS name literal syntax ("name"n).

    Args:
        name: Column name that may be quoted with SAS name literal syntax

    Returns:
        Unquoted column name, or empty string if name is None/empty

    Example:
        >>> unquote_column_name('"Subject ID"n')
        'Subject ID'
        >>> unquote_column_name('USUBJID')
        'USUBJID'
    """
    if not name:
        return ""
    name_str = str(name)
    match = _SAS_NAME_LITERAL_RE.fullmatch(name_str)
    if not match:
        return name_str
    quoted = match.group("quoted")
    # Remove surrounding quotes and unescape doubled quotes
    return quoted[1:-1].replace('""', '"')
