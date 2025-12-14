"""Findings domain transformations.

This module contains transformers for Findings class domains
(VS, LB, QS, etc.) that typically require wide-to-long reshaping.
"""

from .wide_to_long import (
    TestColumnPattern,
    WideToLongTransformer,
)

__all__ = [
    "TestColumnPattern",
    "WideToLongTransformer",
]
