"""Infrastructure layer for CDISC Transpiler.

This layer contains adapters for external systems and I/O operations.
It implements the ports defined in the application layer.
"""

from .container import DependencyContainer, create_default_container

__all__ = [
    "DependencyContainer",
    "create_default_container",
]
