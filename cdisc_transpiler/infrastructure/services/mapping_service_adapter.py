"""Infrastructure adapter for the mapping port.

Today the mapping engines are pure domain services.
This adapter exists so the application layer depends on a port, not on the
concrete mapping implementation.
"""

from typing import override

import pandas as pd

from ...application.ports.repositories import DomainDefinitionRepositoryPort
from ...application.ports.services import MappingPort
from ...domain.entities.column_hints import Hints
from ...domain.entities.mapping import MappingSuggestions
from ...domain.entities.study_metadata import StudyMetadata
from ...domain.services.mapping.factory import create_mapper


class MappingServiceAdapter(MappingPort):
    def __init__(
        self, *, domain_definition_repository: DomainDefinitionRepositoryPort
    ) -> None:
        super().__init__()
        self._domain_definition_repository = domain_definition_repository

    @override
    def suggest(
        self,
        *,
        domain_code: str,
        frame: pd.DataFrame,
        metadata: StudyMetadata | None = None,
        min_confidence: float = 0.5,
        column_hints: Hints | None = None,
    ) -> MappingSuggestions:
        domain = self._domain_definition_repository.get_domain(domain_code)
        engine = create_mapper(
            domain,
            metadata=metadata,
            min_confidence=min_confidence,
            column_hints=column_hints,
        )
        return engine.suggest(frame)
