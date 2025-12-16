"""CDISC Transpiler package.

This package provides tools for transforming clinical trial data into
CDISC-compliant formats, specifically SDTM (Study Data Tabulation Model).

Features:
- Define-XML 2.1 generation
- Dataset-XML 1.0 generation
- XPT (SAS Transport) file generation
- SDTM domain metadata management
- Controlled terminology handling
"""

from importlib.metadata import PackageNotFoundError, version

try:  # pragma: no cover
    __version__ = version("cdisc-transpiler")
except PackageNotFoundError:  # pragma: no cover
    __version__ = "0.0.0"

# Core exports
from cdisc_transpiler.infrastructure.io.dataset_xml.builder import (
    build_dataset_xml_tree,
)
from cdisc_transpiler.infrastructure.io.dataset_xml.writer import write_dataset_xml
from cdisc_transpiler.infrastructure.io.define_xml.metadata_builder import (
    build_study_define_tree,
)
from cdisc_transpiler.domains_module import SDTMDomain, SDTMVariable, get_domain

__all__ = [
    "__version__",
    # Define-XML
    "build_study_define_tree",
    # Dataset-XML
    "build_dataset_xml_tree",
    "write_dataset_xml",
    # Metadata
    "SDTMDomain",
    "SDTMVariable",
    "get_domain",
]
