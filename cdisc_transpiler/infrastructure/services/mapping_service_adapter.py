"""Infrastructure adapter for the mapping port.

Today the mapping engines are pure domain services.
This adapter exists so the application layer depends on a port, not on the
concrete mapping implementation.
"""

from __future__ import annotations

import pandas as pd

from ...application.ports.services import MappingPort
from ...domain.entities.column_hints import Hints
from ...domain.entities.mapping import MappingSuggestions
from ...domain.entities.study_metadata import StudyMetadata
from ...domain.services.mapping import create_mapper


class MappingServiceAdapter(MappingPort):
    def suggest(
        self,
        *,
        domain_code: str,
        frame: pd.DataFrame,
        metadata: StudyMetadata | None = None,
        min_confidence: float = 0.5,
        column_hints: Hints | None = None,
    ) -> MappingSuggestions:
        engine = create_mapper(
            domain_code,
            metadata=metadata,
            min_confidence=min_confidence,
            column_hints=column_hints,
        )
        return engine.suggest(frame)
