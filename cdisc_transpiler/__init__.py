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
from cdisc_transpiler.dataset_xml import (
    build_dataset_xml_tree,
    write_dataset_xml,
)
from cdisc_transpiler.define_xml_module import (
    build_define_tree,
    build_study_define_tree,
    write_define_file,
)
from cdisc_transpiler.domains import SDTMDomain, SDTMVariable, get_domain

__all__ = [
    "__version__",
    # Define-XML
    "build_define_tree",
    "build_study_define_tree",
    "write_define_file",
    # Dataset-XML
    "build_dataset_xml_tree",
    "write_dataset_xml",
    # Metadata
    "SDTMDomain",
    "SDTMVariable",
    "get_domain",
]
