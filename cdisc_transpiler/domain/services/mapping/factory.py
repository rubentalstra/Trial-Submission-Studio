"""Factory helpers for mapping engines.

These helpers live in the domain layer because mapping is pure business logic.
Compatibility wrappers re-export these functions from `cdisc_transpiler.mapping_module`.
"""

from __future__ import annotations

from ...entities.column_hints import Hints
from ...entities.study_metadata import StudyMetadata
from .engine import MappingEngine
from .metadata_mapper import MetadataAwareMapper


def create_mapper(
    domain_code: str,
    metadata: StudyMetadata | None = None,
    *,
    min_confidence: float = 0.5,
    column_hints: Hints | None = None,
) -> MetadataAwareMapper | MappingEngine:
    """Create the most appropriate mapper for the available context.

    If metadata is present (Items/CodeLists), prefer `MetadataAwareMapper`.
    Otherwise fall back to `MappingEngine`.
    """
    if metadata is not None and (metadata.items or metadata.codelists):
        return MetadataAwareMapper(
            domain_code,
            metadata,
            min_confidence=min_confidence,
            column_hints=column_hints,
        )

    return MappingEngine(
        domain_code,
        min_confidence=min_confidence,
        column_hints=column_hints,
    )
