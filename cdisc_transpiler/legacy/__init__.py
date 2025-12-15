"""Legacy service layer implementations.

⚠️ DEPRECATED: The modules in this package are deprecated and will be removed
in the next major version. They have been replaced by the new architecture:

- DomainProcessingCoordinator → application.domain_processing_use_case
- DomainSynthesisCoordinator → application.study_processing_use_case
- StudyOrchestrationService → application.study_processing_use_case

These modules are kept for one release cycle to ensure backward compatibility
during the migration period. Please update your code to use the new architecture.

For migration guidance, see docs/ARCHITECTURE.md.
"""

import warnings
from typing import Any


def _deprecated_import_warning(old_module: str, new_module: str) -> None:
    """Issue a deprecation warning for legacy imports."""
    warnings.warn(
        f"{old_module} is deprecated and will be removed in the next major version. "
        f"Please migrate to {new_module}. See docs/ARCHITECTURE.md for details.",
        DeprecationWarning,
        stacklevel=3,
    )


# Re-export legacy modules with deprecation warnings
def __getattr__(name: str) -> Any:
    """Lazy import with deprecation warning."""
    if name == "DomainProcessingCoordinator":
        _deprecated_import_warning(
            "cdisc_transpiler.legacy.DomainProcessingCoordinator",
            "cdisc_transpiler.application.domain_processing_use_case.DomainProcessingUseCase"
        )
        from .domain_processing_coordinator import DomainProcessingCoordinator
        return DomainProcessingCoordinator
    
    elif name == "DomainSynthesisCoordinator":
        _deprecated_import_warning(
            "cdisc_transpiler.legacy.DomainSynthesisCoordinator",
            "cdisc_transpiler.application.study_processing_use_case.StudyProcessingUseCase"
        )
        from .domain_synthesis_coordinator import DomainSynthesisCoordinator
        return DomainSynthesisCoordinator
    
    elif name == "StudyOrchestrationService":
        _deprecated_import_warning(
            "cdisc_transpiler.legacy.StudyOrchestrationService",
            "cdisc_transpiler.application.study_processing_use_case.StudyProcessingUseCase"
        )
        from .study_orchestration_service import StudyOrchestrationService
        return StudyOrchestrationService
    
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


__all__ = [
    "DomainProcessingCoordinator",
    "DomainSynthesisCoordinator",
    "StudyOrchestrationService",
]
