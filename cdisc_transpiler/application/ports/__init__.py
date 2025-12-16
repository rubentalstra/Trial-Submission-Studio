"""Port interfaces for external dependencies.

This module defines abstract interfaces (protocols) that external
adapters must implement. This enables dependency injection and testing.
"""

from .repositories import (
    CTRepositoryPort,
    DomainDefinitionPort,
    SDTMSpecRepositoryPort,
    StudyDataRepositoryPort,
)
from .services import (
    DatasetXMLWriterPort,
    DefineXmlGeneratorPort,
    FileGeneratorPort,
    LoggerPort,
    MappingPort,
    OutputPreparationPort,
    SASWriterPort,
    XPTWriterPort,
)

__all__ = [
    # Repository Ports
    "CTRepositoryPort",
    "DomainDefinitionPort",
    "SDTMSpecRepositoryPort",
    "StudyDataRepositoryPort",
    # Service Ports
    "LoggerPort",
    "FileGeneratorPort",
    "MappingPort",
    "OutputPreparationPort",
    # Writer Ports
    "XPTWriterPort",
    "DatasetXMLWriterPort",
    "SASWriterPort",
    # Generator Ports
    "DefineXmlGeneratorPort",
]
