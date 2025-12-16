"""Constants for Define-XML 2.1 generation.

This module contains all namespace declarations, OID constants, and default
values used in Define-XML 2.1.0 specification compliance.
"""

from __future__ import annotations

from pathlib import Path

from cdisc_transpiler.constants import Constraints, SDTMVersions

from xml.etree import ElementTree as ET

# Define-XML 2.1 namespace declarations per specification
ODM_NS = "http://www.cdisc.org/ns/odm/v1.3"
DEF_NS = "http://www.cdisc.org/ns/def/v2.1"
XLINK_NS = "http://www.w3.org/1999/xlink"
XML_NS = "http://www.w3.org/XML/1998/namespace"

# Register namespaces for proper prefix handling
ET.register_namespace("", ODM_NS)
ET.register_namespace("def", DEF_NS)
ET.register_namespace("xlink", XLINK_NS)

# Define-XML 2.1 version identifier
DEFINE_VERSION = Constraints.DEFINE_XML_VERSION

# Default SDTM standards aligned to SDTM-MSG v2.0 sample package
DEFAULT_SDTM_VERSION = SDTMVersions.DEFAULT_VERSION
DEFAULT_SDTM_MD_VERSION = "1.1"
DEFAULT_CT_PUBLISHING_SET = "SDTM"
DEFAULT_CT_DEFINE_PUBLISHING_SET = "DEFINE-XML"

# Standard OIDs
IG_STANDARD_OID = "STD.1"
MD_STANDARD_OID = "STD.2_1"
CT_STANDARD_OID_SDTM = "STD.3"
CT_STANDARD_OID_DEFINE = "STD.4"

# Default supporting document references
ACRF_LEAF_ID = "LF.acrf"
ACRF_HREF = "acrf.pdf"
ACRF_TITLE = "Annotated CRF"
DEFAULT_CRF_PAGE_REFS = "1"
CSDRG_LEAF_ID = "LF.csdrg"
CSDRG_HREF = "csdrg.pdf"
CSDRG_TITLE = "Reviewers Guide"
DEFAULT_MEDDRA_VERSION = "26.1"
MEDDRA_HREF = "https://www.meddra.org/"
MEDDRA_CODELIST_NAME = "MedDRA Dictionary"

# Context values per Define-XML 2.1 spec
CONTEXT_SUBMISSION = SDTMVersions.DEFINE_CONTEXT_SUBMISSION
CONTEXT_OTHER = SDTMVersions.DEFINE_CONTEXT_OTHER


def tag(namespace: str, name: str) -> str:
    """Create a namespaced XML tag string."""
    return f"{{{namespace}}}{name}"


def attr(namespace: str, name: str) -> str:
    """Create a namespaced XML attribute string."""
    return f"{{{namespace}}}{name}"


def safe_href(href: str) -> str:
    """Sanitize dataset href to comply with SAS naming constraints."""
    if not href:
        return href
    path = Path(href)
    stem = path.stem[:8]
    new_name = f"{stem}{path.suffix}".lower()
    safe = str(path.with_name(new_name))
    return safe[:64]
