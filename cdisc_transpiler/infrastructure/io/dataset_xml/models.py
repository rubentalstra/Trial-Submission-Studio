"""Data models for Dataset-XML 1.0 generation.

This module contains dataclass definitions for Dataset-XML configuration.
"""

from dataclasses import dataclass


class DatasetXMLError(Exception):
    """Error during Dataset-XML generation."""


@dataclass
class DatasetXMLConfig:
    """Configuration for Dataset-XML generation.

    Attributes:
        study_oid: Study OID identifier
        metadata_version_oid: MetaDataVersion OID reference
        originator: Organization that created the document
        source_system: System that created the document
        source_system_version: Version of source system
        file_type: ODM FileType (Snapshot, Transactional)
        granularity: Data granularity level
    """

    study_oid: str
    metadata_version_oid: str
    originator: str = "CDISC-Transpiler"
    source_system: str = "CDISC-Transpiler"
    source_system_version: str = "1.0"
    file_type: str = "Snapshot"
    granularity: str = "All"  # "All", "Metadata", "AdminData", "ReferenceData", "AllClinicalData", "SingleSite"
