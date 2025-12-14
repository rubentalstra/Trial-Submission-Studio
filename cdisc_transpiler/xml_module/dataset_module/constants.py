"""Constants for Dataset-XML 1.0 generation.

This module contains namespace declarations, version identifiers, and
constants used in Dataset-XML 1.0 specification compliance.
"""

from xml.etree import ElementTree as ET

# XML Namespaces for Dataset-XML 1.0
ODM_NS = "http://www.cdisc.org/ns/odm/v1.3"
DATA_NS = "http://www.cdisc.org/ns/Dataset-XML/v1.0"
XLINK_NS = "http://www.w3.org/1999/xlink"

# Register namespaces for pretty output
ET.register_namespace("", ODM_NS)
ET.register_namespace("xlink", XLINK_NS)
ET.register_namespace("data", DATA_NS)

# Dataset-XML version
DATASET_XML_VERSION = "1.0.0"

# Define-XML version for PriorFileOID reference
DEFINE_XML_VERSION = "2.1.0"
DEFAULT_SDTM_VERSION = "3.4"

# Shared variables that use IT.{VARIABLE} OID pattern (no domain prefix)
# RDOMAIN follows the shared pattern per CDISC examples (see SUPP-- datasets).
SHARED_VARIABLE_OIDS = {"STUDYID", "USUBJID", "RDOMAIN"}
