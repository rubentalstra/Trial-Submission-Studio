"""Controlled terminology loader from CDISC CT CSV files.

This module handles loading and parsing controlled terminology
data from CSV files in the Controlled_Terminology directory.
"""

from __future__ import annotations

import math
from pathlib import Path
from typing import Dict, List

import pandas as pd

from .models import ControlledTerminology


def _split_synonyms(raw: str | None) -> list[str]:
    """Split a synonyms string into individual synonyms.

    Args:
        raw: Raw synonyms string (semicolon or comma separated)

    Returns:
        List of individual synonym strings
    """
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
    """Clean a raw value to string form.

    Handles None, NaN, and other missing value types.

    Args:
        raw: Raw value from CSV

    Returns:
        Cleaned string value
    """
    if raw is None:
        return ""
    if isinstance(raw, (pd.Series, pd.DataFrame)):
        try:
            raw = raw.iloc[0]  # type: ignore[index]
        except Exception:
            return ""
    try:
        if isinstance(raw, float) and math.isnan(raw):
            return ""
    except Exception:
        pass
    if isinstance(raw, str):
        return raw.strip()
    if not isinstance(raw, (pd.Series, pd.DataFrame)):
        try:
            if bool(pd.isna(raw)):
                return ""
        except Exception:
            pass
    return str(raw).strip()


def _iter_ct_files(ct_dir: Path) -> List[Path]:
    """Return all CT CSV files in the specified directory.

    Args:
        ct_dir: Path to controlled terminology directory

    Returns:
        Sorted list of CSV file paths matching *CT_*.csv pattern
    """
    if not ct_dir.exists():
        return []
    return sorted(ct_dir.glob("*CT_*.csv"))


def _load_ct_rows(ct_dir: Path) -> Dict[str, list[dict[str, Any]]]:
    """Load CT rows grouped by codelist code from all CT CSVs.

    Args:
        ct_dir: Path to controlled terminology directory

    Returns:
        Dictionary mapping codelist codes to lists of row dictionaries
    """
    grouped: Dict[str, list[dict[str, Any]]] = {}
    for csv_path in _iter_ct_files(ct_dir):
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
    """Merge two CT objects for the same codelist code.

    Args:
        base: Base CT object (takes precedence)
        other: Other CT object to merge

    Returns:
        Merged ControlledTerminology object
    """
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
        codelist_name=codelist_name,
        submission_values=merged_submission,
        codelist_extensible=merged_extensible,
        synonyms=merged_synonyms,
        nci_codes=merged_nci,
        standards=standards,
        sources=sources,
        definitions=merged_definitions,
        preferred_terms=merged_pref,
        # Use base.variable if available, otherwise fall back to codelist_name
        # since CT variable names often match the codelist name (e.g., SEX)
        variable=base.variable or other.variable or codelist_name,
    )


def build_registry(ct_dir: Path) -> tuple[
    Dict[str, ControlledTerminology], Dict[str, ControlledTerminology]
]:
    """Build registries keyed by codelist code and by codelist name.

    Args:
        ct_dir: Path to controlled terminology directory

    Returns:
        Tuple of (registry_by_code, registry_by_name)
    """
    registry_by_code: Dict[str, ControlledTerminology] = {}
    registry_by_name: Dict[str, ControlledTerminology] = {}
    grouped = _load_ct_rows(ct_dir)

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
