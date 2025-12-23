"""Controlled terminology loader (infrastructure).

This module loads CDISC Controlled Terminology (CT) from CT CSV files on disk.

It intentionally lives in the infrastructure layer because it performs
filesystem I/O and depends on the CT CSV file layout.
"""

import math
from typing import TYPE_CHECKING, Any, cast

import pandas as pd

from ...domain.entities.controlled_terminology import ControlledTerminology

if TYPE_CHECKING:
    from pathlib import Path


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
    if isinstance(raw, pd.Series):
        series = cast("pd.Series[Any]", raw)
        if series.empty:
            return ""
        raw_value: object = series.iloc[0]
    elif isinstance(raw, pd.DataFrame):
        if raw.empty:
            return ""
        raw_value = raw.iloc[0, 0]
    else:
        raw_value = raw

    if isinstance(raw_value, float) and math.isnan(raw_value):
        return ""
    try:
        if pd.isna(cast("Any", raw_value)):
            return ""
    except (TypeError, ValueError):
        pass

    if isinstance(raw_value, str):
        return raw_value.strip()
    return str(raw_value).strip()


def _iter_ct_files(ct_dir: Path) -> list[Path]:
    if not ct_dir.exists():
        return []
    return sorted(ct_dir.glob("*CT_*.csv"))


def _load_ct_rows(ct_dir: Path) -> dict[str, list[dict[str, Any]]]:
    grouped: dict[str, list[dict[str, Any]]] = {}
    for csv_path in _iter_ct_files(ct_dir):
        try:
            records = pd.read_csv(csv_path).to_dict(orient="records")
        except Exception:
            continue
        standard_hint = csv_path.stem
        for raw_row in records:
            row: dict[str, Any] = {str(k): v for k, v in raw_row.items()}
            code = str(row.get("Codelist Code") or "").strip().upper()
            if not code:
                continue
            row.setdefault("Standard and Date", standard_hint)
            row.setdefault("_source_file", csv_path.name)
            grouped.setdefault(code, []).append(row)
    return grouped


def _merge_ct(
    base: ControlledTerminology, other: ControlledTerminology
) -> ControlledTerminology:
    merged_submission = base.submission_values | other.submission_values

    merged_synonyms: dict[str, str] | None = None
    if base.synonyms or other.synonyms:
        merged_synonyms = {}
        if other.synonyms:
            merged_synonyms.update(other.synonyms)
        if base.synonyms:
            merged_synonyms.update(base.synonyms)

    merged_nci = {**other.nci_codes, **base.nci_codes}
    merged_definitions = {**other.definitions, **base.definitions}
    merged_pref = {**other.preferred_terms, **base.preferred_terms}

    merged_synonyms_by_submission: dict[str, tuple[str, ...]] = {}
    if base.submission_value_synonyms or other.submission_value_synonyms:
        all_keys = set(base.submission_value_synonyms.keys()) | set(
            other.submission_value_synonyms.keys()
        )
        for key in all_keys:
            merged = set(base.submission_value_synonyms.get(key, ())) | set(
                other.submission_value_synonyms.get(key, ())
            )
            if merged:
                merged_synonyms_by_submission[key] = tuple(sorted(list(merged)))
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
        submission_value_synonyms=merged_synonyms_by_submission,
        nci_codes=merged_nci,
        standards=standards,
        sources=sources,
        definitions=merged_definitions,
        preferred_terms=merged_pref,
        variable=base.variable or other.variable or codelist_name,
    )


def build_registry(
    ct_dir: Path,
) -> tuple[dict[str, ControlledTerminology], dict[str, ControlledTerminology]]:
    """Build registries keyed by codelist code and by codelist name."""

    registry_by_code: dict[str, ControlledTerminology] = {}
    registry_by_name: dict[str, ControlledTerminology] = {}
    grouped = _load_ct_rows(ct_dir)

    for code, rows in grouped.items():
        if not rows:
            continue

        submission_values: set[str] = set()
        synonyms: dict[str, str] = {}
        synonyms_by_submission: dict[str, set[str]] = {}
        nci_codes: dict[str, str] = {}
        definitions: dict[str, str] = {}
        preferred_terms: dict[str, str] = {}

        extensible = (
            _clean_value(rows[0].get("Codelist Extensible (Yes/No)")).lower() == "yes"
        )
        name_raw = _clean_value(rows[0].get("Codelist Name") or code)
        name_key = name_raw.upper()
        standard = _clean_value(rows[0].get("Standard and Date"))
        source_file = _clean_value(rows[0].get("_source_file"))

        for row in rows:
            submission = _clean_value(row.get("CDISC Submission Value"))
            if not submission:
                continue
            canonical_value = submission
            submission_values.add(canonical_value)

            synonyms_by_submission.setdefault(canonical_value, set())

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

            synonyms[canonical_value.upper()] = canonical_value
            for syn in _split_synonyms(_clean_value(row.get("CDISC Synonym(s)"))):
                synonyms[syn.upper()] = canonical_value
                if syn.strip().upper() != canonical_value.strip().upper():
                    synonyms_by_submission.setdefault(canonical_value, set()).add(syn)

        submission_value_synonyms = {
            canonical: tuple(sorted(list(values)))
            for canonical, values in synonyms_by_submission.items()
            if values
        }

        ct = ControlledTerminology(
            codelist_name=name_raw,
            codelist_code=code,
            submission_values=submission_values,
            codelist_extensible=extensible,
            synonyms=synonyms,
            submission_value_synonyms=submission_value_synonyms,
            nci_codes=nci_codes,
            standards={standard} if standard else set(),
            sources={source_file} if source_file else set(),
            definitions=definitions,
            preferred_terms=preferred_terms,
            variable=name_key,
        )

        if code in registry_by_code:
            ct = _merge_ct(registry_by_code[code], ct)
        registry_by_code[code] = ct

        existing = registry_by_name.get(name_key)
        if existing:
            registry_by_name[name_key] = _merge_ct(existing, ct)
        else:
            registry_by_name[name_key] = ct

    return registry_by_code, registry_by_name
