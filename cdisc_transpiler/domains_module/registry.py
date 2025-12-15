"""Domain registry and lookup."""

from __future__ import annotations

from functools import lru_cache
from pathlib import Path
from typing import Any, Iterable

from .domain_builder import build_domain_from_rows, build_supp_domain
from .general_classes import build_general_class_variables
from .loaders import load_csv_rows, load_dataset_attributes
from ..domain.entities.sdtm_domain import SDTMDomain


def _get_spec_paths() -> tuple[Path, Path, Path]:
    """Get SDTM spec file paths from config or default locations.
    
    Returns:
        Tuple of (sdtmig_path, sdtm_v2_path, datasets_path)
    """
    # Lazy import to avoid circular imports
    from ..config import TranspilerConfig
    
    config = TranspilerConfig()
    spec_dir = config.sdtm_spec_dir
    
    # Make path absolute if relative
    if not spec_dir.is_absolute():
        package_root = Path(__file__).resolve().parent.parent.parent
        spec_dir = package_root / spec_dir
    
    # SDTMIG v3.4 paths
    sdtmig_path = spec_dir / "Variables.csv"
    datasets_path = spec_dir / "Datasets.csv"
    
    # SDTM v2.0 fallback (relative to SDTMIG location)
    sdtm_v2_path = spec_dir.parent / "SDTM_v2.0" / "Variables.csv"
    
    return sdtmig_path, sdtm_v2_path, datasets_path


# Global registries (lazily initialized)
_DOMAIN_DEFINITIONS: dict[str, SDTMDomain] = {}
_sdtmig_cache: dict[str, list[dict[str, Any]]] | None = None
_sdtm_v2_cache: dict[str, list[dict[str, Any]]] | None = None
_dataset_attributes: dict[str, dict[str, str]] = {}
_general_class_variables: dict[str, dict[str, Any]] = {}
_general_class_usage: dict[str, dict[str, set[str]]] = {}
_initialized: bool = False


def _load_sdtmig_cache() -> dict[str, list[dict[str, Any]]]:
    """Load SDTMIG v3.4 metadata from CSV."""
    global _sdtmig_cache
    if _sdtmig_cache is None:
        sdtmig_path, _, _ = _get_spec_paths()
        _sdtmig_cache = load_csv_rows(sdtmig_path)
    return _sdtmig_cache


def _load_sdtm_v2_cache() -> dict[str, list[dict[str, Any]]]:
    """Load SDTM v2.0 metadata from CSV (used as fallback/enrichment)."""
    global _sdtm_v2_cache
    if _sdtm_v2_cache is None:
        _, sdtm_v2_path, _ = _get_spec_paths()
        _sdtm_v2_cache = load_csv_rows(sdtm_v2_path)
    return _sdtm_v2_cache


def _load_dataset_attributes() -> dict[str, dict[str, str]]:
    """Load dataset attributes."""
    global _dataset_attributes
    if not _dataset_attributes:
        _, _, datasets_path = _get_spec_paths()
        _dataset_attributes = load_dataset_attributes(datasets_path)
    return _dataset_attributes


def _initialize_general_classes() -> None:
    """Initialize general class variables and usage."""
    global _general_class_variables, _general_class_usage
    if not _general_class_variables:
        sdtmig = _load_sdtmig_cache()
        sdtm_v2 = _load_sdtm_v2_cache()
        _general_class_variables, _general_class_usage = build_general_class_variables(
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


def _ensure_initialized() -> None:
    """Ensure the domain registry is initialized (lazy initialization)."""
    global _initialized
    if not _initialized:
        _initialize_general_classes()
        _register_all_domains()
        _initialized = True


@lru_cache(maxsize=None)
def get_domain(code: str) -> SDTMDomain:
    """Get domain definition by code."""
    _ensure_initialized()
    
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
    _ensure_initialized()
    return _DOMAIN_DEFINITIONS.keys()


def generalized_identifiers(domain_code: str) -> dict[str, str]:
    """Return mapping of variables to their generalized Identifier/Timing placeholders."""
    domain = get_domain(domain_code)
    return domain.implements_mapping()


# NOTE: Initialization is now lazy - domains are loaded on first access to get_domain() or list_domains()
