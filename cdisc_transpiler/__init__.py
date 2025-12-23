"""CDISC Transpiler package.

This package provides tools for transforming clinical trial data into
CDISC-compliant formats, specifically SDTM (Study Data Tabulation Model).

Features:
- Define-XML 2.1 generation
- Dataset-XML 1.0 generation
- XPT (SAS Transport) file generation
- SDTM domain metadata management
- Controlled terminology handling
"""

from importlib.metadata import PackageNotFoundError, version

try:  # pragma: no cover
    __version__ = version("cdisc-transpiler")
except PackageNotFoundError:  # pragma: no cover
    __version__ = "0.0.0"

__all__ = ["__version__"]
