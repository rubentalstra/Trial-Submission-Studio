"""CLI entry point for the CDISC Transpiler.

This module provides backward compatibility by importing from the new modular CLI structure.
The actual implementation has been moved to the cli/ package for better organization.

For new code, import directly from cdisc_transpiler.cli
"""

from __future__ import annotations

# Import the main CLI app from the new modular structure
from .cli import app


# Export for backward compatibility
__all__ = ["app"]


if __name__ == "__main__":  # pragma: no cover - manual CLI invocation
    app()
