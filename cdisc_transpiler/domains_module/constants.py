"""Constants for SDTM domain metadata.

SDTM Reference:
    SDTMIG v3.4 and SDTM v2.0 define the structure and content of
    Study Data Tabulation Model datasets.
"""

from __future__ import annotations

# Controlled Terminology version used for metadata stamping.
# The SDTM-MSG v2.0 example relies on the 2024-03-29 package. When this
# exact folder is not present locally, the terminology loader falls back
# to the most recent available package.
CT_VERSION = "2024-03-29"

# Default lengths when the source metadata does not provide them.
# Per SDTMIG v3.4, character variables default to 200 characters max.
DEFAULT_CHAR_LENGTH = 200
DEFAULT_NUM_LENGTH = 8

# General Observation Classes (SDTM v2.0 Section 3.2)
# These are the three base classes for observation data in SDTM.
GENERAL_OBSERVATION_CLASSES = {"INTERVENTIONS", "EVENTS", "FINDINGS"}

# Aliases for class names that map to General Observation Classes
# "FINDINGS ABOUT" domains (FA) are treated as FINDINGS class
GENERAL_CLASS_ALIASES = {"FINDINGS ABOUT": "FINDINGS"}

# Variables that should always be propagated across domains in a class.
# These are Identifier and Timing variables shared by all domains in a class.
ALWAYS_PROPAGATE_GENERAL = {
    "STUDYID",  # Study Identifier
    "DOMAIN",  # Domain Abbreviation
    "USUBJID",  # Unique Subject Identifier
    "EPOCH",  # Epoch (study period)
    "VISIT",  # Visit Name
    "VISITNUM",  # Visit Number
    "VISITDY",  # Planned Study Day of Visit
    "SPDEVID",  # Sponsor Device Identifier
}
