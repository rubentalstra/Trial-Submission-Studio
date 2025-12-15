"""Application layer for CDISC Transpiler.

This layer contains use cases and application-level orchestration logic.
It defines ports (interfaces) for external dependencies.
"""

from .models import (
    DomainProcessingResult,
    ProcessDomainRequest,
    ProcessDomainResponse,
    ProcessStudyRequest,
    ProcessStudyResponse,
)

# Avoid circular import by not importing use case at module level
# Import StudyProcessingUseCase directly when needed:
#   from cdisc_transpiler.application.study_processing_use_case import StudyProcessingUseCase

__all__ = [
    "DomainProcessingResult",
    "ProcessDomainRequest",
    "ProcessDomainResponse",
    "ProcessStudyRequest",
    "ProcessStudyResponse",
]
