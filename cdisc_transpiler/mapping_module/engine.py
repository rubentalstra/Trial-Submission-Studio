"""Basic mapping engine for SDTM variable suggestion.

NOTE: This module is a compatibility wrapper. The actual implementation
has been moved to `cdisc_transpiler.domain.services.mapping.engine`.
"""

from __future__ import annotations

# Re-export from domain services for backwards compatibility
from ..domain.services.mapping.engine import MappingEngine

__all__ = ["MappingEngine"]
