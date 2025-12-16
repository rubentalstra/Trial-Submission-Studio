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
    - TrialDesignService: Synthesizes trial design domains (TS, TA, TE, etc.)
    - DomainDiscoveryService: Discovers and classifies domain files
    - FileOrganizationService: Manages output directory structure
    - ProgressReportingService: Reports progress and status to users

Legacy coordinators:
    The legacy coordinator APIs have been removed.
    Migrate to the corresponding application use cases:
    - application.study_processing_use_case.StudyProcessingUseCase
    - application.domain_processing_use_case.DomainProcessingUseCase
"""

from __future__ import annotations

from .domain_discovery_service import DomainDiscoveryService
from .file_organization_service import FileOrganizationService, ensure_acrf_pdf
from .progress_reporting_service import ProgressReportingService
from .trial_design_service import TrialDesignService

__all__ = [
    "TrialDesignService",
    "DomainDiscoveryService",
    "FileOrganizationService",
    "ensure_acrf_pdf",
    "ProgressReportingService",
]
