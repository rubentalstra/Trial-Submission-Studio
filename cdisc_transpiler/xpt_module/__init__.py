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
    XportGenerationError,
    write_xpt_file,
)

# Future exports as modules are created:
# from .builder import (
#     build_domain_dataframe,
#     DomainFrameBuilder,
# )
# from .transformers import (
#     DateTransformer,
#     CodelistTransformer,
#     NumericTransformer,
#     TextTransformer,
# )

__all__ = [
    # Exceptions
    "XportGenerationError",
    # Writing
    "write_xpt_file",
    # Building (to be added)
    # "build_domain_dataframe",
    # "DomainFrameBuilder",
    # Transformers (to be added)
    # "DateTransformer",
    # "CodelistTransformer",
    # "NumericTransformer",
    # "TextTransformer",
]
