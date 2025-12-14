"""Controlled terminology registry and lookup functions.

This module provides the main API for accessing controlled terminology
data, including lookup by codelist code or variable name.

All submission values, labels, synonyms, and units are loaded dynamically from
the CDISC Controlled Terminology CSV files in docs/Controlled_Terminology/,
ensuring the data stays up-to-date with CT releases.

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


# -----------------------------------------------------------------------------
# SDTM Codelist Codes by Variable Type
# These are the official CDISC NCI codelist codes from the CT files
# -----------------------------------------------------------------------------

# --TESTCD codelists by domain (for Findings class domains)
TESTCD_CODELISTS: Dict[str, str] = {
    "VS": "C66741",   # VSTESTCD - Vital Signs Test Code
    "LB": "C65047",   # LBTESTCD - Laboratory Test Code
    "EG": "C71153",   # EGTESTCD - ECG Test Code
    "PE": "C74559",   # PETESTCD - Physical Examination Test Code
    "QS": "C100129",  # QSTESTCD - Questionnaire Test Code
    "DA": "C78735",   # DATESTCD - Drug Accountability Test Code
    "IE": "C66786",   # IETESTCD - Inclusion/Exclusion Criterion
    "SC": "C99078",   # SCTESTCD - Subject Characteristic Test Code
    "FT": "C120526",  # FTTESTCD - Functional Test Test Code
    "CV": "C102580",  # CVTESTCD - Cardiovascular Test Code
    "RE": "C102581",  # RETESTCD - Respiratory Test Code
    "DD": "C117743",  # DDTESTCD - Death Details Test Code
    "FA": "C123973",  # FATESTCD - Findings About Test Code
    "MI": "C117388",  # MITESTCD - Microscopic Findings Test Code
    "MO": "C117389",  # MOTESTCD - Morphology Test Code
    "TU": "C96785",   # TUTESTCD - Tumor Identification Test Code
    "TR": "C96784",   # TRTESTCD - Tumor Results Test Code
    "RS": "C96783",   # RSTESTCD - Response Test Code
}

# --ORRESU/--STRESU unit codelists by domain
UNIT_CODELISTS: Dict[str, str] = {
    "VS": "C66770",   # VSORRESU/VSSTRESU - Units for Vital Signs
    "LB": "C66771",   # LBORRESU/LBSTRESU - Units for Laboratory
    "EG": "C71150",   # EGORRESU/EGSTRESU - Units for ECG
    "PC": "C128686",  # PCORRESU/PCSTRESU - Units for PK Concentrations
    "PP": "C128686",  # PPORRESU/PPSTRESU - Units for PK Parameters
}

# Common variable codelists (not domain-specific)
VARIABLE_CODELISTS: Dict[str, str] = {
    "SEX": "C66731",      # Sex
    "RACE": "C74457",     # Race
    "ETHNIC": "C66790",   # Ethnicity
    "COUNTRY": "C66732",  # Country
    "EPOCH": "C99079",    # Epoch
    "ARMCD": "C66767",    # Arm Code
    "ACTARMCD": "C66767", # Actual Arm Code
}


def _resolve_ct_dir(ct_version: str) -> Path:
    """Select the CT folder for the configured CT_VERSION, fallback to newest."""
    ct_base_dir = (
        Path(__file__).resolve().parent.parent.parent
        / "docs"
        / "Controlled_Terminology"
    )
    target = ct_base_dir / ct_version
    if target.exists():
        return target
    candidates = sorted(ct_base_dir.glob("*"))
    return candidates[-1] if candidates else target


def _get_ct_version() -> str:
    """Get the CT version from domains_module."""
    try:
        from ..domains_module import CT_VERSION
        return CT_VERSION
    except ImportError:
        return "2025-09-26"


# Initialize the CT directory and registries
_CT_DIR = _resolve_ct_dir(_get_ct_version())
_REGISTRY_BY_CODE, _REGISTRY_BY_NAME = build_registry(_CT_DIR)


@lru_cache(maxsize=None)
def _variable_to_codelist() -> Dict[str, str]:
    """Map variable names to codelist codes using domain metadata."""
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
        return _REGISTRY_BY_NAME.get(var_key)

    return None


@lru_cache(maxsize=None)
def get_submission_values(codelist_code: str) -> Set[str]:
    """Get all valid CDISC submission values for a codelist.

    Args:
        codelist_code: NCI codelist code (e.g., "C66741" for VSTESTCD)

    Returns:
        Set of valid submission values
    """
    ct = _REGISTRY_BY_CODE.get(codelist_code.upper())
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
    ct = _REGISTRY_BY_CODE.get(codelist_code.upper())
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
    ct = _REGISTRY_BY_CODE.get(codelist_code.upper())
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
    ct = _REGISTRY_BY_CODE.get(codelist_code.upper())
    if ct is None:
        return {}
    return ct.definitions or {}


# -----------------------------------------------------------------------------
# Domain-Specific Convenience Functions
# -----------------------------------------------------------------------------


def get_testcd_codelist(domain_code: str) -> str | None:
    """Get the --TESTCD codelist code for a domain.

    Args:
        domain_code: SDTM domain code (e.g., "VS", "LB")

    Returns:
        Codelist code or None if not a Findings domain
    """
    return TESTCD_CODELISTS.get(domain_code.upper())


def get_unit_codelist(domain_code: str) -> str | None:
    """Get the unit codelist code for a domain.

    Args:
        domain_code: SDTM domain code (e.g., "VS", "LB")

    Returns:
        Codelist code or None if not defined
    """
    return UNIT_CODELISTS.get(domain_code.upper())


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


# -----------------------------------------------------------------------------
# Legacy/Convenience Functions
# -----------------------------------------------------------------------------


def list_controlled_variables() -> tuple[str, ...]:
    """Return all variables with controlled terminology."""
    vars_from_domains = tuple(sorted(_variable_to_codelist().keys()))
    names = tuple(sorted(_REGISTRY_BY_NAME.keys()))
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


# Backward compatibility aliases (deprecated - use new names)
get_test_labels = get_preferred_terms
get_test_synonyms = get_synonyms
