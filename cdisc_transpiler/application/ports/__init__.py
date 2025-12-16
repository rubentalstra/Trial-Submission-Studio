"""Port interfaces for external dependencies.

This module defines abstract interfaces (protocols) that external
adapters must implement. This enables dependency injection and testing.
"""

from .repositories import (
    CTRepositoryPort,
    DomainDefinitionRepositoryPort,
    SDTMSpecRepositoryPort,
    StudyDataRepositoryPort,
)
from .services import (
    DatasetXMLWriterPort,
    DomainFrameBuilderPort,
    DomainDiscoveryPort,
    DefineXmlGeneratorPort,
    FileGeneratorPort,
    LoggerPort,
    MappingPort,
    OutputPreparationPort,
    SuppqualPort,
    SASWriterPort,
    TerminologyPort,
    XPTWriterPort,
)

__all__ = [
    # Repository Ports
    "CTRepositoryPort",
    "DomainDefinitionRepositoryPort",
    "SDTMSpecRepositoryPort",
    "StudyDataRepositoryPort",
    # Service Ports
    "LoggerPort",
    "FileGeneratorPort",
    "DomainDiscoveryPort",
    "DomainFrameBuilderPort",
    "MappingPort",
    "OutputPreparationPort",
    "SuppqualPort",
    "TerminologyPort",
    # Writer Ports
    "XPTWriterPort",
    "DatasetXMLWriterPort",
    "SASWriterPort",
    # Generator Ports
    "DefineXmlGeneratorPort",
]
