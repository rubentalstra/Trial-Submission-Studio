"""Utilities for generating SAS transformation code."""

from __future__ import annotations

from pandas import isna

from .mapping_module import ColumnMapping
from .terminology import get_controlled_terminology
from .domains import SDTMVariable


def _synonyms(*values: str) -> set[str]:
    """Return uppercased synonyms for canonical value matching."""
    return {value.strip().upper() for value in values if value}


# =============================================================================
# Domain-specific normalizers (not covered by standard CDISC CT)
# =============================================================================

_MENOSTAT = {
    ">=1 YEAR POST-MENOPAUSAL": _synonyms(
        ">=1 YEAR POST-MENOPAUSAL",
        ">=1 YEAR POST MENOPAUSAL",
        "GE 1 YEAR POST-MENOPAUSAL",
    ),
    "HYSTERECTOMY": _synonyms("HYSTERECTOMY"),
    "BILATERAL TUBAL LIGATION": _synonyms("BILATERAL TUBAL LIGATION"),
}

_SMOKE_STATUS = {
    "CURRENT": _synonyms("CURRENT", "SMOKING"),
    "FORMER": _synonyms("FORMER", "EX-SMOKER"),
    "NEVER": _synonyms("NEVER"),
    "UNKNOWN": _synonyms("UNKNOWN", "UNK"),
}

_QS_SEVERITY = {
    "ABSENT": _synonyms("ABSENT", "0"),
    "MILD": _synonyms("MILD", "1"),
    "MODERATE": _synonyms("MODERATE", "2"),
    "SEVERE": _synonyms("SEVERE", "3"),
}

_VSTTYPE = {
    "IN-CLINIC": _synonyms("IN-CLINIC", "IN CLINIC"),
    "VIRTUAL": _synonyms("VIRTUAL"),
    "PHONE": _synonyms("PHONE", "CALL"),
    "IN-HOME": _synonyms("IN-HOME", "IN HOME", "HOME"),
    "VISIT DID NOT OCCUR": _synonyms("VISIT DID NOT OCCUR", "MISSED VISIT"),
}

_IECAT = {
    "INCLUSION": _synonyms("INCLUSION"),
    "EXCLUSION": _synonyms("EXCLUSION"),
    "BOTH": _synonyms("BOTH"),
}

_PREG_TYPE = {
    "SERUM": _synonyms("SERUM"),
    "URINE": _synonyms("URINE"),
}

_PREG_RESULT = {
    "POSITIVE": _synonyms("POSITIVE", "POS", "REACTIVE"),
    "NEGATIVE": _synonyms("NEGATIVE", "NEG", "NONREACTIVE"),
}

_PE_RESULTS = {
    "NORMAL": _synonyms("NORMAL"),
    "ABNORMAL, NCS": _synonyms("ABNORMAL, NCS", "ABNORMAL - NCS"),
    "ABNORMAL, CS": _synonyms("ABNORMAL, CS", "ABNORMAL - CS"),
}

_ALC_FREQUENCY = {
    "DAILY": _synonyms("DAILY", "QD"),
    "WEEKLY": _synonyms("WEEKLY"),
    "MONTHLY": _synonyms("MONTHLY"),
    "OCCASIONALLY": _synonyms("OCCASIONALLY", "OCCASIONAL"),
}

_LIFESTYLE_CHANGES = {
    "VEGETARIAN DIET": _synonyms("VEGETARIAN DIET"),
    "PHYSICAL ACTIVITY": _synonyms("PHYSICAL ACTIVITY"),
    "SMOKING USE": _synonyms("SMOKING USE"),
    "ALCOHOL USE": _synonyms("ALCOHOL USE"),
}

