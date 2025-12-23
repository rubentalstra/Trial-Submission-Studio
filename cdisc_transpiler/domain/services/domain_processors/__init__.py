"""Domain processor registry and factory.

This module provides a registry system for domain-specific processors,
allowing each SDTM domain to have custom post-processing logic.

This is core SDTM business logic that belongs in the domain layer.
"""

from collections.abc import Callable
from typing import TYPE_CHECKING

from .ae import AEProcessor
from .base import BaseDomainProcessor, DefaultDomainProcessor
from .cm import CMProcessor
from .da import DAProcessor

# Import all domain processors
from .dm import DMProcessor
from .ds import DSProcessor
from .ex import EXProcessor
from .ie import IEProcessor
from .lb import LBProcessor
from .mh import MHProcessor
from .pe import PEProcessor
from .pr import PRProcessor
from .qs import QSProcessor
from .se import SEProcessor
from .ta import TAProcessor
from .te import TEProcessor
from .ts import TSProcessor
from .vs import VSProcessor

if TYPE_CHECKING:
    from ...entities.controlled_terminology import ControlledTerminology
    from ...entities.sdtm_domain import SDTMDomain
    from ...entities.study_metadata import StudyMetadata


class DomainProcessorRegistry:
    """Registry for domain-specific processors."""

    def __init__(self):
        super().__init__()
        self._processors: dict[str, type[BaseDomainProcessor]] = {}
        self._default_processor = DefaultDomainProcessor

    def register(self, domain_code: str, processor_class: type[BaseDomainProcessor]):
        """Register a processor for a specific domain."""
        self._processors[domain_code.upper()] = processor_class

    def get_processor(
        self,
        domain: SDTMDomain,
        reference_starts: dict[str, str] | None = None,
        metadata: StudyMetadata | None = None,
        ct_resolver: Callable[[str | None, str | None], ControlledTerminology | None]
        | None = None,
    ) -> BaseDomainProcessor:
        """Get the appropriate processor for a domain."""
        processor_class = self._processors.get(
            domain.code.upper(), self._default_processor
        )
        return processor_class(domain, reference_starts, metadata, ct_resolver)


# Global registry instance
_registry = DomainProcessorRegistry()


# Register all domain processors
_registry.register("DM", DMProcessor)
_registry.register("AE", AEProcessor)
_registry.register("CM", CMProcessor)
_registry.register("DS", DSProcessor)
_registry.register("EX", EXProcessor)
_registry.register("LB", LBProcessor)
_registry.register("VS", VSProcessor)
_registry.register("MH", MHProcessor)
_registry.register("PE", PEProcessor)
_registry.register("QS", QSProcessor)
_registry.register("DA", DAProcessor)
_registry.register("IE", IEProcessor)
_registry.register("PR", PRProcessor)
_registry.register("SE", SEProcessor)
_registry.register("TS", TSProcessor)
_registry.register("TA", TAProcessor)
_registry.register("TE", TEProcessor)


def get_domain_processor(
    domain: SDTMDomain,
    reference_starts: dict[str, str] | None = None,
    metadata: StudyMetadata | None = None,
    ct_resolver: Callable[[str | None, str | None], ControlledTerminology | None]
    | None = None,
) -> BaseDomainProcessor:
    """Get a processor for the specified domain."""
    return _registry.get_processor(domain, reference_starts, metadata, ct_resolver)


__all__ = [
    "BaseDomainProcessor",
    "DefaultDomainProcessor",
    "DomainProcessorRegistry",
    "get_domain_processor",
    # Individual processors
    "AEProcessor",
    "CMProcessor",
    "DAProcessor",
    "DMProcessor",
    "DSProcessor",
    "EXProcessor",
    "IEProcessor",
    "LBProcessor",
    "MHProcessor",
    "PEProcessor",
    "PRProcessor",
    "QSProcessor",
    "SEProcessor",
    "TAProcessor",
    "TEProcessor",
    "TSProcessor",
    "VSProcessor",
]
