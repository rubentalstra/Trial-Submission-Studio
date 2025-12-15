"""Modular column mapping engine and configuration.

This package provides a clean, modular architecture for mapping source data
columns to SDTM target variables with intelligent pattern recognition and
metadata-aware suggestions.

The package is organized into focused modules:
- models: Data models for mappings and configurations - NOW IN domain.entities
- config_io: Configuration file I/O operations
- constants: SDTM inference patterns
- engine: Basic mapping engine with fuzzy matching
- metadata_mapper: Advanced metadata-aware mapper
- utils: Helper functions

Example:
    Basic usage with MappingEngine:
    >>> from cdisc_transpiler.mapping_module import MappingEngine
    >>> engine = MappingEngine("DM", min_confidence=0.7)
    >>> suggestions = engine.suggest(source_df)

    Using the factory function:
    >>> from cdisc_transpiler.mapping_module import create_mapper
    >>> mapper = create_mapper("DM", metadata=study_metadata)
    >>> suggestions = mapper.suggest(source_df)

    Configuration management:
    >>> from cdisc_transpiler.mapping_module import load_config, save_config
    >>> config = load_config("mappings/dm.json")
    >>> save_config(config, "mappings/dm_updated.json")
"""

from ..io_module.models import Hints

# Import from new location and re-export for backward compatibility
from ..domain.entities.study_metadata import StudyMetadata
from ..domain.entities.mapping import (
    ColumnMapping,
    MappingConfig,
    Suggestion,
    MappingSuggestions,
    build_config,
    merge_mappings,
)
from .config_io import (
    load_config,
    save_config,
)
from .engine import (
    MappingEngine,
)
from .metadata_mapper import (
    MetadataAwareMapper,
)
from .utils import (
    normalize_text,
    safe_column_name,
    unquote_column_name,
)


# Factory function for creating appropriate mapper
def create_mapper(
    domain_code: str,
    metadata: StudyMetadata | None = None,
    *,
    min_confidence: float = 0.5,
    column_hints: Hints | None = None,
) -> MetadataAwareMapper | MappingEngine:
    """Factory function to create the appropriate mapper.

    If metadata is provided, returns a MetadataAwareMapper that uses
    Items.csv and CodeLists.csv for intelligent mapping.
    Otherwise, returns the standard MappingEngine.

    Args:
        domain_code: Target SDTM domain code
        metadata: Optional StudyMetadata
        min_confidence: Minimum confidence threshold
        column_hints: Optional column hints

    Returns:
        Appropriate mapper instance

    Example:
        >>> # Without metadata
        >>> mapper = create_mapper("DM")
        >>>
        >>> # With metadata
        >>> from cdisc_transpiler.metadata_module import load_study_metadata
        >>> metadata = load_study_metadata("study_folder")
        >>> mapper = create_mapper("DM", metadata=metadata)
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


__all__ = [
    # Models
    "ColumnMapping",
    "MappingConfig",
    "Suggestion",
    "MappingSuggestions",
    # Model utilities
    "build_config",
    "merge_mappings",
    # Configuration I/O
    "load_config",
    "save_config",
    # Engines
    "MappingEngine",
    "MetadataAwareMapper",
    # Factory
    "create_mapper",
    # Utilities
    "normalize_text",
    "safe_column_name",
    "unquote_column_name",
]
