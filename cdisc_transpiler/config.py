from collections.abc import Mapping
from dataclasses import dataclass, field
import os
from pathlib import Path
import tomllib
from typing import cast
import warnings


@dataclass(frozen=True, slots=True)
class TranspilerConfig:
    sdtm_spec_dir: Path = field(default_factory=lambda: Path("docs/SDTMIG_v3.4"))
    ct_dir: Path = field(default_factory=lambda: Path("docs/Controlled_Terminology"))
    min_confidence: float = 0.5
    chunk_size: int = 1000
    default_date: str = "2023-01-01"
    default_subject: str = "SYNTH001"
    default_country: str | None = None
    xpt_max_label_length: int = 200
    xpt_max_variables: int = 40
    qnam_max_length: int = 8

    def __post_init__(self) -> None:
        if not 0.0 <= self.min_confidence <= 1.0:
            raise ValueError(
                f"min_confidence must be between 0.0 and 1.0, got {self.min_confidence}"
            )
        if self.chunk_size < 1:
            raise ValueError(f"chunk_size must be positive, got {self.chunk_size}")
        if self.xpt_max_label_length < 1:
            raise ValueError(
                f"xpt_max_label_length must be positive, got {self.xpt_max_label_length}"
            )
        if self.xpt_max_variables < 1:
            raise ValueError(
                f"xpt_max_variables must be positive, got {self.xpt_max_variables}"
            )
        if self.qnam_max_length < 1:
            raise ValueError(
                f"qnam_max_length must be positive, got {self.qnam_max_length}"
            )

    @classmethod
    def from_env(cls) -> TranspilerConfig:
        raw_default_country = os.getenv("DEFAULT_COUNTRY")
        default_country = raw_default_country.strip() if raw_default_country else None
        return cls(
            sdtm_spec_dir=Path(os.getenv("SDTM_SPEC_DIR", "docs/SDTMIG_v3.4")),
            ct_dir=Path(os.getenv("CT_DIR", "docs/Controlled_Terminology")),
            min_confidence=float(os.getenv("MIN_CONFIDENCE", "0.5")),
            chunk_size=int(os.getenv("CHUNK_SIZE", "1000")),
            default_date=os.getenv("DEFAULT_DATE", "2023-01-01"),
            default_subject=os.getenv("DEFAULT_SUBJECT", "SYNTH001"),
            default_country=default_country,
        )


class ConfigLoader:
    pass

    @staticmethod
    def load(config_file: Path | None = None) -> TranspilerConfig:
        config = TranspilerConfig.from_env()
        if config_file is None:
            config_file = Path("cdisc_transpiler.toml")
        if config_file.exists():
            try:
                config = ConfigLoader._load_from_toml(config_file, config)
            except Exception as e:
                warnings.warn(
                    f"Failed to load config from {config_file}: {e}", stacklevel=2
                )
        return config

    @staticmethod
    def _load_from_toml(
        config_file: Path, base_config: TranspilerConfig
    ) -> TranspilerConfig:
        with config_file.open("rb") as handle:
            data = tomllib.load(handle)
        paths = _get_table(data, "paths")
        default_section = _get_table(data, "default")
        sdtm_spec_dir = base_config.sdtm_spec_dir
        if value := paths.get("sdtm_spec_dir"):
            sdtm_spec_dir = Path(str(value))
        ct_dir = base_config.ct_dir
        if value := paths.get("ct_dir"):
            ct_dir = Path(str(value))
        min_confidence = base_config.min_confidence
        if (value := default_section.get("min_confidence")) is not None:
            min_confidence = _coerce_float(value, key="default.min_confidence")
        chunk_size = base_config.chunk_size
        if (value := default_section.get("chunk_size")) is not None:
            chunk_size = _coerce_int(value, key="default.chunk_size")
        default_date = base_config.default_date
        if (value := default_section.get("default_date")) is not None:
            default_date = str(value)
        default_subject = base_config.default_subject
        if (value := default_section.get("default_subject")) is not None:
            default_subject = str(value)
        default_country = base_config.default_country
        if "default_country" in default_section:
            raw = default_section.get("default_country")
            if raw is None:
                default_country = None
            else:
                cleaned = str(raw).strip()
                default_country = cleaned or None
        return TranspilerConfig(
            sdtm_spec_dir=sdtm_spec_dir,
            ct_dir=ct_dir,
            min_confidence=min_confidence,
            chunk_size=chunk_size,
            default_date=default_date,
            default_subject=default_subject,
            default_country=default_country,
        )


def _get_table(data: Mapping[str, object], key: str) -> Mapping[str, object]:
    value = data.get(key)
    if isinstance(value, Mapping):
        return cast("Mapping[str, object]", value)
    return {}


def _coerce_float(value: object, *, key: str) -> float:
    if isinstance(value, (int, float)):
        return float(value)
    if isinstance(value, str):
        return float(value)
    raise ValueError(f"{key} must be numeric or string, got {type(value).__name__}")


def _coerce_int(value: object, *, key: str) -> int:
    if isinstance(value, bool):
        raise ValueError(f"{key} must be an int, got bool")
    if isinstance(value, int):
        return value
    if isinstance(value, float):
        return int(value)
    if isinstance(value, str):
        return int(value)
    raise ValueError(f"{key} must be int-like or string, got {type(value).__name__}")
