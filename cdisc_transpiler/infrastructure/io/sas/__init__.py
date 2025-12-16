"""SAS program generation (infrastructure).

This package contains the concrete implementation for generating SAS programs
and writing them to disk.
"""

from .constants import DEFAULT_STUDY_ID, SAS_FILE_ENCODING, SAS_PROGRAM_TEMPLATE
from .generator import SASProgramGenerator, generate_sas_program
from .writer import write_sas_file

__all__ = [
    "DEFAULT_STUDY_ID",
    "SAS_FILE_ENCODING",
    "SAS_PROGRAM_TEMPLATE",
    "SASProgramGenerator",
    "generate_sas_program",
    "write_sas_file",
]
