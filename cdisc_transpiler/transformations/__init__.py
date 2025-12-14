"""Transformation framework.

This module provides a pluggable transformation framework for
data transformations like wide-to-long reshaping, date formatting, etc.
"""

from .base import (
    TransformerPort,
    TransformationContext,
    TransformationResult,
    is_transformer,
)

__all__ = [
    "TransformerPort",
    "TransformationContext",
    "TransformationResult",
    "is_transformer",
]
