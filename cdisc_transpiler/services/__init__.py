"""Service layer for business logic.

This package contains service classes that encapsulate business logic
and coordinate operations across multiple modules. Services are designed
to be reusable, testable, and independent of the CLI layer.

Services:
    - DomainProcessingService: Processes domains from source data
    - FileGenerationService: Generates output files (XPT, XML, SAS)
    - TrialDesignService: Synthesizes trial design domains
"""

from .domain_service import DomainProcessingService, DomainProcessingResult
from .file_generation_service import FileGenerationService, FileGenerationResult
from .trial_design_service import TrialDesignService

__all__ = [
    "DomainProcessingService",
    "DomainProcessingResult",
    "FileGenerationService",
    "FileGenerationResult",
    "TrialDesignService",
]
