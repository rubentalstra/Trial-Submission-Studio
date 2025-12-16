"""Constants for SDTM domain metadata.

These are used by the infrastructure spec loader/registry.
"""

from __future__ import annotations

# Controlled Terminology version used for metadata stamping.
# The SDTM-MSG v2.0 example relies on the 2024-03-29 package. When this
# exact folder is not present locally, the terminology loader falls back
# to the most recent available package.
CT_VERSION = "2024-03-29"

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
