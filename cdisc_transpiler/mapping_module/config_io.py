"""Configuration file I/O operations.

NOTE: This module is a compatibility wrapper. The actual implementation
has been moved to `cdisc_transpiler.infrastructure.repositories.mapping_config_repository`.
"""

from __future__ import annotations


# Re-export from infrastructure for backwards compatibility
from ..infrastructure.repositories.mapping_config_repository import (
    load_mapping_config as load_config,
    save_mapping_config as save_config,
    MappingConfigLoadError,
    MappingConfigSaveError,
)
from ..domain.entities.mapping import MappingConfig

__all__ = [
    "load_config",
    "save_config",
    "MappingConfigLoadError",
    "MappingConfigSaveError",
    "MappingConfig",
]
