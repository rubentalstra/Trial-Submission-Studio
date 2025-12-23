"""Utility helpers for SDTM spec/metadata processing (infrastructure)."""

from ...domain.entities.sdtm_classes import (
    GENERAL_OBSERVATION_CLASSES,
    core_priority,
    normalize_class,
    normalize_general_class,
)


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
    "core_priority",
    "infer_implements",
    "normalize_class",
    "normalize_general_class",
]
