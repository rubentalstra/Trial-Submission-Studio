"""Unit tests for configuration and constants."""

from __future__ import annotations

from pathlib import Path

import pytest

from cdisc_transpiler.config import ConfigLoader, TranspilerConfig
from cdisc_transpiler.constants import Constraints, Defaults, Patterns


class TestTranspilerConfig:
    """Test suite for TranspilerConfig class."""

    def test_default_config(self):
        """Test default configuration values."""
        config = TranspilerConfig()

        assert config.sdtm_spec_dir == Path("docs/SDTMIG_v3.4")
        assert config.ct_dir == Path("docs/Controlled_Terminology")
        assert config.min_confidence == 0.5
        assert config.chunk_size == 1000
        assert config.default_date == "2023-01-01"
        assert config.default_subject == "SYNTH001"
        assert config.xpt_max_label_length == 200
        assert config.xpt_max_variables == 40
        assert config.qnam_max_length == 8

    def test_custom_config(self):
        """Test creating config with custom values."""
        config = TranspilerConfig(
            sdtm_spec_dir=Path("/custom/sdtm"),
            ct_dir=Path("/custom/ct"),
            min_confidence=0.7,
            chunk_size=500,
            default_date="2024-01-01",
            default_subject="CUSTOM001",
        )

        assert config.sdtm_spec_dir == Path("/custom/sdtm")
        assert config.ct_dir == Path("/custom/ct")
        assert config.min_confidence == 0.7
        assert config.chunk_size == 500
        assert config.default_date == "2024-01-01"
        assert config.default_subject == "CUSTOM001"

    def test_config_is_immutable(self):
        """Test that config is frozen and cannot be modified."""
        config = TranspilerConfig()

        with pytest.raises(Exception):  # FrozenInstanceError
            config.min_confidence = 0.8

    def test_config_validation_min_confidence(self):
        """Test validation of min_confidence range."""
        # Valid values
        TranspilerConfig(min_confidence=0.0)
        TranspilerConfig(min_confidence=0.5)
        TranspilerConfig(min_confidence=1.0)

        # Invalid values
        with pytest.raises(ValueError, match="min_confidence must be between"):
            TranspilerConfig(min_confidence=-0.1)

        with pytest.raises(ValueError, match="min_confidence must be between"):
            TranspilerConfig(min_confidence=1.1)

    def test_config_validation_chunk_size(self):
        """Test validation of chunk_size."""
        # Valid values
        TranspilerConfig(chunk_size=1)
        TranspilerConfig(chunk_size=1000)

        # Invalid values
        with pytest.raises(ValueError, match="chunk_size must be positive"):
            TranspilerConfig(chunk_size=0)

        with pytest.raises(ValueError, match="chunk_size must be positive"):
            TranspilerConfig(chunk_size=-100)

    def test_config_from_env(self, monkeypatch):
        """Test loading config from environment variables."""
        # Set environment variables
        monkeypatch.setenv("SDTM_SPEC_DIR", "/env/sdtm")
        monkeypatch.setenv("CT_DIR", "/env/ct")
        monkeypatch.setenv("MIN_CONFIDENCE", "0.8")
        monkeypatch.setenv("CHUNK_SIZE", "2000")
        monkeypatch.setenv("DEFAULT_DATE", "2025-01-01")
        monkeypatch.setenv("DEFAULT_SUBJECT", "ENV001")

        config = TranspilerConfig.from_env()

        assert config.sdtm_spec_dir == Path("/env/sdtm")
        assert config.ct_dir == Path("/env/ct")
        assert config.min_confidence == 0.8
        assert config.chunk_size == 2000
        assert config.default_date == "2025-01-01"
        assert config.default_subject == "ENV001"


