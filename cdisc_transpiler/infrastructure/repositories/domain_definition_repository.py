"""Domain definition repository adapter.

Provides SDTMDomain definitions via an application-facing port, while keeping
spec-loading mechanics (wrapper modules) in the infrastructure layer.
"""

from typing import TYPE_CHECKING, override

from ...application.ports.repositories import DomainDefinitionRepositoryPort
from ..sdtm_spec.registry import get_domain, list_domains

if TYPE_CHECKING:
    from ...domain.entities.sdtm_domain import SDTMDomain


class DomainDefinitionRepository(DomainDefinitionRepositoryPort):
    """Infrastructure adapter for looking up SDTM domain definitions."""

    @override
    def list_domains(self) -> list[str]:
        return list(list_domains())

    @override
    def get_domain(self, domain_code: str) -> SDTMDomain:
        return get_domain(domain_code)
