"""Submission utilities for SDTM data packages.

This module provides utilities for building submission-ready
SDTM data packages, including SUPPQUAL domains.

The module is organized into:
- suppqual: SUPPQUAL (Supplemental Qualifiers) building

Usage:
    from cdisc_transpiler.submission_module import build_suppqual
    from cdisc_transpiler.submission_module import extract_used_columns
"""

from __future__ import annotations

from .suppqual import build_suppqual, extract_used_columns

__all__ = [
    "build_suppqual",
    "extract_used_columns",
]
