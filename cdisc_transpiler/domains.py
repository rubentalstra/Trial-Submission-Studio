"""SDTM metadata definitions loaded from SDTMIG/SDTM CSVs (v3.4 / v2.1)."""

from __future__ import annotations

import csv
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path
from typing import Iterable, Mapping, Sequence

# Controlled Terminology version used for metadata stamping.
# The SDTM-MSG v2.0 example relies on the 2024-03-29 package. When this
# exact folder is not present locally, the terminology loader falls back
# to the most recent available package.
CT_VERSION = "2024-03-29"

# Default lengths when the source metadata does not provide them.
DEFAULT_CHAR_LENGTH = 200
DEFAULT_NUM_LENGTH = 8

# General Observation Class constants (Interventions, Events, Findings)
_GENERAL_OBSERVATION_CLASSES = {"INTERVENTIONS", "EVENTS", "FINDINGS"}
_GENERAL_CLASS_ALIASES = {"FINDINGS ABOUT": "FINDINGS"}


@dataclass(frozen=True)
class SDTMVariable:
    """SDTM variable definition."""

    name: str
    label: str
    type: str  # Char or Num
    length: int
    core: str | None = None  # Core: Req, Exp, Perm
    codelist_code: str | None = None  # CDISC CT codelist code (e.g., C66742)
    variable_order: int | None = None  # CSV: Variable Order
    role: str | None = None  # CSV: Role
    value_list: str | None = None  # CSV: Value List
    described_value_domain: str | None = None  # CSV: Described Value Domain(s)
    codelist_submission_values: str | None = None  # CSV: Codelist Submission Values
    usage_restrictions: str | None = None  # CSV: Usage Restrictions (SDTM v2.1)
    definition: str | None = None  # CSV: Definition/CDISC Notes
    notes: str | None = None  # CSV: CDISC Notes/Notes
    variables_qualified: str | None = None  # CSV: Variables Qualified
    source_dataset: str | None = None  # CSV: Dataset Name
    source_version: str | None = None  # CSV: Version
    # General Observation Class linkage (e.g., --SEQ, --DTC)
    implements: str | None = None

    def pandas_dtype(self) -> str:
        """Return the pandas dtype for the variable."""
        if self.type == "Num":
            return "float64"
        return "string"


@dataclass(frozen=True)
class SDTMDomain:
    """SDTM domain definition."""

    code: str
    description: str
    class_name: str
    structure: str
    label: str | None
    variables: tuple[SDTMVariable, ...]
    dataset_name: str | None = None

    def variable_names(self) -> tuple[str, ...]:
        """Return tuple of variable names in this domain."""
        return tuple(var.name for var in self.variables)

    def implements_mapping(self) -> dict[str, str]:
        """Return mapping of variable name to generalized identifier/timing concept."""
        return {var.name: var.implements for var in self.variables if var.implements}

    def resolved_dataset_name(self) -> str:
        """Return the 8-character dataset name."""
        name = (self.dataset_name or self.code).upper()
        return name[:8]


_DOMAIN_DEFINITIONS: dict[str, SDTMDomain] = {}

# Path to SDTMIG v3.4 metadata (single source of truth)
_SDTMIG_PATH = Path(__file__).resolve().parent.parent / "docs" / "SDTMIG_v3.4" / "Variables.csv"
_SDTM_V2_PATH = Path(__file__).resolve().parent.parent / "docs" / "SDTM_v2.0" / "Variables.csv"
_SDTM_DATASETS_PATH = (
    Path(__file__).resolve().parent.parent / "docs" / "SDTMIG_v3.4" / "Datasets.csv"
)


def _load_dataset_attributes() -> dict[str, dict[str, str]]:
    """Load dataset-level attributes (class/label/structure) from Datasets.csv."""
    if not _SDTM_DATASETS_PATH.exists():
        return {}
    attributes: dict[str, dict[str, str]] = {}
    
    # Read CSV file
    with _SDTM_DATASETS_PATH.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        for row in reader:
            # Expected columns: Version, Class, Dataset Name, Dataset Label, Structure
            dataset_name = (row.get("Dataset Name") or "").strip().upper()
            if not dataset_name:
                continue
            attributes[dataset_name] = {
                "class": (row.get("Class") or "").strip(),
                "label": (row.get("Dataset Label") or "").strip(),
                "structure": (row.get("Structure") or "").strip(),
            }
    return attributes


