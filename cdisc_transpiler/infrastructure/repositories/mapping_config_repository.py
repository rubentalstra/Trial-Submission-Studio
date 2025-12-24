import json
from pathlib import Path

from ...domain.entities.mapping import MappingConfig
from ..io.exceptions import DataParseError, DataSourceNotFoundError


class MappingConfigLoadError(DataParseError):
    pass


class MappingConfigSaveError(DataParseError):
    pass


def load_mapping_config(path: str | Path) -> MappingConfig:
    file_path = Path(path)
    if not file_path.exists():
        raise DataSourceNotFoundError(f"Mapping config not found: {file_path}")
    try:
        with file_path.open("r", encoding="utf-8") as handle:
            data = json.load(handle)
        config = MappingConfig.model_validate(data)
        config.enforce_domain()
        return config
    except json.JSONDecodeError as exc:
        raise MappingConfigLoadError(f"Invalid JSON in {file_path}: {exc}") from exc
    except Exception as exc:
        if isinstance(exc, (DataSourceNotFoundError, MappingConfigLoadError)):
            raise
        raise MappingConfigLoadError(f"Failed to load mapping config: {exc}") from exc


def save_mapping_config(config: MappingConfig, path: str | Path) -> None:
    file_path = Path(path)
    try:
        file_path.parent.mkdir(parents=True, exist_ok=True)
        payload = config.model_dump()
        with file_path.open("w", encoding="utf-8") as handle:
            json.dump(payload, handle, indent=2)
    except Exception as exc:
        raise MappingConfigSaveError(f"Failed to save mapping config: {exc}") from exc
