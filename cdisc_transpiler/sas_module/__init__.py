"""Modular SAS program generation.

This package provides a clean, modular architecture for generating
SAS programs that transform raw data into CDISC SDTM-compliant datasets.

The package is organized into focused modules:
- constants: SAS templates and configuration
- generator: SAS program generation logic
- writer: File writing operations
- utils: Helper functions for assignments and defaults

Example:
    >>> from cdisc_transpiler.sas_module import generate_sas_program, write_sas_file
    >>> program = generate_sas_program("DM", config, "rawdata", "dm")
    >>> write_sas_file(program, "output/sas/dm.sas")
"""

from .constants import (
    DEFAULT_STUDY_ID,
    SAS_FILE_ENCODING,
    SAS_PROGRAM_TEMPLATE,
)
from .generator import (
    SASProgramGenerator,
    generate_sas_program,
)
from .writer import (
    write_sas_file,
)
from .utils import (
    get_assignment_for_mapping,
    get_default_assignments,
    get_default_value_assignment,
    get_keep_clause,
)

__all__ = [
    # Constants
    "DEFAULT_STUDY_ID",
    "SAS_FILE_ENCODING",
    "SAS_PROGRAM_TEMPLATE",
    # Generator
    "SASProgramGenerator",
    "generate_sas_program",
    # Writer
    "write_sas_file",
    # Utils
    "get_assignment_for_mapping",
    "get_default_assignments",
    "get_default_value_assignment",
    "get_keep_clause",
]
