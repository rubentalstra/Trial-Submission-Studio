"""Utility functions for domain metadata processing."""

from __future__ import annotations

from .constants import GENERAL_CLASS_ALIASES, GENERAL_OBSERVATION_CLASSES


def get_domain_class(domain_code: str) -> str:
    """Get the SDTM class for a domain dynamically from metadata.

    This function retrieves the General Observation Class (e.g., 'EVENTS',
    'FINDINGS', 'INTERVENTIONS') for a given domain code by looking up
    the domain definition from the SDTMIG metadata.

    Args:
        domain_code: SDTM domain code (e.g., 'DM', 'AE', 'LB')

    Returns:
        SDTM class name (e.g., 'EVENTS', 'FINDINGS') or 'Unknown' if not found.
        For SUPP domains, returns 'Supplemental Qualifiers'.

    Example:
        >>> get_domain_class('AE')
        'EVENTS'
        >>> get_domain_class('LB')
        'FINDINGS'
        >>> get_domain_class('SUPPAE')
        'Supplemental Qualifiers'
    """
    # Import here to avoid circular imports
    from .registry import get_domain

    code = domain_code.upper()
    # Handle SUPP domains specially
    if code.startswith("SUPP"):
        return "Supplemental Qualifiers"

    try:
        domain = get_domain(code)
        return domain.class_name or "Unknown"
    except KeyError:
        return "Unknown"


def normalize_class(value: str | None) -> str:
    """Normalize class strings for comparisons."""
    if not value:
        return ""
    return value.strip().upper().replace("-", " ")


def normalize_general_class(value: str | None) -> str:
    """Map class names to the three General Observation Classes."""
    normalized = normalize_class(value)
    return GENERAL_CLASS_ALIASES.get(normalized, normalized)


def normalize_type(raw: str | None) -> str:
    """Map CSV type strings to SDTM types."""
    if not raw:
        return "Char"
    lower = raw.strip().lower()
    return "Num" if lower.startswith("num") else "Char"


def clean_codelist(raw: str | None) -> str | None:
    """Normalize codelist strings from CSV to a single CDISC CT code."""
    if not raw:
        return None
    text = raw.strip()
    if not text:
        return None
    # Some cells may contain multiple codes separated by delimiters; take the first.
    for sep in [";", ",", " "]:
        if sep in text:
            parts = [p for p in (part.strip() for part in text.split(sep)) if p]
            if parts:
                return parts[0]
    return text


def core_priority(core: str | None) -> int:
    """Priority of Core column for selecting canonical templates."""
    order = {"REQ": 3, "EXP": 2, "PERM": 1}
    return order.get((core or "").strip().upper(), 0)


def infer_implements(
    var_name: str, domain_code: str, class_name: str, role: str | None
) -> str | None:
    """Return generalized placeholder (e.g., --SEQ) for Identifier/Timing variables."""
    general_class = normalize_general_class(class_name)
    if general_class not in GENERAL_OBSERVATION_CLASSES:
        return None
    if (role or "").strip().lower() not in ("identifier", "timing"):
        return None
    name = (var_name or "").strip().upper()
    if not name:
        return None
    dom = (domain_code or "").strip().upper()
    if name.startswith(dom) and len(name) > len(dom):
        return f"--{name[len(dom) :]}"
    return name
