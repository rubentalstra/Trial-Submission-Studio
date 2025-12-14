"""Controlled terminology registry and lookup functions.

This module provides the main API for accessing controlled terminology
data, including lookup by codelist code or variable name.
"""

from __future__ import annotations

from functools import lru_cache
from pathlib import Path
from typing import Dict

from .loader import build_registry
from .models import ControlledTerminology


def _resolve_ct_dir(ct_version: str) -> Path:
    """Select the CT folder for the configured CT_VERSION, fallback to newest.

    Args:
        ct_version: Target CT version string

    Returns:
        Path to the controlled terminology directory
    """
    ct_base_dir = Path(__file__).resolve().parent.parent.parent / "docs" / "Controlled_Terminology"
    target = ct_base_dir / ct_version
    if target.exists():
        return target
    # Fallback to most recent folder if exact version folder not present
    candidates = sorted(ct_base_dir.glob("*"))
    return candidates[-1] if candidates else target


# Import CT_VERSION from domains_module for configuration
def _get_ct_version() -> str:
    """Get the CT version from domains_module."""
    try:
        from ..domains_module import CT_VERSION
        return CT_VERSION
    except ImportError:
        return "2025-09-26"  # Default fallback


# Initialize the CT directory and registries
_CT_DIR = _resolve_ct_dir(_get_ct_version())
_REGISTRY_BY_CODE, _REGISTRY_BY_NAME = build_registry(_CT_DIR)


@lru_cache(maxsize=None)
def _variable_to_codelist() -> Dict[str, str]:
    """Map variable names to codelist codes using domain metadata.

    Returns:
        Dictionary mapping uppercase variable names to codelist codes
    """
    from ..domains_module import get_domain, list_domains

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
    """Get controlled terminology by codelist code or variable name.

    Args:
        codelist_code: NCI codelist code (e.g., "C66767")
        variable: Variable name (e.g., "SEX")

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
        # Fallback to codelist name matching (e.g., SEX)
        return _REGISTRY_BY_NAME.get(var_key)

    return None


def list_controlled_variables() -> tuple[str, ...]:
    """Return all variables with controlled terminology.

    Returns:
        Tuple of variable names (uppercase)
    """
    vars_from_domains = tuple(sorted(_variable_to_codelist().keys()))
    names = tuple(sorted(_REGISTRY_BY_NAME.keys()))
    # Merge and deduplicate while preserving lexical order
    merged = sorted(set(vars_from_domains) | set(names))
    return tuple(merged)


def get_nci_code(variable: str, value: str) -> str | None:
    """Return the NCI code for a variable/value combination.

    Args:
        variable: Variable name
        value: The value to look up

    Returns:
        NCI code or None if not found
    """
    ct = get_controlled_terminology(variable=variable)
    if ct is None:
        return None
    return ct.get_nci_code(value)


def get_codelist_code(variable: str) -> str | None:
    """Return the codelist code for a variable.

    Args:
        variable: Variable name

    Returns:
        Codelist code or None if not found
    """
    ct = get_controlled_terminology(variable=variable)
    if ct is None:
        return None
    return ct.codelist_code


@lru_cache(maxsize=None)
def get_test_labels(codelist_code: str) -> Dict[str, str]:
    """Return a dictionary mapping test codes to test names/labels from CT.

    This dynamically loads test labels from controlled terminology files,
    avoiding the need for hardcoded test label dictionaries.

    Args:
        codelist_code: The codelist code (e.g., "C66741" for VSTESTCD, 
                      "C65047" for LBTESTCD)

    Returns:
        Dictionary mapping test codes to their full names/labels
        
    Example:
        >>> labels = get_test_labels("C66741")  # VS Test Codes
        >>> labels.get("HR")
        'Heart Rate'
    """
    ct = _REGISTRY_BY_CODE.get(codelist_code.upper())
    if ct is None:
        return {}
    
    # Build mapping from submission values to their preferred terms/definitions
    labels: Dict[str, str] = {}
    for value in ct.submission_values:
        # Use preferred term if available, then definition, then value itself
        if value in ct.preferred_terms:
            label = ct.preferred_terms[value]
        elif value in ct.definitions:
            label = ct.definitions[value]
        else:
            label = value
        labels[value] = label
    
    return labels


@lru_cache(maxsize=None)
def get_vs_test_labels() -> Dict[str, str]:
    """Return VS (Vital Signs) test code to label mapping from CT.
    
    Loads from CDISC Controlled Terminology codelist C66741 (VSTESTCD).
    
    Returns:
        Dictionary mapping VSTESTCD values to their labels
        
    Example:
        >>> labels = get_vs_test_labels()
        >>> labels.get("HR")
        'Heart Rate'
    """
    return get_test_labels("C66741")


@lru_cache(maxsize=None)
def get_lb_test_labels() -> Dict[str, str]:
    """Return LB (Laboratory) test code to label mapping from CT.
    
    Loads from CDISC Controlled Terminology codelist C65047 (LBTESTCD).
    
    Returns:
        Dictionary mapping LBTESTCD values to their labels
        
    Example:
        >>> labels = get_lb_test_labels()
        >>> labels.get("GLUC")
        'Glucose'
    """
    return get_test_labels("C65047")
