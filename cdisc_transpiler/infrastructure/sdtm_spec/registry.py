"""SDTM domain registry and lookup (infrastructure).

This replaces the former `domains_module.registry` package.
"""

from __future__ import annotations

from collections.abc import Iterable
from functools import cache, lru_cache
from pathlib import Path
from typing import Any

from cdisc_transpiler.config import TranspilerConfig
from cdisc_transpiler.domain.entities.sdtm_domain import SDTMDomain

from .domain_builder import build_domain_from_rows, build_supp_domain
from .loaders import load_csv_rows, load_dataset_attributes


def _get_spec_paths() -> tuple[Path, Path, Path]:
    config = TranspilerConfig()
    spec_dir = config.sdtm_spec_dir

    if not spec_dir.is_absolute():
        # In-repo runs (tests/CLI) typically use repo-relative paths like
        # docs/SDTMIG_v3.4. Resolve relative to CWD first, then fall back to
        # the repository root (one level above the package directory).
        cwd_candidate = Path.cwd() / spec_dir
        if cwd_candidate.exists():
            spec_dir = cwd_candidate
        else:
            repo_root = Path(__file__).resolve().parents[3]
            spec_dir = repo_root / spec_dir

    sdtmig_path = spec_dir / "Variables.csv"
    datasets_path = spec_dir / "Datasets.csv"
    sdtm_v2_path = spec_dir.parent / "SDTM_v2.0" / "Variables.csv"
    return sdtmig_path, sdtm_v2_path, datasets_path


@lru_cache(maxsize=1)
def _sdtmig_cache() -> dict[str, list[dict[str, Any]]]:
    sdtmig_path, _, _ = _get_spec_paths()
    return load_csv_rows(sdtmig_path)


@lru_cache(maxsize=1)
def _sdtm_v2_cache() -> dict[str, list[dict[str, Any]]]:
    _, sdtm_v2_path, _ = _get_spec_paths()
    return load_csv_rows(sdtm_v2_path)


@lru_cache(maxsize=1)
def _dataset_attrs() -> dict[str, dict[str, str]]:
    _, _, datasets_path = _get_spec_paths()
    return load_dataset_attributes(datasets_path)


_DOMAIN_DEFINITIONS: dict[str, SDTMDomain] = {}


def _register(domain: SDTMDomain) -> None:
    _DOMAIN_DEFINITIONS[domain.code.upper()] = domain


def _build_domain_from_cache(code: str) -> SDTMDomain | None:
    attrs = _dataset_attrs()

    rows = _sdtmig_cache().get(code)
    if rows:
        return build_domain_from_rows(code, rows, "SDTMIG v3.4", attrs)

    rows = _sdtm_v2_cache().get(code)
    if rows:
        return build_domain_from_rows(code, rows, "SDTM v2.0", attrs)

    return None


def _ensure_registry_built() -> None:
    if _DOMAIN_DEFINITIONS:
        return

    attrs = _dataset_attrs()

    for code, rows in sorted(_sdtm_v2_cache().items()):
        domain = build_domain_from_rows(code, rows, "SDTM v2.0", attrs)
        if domain:
            _register(domain)

    for code, rows in sorted(_sdtmig_cache().items()):
        domain = build_domain_from_rows(code, rows, "SDTMIG v3.4", attrs)
        if domain:
            _register(domain)


@cache
def get_domain(code: str) -> SDTMDomain:
    _ensure_registry_built()

    key = code.upper()
    if key in _DOMAIN_DEFINITIONS:
        return _DOMAIN_DEFINITIONS[key]

    if key.startswith("SUPP") and len(key) == 6:
        suppqual_base = _DOMAIN_DEFINITIONS.get("SUPPQUAL") or _build_domain_from_cache(
            "SUPPQUAL"
        )
        supp = build_supp_domain(key, suppqual_base)
        _register(supp)
        return supp

    domain = _build_domain_from_cache(key)
    if domain:
        _register(domain)
        return domain

    raise KeyError(f"Unknown SDTM domain '{code}'")


def list_domains() -> Iterable[str]:
    _ensure_registry_built()
    return _DOMAIN_DEFINITIONS.keys()


def generalized_identifiers(domain_code: str) -> dict[str, str]:
    return get_domain(domain_code).implements_mapping()


__all__ = ["generalized_identifiers", "get_domain", "list_domains"]
