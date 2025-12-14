"""SAS value normalization utilities.

This module provides utilities for normalizing values in SAS code generation,
including domain-specific normalizers and controlled terminology handling.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ..domains_module import SDTMVariable
    from ..mapping_module import ColumnMapping

from ..terminology_module import get_controlled_terminology


def _synonyms(*values: str) -> set[str]:
    """Return uppercased synonyms for canonical value matching.

    Args:
        values: String values to convert to synonyms

    Returns:
        Set of uppercase synonyms
    """
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
    """Build value normalizers for variables not in standard CDISC CT.

    Returns:
        Dictionary mapping variable names to value maps
    """
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
    """Get value map from controlled terminology registry.

    Args:
        variable_name: Name of the variable

    Returns:
        Value map or None if not found in CT
    """

    ct = get_controlled_terminology(variable=variable_name)
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


def render_assignment(mapping: "ColumnMapping", variable: "SDTMVariable | None") -> str:
    """Return SAS statements that assign a target variable with normalization.

    Args:
        mapping: Column mapping configuration
        variable: Target SDTM variable definition (optional)

    Returns:
        SAS assignment statement(s)
    """
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


def _render_value_map(mapping: "ColumnMapping", value_map: dict[str, set[str]]) -> str:
    """Render SAS SELECT statement for value normalization.

    Args:
        mapping: Column mapping configuration
        value_map: Dictionary of canonical values to synonym sets

    Returns:
        SAS SELECT statement
    """
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
    """Quote a string for SAS.

    Args:
        value: String to quote

    Returns:
        SAS-quoted string
    """
    escaped = value.replace("'", "''")
    return f"'{escaped}'"


def _should_upcase(variable: "SDTMVariable | None", target_name: str) -> bool:
    """Determine if a variable value should be uppercased.

    Args:
        variable: SDTM variable definition
        target_name: Target variable name

    Returns:
        True if values should be uppercased
    """

    if variable and variable.codelist_code:
        return True
    # Check if variable has CT defined
    if get_controlled_terminology(variable=target_name):
        return True
    return target_name in _UPCASE_VARIABLES
