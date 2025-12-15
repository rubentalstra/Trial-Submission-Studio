"""Domain definition repository adapter.

Provides SDTMDomain definitions via an application-facing port, while keeping
spec-loading mechanics (wrapper modules) in the infrastructure layer.
"""

from __future__ import annotations

from ...application.ports.repositories import DomainDefinitionPort
from ...domains_module import get_domain


class DomainDefinitionRepository(DomainDefinitionPort):
    """Infrastructure adapter for looking up SDTM domain definitions."""

    def get_domain(self, code: str):
        return get_domain(code)
