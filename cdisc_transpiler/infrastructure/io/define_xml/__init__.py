"""Define-XML 2.1 generation (infrastructure).

Keep this package's public surface minimal; import implementation details from
their defining modules (no broad re-exports).
"""

from .constants import CONTEXT_OTHER, CONTEXT_SUBMISSION, DEFINE_VERSION
from .models import DefineGenerationError, StudyDataset
from .xml_writer import build_study_define_tree, write_study_define_file

__all__ = [
    "CONTEXT_OTHER",
    "CONTEXT_SUBMISSION",
    "DEFINE_VERSION",
    "DefineGenerationError",
    "StudyDataset",
    "build_study_define_tree",
    "write_study_define_file",
]
