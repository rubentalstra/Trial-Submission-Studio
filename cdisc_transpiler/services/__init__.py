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

Active Services:
    - FileGenerationService: Generates output files (XPT, XML, SAS)
    - TrialDesignService: Synthesizes trial design domains (TS, TA, TE, etc.)
    - DomainDiscoveryService: Discovers and classifies domain files
    - FileOrganizationService: Manages output directory structure
    - ProgressReportingService: Reports progress and status to users

Deprecated Services (moved to legacy package):
    ⚠️ The following services are deprecated and will be removed in the next major version:
    - StudyOrchestrationService → use application.study_processing_use_case.StudyProcessingUseCase
    - DomainProcessingCoordinator → use application.domain_processing_use_case.DomainProcessingUseCase
    - DomainSynthesisCoordinator → use application.study_processing_use_case.StudyProcessingUseCase
    
    These are re-exported from the legacy package for backward compatibility.
    See MIGRATION.md for migration guidance.
"""

from .file_generation_service import FileGenerationService, FileGenerationResult
from .trial_design_service import TrialDesignService
from .domain_discovery_service import DomainDiscoveryService
from .file_organization_service import FileOrganizationService
from .progress_reporting_service import ProgressReportingService

# Import deprecated services from legacy package (with deprecation warnings)
from ..legacy import (
    StudyOrchestrationService,
    DomainProcessingCoordinator,
    DomainSynthesisCoordinator,
)

__all__ = [
    "FileGenerationService",
    "FileGenerationResult",
    "TrialDesignService",
    "DomainDiscoveryService",
    "FileOrganizationService",
    "ProgressReportingService",
    # Deprecated - kept for backward compatibility
    "StudyOrchestrationService",
    "DomainProcessingCoordinator",
    "DomainSynthesisCoordinator",
]
