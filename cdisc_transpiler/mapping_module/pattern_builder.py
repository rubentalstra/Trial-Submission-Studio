"""Dynamic pattern generation for SDTM variable mapping.

NOTE: This module is a compatibility wrapper. The actual implementation
has been moved to `cdisc_transpiler.domain.services.mapping.pattern_builder`.
"""

from __future__ import annotations

# Re-export from domain services for backwards compatibility
from ..domain.services.mapping.pattern_builder import (
    build_variable_patterns,
    get_domain_suffix_patterns,
)

__all__ = ["build_variable_patterns", "get_domain_suffix_patterns"]
