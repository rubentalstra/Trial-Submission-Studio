"""Constants for SDTM domain metadata."""

from __future__ import annotations

# Controlled Terminology version used for metadata stamping.
# The SDTM-MSG v2.0 example relies on the 2024-03-29 package. When this
# exact folder is not present locally, the terminology loader falls back
# to the most recent available package.
CT_VERSION = "2024-03-29"

# Default lengths when the source metadata does not provide them.
DEFAULT_CHAR_LENGTH = 200
DEFAULT_NUM_LENGTH = 8

# General Observation Class constants (Interventions, Events, Findings)
GENERAL_OBSERVATION_CLASSES = {"INTERVENTIONS", "EVENTS", "FINDINGS"}
GENERAL_CLASS_ALIASES = {"FINDINGS ABOUT": "FINDINGS"}

# Variables that should always be propagated across domains in a class
ALWAYS_PROPAGATE_GENERAL = {
    "STUDYID",
    "DOMAIN",
    "USUBJID",
    "EPOCH",
    "VISIT",
    "VISITNUM",
    "VISITDY",
    "SPDEVID",
}
