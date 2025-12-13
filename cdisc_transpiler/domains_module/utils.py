"""Utility functions for domain metadata processing."""

from __future__ import annotations

from .constants import GENERAL_CLASS_ALIASES, GENERAL_OBSERVATION_CLASSES


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
