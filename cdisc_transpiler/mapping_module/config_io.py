"""Configuration file I/O operations.

This module handles loading and saving mapping configurations from/to JSON files.
"""

from __future__ import annotations

import json
from pathlib import Path

from .models import MappingConfig


def load_config(path: str | Path) -> MappingConfig:
    """Load a MappingConfig from a JSON file.

    Args:
        path: Path to JSON configuration file

    Returns:
        Loaded and validated mapping configuration

    Example:
        >>> config = load_config("mappings/dm.json")
    """
    file_path = Path(path)
    with file_path.open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    config = MappingConfig.model_validate(data)
    config.enforce_domain()
    return config


def save_config(config: MappingConfig, path: str | Path) -> None:
    """Save a MappingConfig to a JSON file.

    Args:
        config: Mapping configuration to save
        path: Path where JSON file should be written

    Example:
        >>> save_config(config, "mappings/dm.json")
    """
    file_path = Path(path)
    file_path.parent.mkdir(parents=True, exist_ok=True)
    payload = config.model_dump()
    with file_path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2)