_LCPHYSA_STATUS = {
    "PERFORMS VIGOROUS PHYSICAL ACTIVITIES (EG, HEAVY LIFTING, DIGGING, AEROBICS, OR FAST BICYCLING)": _synonyms(
        "Performs vigorous physical activities (eg, heavy lifting, digging, aerobics, or fast bicycling)"
    ),
    "PERFORMS ONLY MODERATE PHYSICAL ACTIVITIES (EG, CARRYING LIGHT LOADS, BICYCLING AT A REGULAR PACE, OR DOUBLES TENNIS; DOES NOT INCLUDE WALKING)": _synonyms(
        "Performs only moderate physical activities (eg, carrying light loads, bicycling at a regular pace, or doubles tennis; does not include walking)"
    ),
    "PERFORMS NO VIGOROUS OR MODERATE PHYSICAL ACTIVITIES AS DESCRIBED ABOVE AND IS ESSENTIALLY SEDENTARY FOR MORE THAN 5 DAYS/WEEK": _synonyms(
        "Performs no vigorous or moderate physical activities as described above and is essentially sedentary for more than 5 days/week"
    ),
}

_AGEU = {
    "YEARS": _synonyms("YEAR", "YEARS", "YRS"),
    "MONTHS": _synonyms("MONTH", "MONTHS", "MOS"),
}

# Variables that use Yes/No CT but need local mapping for SAS generation
_YES_NO_TARGETS = {
    "ICYN",
    "AESER",
    "CHILDPOT",
    "CHILDPOTY",
    "COMPLYN",
    "VSTATYN",
    "LCCHGYN",
    "LCVEGYN",
    "DAFMLYN",
    "DAMSSYN",
    "LBCCCOND",
    "LBCCPERF",
    "LBHMPERF",
    "LBSAPERF",
    "LBURPERF",
    "PEPERF",
    "PREGPERF",
    "PGAPERF",
    "VSPERF",
    "DISAMT_DAPERF",
    "RETAMT_DAPERF",
    "IEORRES",
    "MHONGO",
}


def _build_value_normalizers() -> dict[str, dict[str, set[str]]]:
    """Build value normalizers for variables not in standard CDISC CT."""
    normalizers: dict[str, dict[str, set[str]]] = {
        # Domain-specific normalizers
        "AGEU": _AGEU,
        "MENOSTAT": _MENOSTAT,
        "SMOKE_SUNCF": _SMOKE_STATUS,
        "ALC_SUNCF": _SMOKE_STATUS,
        "QSPGARS": _QS_SEVERITY,
        "VSTTYPE": _VSTTYPE,
        "IECAT": _IECAT,
        "PREGTYPE": _PREG_TYPE,
        "PREGORRES": _PREG_RESULT,
        "PE_ORRES": _PE_RESULTS,
        "ALC_FRQ": _ALC_FREQUENCY,
        "LCPHYSA": _LCPHYSA_STATUS,
    }

    # Lifestyle change targets
    for lifestyle_target in ("LCCHANGE1", "LCCHANGE2", "LCCHANGE3", "LCCHANGE4"):
        normalizers[lifestyle_target] = _LIFESTYLE_CHANGES

    return normalizers


# Canonical uppercase values mapped to sets of synonyms (non-CT variables only)
_VALUE_NORMALIZERS: dict[str, dict[str, set[str]]] = _build_value_normalizers()


# Variables that should be uppercased
_UPCASE_VARIABLES = {
    "AGEU",
    "MENOSTAT",
    "SMOKE_SUNCF",
    "ALC_SUNCF",
    "QSPGARS",
    "VSTTYPE",
    "IECAT",
    "PREGTYPE",
    "PREGORRES",
    "PE_ORRES",
    "ALC_FRQ",
    "LCPHYSA",
} | _YES_NO_TARGETS


def _get_ct_value_map(variable_name: str) -> dict[str, set[str]] | None:
    """Get value map from controlled terminology registry."""
    ct = get_controlled_terminology(variable_name)
    if ct is None:
        return None

    value_map: dict[str, set[str]] = {}
    for canonical in ct.submission_values:
        synonyms = {canonical}
        # Add synonyms from CT if available
        if ct.synonyms:
            for syn_key, syn_canonical in ct.synonyms.items():
                if syn_canonical == canonical:
                    synonyms.add(syn_key)
        value_map[canonical] = synonyms

    return value_map


