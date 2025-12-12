"""Modular XPT (SAS Transport) file generation.

This package provides a clean, modular architecture for building and writing
SAS transport (XPORT) files that comply with CDISC SDTM standards.

The package is organized into focused modules:
- writer: XPT file writing with pyreadstat
- builder: DataFrame construction and orchestration
- transformers: Data transformations (date, codelist, numeric, text)
- validators: XPT-specific validation rules

Example:
    >>> from cdisc_transpiler.xpt_module import build_domain_dataframe, write_xpt_file
    >>> df = build_domain_dataframe(source_frame, config)
    >>> write_xpt_file(df, "DM", "output/xpt/dm.xpt")
"""

from .writer import (
    write_xpt_file,
)
from .builder import (
    XportGenerationError,
    build_domain_dataframe,
    DomainFrameBuilder,
)
from .transformers import (
    DateTransformer,
    CodelistTransformer,
    NumericTransformer,
    TextTransformer,
)
from .validators import (
    XPTValidator,
)

__all__ = [
    # Exceptions
    "XportGenerationError",
    # Writing
    "write_xpt_file",
    # Building
    "build_domain_dataframe",
    "DomainFrameBuilder",
    # Transformers
    "DateTransformer",       # Step 3 ✓
    "CodelistTransformer",   # Step 4 ✓
    "NumericTransformer",    # Step 5 ✓
    "TextTransformer",       # Step 5 ✓
    # Validators
    "XPTValidator",          # Step 6 ✓
]
