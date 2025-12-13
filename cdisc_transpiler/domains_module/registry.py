"""Domain registry and lookup."""

from __future__ import annotations

from functools import lru_cache
from pathlib import Path
from typing import Iterable

from .domain_builder import build_domain_from_rows, build_supp_domain
from .general_classes import build_general_class_variables
from .loaders import load_csv_rows, load_dataset_attributes
from .models import SDTMDomain

# Path to SDTMIG v3.4 metadata (single source of truth)
_SDTMIG_PATH = Path(__file__).resolve().parent.parent.parent / "docs" / "SDTMIG_v3.4" / "Variables.csv"
_SDTM_V2_PATH = Path(__file__).resolve().parent.parent.parent / "docs" / "SDTM_v2.0" / "Variables.csv"
_SDTM_DATASETS_PATH = (
    Path(__file__).resolve().parent.parent.parent / "docs" / "SDTMIG_v3.4" / "Datasets.csv"
)

# Global registries
_DOMAIN_DEFINITIONS: dict[str, SDTMDomain] = {}
_SDTMIG_CACHE: dict[str, list[dict]] | None = None
_SDTM_V2_CACHE: dict[str, list[dict]] | None = None
_DATASET_ATTRIBUTES: dict[str, dict[str, str]] = {}
_GENERAL_CLASS_VARIABLES: dict[str, dict[str, any]] = {}
_GENERAL_CLASS_USAGE: dict[str, dict[str, set[str]]] = {}


def _load_sdtmig_cache() -> dict[str, list[dict]]:
    """Load SDTMIG v3.4 metadata from CSV."""
    global _SDTMIG_CACHE
    if _SDTMIG_CACHE is None:
        _SDTMIG_CACHE = load_csv_rows(_SDTMIG_PATH)
    return _SDTMIG_CACHE


def _load_sdtm_v2_cache() -> dict[str, list[dict]]:
    """Load SDTM v2.0 metadata from CSV (used as fallback/enrichment)."""
    global _SDTM_V2_CACHE
    if _SDTM_V2_CACHE is None:
        _SDTM_V2_CACHE = load_csv_rows(_SDTM_V2_PATH)
    return _SDTM_V2_CACHE


def _load_dataset_attributes() -> dict[str, dict[str, str]]:
    """Load dataset attributes."""
    global _DATASET_ATTRIBUTES
    if not _DATASET_ATTRIBUTES:
        _DATASET_ATTRIBUTES = load_dataset_attributes(_SDTM_DATASETS_PATH)
    return _DATASET_ATTRIBUTES


def _initialize_general_classes() -> None:
    """Initialize general class variables and usage."""
    global _GENERAL_CLASS_VARIABLES, _GENERAL_CLASS_USAGE
    if not _GENERAL_CLASS_VARIABLES:
        sdtmig = _load_sdtmig_cache()
        sdtm_v2 = _load_sdtm_v2_cache()
        _GENERAL_CLASS_VARIABLES, _GENERAL_CLASS_USAGE = build_general_class_variables(
            sdtmig, sdtm_v2
        )


def _register(domain: SDTMDomain) -> None:
    """Register a domain definition."""
    _DOMAIN_DEFINITIONS[domain.code.upper()] = domain


def _build_domain_from_cache(code: str) -> SDTMDomain | None:
    """Lookup domain rows from metadata caches (SDTMIG v3.4 then SDTM v2.0)."""
    attrs = _load_dataset_attributes()
    cache_v34 = _load_sdtmig_cache()
    rows = cache_v34.get(code)
    if rows:
        return build_domain_from_rows(code, rows, "SDTMIG v3.4", attrs)
    cache_v2 = _load_sdtm_v2_cache()
    rows = cache_v2.get(code)
    if rows:
        return build_domain_from_rows(code, rows, "SDTM v2.0", attrs)
    return None


def _register_all_domains() -> None:
    """Register all domains defined in the CSV metadata (v3.4 overriding v2.0)."""
    attrs = _load_dataset_attributes()
    
    # Register SDTM v2.0 first
    cache_v2 = _load_sdtm_v2_cache()
    for code, rows in sorted(cache_v2.items()):
        domain = build_domain_from_rows(code, rows, "SDTM v2.0", attrs)
        if domain:
            _register(domain)

    # Register SDTMIG v3.4 (newer) to override with latest metadata
    cache_v34 = _load_sdtmig_cache()
    for code, rows in sorted(cache_v34.items()):
        domain = build_domain_from_rows(code, rows, "SDTMIG v3.4", attrs)
        if domain:
            _register(domain)


@lru_cache(maxsize=None)
def get_domain(code: str) -> SDTMDomain:
    """Get domain definition by code."""
    key = code.upper()
    if key in _DOMAIN_DEFINITIONS:
        return _DOMAIN_DEFINITIONS[key]

    # Supplemental qualifiers: build SUPP-- domains from SUPPQUAL metadata
    if key.startswith("SUPP") and len(key) == 6:
        suppqual_base = _DOMAIN_DEFINITIONS.get("SUPPQUAL") or _build_domain_from_cache("SUPPQUAL")
        supp = build_supp_domain(key, suppqual_base)
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


# Initialize CSV-driven domains at import time
_initialize_general_classes()
_register_all_domains()
