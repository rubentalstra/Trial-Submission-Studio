"""Controlled terminology registry and lookup functions.

This module provides the main API for accessing controlled terminology
data, including lookup by codelist code or variable name.

All codelist codes are loaded dynamically from the SDTMIG Variables.csv file
via the domains_module, and terminology values are loaded from the CT CSV files.
There are NO hardcoded codelist codes in this module.

SDTM Reference:
    CDISC Controlled Terminology provides standardized values for SDTM variables.
    Each codelist has:
    - Submission Values: The exact values to use in SDTM datasets
    - Synonyms: Alternative names that map to submission values
    - Preferred Terms: Human-readable labels for each value
    - Definitions: Full descriptions of each value
"""

from __future__ import annotations

from functools import lru_cache
from pathlib import Path
from typing import Dict, Set

from .loader import build_registry
from .models import ControlledTerminology
from ..domains_module import get_domain, list_domains


def _get_ct_dir() -> Path:
    """Get CT directory from config, with version resolution.

    Returns:
        Path to the CT directory (with version subdirectory resolved)
    """
    # Lazy import to avoid circular imports
    from ..config import TranspilerConfig

    config = TranspilerConfig()
    ct_base = config.ct_dir

    # Make path absolute if relative
    if not ct_base.is_absolute():
        package_root = Path(__file__).resolve().parent.parent.parent
        ct_base = package_root / ct_base

    if not ct_base.exists():
        return ct_base

    # Try to find the latest version folder
    candidates = sorted(
        [d for d in ct_base.iterdir() if d.is_dir() and not d.name.startswith(".")]
    )

    if candidates:
        return candidates[-1]  # Latest by name (ISO date naming)

    return ct_base


# Global registries (lazily initialized)
_REGISTRY_BY_CODE: dict[str, ControlledTerminology] | None = None
_REGISTRY_BY_NAME: dict[str, ControlledTerminology] | None = None


def _ensure_registry_initialized() -> tuple[
    dict[str, ControlledTerminology], dict[str, ControlledTerminology]
]:
    """Ensure CT registries are initialized (lazy initialization).

    Returns:
        Tuple of (registry_by_code, registry_by_name)
    """
    global _REGISTRY_BY_CODE, _REGISTRY_BY_NAME

    if _REGISTRY_BY_CODE is None or _REGISTRY_BY_NAME is None:
        ct_dir = _get_ct_dir()
        _REGISTRY_BY_CODE, _REGISTRY_BY_NAME = build_registry(ct_dir)

    return _REGISTRY_BY_CODE, _REGISTRY_BY_NAME


# -----------------------------------------------------------------------------
# Dynamic Codelist Code Discovery from Domain Variables
# All codelist codes are loaded from SDTMIG Variables.csv via domains_module
# -----------------------------------------------------------------------------


@lru_cache(maxsize=None)
def _get_domain_variable_codelists(domain_code: str) -> Dict[str, str]:
    """Get all codelist codes for variables in a domain.

    Loads from the domain's variable definitions (sourced from Variables.csv).

    Args:
        domain_code: SDTM domain code (e.g., "VS", "LB")

    Returns:
        Dictionary mapping variable names to codelist codes
    """
    try:
        domain = get_domain(domain_code)
    except (KeyError, ValueError):
        return {}

    codelists: Dict[str, str] = {}
    for var in domain.variables:
        if var.codelist_code:
            codelists[var.name.upper()] = var.codelist_code
    return codelists


@lru_cache(maxsize=None)
def get_variable_codelist(domain_code: str, variable_name: str) -> str | None:
    """Get the codelist code for a specific variable in a domain.

    Dynamically loads from the domain's variable definitions.

    Args:
        domain_code: SDTM domain code (e.g., "VS", "LB")
        variable_name: Variable name (e.g., "VSTESTCD", "LBORRESU")

    Returns:
        Codelist code or None if not found
    """
    codelists = _get_domain_variable_codelists(domain_code)
    return codelists.get(variable_name.upper())


