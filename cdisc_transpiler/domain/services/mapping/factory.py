"""Factory helpers for mapping engines.

These helpers live in the domain layer because mapping is pure business logic.
"""

from ...entities.column_hints import Hints
from ...entities.sdtm_domain import SDTMDomain
from ...entities.study_metadata import StudyMetadata
from .engine import MappingEngine
from .metadata_mapper import MetadataAwareMapper


def create_mapper(
    domain: SDTMDomain,
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
            domain,
            metadata,
            min_confidence=min_confidence,
            column_hints=column_hints,
        )

    return MappingEngine(
        domain,
        min_confidence=min_confidence,
        column_hints=column_hints,
    )
