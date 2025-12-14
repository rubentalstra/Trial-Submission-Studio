"""XML generation module for CDISC standards.

This module provides shared utilities and sub-modules for generating
CDISC-compliant XML documents:
- Define-XML 2.1 (define/)
- Dataset-XML 1.0 (dataset/)

Shared utilities include namespace handling, XML tag/attribute helpers,
and common data models.
"""

# Re-export from submodules for convenience
from .define_module import (
    build_define_tree,
    build_study_define_tree,
    write_define_file,
    write_study_define_file,
    DefineGenerationError,
    StudyDataset as DefineStudyDataset,
)

from .dataset_module import (
    build_dataset_xml_tree,
    write_dataset_xml,
    DatasetXMLError,
)

__all__ = [
    # Define-XML
    "build_define_tree",
    "build_study_define_tree",
    "write_define_file",
    "write_study_define_file",
    "DefineGenerationError",
    "DefineStudyDataset",
    # Dataset-XML
    "build_dataset_xml_tree",
    "write_dataset_xml",
    "DatasetXMLError",
]
