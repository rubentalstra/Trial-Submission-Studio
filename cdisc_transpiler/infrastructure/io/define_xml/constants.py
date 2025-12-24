from xml.etree import ElementTree as ET

from cdisc_transpiler.constants import Constraints, SDTMVersions

ODM_NS = "http://www.cdisc.org/ns/odm/v1.3"
DEF_NS = "http://www.cdisc.org/ns/def/v2.1"
XLINK_NS = "http://www.w3.org/1999/xlink"
XML_NS = "http://www.w3.org/XML/1998/namespace"
ET.register_namespace("", ODM_NS)
ET.register_namespace("def", DEF_NS)
ET.register_namespace("xlink", XLINK_NS)
DEFINE_VERSION = Constraints.DEFINE_XML_VERSION
DEFAULT_SDTM_VERSION = SDTMVersions.DEFAULT_VERSION
DEFAULT_SDTM_MD_VERSION = "1.1"
DEFAULT_CT_PUBLISHING_SET = "SDTM"
DEFAULT_CT_DEFINE_PUBLISHING_SET = "DEFINE-XML"
IG_STANDARD_OID = "STD.1"
MD_STANDARD_OID = "STD.2_1"
CT_STANDARD_OID_SDTM = "STD.3"
CT_STANDARD_OID_DEFINE = "STD.4"
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
CONTEXT_SUBMISSION = SDTMVersions.DEFINE_CONTEXT_SUBMISSION
CONTEXT_OTHER = SDTMVersions.DEFINE_CONTEXT_OTHER