def get_testcd_codelist(domain_code: str) -> str | None:
    """Get the --TESTCD codelist code for a domain.

    Dynamically loads from the domain's variable definitions.

    Args:
        domain_code: SDTM domain code (e.g., "VS", "LB")

    Returns:
        Codelist code or None if not a Findings domain with TESTCD
    """
    testcd_var = f"{domain_code.upper()}TESTCD"
    return get_variable_codelist(domain_code, testcd_var)


def get_unit_codelist(domain_code: str) -> str | None:
    """Get the --ORRESU or --STRESU codelist code for a domain.

    Dynamically loads from the domain's variable definitions.

    Args:
        domain_code: SDTM domain code (e.g., "VS", "LB")

    Returns:
        Codelist code or None if not defined
    """
    # Try --ORRESU first, then --STRESU
    orresu_var = f"{domain_code.upper()}ORRESU"
    code = get_variable_codelist(domain_code, orresu_var)
    if code:
        return code

    stresu_var = f"{domain_code.upper()}STRESU"
    return get_variable_codelist(domain_code, stresu_var)


@lru_cache(maxsize=None)
def _variable_to_codelist() -> Dict[str, str]:
    """Map variable names to codelist codes using domain metadata.

    Returns:
        Dictionary mapping uppercase variable names to codelist codes
    """
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


# -----------------------------------------------------------------------------
# Core CT Lookup Functions
# -----------------------------------------------------------------------------


def get_controlled_terminology(
    codelist_code: str | None = None, variable: str | None = None
) -> ControlledTerminology | None:
    """Get controlled terminology by codelist code or variable name.

    Args:
        codelist_code: NCI codelist code (e.g., "C66731" for SEX)
        variable: Variable name (e.g., "SEX", "VSTESTCD")

    Returns:
        ControlledTerminology object or None if not found
    """
    if not codelist_code and not variable:
        return None

    registry_by_code, registry_by_name = _ensure_registry_initialized()

    if codelist_code:
        ct = registry_by_code.get(codelist_code.strip().upper())
        if ct:
            return ct

    if variable:
        var_key = variable.upper()
        code = _variable_to_codelist().get(var_key)
        if code:
            ct = registry_by_code.get(code)
            if ct:
                return ct
        return registry_by_name.get(var_key)

    return None


@lru_cache(maxsize=None)
def get_submission_values(codelist_code: str) -> Set[str]:
    """Get all valid CDISC submission values for a codelist.

    Args:
        codelist_code: NCI codelist code (e.g., "C66741" for VSTESTCD)

    Returns:
        Set of valid submission values
    """
    registry_by_code, _ = _ensure_registry_initialized()
    ct = registry_by_code.get(codelist_code.upper())
    if ct is None:
        return set()
    return ct.submission_values


@lru_cache(maxsize=None)
def get_preferred_terms(codelist_code: str) -> Dict[str, str]:
    """Get mapping of submission values to their preferred terms (labels).

    Args:
        codelist_code: NCI codelist code

    Returns:
        Dictionary mapping submission values to preferred terms

    Example:
        >>> terms = get_preferred_terms("C66741")  # VSTESTCD
        >>> terms.get("HR")
        'Heart Rate'
    """
    registry_by_code, _ = _ensure_registry_initialized()
    ct = registry_by_code.get(codelist_code.upper())
    if ct is None:
        return {}

    labels: Dict[str, str] = {}
    for value in ct.submission_values:
        if value in ct.preferred_terms:
            labels[value] = ct.preferred_terms[value]
        elif value in ct.definitions:
            labels[value] = ct.definitions[value]
        else:
            labels[value] = value

    return labels


@lru_cache(maxsize=None)
def get_synonyms(codelist_code: str) -> Dict[str, str]:
    """Get mapping of synonyms to their CDISC submission values.

    This enables normalization of source data that uses non-standard codes.

    Args:
        codelist_code: NCI codelist code

    Returns:
        Dictionary mapping synonym (uppercase) to submission value

    Example:
        >>> synonyms = get_synonyms("C66741")  # VSTESTCD
        >>> synonyms.get("PULSE")  # Maps to "HR"
    """
    registry_by_code, _ = _ensure_registry_initialized()
    ct = registry_by_code.get(codelist_code.upper())
    if ct is None:
        return {}
    return ct.synonyms or {}


