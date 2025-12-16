"""Compatibility models for the terminology module.

The codebase is migrating toward Ports & Adapters / Clean Architecture.
The canonical controlled terminology entity now lives in the domain layer.

This module remains as a compatibility shim so legacy imports like:

    from cdisc_transpiler.terminology_module.models import ControlledTerminology

keep working during the transition.
"""

from __future__ import annotations

from cdisc_transpiler.domain.entities.controlled_terminology import (
    ControlledTerminology,
)

__all__ = ["ControlledTerminology"]
