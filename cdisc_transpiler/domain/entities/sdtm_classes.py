GENERAL_OBSERVATION_CLASSES = {"INTERVENTIONS", "EVENTS", "FINDINGS"}
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
