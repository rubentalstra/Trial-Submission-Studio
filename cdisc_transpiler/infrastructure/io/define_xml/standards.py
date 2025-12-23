"""Standards configuration and management for Define-XML.

This module handles the creation and management of CDISC standards definitions
used in Define-XML documents.
"""

from cdisc_transpiler.infrastructure.sdtm_spec.constants import CT_VERSION

from .constants import (
    CT_STANDARD_OID_DEFINE,
    CT_STANDARD_OID_SDTM,
    DEFAULT_CT_DEFINE_PUBLISHING_SET,
    DEFAULT_CT_PUBLISHING_SET,
    DEFAULT_SDTM_MD_VERSION,
    DEFAULT_SDTM_VERSION,
    IG_STANDARD_OID,
    MD_STANDARD_OID,
)
from .models import CommentDefinition, StandardDefinition


def get_default_standard_comments() -> list[CommentDefinition]:
    """Return default comments used by the MSG sample package standards.

    Returns:
        List of CommentDefinition objects for standard comments
    """
    return [
        CommentDefinition(
            oid="COM.ST1",
            text=(
                "Study Data Tabulation Model Implementation Guide: "
                "Human Clinical Trials Version 3.4"
            ),
        ),
        CommentDefinition(
            oid="COM.ST2",
            text="Study Data Tabulation Model Implementation Guide for Medical Devices Version 1.0",
        ),
        CommentDefinition(
            oid="COM.ST3",
            text=(
                "This was the latest release of CDISC CT available when this sample "
                "submission was completed."
            ),
        ),
        CommentDefinition(
            oid="COM.ST4",
            text=(
                "This was the CDISC CT Package associated to the CDISC Define-XML "
                "Specification Version 2.1 when this sample submission was completed."
            ),
        ),
    ]


def get_default_standards(
    sdtm_version: str = DEFAULT_SDTM_VERSION,
    ct_version: str | None = None,
    *,
    md_version: str = DEFAULT_SDTM_MD_VERSION,
) -> list[StandardDefinition]:
    """Return the default standard definitions for SDTM submissions.

    Args:
        sdtm_version: SDTMIG version (default: DEFAULT_SDTM_VERSION)
        ct_version: Controlled Terminology version (e.g., "2024-03-29")
        md_version: SDTM-MD (Medical Devices) version (default: "1.1")

    Returns:
        List of StandardDefinition objects for the study
    """
    if ct_version is None:
        ct_version = CT_VERSION

    return [
        StandardDefinition(
            oid=IG_STANDARD_OID,
            name="SDTMIG",
            type="IG",
            version=sdtm_version,
            status="Final",
            comment_oid="COM.ST1",
        ),
        StandardDefinition(
            oid=MD_STANDARD_OID,
            name="SDTMIG-MD",
            type="IG",
            version=md_version,
            status="Final",
            comment_oid="COM.ST2",
        ),
        StandardDefinition(
            oid=CT_STANDARD_OID_SDTM,
            name="CDISC/NCI",
            type="CT",
            version=ct_version,
            status="Final",
            publishing_set=DEFAULT_CT_PUBLISHING_SET,
            comment_oid="COM.ST3",
        ),
        StandardDefinition(
            oid=CT_STANDARD_OID_DEFINE,
            name="CDISC/NCI",
            type="CT",
            version=ct_version,
            status="Final",
            publishing_set=DEFAULT_CT_DEFINE_PUBLISHING_SET,
            comment_oid="COM.ST4",
        ),
    ]
