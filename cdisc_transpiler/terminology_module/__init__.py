"""Controlled terminology registry and validation helpers.

This module provides access to CDISC Controlled Terminology loaded from
CSV files in the Controlled_Terminology directory.

All submission values, labels, synonyms, and units are loaded dynamically
from the CT files, ensuring the data stays up-to-date with CT releases.

Usage:
    from cdisc_transpiler.terminology_module import (
        # Core CT lookup
        get_controlled_terminology,
        get_submission_values,
        get_preferred_terms,
        get_synonyms,
        
        # Domain-specific lookups
        get_domain_testcd_values,
        get_domain_testcd_labels,
        get_domain_testcd_synonyms,
        get_domain_valid_units,
        
        # Normalization
        normalize_testcd,
        normalize_to_submission_value,
        get_testcd_label,
        
        # Codelist lookups
        TESTCD_CODELISTS,
        UNIT_CODELISTS,
    )
"""

from __future__ import annotations

from .models import ControlledTerminology
from .registry import (
    # Codelist mappings
    TESTCD_CODELISTS,
    UNIT_CODELISTS,
    VARIABLE_CODELISTS,
    # Core CT functions
    get_controlled_terminology,
    get_submission_values,
    get_preferred_terms,
    get_synonyms,
    get_definitions,
    # Codelist lookups
    get_testcd_codelist,
    get_unit_codelist,
    # Domain-specific functions
    get_domain_testcd_values,
    get_domain_testcd_labels,
    get_domain_testcd_synonyms,
    get_domain_valid_units,
    # Normalization functions
    normalize_to_submission_value,
    normalize_testcd,
    get_testcd_label,
    # Legacy functions
    list_controlled_variables,
    get_nci_code,
    get_codelist_code,
    # Backward compatibility
    get_test_labels,
    get_test_synonyms,
)

__all__ = [
    # Models
    "ControlledTerminology",
    # Codelist mappings
    "TESTCD_CODELISTS",
    "UNIT_CODELISTS",
    "VARIABLE_CODELISTS",
    # Core CT functions
    "get_controlled_terminology",
    "get_submission_values",
    "get_preferred_terms",
    "get_synonyms",
    "get_definitions",
    # Codelist lookups
    "get_testcd_codelist",
    "get_unit_codelist",
    # Domain-specific functions
    "get_domain_testcd_values",
    "get_domain_testcd_labels",
    "get_domain_testcd_synonyms",
    "get_domain_valid_units",
    # Normalization functions
    "normalize_to_submission_value",
    "normalize_testcd",
    "get_testcd_label",
    # Legacy functions
    "list_controlled_variables",
    "get_nci_code",
    "get_codelist_code",
    # Backward compatibility (deprecated)
    "get_test_labels",
    "get_test_synonyms",
]
