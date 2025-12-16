"""Infrastructure I/O layer.

This package contains adapters and concrete implementations for reading and
writing files (CSV, XPT, Dataset-XML, Define-XML, SAS).

Architecture note:
- Avoid re-exporting symbols from here; import from the defining modules.
- Application DTOs live in cdisc_transpiler.application.models.
"""

# Lightweight re-exports for convenience and backwards compatibility.
# Internal modules should still import from defining modules to avoid cycles.

from .csv_reader import CSVReader, CSVReadOptions
from .dataset_xml_writer import DatasetXMLWriter
from .define_xml_generator import DefineXMLGenerator
from .exceptions import (
    DataParseError,
    DataSourceError,
    DataSourceNotFoundError,
    DataValidationError,
)
from .file_generator import FileGenerator
from .output_preparer import OutputPreparer
from .sas_writer import SASWriter
from .xpt_writer import XPTWriter

__all__ = [
    "CSVReader",
    "CSVReadOptions",
    "DataParseError",
    "DataSourceError",
    "DataSourceNotFoundError",
    "DataValidationError",
    "FileGenerator",
    "OutputPreparer",
    "XPTWriter",
    "DatasetXMLWriter",
    "SASWriter",
    "DefineXMLGenerator",
]