@lru_cache(maxsize=None)
def get_definitions(codelist_code: str) -> Dict[str, str]:
    """Get CDISC definitions for submission values.

    Args:
        codelist_code: NCI codelist code

    Returns:
        Dictionary mapping submission values to definitions
    """
    registry_by_code, _ = _ensure_registry_initialized()
    ct = registry_by_code.get(codelist_code.upper())
    if ct is None:
        return {}
    return ct.definitions or {}


# -----------------------------------------------------------------------------
# Domain-Specific Convenience Functions
# All these functions dynamically discover codelist codes from domain variables
# -----------------------------------------------------------------------------


@lru_cache(maxsize=None)
def get_domain_testcd_values(domain_code: str) -> Set[str]:
    """Get all valid --TESTCD values for a domain from CT.

    Args:
        domain_code: SDTM domain code (e.g., "VS", "LB")

    Returns:
        Set of valid test codes
    """
    codelist = get_testcd_codelist(domain_code)
    if not codelist:
        return set()
    return get_submission_values(codelist)


@lru_cache(maxsize=None)
def get_domain_testcd_labels(domain_code: str) -> Dict[str, str]:
    """Get --TESTCD to --TEST label mapping for a domain.

    Args:
        domain_code: SDTM domain code (e.g., "VS", "LB")

    Returns:
        Dictionary mapping test codes to their labels

    Example:
        >>> labels = get_domain_testcd_labels("VS")
        >>> labels.get("HR")
        'Heart Rate'
    """
    codelist = get_testcd_codelist(domain_code)
    if not codelist:
        return {}
    return get_preferred_terms(codelist)


@lru_cache(maxsize=None)
def get_domain_testcd_synonyms(domain_code: str) -> Dict[str, str]:
    """Get synonym to --TESTCD mapping for a domain.

    Args:
        domain_code: SDTM domain code (e.g., "VS", "LB")

    Returns:
        Dictionary mapping synonyms to standard test codes
    """
    codelist = get_testcd_codelist(domain_code)
    if not codelist:
        return {}
    return get_synonyms(codelist)


@lru_cache(maxsize=None)
def get_domain_valid_units(domain_code: str) -> Set[str]:
    """Get valid --ORRESU/--STRESU values for a domain.

    Args:
        domain_code: SDTM domain code (e.g., "VS", "LB")

    Returns:
        Set of valid unit values
    """
    codelist = get_unit_codelist(domain_code)
    if not codelist:
        return set()
    return get_submission_values(codelist)


# -----------------------------------------------------------------------------
# Value Normalization Functions
# -----------------------------------------------------------------------------


def normalize_to_submission_value(codelist_code: str, source_value: str) -> str | None:
    """Normalize a source value to its CDISC submission value.

    Uses synonyms from CT to map non-standard values to standard ones.

    Args:
        codelist_code: NCI codelist code
        source_value: Source value to normalize

    Returns:
        CDISC submission value or None if not found
    """
    if not source_value:
        return None

    source_upper = source_value.upper().strip()
    submission_values = get_submission_values(codelist_code)
    synonyms = get_synonyms(codelist_code)

    # Check if already a valid submission value
    if source_upper in submission_values:
        return source_upper

    # Check synonyms
    if source_upper in synonyms:
        return synonyms[source_upper]

    return None


def normalize_testcd(domain_code: str, source_code: str) -> str | None:
    """Normalize a source test code to the CDISC --TESTCD submission value.

    Args:
        domain_code: SDTM domain code (e.g., "VS", "LB")
        source_code: Source test code to normalize

    Returns:
        Standard test code or None if not found

    Example:
        >>> normalize_testcd("VS", "PULSE")
        'HR'
        >>> normalize_testcd("VS", "HR")
        'HR'
    """
    codelist = get_testcd_codelist(domain_code)
    if not codelist:
        return None
    return normalize_to_submission_value(codelist, source_code)


def get_testcd_label(domain_code: str, testcd: str) -> str:
    """Get the --TEST label for a --TESTCD value.

    Args:
        domain_code: SDTM domain code (e.g., "VS", "LB")
        testcd: The test code (e.g., "HR", "GLUC")

    Returns:
        The label for the test code, or the test code itself if not found
    """
    labels = get_domain_testcd_labels(domain_code)
    return labels.get(testcd.upper(), testcd)
