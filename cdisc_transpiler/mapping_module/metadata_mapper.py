"""Metadata-aware mapping engine for intelligent SDTM variable suggestion.

NOTE: This module is a compatibility wrapper. The actual implementation
has been moved to `cdisc_transpiler.domain.services.mapping.metadata_mapper`.
"""

from __future__ import annotations

# Re-export from domain services for backwards compatibility
from ..domain.services.mapping.metadata_mapper import MetadataAwareMapper

__all__ = ["MetadataAwareMapper"]
