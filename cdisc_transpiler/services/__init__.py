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

Deprecated Services (moved to legacy package):
    ⚠️ The following services are deprecated and will be removed in the next major version:
    - FileGenerationService → use infrastructure.io.FileGenerator with FileGeneratorPort
    - StudyOrchestrationService → use application.study_processing_use_case.StudyProcessingUseCase
    - DomainProcessingCoordinator → use application.domain_processing_use_case.DomainProcessingUseCase
    - DomainSynthesisCoordinator → use application.study_processing_use_case.StudyProcessingUseCase

    These are re-exported from the legacy package for backward compatibility.
    See docs/ARCHITECTURE.md for migration guidance.
"""

from __future__ import annotations

import warnings
from typing import TYPE_CHECKING

from .domain_discovery_service import DomainDiscoveryService
from .file_organization_service import FileOrganizationService, ensure_acrf_pdf
from .progress_reporting_service import ProgressReportingService
from .trial_design_service import TrialDesignService

if TYPE_CHECKING:
    from ..legacy import (
        DomainProcessingCoordinator,
        DomainSynthesisCoordinator,
        StudyOrchestrationService,
    )

__all__ = [
    "TrialDesignService",
    "DomainDiscoveryService",
    "FileOrganizationService",
    "ensure_acrf_pdf",
    "ProgressReportingService",
    # Deprecated - kept for backward compatibility
    "StudyOrchestrationService",
    "DomainProcessingCoordinator",
    "DomainSynthesisCoordinator",
]


_DEPRECATED_LEGACY_EXPORTS = {
    "StudyOrchestrationService",
    "DomainProcessingCoordinator",
    "DomainSynthesisCoordinator",
}


def __getattr__(name: str):
    if name not in _DEPRECATED_LEGACY_EXPORTS:
        raise AttributeError(name)

    warnings.warn(
        f"cdisc_transpiler.services.{name} is deprecated and will be removed in the next major version. "
        "Please import it from cdisc_transpiler.legacy, or migrate to the corresponding application use case. "
        "See docs/ARCHITECTURE.md for details.",
        DeprecationWarning,
        stacklevel=2,
    )

    from .. import legacy as _legacy

    return getattr(_legacy, name)
