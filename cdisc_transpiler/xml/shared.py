"""Shared utilities for XML generation.

Common functions used by both Define-XML and Dataset-XML modules.
"""

from pathlib import Path


def tag(namespace: str, name: str) -> str:
    """Create a namespaced XML tag string.
    
    Args:
        namespace: The XML namespace URI
        name: The element name
        
    Returns:
        Namespaced tag string in format {namespace}name
    """
    return f"{{{namespace}}}{name}"


def attr(namespace: str, name: str) -> str:
    """Create a namespaced XML attribute string.
    
    Args:
        namespace: The XML namespace URI
        name: The attribute name
        
    Returns:
        Namespaced attribute string in format {namespace}name
    """
    return f"{{{namespace}}}{name}"


def safe_href(href: str) -> str:
    """Sanitize dataset href to comply with SAS naming constraints.
    
    Clamps dataset href to SAS 8-char dataset name plus extension and length cap.
    
    Args:
        href: The original href string
        
    Returns:
        Sanitized href string (max 64 chars, 8-char stem, lowercase)
    """
    if not href:
        return href
    path = Path(href)
    stem = path.stem[:8]
    new_name = f"{stem}{path.suffix}".lower()
    safe = str(path.with_name(new_name))
    return safe[:64]
