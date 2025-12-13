"""SDTM domain metadata module.

This module provides access to SDTM domain and variable definitions loaded
from SDTMIG v3.4 and SDTM v2.0 CSV metadata files.

The module is organized into focused components:
- constants: Configuration values and constants
- models: Data classes for domains and variables
- utils: Normalization and helper functions
- loaders: CSV file loading
- variable_builder: Variable construction from CSV
- domain_builder: Domain construction from CSV
- general_classes: General Observation Class logic
- registry: Domain registration and lookup
"""

from __future__ import annotations

from .constants import CT_VERSION
from .models import SDTMDomain, SDTMVariable
from .registry import generalized_identifiers, get_domain, list_domains

__all__ = [
    "CT_VERSION",
    "SDTMDomain",
    "SDTMVariable",
    "get_domain",
    "list_domains",
    "generalized_identifiers",
]
