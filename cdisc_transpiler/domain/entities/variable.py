"""Variable construction from CSV metadata."""

from __future__ import annotations

from collections.abc import Mapping

from .sdtm_classes import GENERAL_OBSERVATION_CLASSES, normalize_general_class
from .sdtm_domain import SDTMVariable

# Import constants directly to avoid circular import
DEFAULT_CHAR_LENGTH = 200
DEFAULT_NUM_LENGTH = 8


def _parse_int(value: str | None) -> int | None:
    if value is None:
        return None
    text = str(value).strip().strip('"')
    if not text:
        return None
    try:
        parsed = int(float(text))
    except (TypeError, ValueError):
        return None
    return parsed if parsed > 0 else None


def extract_variable_length(row: Mapping[str, str]) -> int | None:
    """Extract variable length from CSV metadata when available.

    SDTMIG v3.4 Variables.csv (as shipped in this repo) does not include a
    dedicated Length column. However, other structured sources (or future
    versions) may include one. This extractor is defensive and returns None
    when no length can be found.
    """
    for key in (
        "Length",
        "Variable Length",
        "Variable Length (Char)",
        "Max Length",
        "Maximum Length",
    ):
        if key in row:
            value = _parse_int(row.get(key))
            if value is not None:
                return value
    return None


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


def normalize_type(raw: str | None) -> str:
    """Map CSV type strings to SDTM types."""
    if not raw:
        return "Char"
    lower = raw.strip().lower()
    return "Num" if lower.startswith("num") else "Char"


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
        row.get("Described Value Domain(s)") or row.get("Described Value Domain") or ""
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

    Args:
        row: Dictionary containing CSV metadata fields (Variable Name, Type, Label, etc.)
        code: Domain code (e.g., 'DM', 'AE') used for the source dataset
        class_name: SDTM class name (e.g., 'FINDINGS', 'EVENTS') for the domain

    Returns:
        SDTMVariable: Fully constructed variable definition with metadata
    """
    name = extract_variable_name(row)
    label = extract_variable_label(row, name)
    vtype = normalize_type(row.get("Type"))
    core = extract_core_value(row)
    length = extract_variable_length(row) or determine_length(name, vtype)
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
