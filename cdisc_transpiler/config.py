"""Centralized configuration for CDISC Transpiler.

This module provides immutable configuration with support for
environment variables and TOML configuration files.
"""

from __future__ import annotations

from dataclasses import dataclass, field
import os
from pathlib import Path


@dataclass(frozen=True)
class TranspilerConfig:
    """Immutable configuration for the CDISC Transpiler.

    This centralizes all configuration values that were previously
    scattered throughout the codebase as magic values.

    Configuration can be loaded from:
    1. Default values (defined here)
    2. Environment variables (via from_env())
    3. TOML configuration file (via ConfigLoader)

    Attributes:
        sdtm_spec_dir: Directory containing SDTM specification JSON files
        ct_dir: Directory containing CDISC Controlled Terminology files
        min_confidence: Minimum confidence threshold for fuzzy matching (0.0-1.0)
        chunk_size: Chunk size for streaming processing
        default_date: Default date string for synthesis (ISO 8601 format)
        default_subject: Default subject identifier for synthesis
        xpt_max_label_length: Maximum label length for XPT files (SAS constraint)
        xpt_max_variables: Maximum number of variables per XPT file (SAS constraint)
        qnam_max_length: Maximum length for QNAM in SUPPQUAL (SDTM constraint)
    """

    # Paths
    sdtm_spec_dir: Path = field(default_factory=lambda: Path("docs/SDTMIG_v3.4"))
    ct_dir: Path = field(default_factory=lambda: Path("docs/Controlled_Terminology"))

    # Processing defaults
    min_confidence: float = 0.5
    chunk_size: int = 1000

    # Synthesis defaults
    default_date: str = "2023-01-01"
    default_subject: str = "SYNTH001"

    # Study defaults
    # DM.COUNTRY is required and represents the country of the investigational site.
    # SDTMIG guidance generally uses ISO 3166-1 Alpha-3, but requirements can vary.
    # Provide this explicitly via env/TOML when the source data doesn't include it.
    default_country: str | None = None

    # Format constraints (from SDTM and SAS specifications)
    xpt_max_label_length: int = 200
    xpt_max_variables: int = 40
    qnam_max_length: int = 8

    def __post_init__(self):
        """Validate configuration values after initialization."""
        # Validate confidence is in valid range
        if not 0.0 <= self.min_confidence <= 1.0:
            raise ValueError(
                f"min_confidence must be between 0.0 and 1.0, got {self.min_confidence}"
            )

        # Validate chunk_size is positive
        if self.chunk_size < 1:
            raise ValueError(f"chunk_size must be positive, got {self.chunk_size}")

        # Validate constraints are positive
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
        """Create configuration from environment variables.

        Environment variables:
            SDTM_SPEC_DIR: Path to SDTM specification directory
            CT_DIR: Path to CDISC CT directory
            MIN_CONFIDENCE: Minimum confidence threshold (float)
            CHUNK_SIZE: Chunk size for streaming (int)
            DEFAULT_DATE: Default date for synthesis
            DEFAULT_SUBJECT: Default subject ID for synthesis

        Returns:
            TranspilerConfig with values from environment (falls back to defaults)
        """
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
    """Loader for configuration from multiple sources.

    This class implements the configuration loading strategy with
    the following precedence (highest to lowest):
    1. Explicit constructor arguments (in application code)
    2. TOML configuration file (if exists)
    3. Environment variables
    4. Default values
    """

    @staticmethod
    def load(config_file: Path | None = None) -> TranspilerConfig:
        """Load configuration with precedence: TOML > Env > Defaults.

        Args:
            config_file: Optional path to TOML configuration file.
                        If None, looks for 'cdisc_transpiler.toml' in current directory.

        Returns:
            TranspilerConfig with values from highest precedence source

        Note:
            TOML file support is optional. If tomllib (Python 3.11+) or
            tomli (fallback) is not available, only environment variables
            and defaults will be used.
        """
        # Start with environment variables (overrides defaults)
        config = TranspilerConfig.from_env()

        # Try to load TOML config if file exists
        if config_file is None:
            config_file = Path("cdisc_transpiler.toml")

        if config_file.exists():
            try:
                config = ConfigLoader._load_from_toml(config_file, config)
            except Exception as e:
                # Log warning but don't fail - fall back to env/defaults
                import warnings

                warnings.warn(f"Failed to load config from {config_file}: {e}")

        return config

    @staticmethod
    def _load_from_toml(
        config_file: Path,
        base_config: TranspilerConfig,
    ) -> TranspilerConfig:
        """Load configuration from TOML file.

        Args:
            config_file: Path to TOML file
            base_config: Base configuration to override

        Returns:
            New TranspilerConfig with TOML values applied

        Raises:
            ImportError: If TOML library not available
            ValueError: If TOML file is malformed
        """
        try:
            # Try Python 3.11+ built-in tomllib
            import tomllib
        except ImportError:
            try:
                # Fall back to tomli for Python < 3.11
                import tomli as tomllib  # type: ignore[import-untyped, no-redef]
            except ImportError:
                raise ImportError(
                    "TOML support requires Python 3.11+ or the 'tomli' package. "
                    "Install with: pip install tomli"
                )

        with open(config_file, "rb") as f:
            data = tomllib.load(f)

        # Extract sections
        paths = data.get("paths", {})
        default_section = data.get("default", {})

        # Build kwargs from TOML data
        kwargs = {}

        # Paths
        if "sdtm_spec_dir" in paths:
            kwargs["sdtm_spec_dir"] = Path(paths["sdtm_spec_dir"])
        if "ct_dir" in paths:
            kwargs["ct_dir"] = Path(paths["ct_dir"])

        # Defaults
        if "min_confidence" in default_section:
            kwargs["min_confidence"] = float(default_section["min_confidence"])

        if "default_country" in default_section:
            raw = default_section["default_country"]
            if raw is None:
                kwargs["default_country"] = None
            else:
                value = str(raw).strip()
                kwargs["default_country"] = value or None
        if "chunk_size" in default_section:
            kwargs["chunk_size"] = int(default_section["chunk_size"])
        if "default_date" in default_section:
            kwargs["default_date"] = default_section["default_date"]
        if "default_subject" in default_section:
            kwargs["default_subject"] = default_section["default_subject"]

        # Create new config with TOML overrides
        # Use base config values as defaults, override with TOML values
        return TranspilerConfig(
            sdtm_spec_dir=kwargs.get("sdtm_spec_dir", base_config.sdtm_spec_dir),
            ct_dir=kwargs.get("ct_dir", base_config.ct_dir),
            min_confidence=kwargs.get("min_confidence", base_config.min_confidence),
            chunk_size=kwargs.get("chunk_size", base_config.chunk_size),
            default_date=kwargs.get("default_date", base_config.default_date),
            default_subject=kwargs.get("default_subject", base_config.default_subject),
            default_country=kwargs.get("default_country", base_config.default_country),
        )
