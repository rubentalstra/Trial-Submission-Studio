"""Service layer for business logic.

This package contains service classes that encapsulate business logic
and coordinate operations across multiple modules. Services are designed
to be reusable, testable, and independent of the CLI layer.

Services:
    - DomainProcessingService: Processes domains from source data
    - FileGenerationService: Generates output files (XPT, XML, SAS)
    - TrialDesignService: Synthesizes trial design domains
    - DomainDiscoveryService: Discovers and classifies domain files
    - FileOrganizationService: Manages output directory structure
    - ProgressReportingService: Reports progress and status to users
    - StudyOrchestrationService: Orchestrates study processing workflows
"""

from .domain_service import DomainProcessingService, DomainProcessingResult
from .file_generation_service import FileGenerationService, FileGenerationResult
from .trial_design_service import TrialDesignService
from .domain_discovery_service import DomainDiscoveryService
from .file_organization_service import FileOrganizationService
from .progress_reporting_service import ProgressReportingService
from .study_orchestration_service import StudyOrchestrationService

__all__ = [
    "DomainProcessingService",
    "DomainProcessingResult",
    "FileGenerationService",
    "FileGenerationResult",
    "TrialDesignService",
    "DomainDiscoveryService",
    "FileOrganizationService",
    "ProgressReportingService",
    "StudyOrchestrationService",
]
