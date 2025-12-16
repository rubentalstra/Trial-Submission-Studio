"""Domain definition repository adapter.

Provides SDTMDomain definitions via an application-facing port, while keeping
spec-loading mechanics (wrapper modules) in the infrastructure layer.
"""

from __future__ import annotations

from ...application.ports.repositories import DomainDefinitionRepositoryPort
from ..sdtm_spec.registry import get_domain, list_domains


class DomainDefinitionRepository(DomainDefinitionRepositoryPort):
    """Infrastructure adapter for looking up SDTM domain definitions."""

    def list_domains(self) -> list[str]:
        return list(list_domains())

    def get_domain(self, domain_code: str):
        return get_domain(domain_code)
