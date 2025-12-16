"""Shared utilities for XML generation.

Common helpers used by Dataset-XML and Define-XML writers.
"""

from __future__ import annotations

from pathlib import Path


def tag(namespace: str, name: str) -> str:
    """Create a namespaced XML tag string."""
    return f"{{{namespace}}}{name}"


def attr(namespace: str, name: str) -> str:
    """Create a namespaced XML attribute string."""
    return f"{{{namespace}}}{name}"


def safe_href(href: str) -> str:
    """Sanitize dataset href to comply with SAS naming constraints."""
    if not href:
        return href
    path = Path(href)
    stem = path.stem[:8]
    new_name = f"{stem}{path.suffix}".lower()
    safe = str(path.with_name(new_name))
    return safe[:64]
