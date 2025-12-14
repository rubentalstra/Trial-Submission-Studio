"""Controlled terminology registry and validation helpers.

This module provides access to CDISC Controlled Terminology loaded from
CSV files in the Controlled_Terminology directory.

All codelist codes are loaded DYNAMICALLY from:
- SDTMIG Variables.csv (via domains_module) for variable-to-codelist mappings
- CT CSV files for submission values, synonyms, and labels

There are NO hardcoded codelist codes - everything is loaded from source files.

Usage:
    from cdisc_transpiler.terminology_module import (
        # Dynamic codelist discovery
        get_variable_codelist,
        get_testcd_codelist,
        get_unit_codelist,
        
        # Core CT lookup
        get_controlled_terminology,
        get_submission_values,
        get_preferred_terms,
        get_synonyms,
        get_definitions,
        get_nci_code,
        
        # Domain-specific lookups
        get_domain_testcd_values,
        get_domain_testcd_labels,
        get_domain_testcd_synonyms,
        get_domain_valid_units,
        
        # Normalization
        normalize_testcd,
        normalize_to_submission_value,
        get_testcd_label,
    )
"""

from __future__ import annotations

from .models import ControlledTerminology
from .registry import (
    # Dynamic codelist discovery from domain variables
    get_variable_codelist,
    get_testcd_codelist,
    get_unit_codelist,
    # Core CT functions
    get_controlled_terminology,
    get_submission_values,
    get_preferred_terms,
    get_synonyms,
    get_definitions,
    get_nci_code,
    # Domain-specific functions
    get_domain_testcd_values,
    get_domain_testcd_labels,
    get_domain_testcd_synonyms,
    get_domain_valid_units,
    # Normalization functions
    normalize_to_submission_value,
    normalize_testcd,
    get_testcd_label,
)

__all__ = [
    # Models
    "ControlledTerminology",
    # Dynamic codelist discovery
    "get_variable_codelist",
    "get_testcd_codelist",
    "get_unit_codelist",
    # Core CT functions
    "get_controlled_terminology",
    "get_submission_values",
    "get_preferred_terms",
    "get_synonyms",
    "get_definitions",
    "get_nci_code",
    # Domain-specific functions
    "get_domain_testcd_values",
    "get_domain_testcd_labels",
    "get_domain_testcd_synonyms",
    "get_domain_valid_units",
    # Normalization functions
    "normalize_to_submission_value",
    "normalize_testcd",
    "get_testcd_label",
]
