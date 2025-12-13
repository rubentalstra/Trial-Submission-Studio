"""Variable construction from CSV metadata."""

from __future__ import annotations

from typing import Mapping

from .constants import DEFAULT_CHAR_LENGTH, DEFAULT_NUM_LENGTH
from .models import SDTMVariable
from .utils import clean_codelist, infer_implements, normalize_type


def extract_variable_order(row: Mapping[str, str]) -> int | None:
    """Extract variable order from CSV row."""
    try:
        return int((row.get("Variable Order") or "").strip())
    except ValueError:
        return None


def extract_variable_name(row: Mapping[str, str]) -> str:
    """Extract and normalize variable name from CSV row."""
    return (row.get("Variable Name") or "").strip().upper()


def extract_variable_label(row: Mapping[str, str], name: str) -> str:
    """Extract variable label from CSV row, defaulting to name."""
    return (row.get("Variable Label") or name).strip()


def extract_codelist_code(row: Mapping[str, str]) -> str | None:
    """Extract codelist code from CSV row."""
    return clean_codelist(
        row.get("CDISC CT Codelist Code(s)") or row.get("Variable C-Code")
    )


def extract_described_value_domain(row: Mapping[str, str]) -> str | None:
    """Extract described value domain from CSV row."""
    value = (
        row.get("Described Value Domain(s)")
        or row.get("Described Value Domain")
        or ""
    ).strip()
    return value or None


def extract_core_value(row: Mapping[str, str]) -> str | None:
    """Extract core value from CSV row."""
    core_raw = (row.get("Core") or "").strip()
    return core_raw or None


def extract_role(row: Mapping[str, str]) -> str | None:
    """Extract role from CSV row."""
    return (row.get("Role") or "").strip() or None


def extract_source_version(row: Mapping[str, str]) -> str | None:
    """Extract source version from CSV row."""
    return (row.get("Version") or "").strip() or None


def determine_length(name: str, vtype: str) -> int:
    """Determine variable length based on type and name."""
    if name in {"DOMAIN", "RDOMAIN"}:
        return 2
    return DEFAULT_NUM_LENGTH if vtype == "Num" else DEFAULT_CHAR_LENGTH


def variable_from_row(
    row: Mapping[str, str], code: str, class_name: str
) -> SDTMVariable:
    """Create an SDTMVariable from a CSV row.
    
    This function extracts metadata from a CSV row and constructs a standardized
    SDTMVariable object. It handles various CSV field variations and applies
    SDTM conventions.
    """
    name = extract_variable_name(row)
    label = extract_variable_label(row, name)
    vtype = normalize_type(row.get("Type"))
    core = extract_core_value(row)
    length = determine_length(name, vtype)
    codelist = extract_codelist_code(row)
    variable_order = extract_variable_order(row)
    source_version = extract_source_version(row)
    role = extract_role(row)
    implements = infer_implements(name, code, class_name, role)

    return SDTMVariable(
        name=name,
        label=label,
        type=vtype,
        length=length,
        core=core,
        codelist_code=codelist,
        variable_order=variable_order,
        role=role,
        value_list=(row.get("Value List") or "").strip() or None,
        described_value_domain=extract_described_value_domain(row),
        codelist_submission_values=(row.get("Codelist Submission Values") or "").strip()
        or None,
        usage_restrictions=(row.get("Usage Restrictions") or "").strip() or None,
        definition=(row.get("Definition") or "").strip() or None,
        notes=(row.get("CDISC Notes") or row.get("Notes") or "").strip() or None,
        variables_qualified=(row.get("Variables Qualified") or "").strip() or None,
        source_dataset=code,
        source_version=source_version,
        implements=implements,
    )
