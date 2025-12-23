"""Transformation framework.

This module provides a pluggable transformation framework for
data transformations like wide-to-long reshaping, date formatting, etc.
"""

from .base import (
    TransformationContext,
    TransformationResult,
    TransformerPort,
    is_transformer,
)
from .pipeline import TransformationPipeline

__all__ = [
    "TransformationContext",
    "TransformationPipeline",
    "TransformationResult",
    "TransformerPort",
    "is_transformer",
]
