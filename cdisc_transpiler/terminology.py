"""Controlled terminology registry and validation helpers driven by CDISC CT CSV."""

from __future__ import annotations

from dataclasses import dataclass, field
from functools import lru_cache
from pathlib import Path
from typing import Dict, List

import pandas as pd
import math
from .domains_module import CT_VERSION

# Root directory for controlled terminology packages (contains SDTM, SEND, ADaM, etc.)
_CT_BASE_DIR = Path(__file__).resolve().parent.parent / "docs" / "Controlled_Terminology"


def _resolve_ct_dir() -> Path:
    """Select the CT folder for the configured CT_VERSION, fallback to newest."""
    target = _CT_BASE_DIR / CT_VERSION
    if target.exists():
        return target
    # fallback to most recent folder if exact version folder not present
    candidates = sorted(_CT_BASE_DIR.glob("*"))
    return candidates[-1] if candidates else target


_CT_DIR = _resolve_ct_dir()


@dataclass(frozen=True)
class ControlledTerminology:
    """Represents controlled terminology using column-aligned names from CT CSVs."""

    codelist_code: str | None
    codelist_name: str
    submission_values: set[str]
    codelist_extensible: bool = False
    synonyms: dict[str, str] | None = None  # CDISC Synonym(s)
    nci_codes: dict[str, str] = field(default_factory=dict)  # Code
    standards: set[str] = field(default_factory=set)  # Standard and Date
    sources: set[str] = field(default_factory=set)  # Source CSV filenames
    definitions: dict[str, str] = field(default_factory=dict)  # CDISC Definition
    preferred_terms: dict[str, str] = field(default_factory=dict)  # NCI Preferred Term
    variable: str | None = (
        None  # Optional mapping back to variable name (for convenience)
    )

    def normalize(self, raw_value: object) -> str:
        """Normalize a raw input value to canonical form preserving CDISC case."""
        if raw_value is None:
            return ""
        text = str(raw_value).strip()
        if not text:
            return ""
        lookup_key = text.upper()
        if self.synonyms:
            canonical = self.synonyms.get(lookup_key)
            if canonical is not None:
                return canonical
        return text

    def get_nci_code(self, canonical_value: str) -> str | None:
        """Return the NCI code for a canonical value."""
        if not canonical_value:
            return None
        return self.nci_codes.get(canonical_value) or self.nci_codes.get(
            canonical_value.upper()
        )

    def invalid_values(self, series: pd.Series) -> set[str]:
        """Return invalid raw values given the canonical CT list."""
        invalid: set[str] = set()
        for raw_value in series.dropna().unique():
            normalized = self.normalize(raw_value)
            if not normalized:
                continue
            if normalized in self.submission_values:
                continue
            if self.codelist_extensible:
                continue
            invalid.add(str(raw_value))
        return invalid


def _split_synonyms(raw: str | None) -> list[str]:
    if not raw:
        return []
    tokens: list[str] = []
    for sep in [";", ","]:
        if sep in raw:
            tokens = [t.strip() for t in raw.split(sep)]
            break
    if not tokens:
        tokens = [raw.strip()]
    return [t for t in tokens if t]


def _clean_value(raw: object) -> str:
    if raw is None:
        return ""
    try:
        if isinstance(raw, float) and math.isnan(raw):
            return ""
    except Exception:
        pass
    if isinstance(raw, str):
        return raw.strip()
    if pd.isna(raw):
        return ""
    return str(raw).strip()


def _iter_ct_files() -> List[Path]:
    """Return all CT CSV files in the configured CT directory."""
    if not _CT_DIR.exists():
        return []
    return sorted(_CT_DIR.glob("*CT_*.csv"))


def _load_ct_rows() -> Dict[str, list[dict]]:
    """Load CT rows grouped by codelist code from all CT CSVs in the package."""
    grouped: Dict[str, list[dict]] = {}
    for csv_path in _iter_ct_files():
        try:
            records = pd.read_csv(csv_path).to_dict(orient="records")
        except Exception:
            continue  # Skip unreadable files rather than failing the entire registry
        standard_hint = csv_path.stem  # e.g., SDTM_CT_2025-09-26
        for row in records:
            code = str(row.get("Codelist Code") or "").strip().upper()
            if not code:
                continue
            # Stash the source standard/date if the CSV omitted it
            row.setdefault("Standard and Date", standard_hint)
            row.setdefault("_source_file", csv_path.name)
            grouped.setdefault(code, []).append(row)
    return grouped


def _merge_ct(
    base: ControlledTerminology, other: ControlledTerminology
) -> ControlledTerminology:
    """Merge two CT objects for the same codelist code."""
    merged_submission = base.submission_values | other.submission_values
    merged_synonyms = None
    if base.synonyms or other.synonyms:
        merged_synonyms = {}
        if other.synonyms:
            merged_synonyms.update(other.synonyms)
        if base.synonyms:
            merged_synonyms.update(base.synonyms)
    merged_nci = {**other.nci_codes, **base.nci_codes}
    merged_definitions = {**other.definitions, **base.definitions}
    merged_pref = {**other.preferred_terms, **base.preferred_terms}
    merged_extensible = base.codelist_extensible or other.codelist_extensible
    standards = set(base.standards) | set(other.standards)
    sources = set(base.sources) | set(other.sources)
    codelist_name = base.codelist_name or other.codelist_name

    return ControlledTerminology(
        codelist_code=base.codelist_code or other.codelist_code,
        codelist_name=codelist_name or base.codelist_name,
        submission_values=merged_submission,
        codelist_extensible=merged_extensible,
        synonyms=merged_synonyms,
        nci_codes=merged_nci,
        standards=standards,
        sources=sources,
        definitions=merged_definitions,
        preferred_terms=merged_pref,
        variable=codelist_name or base.variable,
    )