_DATASET_ATTRIBUTES = _load_dataset_attributes()


def _normalize_class(value: str | None) -> str:
    """Normalize class strings for comparisons."""
    if not value:
        return ""
    return value.strip().upper().replace("-", " ")


def _normalize_general_class(value: str | None) -> str:
    """Map class names to the three General Observation Classes."""
    normalized = _normalize_class(value)
    return _GENERAL_CLASS_ALIASES.get(normalized, normalized)


def _infer_implements(
    var_name: str, domain_code: str, class_name: str, role: str | None
) -> str | None:
    """Return generalized placeholder (e.g., --SEQ) for Identifier/Timing variables."""
    general_class = _normalize_general_class(class_name)
    if general_class not in _GENERAL_OBSERVATION_CLASSES:
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


def _core_priority(core: str | None) -> int:
    """Priority of Core column for selecting canonical templates."""
    order = {"REQ": 3, "EXP": 2, "PERM": 1}
    return order.get((core or "").strip().upper(), 0)


def _register(domain: SDTMDomain) -> None:
    """Register a domain definition."""
    _DOMAIN_DEFINITIONS[domain.code.upper()] = domain


@lru_cache(maxsize=None)
def get_domain(code: str) -> SDTMDomain:
    """Get domain definition by code."""
    key = code.upper()
    if key in _DOMAIN_DEFINITIONS:
        return _DOMAIN_DEFINITIONS[key]

    # Supplemental qualifiers: build SUPP-- domains from SUPPQUAL metadata
    if key.startswith("SUPP") and len(key) == 6:
        supp = _build_supp_domain(key)
        _register(supp)
        return supp

    # Attempt to build from CSV metadata on demand
    domain = _build_domain_from_cache(key)
    if domain:
        _register(domain)
        return domain

    raise KeyError(f"Unknown SDTM domain '{code}'")


def list_domains() -> Iterable[str]:
    """List all registered domain codes."""
    return _DOMAIN_DEFINITIONS.keys()


def generalized_identifiers(domain_code: str) -> dict[str, str]:
    """Return mapping of variables to their generalized Identifier/Timing placeholders."""
    domain = get_domain(domain_code)
    return domain.implements_mapping()


def _build_supp_domain(code: str) -> SDTMDomain:
    """Create a SUPP-- domain definition based on SUPPQUAL metadata from CSV."""
    dataset_name = code.upper()
    # Prefer the registered SUPPQUAL definition that comes from SDTMIG_v3.4.csv
    base = _DOMAIN_DEFINITIONS.get("SUPPQUAL") or _build_domain_from_cache("SUPPQUAL")
    if base is None:
        raise KeyError(
            "SUPPQUAL metadata not available; cannot build supplemental domain."
        )
    return SDTMDomain(
        code=dataset_name,
        description=f"Supplemental Qualifiers for {dataset_name[4:]}",
        class_name=base.class_name,
        structure=base.structure,
        label=base.label,
        variables=base.variables,
        dataset_name=dataset_name,
    )


# =============================================================================
# CSV-driven domain loading
# =============================================================================


def _load_csv_rows(
    path: Path, dataset_field: str = "Dataset Name"
) -> dict[str, list[dict]]:
    """Load SDTM metadata rows keyed by dataset/domain code."""
    if not path.exists():
        return {}
    data: dict[str, list[dict]] = {}
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        for row in reader:
            code = (row.get(dataset_field) or "").strip().upper()
            var = (row.get("Variable Name") or "").strip()
            if not code or not var:
                continue
            data.setdefault(code, []).append(row)
    return data


_SDTMIG_CACHE: dict[str, list[dict]] | None = None
_SDTM_V2_CACHE: dict[str, list[dict]] | None = None


def _load_sdtmig_cache() -> dict[str, list[dict]]:
    """Load SDTMIG v3.4 metadata from CSV."""
    global _SDTMIG_CACHE
    if _SDTMIG_CACHE is None:
        _SDTMIG_CACHE = _load_csv_rows(_SDTMIG_PATH)
    return _SDTMIG_CACHE


