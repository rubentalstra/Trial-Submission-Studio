"""SAS value normalization utilities.

This module provides utilities for normalizing values in SAS code generation,
including domain-specific normalizers and controlled terminology handling.
"""

from __future__ import annotations

from functools import lru_cache
import re
from typing import TYPE_CHECKING

from ...repositories.ct_repository import CTRepository

if TYPE_CHECKING:
    from cdisc_transpiler.domain.entities.sdtm_domain import SDTMVariable
    from cdisc_transpiler.domain.entities.mapping import ColumnMapping


@lru_cache(maxsize=1)
def _ct_repository() -> CTRepository:
    return CTRepository()


_WHITESPACE_RE = re.compile(r"\s+")


def _normalize_token(value: str) -> str:
    """Normalize a token for matching.

    This is intentionally generic (no variable-specific rules): the goal is to
    make matching resilient to common formatting differences in source data
    (spacing/hyphen/comma variations) while still grounding the canonical
    values in CT/spec metadata.
    """
    upper = (value or "").strip().upper()
    upper = upper.replace("_", " ").replace("-", " ")
    upper = _WHITESPACE_RE.sub(" ", upper)
    return upper


def _token_variants(value: str) -> set[str]:
    """Return a small set of normalized variants for matching."""
    raw = (value or "").strip()
    if not raw:
        return set()
    base = raw.upper().strip()
    variants = {base, _normalize_token(base)}
    # Handle common punctuation variants generically.
    variants.add(_normalize_token(base.replace(",", " ")))
    variants.add(_normalize_token(base.replace("/", " ")))
    return {v for v in variants if v}


def _parse_submission_values(raw: str | None) -> set[str]:
    """Parse SDTMIG "Codelist Submission Values" into a canonical set.

    The SDTMIG Variables.csv sometimes includes a concrete list of permissible
    submission values even when the variable is not linked to a CT codelist.
    """
    if not raw:
        return set()
    text = str(raw).strip()
    if not text:
        return set()

    tokens: list[str] = []
    for sep in [";", ","]:
        if sep in text:
            tokens = [t.strip() for t in text.split(sep)]
            break
    if not tokens:
        tokens = [text]

    cleaned = {t.strip().strip('"').upper() for t in tokens if t.strip()}
    return {c for c in cleaned if c}


def _get_ct_value_map(
    variable_name: str, variable: "SDTMVariable | None"
) -> dict[str, set[str]] | None:
    """Get value map from controlled terminology repository."""

    ct = None
    if variable is not None and variable.codelist_code:
        ct = _ct_repository().get_by_code(variable.codelist_code)
    if ct is None:
        ct = _ct_repository().get_by_name(variable_name)
    if ct is None:
        return None

    value_map: dict[str, set[str]] = {}

    # Seed with canonicals.
    for canonical in ct.submission_values:
        value_map[canonical] = set(_token_variants(canonical)) | {canonical}

    # Expand with CT-provided synonyms, plus generic formatting variants.
    if ct.synonyms:
        for syn_key, canonical in ct.synonyms.items():
            if not canonical:
                continue
            bucket = value_map.setdefault(canonical, set())
            bucket.update(_token_variants(syn_key))
            bucket.update(_token_variants(canonical))

    return value_map


def _get_spec_value_map(variable: "SDTMVariable | None") -> dict[str, set[str]] | None:
    """Get a value map from SDTMIG "Codelist Submission Values" when present."""
    if variable is None:
        return None
    canonicals = _parse_submission_values(variable.codelist_submission_values)
    if not canonicals:
        return None
    return {c: set(_token_variants(c)) | {c} for c in canonicals}


def _render_value_map(mapping: "ColumnMapping", value_map: dict[str, set[str]]) -> str:
    """Render a SAS select/when map for value normalization."""
    source = mapping.source_column
    target = mapping.target_variable

    lines = [
        f"{target} = strip(coalescec({source}, ''));",
        f"{target} = upcase({target});",
        f"select ({target});",
    ]

    for canonical, synonyms in value_map.items():
        formatted = ", ".join(f"'{s}'" for s in sorted(synonyms))
        lines.append(f"    when ({formatted}) {target} = '{canonical}';")

    lines.append("    otherwise; /* keep as-is */")
    lines.append("end;")
    return "\n".join(lines)


def render_assignment(mapping: "ColumnMapping", variable: "SDTMVariable | None") -> str:
    """Return SAS statements that assign a target variable with normalization."""
    target_name = mapping.target_variable.upper()

    if mapping.transformation:
        expr = mapping.transformation
        return f"{mapping.target_variable} = {expr};"

    # First check controlled terminology registry
    ct_value_map = _get_ct_value_map(target_name, variable)
    if ct_value_map:
        return _render_value_map(mapping, ct_value_map)

    # Then check SDTMIG-provided submission values (when CT is not linked)
    spec_value_map = _get_spec_value_map(variable)
    if spec_value_map:
        return _render_value_map(mapping, spec_value_map)

    # Default assignment
    expr = mapping.source_column
    is_character = False
    if variable:
        is_character = variable.type.lower() == "char"

    if is_character:
        expr = f"coalescec({expr}, '')"
        expr = f"strip({expr})"
        # Only upcase when we have a reason grounded in metadata:
        # - identifiers are case-insensitive keys
        # - variables with explicit value lists (CT/spec) are compared uppercased
        if variable is not None:
            role = (variable.role or "").strip().lower()
            has_explicit_values = bool(variable.codelist_code) or bool(
                _parse_submission_values(variable.codelist_submission_values)
            )
            if role == "identifier" or has_explicit_values:
                expr = f"upcase({expr})"

    return f"{mapping.target_variable} = {expr};"
