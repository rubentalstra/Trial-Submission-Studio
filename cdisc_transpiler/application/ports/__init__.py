"""Port interfaces for external dependencies.

This module defines abstract interfaces (protocols) that external
adapters must implement. This enables dependency injection and testing.
"""

from .repositories import (
    CTRepositoryPort,
    SDTMSpecRepositoryPort,
    StudyDataRepositoryPort,
)
from .services import FileGeneratorPort, LoggerPort

__all__ = [
    # Repository Ports
    "CTRepositoryPort",
    "SDTMSpecRepositoryPort",
    "StudyDataRepositoryPort",
    # Service Ports
    "LoggerPort",
    "FileGeneratorPort",
]
