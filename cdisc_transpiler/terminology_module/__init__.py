"""Controlled terminology registry and validation helpers.

This module provides access to CDISC Controlled Terminology loaded from
CSV files in the Controlled_Terminology directory.

The module is organized into:
- models: Data classes (ControlledTerminology)
- loader: CSV file loading and parsing
- registry: Lookup functions and registries

Usage:
    from cdisc_transpiler.terminology_module import (
        get_controlled_terminology,
        list_controlled_variables,
        get_nci_code,
        get_codelist_code,
        get_test_labels,
        get_vs_test_labels,
        get_lb_test_labels,
        ControlledTerminology,
    )
"""

from __future__ import annotations

from .models import ControlledTerminology
from .registry import (
    get_codelist_code,
    get_controlled_terminology,
    get_nci_code,
    list_controlled_variables,
    get_test_labels,
    get_vs_test_labels,
    get_lb_test_labels,
)

__all__ = [
    # Models
    "ControlledTerminology",
    # Registry functions
    "get_controlled_terminology",
    "list_controlled_variables",
    "get_nci_code",
    "get_codelist_code",
    # Dynamic test label loaders
    "get_test_labels",
    "get_vs_test_labels",
    "get_lb_test_labels",
]