class TestConfigLoader:
    """Test suite for ConfigLoader class."""

    def test_load_with_no_toml_file(self):
        """Test loading config when TOML file doesn't exist."""
        config = ConfigLoader.load(config_file=Path("/nonexistent/config.toml"))

        # Should fall back to defaults
        assert config.min_confidence == 0.5
        assert config.chunk_size == 1000

    def test_load_from_toml(self, tmp_path: Path):
        """Test loading config from TOML file."""
        # Create TOML file
        toml_file = tmp_path / "config.toml"
        toml_file.write_text("""
[paths]
sdtm_spec_dir = "/toml/sdtm"
ct_dir = "/toml/ct"

[default]
min_confidence = 0.9
chunk_size = 3000
default_date = "2026-01-01"
default_subject = "TOML001"
""")

        config = ConfigLoader.load(config_file=toml_file)

        assert config.sdtm_spec_dir == Path("/toml/sdtm")
        assert config.ct_dir == Path("/toml/ct")
        assert config.min_confidence == 0.9
        assert config.chunk_size == 3000
        assert config.default_date == "2026-01-01"
        assert config.default_subject == "TOML001"

    def test_load_toml_with_partial_config(self, tmp_path: Path):
        """Test loading TOML with only some values (others use defaults)."""
        toml_file = tmp_path / "config.toml"
        toml_file.write_text("""
[default]
min_confidence = 0.75
""")

        config = ConfigLoader.load(config_file=toml_file)

        # TOML value
        assert config.min_confidence == 0.75

        # Default values
        assert config.chunk_size == 1000
        assert config.default_date == "2023-01-01"


class TestDefaults:
    """Test suite for Defaults constants."""

    def test_defaults_values(self):
        """Test that default values are as expected."""
        assert Defaults.DATE == "2023-01-01"
        assert Defaults.SUBJECT_ID == "SYNTH001"
        assert Defaults.MIN_CONFIDENCE == 0.5
        assert Defaults.CHUNK_SIZE == 1000
        assert Defaults.OUTPUT_FORMAT == "both"
        assert Defaults.GENERATE_DEFINE is True
        assert Defaults.GENERATE_SAS is True


class TestConstraints:
    """Test suite for Constraints constants."""

    def test_xpt_constraints(self):
        """Test XPT format constraints."""
        assert Constraints.XPT_MAX_LABEL_LENGTH == 200
        assert Constraints.XPT_MAX_VARIABLES == 40
        assert Constraints.XPT_MAX_NAME_LENGTH == 8

    def test_sdtm_constraints(self):
        """Test SDTM constraints."""
        assert Constraints.QNAM_MAX_LENGTH == 8
        assert Constraints.STUDYID_MAX_LENGTH == 20
        assert Constraints.DOMAIN_MAX_LENGTH == 2

    def test_define_xml_constraints(self):
        """Test Define-XML version constraints."""
        assert Constraints.DEFINE_XML_VERSION == "2.1.0"
        assert Constraints.DATASET_XML_VERSION == "1.0.0"


class TestPatterns:
    """Test suite for regex patterns."""

    def test_sdtm_variable_name_pattern(self):
        """Test SDTM variable name pattern."""
        import re

        pattern = re.compile(Patterns.SDTM_VARIABLE_NAME)

        # Valid names
        assert pattern.match("USUBJID")
        assert pattern.match("AESEQ")
        assert pattern.match("VS_TEST")
        assert pattern.match("A")
        assert pattern.match("A1234567")  # 8 chars

        # Invalid names
        assert not pattern.match("usubjid")  # lowercase
        assert not pattern.match("1AESEQ")  # starts with number
        assert not pattern.match("TOOLONGNAME")  # > 8 chars
        assert not pattern.match("TEST-VAR")  # invalid char

    def test_iso_date_patterns(self):
        """Test ISO 8601 date patterns."""
        import re

        full = re.compile(Patterns.ISO_DATE_FULL)
        month = re.compile(Patterns.ISO_DATE_PARTIAL_MONTH)
        year = re.compile(Patterns.ISO_DATE_PARTIAL_YEAR)

        # Valid full dates
        assert full.match("2023-01-01")
        assert full.match("2023-12-31")

        # Valid partial dates
        assert month.match("2023-01")
        assert year.match("2023")

        # Invalid formats
        assert not full.match("2023-1-1")  # no padding
        assert not full.match("23-01-01")  # 2-digit year
        assert not month.match("2023-1")  # no padding
