"""I/O adapters for file operations.

This module provides unified interfaces for reading and writing files.

NOTE: OutputDirs, OutputRequest, OutputResult are application-layer DTOs.
Import them from cdisc_transpiler.application.models instead.
"""

from .csv_reader import CSVReader, CSVReadOptions
from .dataset_xml_writer import DatasetXMLWriter
from .define_xml_generator import DefineXmlGenerator
from .exceptions import (
    DataParseError,
    DataSourceError,
    DataSourceNotFoundError,
    DataValidationError,
)
from .file_generator import FileGenerator
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
    "XPTWriter",
    "DatasetXMLWriter",
    "SASWriter",
    "DefineXmlGenerator",
]
