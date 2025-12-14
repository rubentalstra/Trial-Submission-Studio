"""Service layer for SDTM data processing and generation.

This package contains service classes that encapsulate business logic
and coordinate operations across multiple modules. Services are designed
to be reusable, testable, and independent of the CLI layer.

SDTM Reference:
    Services implement the processing workflow for SDTM-compliant data:
    - Source data loading and validation
    - Variable mapping to SDTM domains
    - Controlled terminology application
    - Output file generation (XPT, Dataset-XML, Define-XML)

Services:
    - FileGenerationService: Generates output files (XPT, XML, SAS)
    - TrialDesignService: Synthesizes trial design domains (TS, TA, TE, etc.)
    - DomainDiscoveryService: Discovers and classifies domain files
    - FileOrganizationService: Manages output directory structure
    - ProgressReportingService: Reports progress and status to users
    - StudyOrchestrationService: Orchestrates study processing workflows
    - DomainProcessingCoordinator: Coordinates domain file processing workflow
    - DomainSynthesisCoordinator: Coordinates domain synthesis with file generation
"""

from .file_generation_service import FileGenerationService, FileGenerationResult
from .trial_design_service import TrialDesignService
from .domain_discovery_service import DomainDiscoveryService
from .file_organization_service import FileOrganizationService
from .progress_reporting_service import ProgressReportingService
from .study_orchestration_service import StudyOrchestrationService
from .domain_processing_coordinator import DomainProcessingCoordinator
from .domain_synthesis_coordinator import DomainSynthesisCoordinator

__all__ = [
    "FileGenerationService",
    "FileGenerationResult",
    "TrialDesignService",
    "DomainDiscoveryService",
    "FileOrganizationService",
    "ProgressReportingService",
    "StudyOrchestrationService",
    "DomainProcessingCoordinator",
    "DomainSynthesisCoordinator",
]