def render_assignment(mapping: ColumnMapping, variable: SDTMVariable | None) -> str:
    """Return SAS statements that assign a target variable with normalization."""
    target_name = mapping.target_variable.upper()

    if mapping.transformation:
        expr = mapping.transformation
        return f"{mapping.target_variable} = {expr};"

    # First check controlled terminology registry
    ct_value_map = _get_ct_value_map(target_name)
    if ct_value_map:
        return _render_value_map(mapping, ct_value_map)

    # Then check local normalizers
    value_map = _VALUE_NORMALIZERS.get(target_name)
    if value_map:
        return _render_value_map(mapping, value_map)

    # Default assignment
    expr = mapping.source_column
    is_character = False
    if variable:
        is_character = variable.type.lower() == "char"
    elif target_name in _UPCASE_VARIABLES:
        is_character = True

    if is_character:
        expr = f"coalescec({expr}, '')"
        expr = f"strip({expr})"
        if _should_upcase(variable, target_name):
            expr = f"upcase({expr})"
    return f"{mapping.target_variable} = {expr};"


def normalize_iso8601(raw_value) -> str:
    """Normalize date/time-ish strings to ISO8601; return original if invalid.

    Uses :func:`pandas.isna` to safely handle ``pd.NA`` and other missing
    markers without triggering "boolean value of NA is ambiguous" errors
    when called from ``Series.apply``.

    SDTM supports partial dates with unknown components. This function handles:
    - Full dates: 2023-09-15 -> 2023-09-15
    - Partial dates: 2023-09-NK, 2023-NK-NK -> preserved as-is (invalid but kept)
    - Non-standard formats: cleaned up to ISO 8601

    Unknown date components should NOT contain letters like 'NK'.
    Per SDTM IG, unknown parts should be omitted (e.g., 2023-09 for unknown day).
    """
    import re

    # Treat all missing-like values (None, NaN, pd.NA, empty string) as empty
    if isna(raw_value) or raw_value == "":
        return ""

    text = str(raw_value).strip()

    # Handle partial dates with "NK" (Not Known) - convert to proper ISO 8601 partial date
    # E.g., "2023-10-NK" -> "2023-10" (unknown day is omitted)
    # E.g., "2023-NK-NK" -> "2023" (unknown month and day)
    if "NK" in text.upper() or "UN" in text.upper() or "UNK" in text.upper():
        # Replace NK/UN/UNK patterns with empty
        cleaned = re.sub(r"-?(NK|UN|UNK)", "", text, flags=re.IGNORECASE)
        cleaned = cleaned.rstrip("-")  # Remove trailing dashes
        if cleaned:
            return cleaned
        return ""

    try:
        import pandas as pd

        parsed = pd.to_datetime(raw_value, errors="coerce", utc=False)
        if pd.isna(parsed):
            # If parsing fails, still return the original if it looks like a partial date
            if re.match(r"^\d{4}(-\d{2})?(-\d{2})?", text):
                return text
            return str(raw_value)
        return parsed.isoformat()
    except Exception:
        return str(raw_value)


