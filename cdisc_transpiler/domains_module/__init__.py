"""SDTM Domain Metadata Module.

This module provides access to SDTM domain and variable definitions loaded
from SDTMIG v3.4 and SDTM v2.0 CSV metadata files.

SDTM Reference:
    Study Data Tabulation Model (SDTM) defines a standard structure for
    organizing clinical trial data. Key concepts include:
    
    - Domains: Collections of observations with topic-specific commonality
      (e.g., DM=Demographics, AE=Adverse Events, LB=Laboratory Results)
    
    - General Observation Classes: Three base classes for observation data
      * Interventions: Treatments administered (EX, CM, EC, SU, PR, AG)
      * Events: Occurrences/incidents (AE, DS, MH, DV, CE, HO)
      * Findings: Measurements/assessments (LB, VS, EG, PE, QS, SC, FA)
    
    - Variable Roles: Each variable has a role defining its purpose
      * Identifier: Subject/record identification (STUDYID, USUBJID, --SEQ)
      * Topic: What the observation is about (--TRT, --TERM, --TESTCD)
      * Timing: When the observation occurred (--DTC, --STDTC, VISIT)
      * Qualifier: Additional context/attributes (--ORRES, --CAT, --LOC)

The module is organized into focused components:
- constants: SDTM configuration values (CT_VERSION, domain classes)
- models: Data classes (SDTMDomain, SDTMVariable)
- utils: Normalization and helper functions
- loaders: CSV file loading for SDTMIG metadata
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