def _load_sdtm_v2_cache() -> dict[str, list[dict]]:
    """Load SDTM v2.0 metadata from CSV (used as fallback/enrichment)."""
    global _SDTM_V2_CACHE
    if _SDTM_V2_CACHE is None:
        _SDTM_V2_CACHE = _load_csv_rows(_SDTM_V2_PATH)
    return _SDTM_V2_CACHE


def _clean_codelist(raw: str | None) -> str | None:
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


def _normalize_type(raw: str | None) -> str:
    """Map CSV type strings to SDTM types."""
    if not raw:
        return "Char"
    lower = raw.strip().lower()
    return "Num" if lower.startswith("num") else "Char"


def _variable_from_row(
    row: Mapping[str, str], code: str, class_name: str
) -> SDTMVariable:
    """Create an SDTMVariable from a CSV row."""
    name = (row.get("Variable Name") or "").strip().upper()
    label = (row.get("Variable Label") or name).strip()
    vtype = _normalize_type(row.get("Type"))
    core_raw = (row.get("Core") or "").strip()
    length = DEFAULT_NUM_LENGTH if vtype == "Num" else DEFAULT_CHAR_LENGTH
    if name in {"DOMAIN", "RDOMAIN"}:
        length = 2
    codelist = _clean_codelist(
        row.get("CDISC CT Codelist Code(s)") or row.get("Variable C-Code")
    )
    try:
        variable_order = int((row.get("Variable Order") or "").strip())
    except ValueError:
        variable_order = None
    source_version = (row.get("Version") or "").strip() or None
    role = (row.get("Role") or "").strip() or None
    implements = _infer_implements(name, code, class_name, role)

    return SDTMVariable(
        name=name,
        label=label,
        type=vtype,
        length=length,
        core=core_raw or None,
        codelist_code=codelist,
        variable_order=variable_order,
        role=role,
        value_list=(row.get("Value List") or "").strip() or None,
        described_value_domain=(
            (
                row.get("Described Value Domain(s)")
                or row.get("Described Value Domain")
                or ""
            ).strip()
            or None
        ),
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


def _is_preferred_variable(
    candidate: SDTMVariable, existing: SDTMVariable | None
) -> bool:
    """Select the better variable template when duplicates exist."""
    if existing is None:
        return True
    cand_rank = _core_priority(candidate.core)
    exist_rank = _core_priority(existing.core)
    if cand_rank != exist_rank:
        return cand_rank > exist_rank
    cand_order = candidate.variable_order or 1_000_000
    exist_order = existing.variable_order or 1_000_000
    if cand_order != exist_order:
        return cand_order < exist_order
    return True


def _build_general_class_variables() -> tuple[
    dict[str, dict[str, SDTMVariable]], dict[str, dict[str, set[str]]]
]:
    """Collect Identifier/Timing templates grouped by General Observation Class."""
    registry: dict[str, dict[str, SDTMVariable]] = {}
    usage: dict[str, dict[str, set[str]]] = {}
    # Process older standard first so SDTMIG (newer) wins ties
    caches = [_load_sdtm_v2_cache(), _load_sdtmig_cache()]

    for cache in caches:
        for code, rows in cache.items():
            if not rows:
                continue
            class_name = _normalize_class(rows[0].get("Class"))
            general_class = _normalize_general_class(class_name)
            if general_class not in _GENERAL_OBSERVATION_CLASSES:
                continue
            for row in rows:
                role = (row.get("Role") or "").strip().lower()
                if role not in ("identifier", "timing"):
                    continue
                variable = _variable_from_row(row, code, class_name)
                implements = variable.implements
                if not implements:
                    continue
                usage.setdefault(general_class, {}).setdefault(implements, set()).add(
                    code
                )
                existing = registry.setdefault(general_class, {}).get(implements)
                if _is_preferred_variable(variable, existing):
                    registry[general_class][implements] = variable

    return registry, usage


_GENERAL_CLASS_VARIABLES, _GENERAL_CLASS_USAGE = _build_general_class_variables()
_ALWAYS_PROPAGATE_GENERAL = {
    "STUDYID",
    "DOMAIN",
    "USUBJID",
    "EPOCH",
    "VISIT",
    "VISITNUM",
    "VISITDY",
    "SPDEVID",
}


def _should_propagate_general(implements: str, general_class: str) -> bool:
    """Decide whether to add a generalized variable to all domains in its class."""
    if implements.startswith("--"):
        return True
    if implements in _ALWAYS_PROPAGATE_GENERAL:
        return True
    domains = _GENERAL_CLASS_USAGE.get(general_class, {}).get(implements, set())
    return len(domains) > 1


def _augment_general_class_variables(
    variables: list[SDTMVariable], class_name: str, code: str
) -> list[SDTMVariable]:
    """Add missing Identifier/Timing variables shared within the class."""
    general_class = _normalize_general_class(class_name)
    templates = _GENERAL_CLASS_VARIABLES.get(general_class)
    if not templates:
        return variables

    existing = {v.name for v in variables}
    for implements, template in templates.items():
        if not _should_propagate_general(implements, general_class):
            continue
        target_name = (
            f"{code}{implements[2:]}" if implements.startswith("--") else implements
        )
        if target_name in existing:
            continue
        variables.append(
            SDTMVariable(
                name=target_name,
                label=template.label,
                type=template.type,
                length=template.length,
                core=template.core or "Perm",
                codelist_code=template.codelist_code,
                variable_order=None,
                role=template.role,
                value_list=template.value_list,
                described_value_domain=template.described_value_domain,
                codelist_submission_values=template.codelist_submission_values,
                usage_restrictions=template.usage_restrictions,
                definition=template.definition,
                notes=template.notes,
                variables_qualified=template.variables_qualified,
                source_dataset=code,
                source_version=template.source_version,
                implements=implements,
            )
        )
        existing.add(target_name)
    return variables


def _build_domain_from_rows(
    code: str, rows: Sequence[Mapping[str, str]], source: str
) -> SDTMDomain | None:
    """Construct a domain definition from CSV rows."""
    if not rows:
        return None

    # Preserve CSV ordering; fall back to file order when Variable Order is missing
    def _order(row: Mapping[str, str], idx: int) -> tuple[int, int]:
        raw = (row.get("Variable Order") or "").strip()
        try:
            return (int(raw), idx)
        except ValueError:
            return (1_000_000, idx)

    rows_list = list(rows)
    ordered = [
        r
        for _, r in sorted(
            ((_order(r, i), r) for i, r in enumerate(rows_list)), key=lambda x: x[0]
        )
    ]
    dataset_meta = _DATASET_ATTRIBUTES.get(code, {})
    dataset_label = dataset_meta.get("label")
    dataset_structure = dataset_meta.get("structure", "")
    class_name = _normalize_class(
        dataset_meta.get("class") or (ordered[0].get("Class") or "GENERAL")
    )

    variables: list[SDTMVariable] = [
        _variable_from_row(row, code, class_name) for row in ordered
    ]
    description = dataset_label or f"{code} domain (from {source})"

    return SDTMDomain(
        code=code,
        description=description,
        class_name=class_name,
        structure=dataset_structure,
        label=dataset_label,
        variables=tuple(variables),
        dataset_name=code,
    )


def _build_domain_from_cache(code: str) -> SDTMDomain | None:
    """Lookup domain rows from metadata caches (SDTMIG v3.4 then SDTM v2.0)."""
    cache_v34 = _load_sdtmig_cache()
    rows = cache_v34.get(code)
    if rows:
        return _build_domain_from_rows(code, rows, "SDTMIG v3.4")
    cache_v2 = _load_sdtm_v2_cache()
    rows = cache_v2.get(code)
    if rows:
        return _build_domain_from_rows(code, rows, "SDTM v2.0")
    return None


def _register_all_domains() -> None:
    """Register all domains defined in the CSV metadata (v3.4 overriding v2.0)."""
    # Register SDTM v2.0 first
    cache_v2 = _load_sdtm_v2_cache()
    for code, rows in sorted(cache_v2.items()):
        domain = _build_domain_from_rows(code, rows, "SDTM v2.0")
        if domain:
            _register(domain)

    # Register SDTMIG v3.4 (newer) to override with latest metadata
    cache_v34 = _load_sdtmig_cache()
    for code, rows in sorted(cache_v34.items()):
        domain = _build_domain_from_rows(code, rows, "SDTMIG v3.4")
        if domain:
            _register(domain)
    # SUPPQUAL is included in the CSVs; no manual overrides necessary.


# Initialize CSV-driven domains at import time
_register_all_domains()
