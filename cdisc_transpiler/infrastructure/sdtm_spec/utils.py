"""Utility helpers for SDTM spec/metadata processing (infrastructure)."""

from __future__ import annotations

from ...domain.entities.sdtm_classes import (
    GENERAL_OBSERVATION_CLASSES,
    core_priority,
    normalize_class,
    normalize_general_class,
)


def get_domain_class(domain_code: str) -> str:
    """Return the SDTM class for a domain via the registry."""
    from .registry import get_domain

    code = domain_code.upper()
    if code.startswith("SUPP"):
        return "Supplemental Qualifiers"

    try:
        domain = get_domain(code)
        return domain.class_name or "Unknown"
    except KeyError:
        return "Unknown"


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


__all__ = [
    "GENERAL_OBSERVATION_CLASSES",
    "normalize_class",
    "normalize_general_class",
    "core_priority",
    "get_domain_class",
    "infer_implements",
]
