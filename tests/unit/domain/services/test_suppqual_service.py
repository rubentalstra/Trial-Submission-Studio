"""Tests for SUPPQUAL service."""

import pandas as pd
import pytest

from cdisc_transpiler.domain.services.suppqual_service import (
    build_suppqual,
    extract_used_columns,
    finalize_suppqual,
    sanitize_qnam,
)


class TestSanitizeQnam:
    """Tests for QNAM sanitization."""

    def test_basic_alphanumeric(self):
        """Test basic alphanumeric name."""
        assert sanitize_qnam("PatientAge") == "PATIENTA"

    def test_name_starting_with_digit(self):
        """Test name starting with digit gets Q prefix."""
        assert sanitize_qnam("123Value") == "Q123VALU"

    def test_special_characters_replaced(self):
        """Test special characters are replaced with underscores."""
        result = sanitize_qnam("Data-Point")
        assert result == "DATA_POI"

    def test_max_length_8(self):
        """Test name is truncated to 8 characters."""
        assert len(sanitize_qnam("VeryLongColumnName")) == 8

    def test_empty_string(self):
        """Test empty string becomes QVAL."""
        assert sanitize_qnam("") == "QVAL"

    def test_only_special_chars(self):
        """Test string with only special chars becomes QVAL."""
        assert sanitize_qnam("---") == "QVAL"

    def test_consecutive_underscores_collapsed(self):
        """Test consecutive underscores are collapsed."""
        result = sanitize_qnam("A__B")
        assert "__" not in result

    def test_leading_trailing_underscores_stripped(self):
        """Test leading/trailing underscores are stripped."""
        result = sanitize_qnam("_Test_")
        assert not result.startswith("_")
        assert not result.endswith("_")


class TestBuildSuppqual:
    """Tests for SUPPQUAL building."""

    @pytest.fixture
    def mock_domain_def(self):
        """Create a mock domain definition."""

        class MockVariable:
            def __init__(self, name):
                self.name = name

        class MockDomain:
            def __init__(self):
                self.variables = [
                    MockVariable("STUDYID"),
                    MockVariable("USUBJID"),
                    MockVariable("SEQ"),
                ]

            def variable_names(self):
                return ["STUDYID", "USUBJID", "SEQ"]

        return MockDomain()

    def test_empty_source_returns_none(self, mock_domain_def):
        """Test empty source DataFrame returns None."""
        source_df = pd.DataFrame()
        mapped_df = pd.DataFrame({"USUBJID": ["U1"]})

        result, used_cols = build_suppqual(
            domain_code="DM",
            source_df=source_df,
            mapped_df=mapped_df,
            domain_def=mock_domain_def,
        )

        assert result is None
        assert used_cols == set()

    def test_no_extra_columns_returns_none(self, mock_domain_def):
        """Test when all columns are in domain model returns None."""
        source_df = pd.DataFrame(
            {
                "STUDYID": ["S1"],
                "USUBJID": ["U1"],
            }
        )
        mapped_df = pd.DataFrame({"USUBJID": ["U1"]})

        result, used_cols = build_suppqual(
            domain_code="DM",
            source_df=source_df,
            mapped_df=mapped_df,
            domain_def=mock_domain_def,
            used_source_columns={"STUDYID", "USUBJID"},
        )

        assert result is None
        assert used_cols == set()

    def test_extra_column_creates_suppqual(self, mock_domain_def):
        """Test extra column creates SUPPQUAL records."""
        source_df = pd.DataFrame(
            {
                "STUDYID": ["STUDY1"],
                "USUBJID": ["U001"],
                "EXTRA_COL": ["extra_value"],
            }
        )
        mapped_df = pd.DataFrame(
            {
                "STUDYID": ["STUDY1"],
                "USUBJID": ["U001"],
            }
        )

        result, used_cols = build_suppqual(
            domain_code="DM",
            source_df=source_df,
            mapped_df=mapped_df,
            domain_def=mock_domain_def,
            used_source_columns={"STUDYID", "USUBJID"},
        )

        assert result is not None
        assert "EXTRA_COL" in used_cols
        assert len(result) == 1
        assert result.iloc[0]["QNAM"] == "EXTRA_CO"
        assert result.iloc[0]["QVAL"] == "extra_value"

    def test_missing_usubjid_rows_dropped(self, mock_domain_def):
        """Test rows with missing USUBJID are dropped."""
        source_df = pd.DataFrame(
            {
                "STUDYID": ["S1", "S1"],
                "USUBJID": ["U1", ""],  # Second row has empty USUBJID
                "EXTRA": ["val1", "val2"],
            }
        )
        mapped_df = pd.DataFrame(
            {
                "USUBJID": ["U1", "U2"],
            }
        )

        result, _ = build_suppqual(
            domain_code="DM",
            source_df=source_df,
            mapped_df=mapped_df,
            domain_def=mock_domain_def,
        )

        # Only 1 row should be processed (row with valid USUBJID)
        if result is not None:
            usubjids = result["USUBJID"].unique()
            assert "" not in usubjids

    def test_empty_values_not_included(self, mock_domain_def):
        """Test empty values are not included in SUPPQUAL."""
        source_df = pd.DataFrame(
            {
                "STUDYID": ["S1"],
                "USUBJID": ["U1"],
                "EMPTY_COL": [""],
            }
        )
        mapped_df = pd.DataFrame({"USUBJID": ["U1"]})

        result, _ = build_suppqual(
            domain_code="DM",
            source_df=source_df,
            mapped_df=mapped_df,
            domain_def=mock_domain_def,
        )

        assert result is None


class TestFinalizeSuppqual:
    """Tests for SUPPQUAL finalization."""

    def test_deduplication(self):
        """Test duplicate records are removed."""
        supp_df = pd.DataFrame(
            {
                "STUDYID": ["S1", "S1"],
                "USUBJID": ["U1", "U1"],
                "IDVAR": ["SEQ", "SEQ"],
                "IDVARVAL": ["1", "1"],
                "QNAM": ["COL1", "COL1"],  # Duplicate
                "QVAL": ["val1", "val2"],
            }
        )

        result = finalize_suppqual(supp_df)

        assert len(result) == 1

    def test_sorting(self):
        """Test records are sorted by USUBJID and IDVARVAL."""
        supp_df = pd.DataFrame(
            {
                "STUDYID": ["S1", "S1", "S1"],
                "USUBJID": ["U2", "U1", "U1"],
                "IDVAR": ["SEQ", "SEQ", "SEQ"],
                "IDVARVAL": ["2", "1", "2"],
                "QNAM": ["A", "B", "C"],
                "QVAL": ["v1", "v2", "v3"],
            }
        )

        result = finalize_suppqual(supp_df)

        # U1 should come before U2
        assert result.iloc[0]["USUBJID"] == "U1"


class TestExtractUsedColumns:
    """Tests for extract_used_columns."""

    def test_none_config(self):
        """Test None config returns empty set."""
        result = extract_used_columns(None)
        assert result == set()
