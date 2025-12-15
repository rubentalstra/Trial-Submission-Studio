"""Dataset-XML 1.0 generation module.

This module provides functionality for generating Dataset-XML documents
compliant with CDISC Dataset-XML 1.0 specification.

The module is organized into focused components:
- constants: Namespace declarations and version identifiers
- models: Data classes for Dataset-XML configuration
- utils: Helper functions
- builder: Document tree construction
- writer: XML serialization and file I/O
"""

from .models import DatasetXMLError, DatasetXMLConfig
from .builder import build_dataset_xml_tree
from .writer import write_dataset_xml

__all__ = [
    "DatasetXMLError",
    "DatasetXMLConfig",
    "build_dataset_xml_tree",
    "write_dataset_xml",
]
