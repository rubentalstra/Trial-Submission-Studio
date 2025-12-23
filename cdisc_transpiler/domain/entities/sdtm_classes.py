"""General Observation Class helpers.

These are domain-level concepts used to relate domain-specific variable names
(e.g., AESTDTC) to generalized placeholders (e.g., --STDTC).

Kept in the domain layer because this is pure business logic.
"""

# General Observation Classes (SDTM v2.0 Section 3.2)
GENERAL_OBSERVATION_CLASSES = {"INTERVENTIONS", "EVENTS", "FINDINGS"}

# Aliases for class names that map to General Observation Classes.
# "FINDINGS ABOUT" domains (FA) are treated as FINDINGS class.
GENERAL_CLASS_ALIASES = {"FINDINGS ABOUT": "FINDINGS"}


def normalize_class(value: str | None) -> str:
    if not value:
        return ""
    return value.strip().upper().replace("-", " ")


def normalize_general_class(value: str | None) -> str:
    normalized = normalize_class(value)
    return GENERAL_CLASS_ALIASES.get(normalized, normalized)


def core_priority(core: str | None) -> int:
    order = {"REQ": 3, "EXP": 2, "PERM": 1}
    return order.get((core or "").strip().upper(), 0)