def _build_registry() -> tuple[
    Dict[str, ControlledTerminology], Dict[str, ControlledTerminology]
]:
    """Build registries keyed by codelist code and by codelist name from all CT files."""
    registry_by_code: Dict[str, ControlledTerminology] = {}
    registry_by_name: Dict[str, ControlledTerminology] = {}
    grouped = _load_ct_rows()
    for code, rows in grouped.items():
        if not rows:
            continue
        submission_values: set[str] = set()
        synonyms: dict[str, str] = {}
        nci_codes: dict[str, str] = {}
        definitions: dict[str, str] = {}
        preferred_terms: dict[str, str] = {}
        extensible = (
            _clean_value(rows[0].get("Codelist Extensible (Yes/No)")).lower() == "yes"
        )
        name = _clean_value(rows[0].get("Codelist Name") or code).upper()
        standard = _clean_value(rows[0].get("Standard and Date"))
        source_file = _clean_value(rows[0].get("_source_file"))
        for row in rows:
            submission = _clean_value(row.get("CDISC Submission Value"))
            if not submission:
                continue
            canonical_value = submission
            submission_values.add(canonical_value)
            nci = _clean_value(row.get("Code"))
            if nci:
                nci_codes[canonical_value] = nci
                nci_codes[canonical_value.upper()] = nci
            definition = _clean_value(row.get("CDISC Definition"))
            if definition:
                definitions[canonical_value] = definition
            pref_term = _clean_value(row.get("NCI Preferred Term"))
            if pref_term:
                preferred_terms[canonical_value] = pref_term
            # Store synonyms with uppercase keys for case-insensitive lookup
            synonyms[canonical_value.upper()] = canonical_value
            for syn in _split_synonyms(_clean_value(row.get("CDISC Synonym(s)"))):
                synonyms[syn.upper()] = canonical_value

        ct = ControlledTerminology(
            codelist_name=name,
            codelist_code=code,
            submission_values=submission_values,
            codelist_extensible=extensible,
            synonyms=synonyms,
            nci_codes=nci_codes,
            standards={standard} if standard else set(),
            sources={source_file} if source_file else set(),
            definitions=definitions,
            preferred_terms=preferred_terms,
            variable=name,
        )

        if code in registry_by_code:
            ct = _merge_ct(registry_by_code[code], ct)
        registry_by_code[code] = ct
        # Keep the most complete CT for a given codelist name
        existing = registry_by_name.get(name)
        if existing:
            registry_by_name[name] = _merge_ct(existing, ct)
        else:
            registry_by_name[name] = ct

    return registry_by_code, registry_by_name


_REGISTRY_BY_CODE, _REGISTRY_BY_NAME = _build_registry()


@lru_cache(maxsize=None)
def _variable_to_codelist() -> Dict[str, str]:
    """Map variable names to codelist codes using domain metadata."""
    from .domains_module import get_domain, list_domains

    mapping: Dict[str, str] = {}
    for domain_code in list_domains():
        try:
            domain = get_domain(domain_code)
        except KeyError:
            continue
        for variable in domain.variables:
            if variable.codelist_code:
                mapping[variable.name.upper()] = variable.codelist_code
    return mapping


def get_controlled_terminology(
    codelist_code: str | None = None, variable: str | None = None
) -> ControlledTerminology | None:
    """Get controlled terminology by codelist code or variable name."""
    if not codelist_code and not variable:
        return None

    if codelist_code:
        ct = _REGISTRY_BY_CODE.get(codelist_code.strip().upper())
        if ct:
            return ct

    if variable:
        var_key = variable.upper()
        code = _variable_to_codelist().get(var_key)
        if code:
            ct = _REGISTRY_BY_CODE.get(code)
            if ct:
                return ct
        # Fallback to codelist name matching (e.g., SEX)
        return _REGISTRY_BY_NAME.get(var_key)

    return None


def list_controlled_variables() -> tuple[str, ...]:
    """Return all variables with controlled terminology."""
    vars_from_domains = tuple(sorted(_variable_to_codelist().keys()))
    names = tuple(sorted(_REGISTRY_BY_NAME.keys()))
    # Merge and deduplicate while preserving lexical order
    merged = sorted(set(vars_from_domains) | set(names))
    return tuple(merged)


def get_nci_code(variable: str, value: str) -> str | None:
    """Return the NCI code for a variable/value combination."""
    ct = get_controlled_terminology(variable=variable)
    if ct is None:
        return None
    return ct.get_nci_code(value)


def get_codelist_code(variable: str) -> str | None:
    """Return the codelist code for a variable."""
    ct = get_controlled_terminology(variable=variable)
    if ct is None:
        return None
    return ct.codelist_code
