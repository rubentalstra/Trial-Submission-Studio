"""I/O adapters for file operations.

This module provides unified interfaces for reading and writing files.
"""

from .csv_reader import CSVReader, CSVReadOptions
from .exceptions import (
    DataParseError,
    DataSourceError,
    DataSourceNotFoundError,
    DataValidationError,
)

__all__ = [
    "CSVReader",
    "CSVReadOptions",
    "DataParseError",
    "DataSourceError",
    "DataSourceNotFoundError",
    "DataValidationError",
]
