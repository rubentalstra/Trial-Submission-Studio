"""Domain entities.

Core domain objects like SDTMDomain, Variable, StudyMetadata, etc.
"""

from .sdtm_domain import SDTMDomain, SDTMVariable
from .variable import (
    extract_variable_order,
    extract_variable_name,
    extract_variable_label,
    extract_codelist_code,
    extract_described_value_domain,
    extract_core_value,
    extract_role,
    extract_source_version,
    determine_length,
    variable_from_row,
)
from .study_metadata import (
    CodeList,
    CodeListValue,
    SourceColumn,
    StudyMetadata,
)
from .mapping import (
    ColumnMapping,
    MappingConfig,
    Suggestion,
    MappingSuggestions,
    merge_mappings,
    build_config,
)

__all__ = [
    # SDTM Domain entities
    "SDTMDomain",
    "SDTMVariable",
    # Variable building functions
    "extract_variable_order",
    "extract_variable_name",
    "extract_variable_label",
    "extract_codelist_code",
    "extract_described_value_domain",
    "extract_core_value",
    "extract_role",
    "extract_source_version",
    "determine_length",
    "variable_from_row",
    # Study Metadata entities
    "CodeList",
    "CodeListValue",
    "SourceColumn",
    "StudyMetadata",
    # Mapping entities
    "ColumnMapping",
    "MappingConfig",
    "Suggestion",
    "MappingSuggestions",
    "merge_mappings",
    "build_config",
]