def normalize_iso8601_duration(raw_value) -> str:
    """Normalize elapsed time/duration values to ISO 8601 duration format.

    ISO 8601 durations: PnYnMnDTnHnMnS
    Examples: PT1H (1 hour), PT30M (30 minutes), P1D (1 day), PT1H30M (1.5 hours)

    Common input formats:
    - "1 hour", "30 minutes", "2 hours 30 minutes"
    - "1:30" (hours:minutes)
    - "PT1H30M" (already ISO 8601)
    - "30" or "30 min" (assumed minutes)
    """
    import re

    # Treat all missing-like values (None, NaN, pd.NA, empty string) as empty
    if isna(raw_value) or raw_value == "":
        return ""

    text = str(raw_value).strip().upper()

    # Already ISO 8601 duration format
    if re.match(r"^P(\d+Y)?(\d+M)?(\d+D)?(T(\d+H)?(\d+M)?(\d+(\.\d+)?S)?)?$", text):
        return text

    # Clean up common variations
    text_clean = text.replace("HOURS", "H").replace("HOUR", "H")
    text_clean = (
        text_clean.replace("MINUTES", "M").replace("MINUTE", "M").replace("MIN", "M")
    )
    text_clean = (
        text_clean.replace("SECONDS", "S").replace("SECOND", "S").replace("SEC", "S")
    )
    text_clean = text_clean.replace("DAYS", "D").replace("DAY", "D")

    hours = 0
    minutes = 0
    seconds = 0
    days = 0

    # Match patterns like "1H 30M" or "1H30M"
    h_match = re.search(r"(\d+(?:\.\d+)?)\s*H", text_clean)
    m_match = re.search(r"(\d+(?:\.\d+)?)\s*M(?!O)", text_clean)  # M but not MONTH
    s_match = re.search(r"(\d+(?:\.\d+)?)\s*S", text_clean)
    d_match = re.search(r"(\d+)\s*D", text_clean)

    if h_match:
        hours = float(h_match.group(1))
    if m_match:
        minutes = float(m_match.group(1))
    if s_match:
        seconds = float(s_match.group(1))
    if d_match:
        days = int(d_match.group(1))

    # If none matched, try HH:MM:SS or HH:MM format
    if not any([h_match, m_match, s_match, d_match]):
        time_match = re.match(r"^(\d{1,2}):(\d{2})(?::(\d{2}))?$", text)
        if time_match:
            hours = int(time_match.group(1))
            minutes = int(time_match.group(2))
            seconds = int(time_match.group(3) or 0)
        else:
            # Try plain number (assume minutes if small, hours if with decimal)
            num_match = re.match(r"^(\d+(?:\.\d+)?)$", text)
            if num_match:
                value = float(num_match.group(1))
                if value < 24:
                    # Could be hours, but most likely small values are minutes
                    minutes = value
                else:
                    minutes = value

    # If still nothing parsed, return empty
    if days == 0 and hours == 0 and minutes == 0 and seconds == 0:
        return ""

    # Build ISO 8601 duration string
    duration = "P"
    if days > 0:
        duration += f"{days}D"
    if hours > 0 or minutes > 0 or seconds > 0:
        duration += "T"
        if hours > 0:
            if hours == int(hours):
                duration += f"{int(hours)}H"
            else:
                duration += f"{hours}H"
        if minutes > 0:
            if minutes == int(minutes):
                duration += f"{int(minutes)}M"
            else:
                duration += f"{minutes}M"
        if seconds > 0:
            if seconds == int(seconds):
                duration += f"{int(seconds)}S"
            else:
                duration += f"{seconds}S"

    return duration if duration != "P" else ""


def _render_value_map(mapping: ColumnMapping, value_map: dict[str, set[str]]) -> str:
    """Render SAS SELECT statement for value normalization."""
    normalized_expr = f"strip(upcase(coalescec({mapping.source_column}, '')))"
    lines: list[str] = [f"select ({normalized_expr});"]
    for canonical, synonyms in value_map.items():
        values = sorted({canonical.upper(), *(value.upper() for value in synonyms)})
        quoted = ", ".join(_quote(value) for value in values)
        lines.append(
            f"    when ({quoted}) {mapping.target_variable} = {_quote(canonical)};"
        )
    lines.append(f"    otherwise {mapping.target_variable} = {normalized_expr};")
    lines.append("end;")
    return "\n".join(lines)


def _quote(value: str) -> str:
    """Quote a string for SAS."""
    escaped = value.replace("'", "''")
    return f"'{escaped}'"


def _should_upcase(variable: SDTMVariable | None, target_name: str) -> bool:
    """Determine if a variable value should be uppercased."""
    if variable and variable.codelist_code:
        return True
    # Check if variable has CT defined
    if get_controlled_terminology(target_name):
        return True
    return target_name in _UPCASE_VARIABLES
