"""Define-XML 2.1 generation module.

This module provides a clean, modular architecture for generating Define-XML
documents compliant with CDISC Define-XML 2.1.0 specification.

The module is organized into focused components:
- constants: Namespace declarations, OIDs, and default values
- models: Data classes for Define-XML elements
- standards: Standards configuration and management
- utils: Helper functions

For backward compatibility, the main entry points are re-exported:
- write_define_file: Generate Define-XML from ElementTree
- write_study_define_file: Generate Define-XML from study datasets

Example:
    >>> from cdisc_transpiler.define_xml_module import write_study_define_file
    >>> write_study_define_file(
    ...     study_id="STUDY001",
    ...     datasets=datasets,
    ...     output_path="define.xml"
    ... )
"""

from .models import (
    DefineGenerationError,
    StandardDefinition,
    OriginDefinition,
    MethodDefinition,
    CommentDefinition,
    WhereClauseDefinition,
    ValueListItemDefinition,
    ValueListDefinition,
    StudyDataset,
)

from .standards import (
    get_default_standards,
    get_default_standard_comments,
)

# Import main functions from the original module for backward compatibility
# These will be migrated in subsequent steps
from ..define_xml import (
    write_define_file,
    write_study_define_file,
    build_define_tree,
    build_study_define_tree,
)

__all__ = [
    # Exceptions
    "DefineGenerationError",
    # Models
    "StandardDefinition",
    "OriginDefinition",
    "MethodDefinition",
    "CommentDefinition",
    "WhereClauseDefinition",
    "ValueListItemDefinition",
    "ValueListDefinition",
    "StudyDataset",
    # Standards
    "get_default_standards",
    "get_default_standard_comments",
    # Main API (backward compatibility)
    "write_define_file",
    "write_study_define_file",
    "build_define_tree",
    "build_study_define_tree",
]
