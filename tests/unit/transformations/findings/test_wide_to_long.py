"""Unit tests for generic wide-to-long transformer.

Tests for TestColumnPattern and WideToLongTransformer classes.
TestDefinition is tested as an internal implementation detail.
"""

import pandas as pd
import pytest

from cdisc_transpiler.transformations.findings.wide_to_long import (
    TestColumnPattern,
    TestDefinition,  # Internal class, not exported publicly
    WideToLongTransformer,
)
from cdisc_transpiler.transformations.base import TransformationContext


class TestTestColumnPattern:
    """Tests for TestColumnPattern dataclass."""

    def test_pattern_match_success(self):
        """Test successful pattern matching."""
        pattern = TestColumnPattern(
            pattern=r"^ORRES_([A-Z]+)$",
            column_type="orres",
            description="Original result columns",
        )

        result = pattern.match("ORRES_HEIGHT")
        assert result == "HEIGHT"

        result = pattern.match("ORRES_WEIGHT")
        assert result == "WEIGHT"

    def test_pattern_match_case_insensitive(self):
        """Test that pattern matching is case-insensitive."""
        pattern = TestColumnPattern(pattern=r"^ORRES_([A-Z]+)$", column_type="orres")

        assert pattern.match("orres_height") == "HEIGHT"
        assert pattern.match("Orres_Weight") == "WEIGHT"
        assert pattern.match("ORRES_glucose") == "GLUCOSE"

    def test_pattern_match_failure(self):
        """Test pattern matching returns None for non-matches."""
        pattern = TestColumnPattern(pattern=r"^ORRES_([A-Z]+)$", column_type="orres")

        assert pattern.match("OTHER_COLUMN") is None
        assert pattern.match("HEIGHT") is None
        assert pattern.match("ORRES_") is None

    def test_pattern_with_complex_regex(self):
        """Test pattern with more complex regex."""
        pattern = TestColumnPattern(
            pattern=r"^([A-Za-z0-9]+)\s+result or finding in original units$",
            column_type="orres",
        )

        assert pattern.match("Glucose result or finding in original units") == "GLUCOSE"
        assert pattern.match("HbA1c result or finding in original units") == "HBA1C"

    def test_pattern_with_numeric_test_codes(self):
        """Test pattern matching with alphanumeric test codes."""
        pattern = TestColumnPattern(
            pattern=r"^TEST_([A-Za-z0-9]+)$", column_type="label"
        )

        assert pattern.match("TEST_HBA1C") == "HBA1C"
        assert pattern.match("TEST_B12") == "B12"


class TestTestDefinition:
    """Tests for TestDefinition dataclass."""

    def test_create_test_definition(self):
        """Test creating a test definition."""
        test_def = TestDefinition(
            test_code="HEIGHT",
            columns={"orres": "ORRES_HEIGHT", "unit": "ORRESU_HEIGHT"},
        )

        assert test_def.test_code == "HEIGHT"
        assert test_def.columns["orres"] == "ORRES_HEIGHT"
        assert test_def.columns["unit"] == "ORRESU_HEIGHT"

    def test_get_column_exists(self):
        """Test getting a column that exists."""
        test_def = TestDefinition(test_code="WEIGHT", columns={"orres": "ORRES_WEIGHT"})

        assert test_def.get_column("orres") == "ORRES_WEIGHT"

    def test_get_column_not_exists(self):
        """Test getting a column that doesn't exist."""
        test_def = TestDefinition(test_code="HEIGHT", columns={"orres": "ORRES_HEIGHT"})

        assert test_def.get_column("unit") is None
        assert test_def.get_column("nrlo") is None

    def test_has_result_true(self):
        """Test has_result returns True when orres exists."""
        test_def = TestDefinition(
            test_code="TEMP", columns={"orres": "ORRES_TEMP", "unit": "ORRESU_TEMP"}
        )

        assert test_def.has_result() is True

    def test_has_result_false(self):
        """Test has_result returns False when orres doesn't exist."""
        test_def = TestDefinition(test_code="TEMP", columns={"unit": "ORRESU_TEMP"})

        assert test_def.has_result() is False


class TestWideToLongTransformer:
    """Tests for WideToLongTransformer class."""

    def test_initialization(self):
        """Test transformer initialization."""
        patterns = [
            TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres"),
            TestColumnPattern(r"^ORRESU_([A-Z]+)$", "unit"),
        ]

        transformer = WideToLongTransformer(
            domain="VS",
            patterns=patterns,
            column_renames={"Subject Id": "USUBJID"},
            output_mapping={"TESTCD": "VSTESTCD", "TEST": "VSTEST"},
        )

        assert transformer.domain == "VS"
        assert len(transformer.patterns) == 2
        assert transformer.column_renames == {"Subject Id": "USUBJID"}
        assert transformer.output_mapping == {"TESTCD": "VSTESTCD", "TEST": "VSTEST"}

    def test_can_transform_correct_domain(self):
        """Test can_transform returns True for matching domain with test columns."""
        patterns = [TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres")]
        transformer = WideToLongTransformer(domain="VS", patterns=patterns)

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_HEIGHT": [170],
                "ORRES_WEIGHT": [70],
            }
        )

        assert transformer.can_transform(df, "VS") is True

    def test_can_transform_wrong_domain(self):
        """Test can_transform returns False for non-matching domain."""
        patterns = [TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres")]
        transformer = WideToLongTransformer(domain="VS", patterns=patterns)

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_HEIGHT": [170],
            }
        )

        assert transformer.can_transform(df, "LB") is False

    def test_can_transform_no_test_columns(self):
        """Test can_transform returns False when no test columns found."""
        patterns = [TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres")]
        transformer = WideToLongTransformer(domain="VS", patterns=patterns)

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "OTHER_COLUMN": ["value"],
            }
        )

        assert transformer.can_transform(df, "VS") is False

    def test_discover_tests(self):
        """Test discovering test definitions from columns."""
        patterns = [
            TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres"),
            TestColumnPattern(r"^ORRESU_([A-Z]+)$", "unit"),
        ]
        transformer = WideToLongTransformer(domain="VS", patterns=patterns)

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_HEIGHT": [170],
                "ORRESU_HEIGHT": ["cm"],
                "ORRES_WEIGHT": [70],
                "ORRESU_WEIGHT": ["kg"],
                "OTHER": ["value"],
            }
        )

        test_defs = transformer._discover_tests(df)

        assert len(test_defs) == 2
        assert test_defs[0].test_code == "HEIGHT"
        assert test_defs[0].get_column("orres") == "ORRES_HEIGHT"
        assert test_defs[0].get_column("unit") == "ORRESU_HEIGHT"
        assert test_defs[1].test_code == "WEIGHT"

    def test_discover_tests_skips_cd_columns(self):
        """Test that *CD (coded) columns are skipped."""
        patterns = [TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres")]
        transformer = WideToLongTransformer(domain="VS", patterns=patterns)

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_HEIGHT": [170],
                "ORRES_STATUSCD": ["NORMAL"],  # Should be skipped
            }
        )

        test_defs = transformer._discover_tests(df)

        assert len(test_defs) == 1
        assert test_defs[0].test_code == "HEIGHT"

    def test_normalize_columns(self):
        """Test column name normalization."""
        transformer = WideToLongTransformer(
            domain="VS",
            patterns=[],
            column_renames={
                "Subject Id": "USUBJID",
                "Event name": "VISIT",
            },
        )

        df = pd.DataFrame(
            {
                "Subject Id": ["001"],
                "Event name": ["Visit 1"],
                "Other": ["value"],
            }
        )

        normalized = transformer._normalize_columns(df)

        assert "USUBJID" in normalized.columns
        assert "VISIT" in normalized.columns
        assert "Other" in normalized.columns
        assert "Subject Id" not in normalized.columns

    def test_extract_row_identifiers(self):
        """Test extracting identifiers from a row."""
        transformer = WideToLongTransformer(domain="VS", patterns=[])

        row = pd.Series(
            {
                "USUBJID": "001",
                "VISITNUM": 1.0,
                "VISIT": "Visit 1",
                "VSDTC": "2023-01-15",
            }
        )

        identifiers = transformer._extract_row_identifiers(row)

        assert identifiers["USUBJID"] == "001"
        assert identifiers["VISITNUM"] == 1.0
        assert identifiers["VISIT"] == "Visit 1"
        assert identifiers["VSDTC"] == "2023-01-15"

    def test_extract_row_identifiers_skips_invalid_usubjid(self):
        """Test that invalid USUBJID values are skipped."""
        transformer = WideToLongTransformer(domain="VS", patterns=[])

        # Empty USUBJID
        row1 = pd.Series({"USUBJID": ""})
        assert "USUBJID" not in transformer._extract_row_identifiers(row1)

        # USUBJID header value
        row2 = pd.Series({"USUBJID": "usubjid"})
        assert "USUBJID" not in transformer._extract_row_identifiers(row2)

    def test_extract_value_simple(self):
        """Test extracting simple values."""
        transformer = WideToLongTransformer(domain="VS", patterns=[])

        row = pd.Series({"HEIGHT": 170, "WEIGHT": 70})

        assert transformer._extract_value(row, "HEIGHT") == 170
        assert transformer._extract_value(row, "WEIGHT") == 70

    def test_extract_value_handles_na(self):
        """Test extracting NA values returns None."""
        transformer = WideToLongTransformer(domain="VS", patterns=[])

        row = pd.Series({"HEIGHT": pd.NA, "WEIGHT": None})

        assert transformer._extract_value(row, "HEIGHT") is None
        assert transformer._extract_value(row, "WEIGHT") is None

    def test_extract_value_handles_series(self):
        """Test extracting from Series (duplicate columns)."""
        transformer = WideToLongTransformer(domain="VS", patterns=[])

        row = pd.Series({"HEIGHT": pd.Series([170, 175])})

        result = transformer._extract_value(row, "HEIGHT")
        assert result == 170  # First non-NA value

    def test_map_output_name(self):
        """Test mapping generic names to domain-specific names."""
        transformer = WideToLongTransformer(
            domain="VS",
            patterns=[],
            output_mapping={
                "TESTCD": "VSTESTCD",
                "TEST": "VSTEST",
                "ORRES": "VSORRES",
            },
        )

        assert transformer._map_output_name("TESTCD") == "VSTESTCD"
        assert transformer._map_output_name("TEST") == "VSTEST"
        assert transformer._map_output_name("ORRES") == "VSORRES"
        assert transformer._map_output_name("UNMAPPED") == "UNMAPPED"

    def test_transform_simple_wide_to_long(self):
        """Test complete transformation from wide to long format."""
        patterns = [
            TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres"),
            TestColumnPattern(r"^ORRESU_([A-Z]+)$", "unit"),
        ]

        transformer = WideToLongTransformer(
            domain="VS",
            patterns=patterns,
            output_mapping={
                "TESTCD": "VSTESTCD",
                "TEST": "VSTEST",
                "ORRES": "VSORRES",
                "ORRESU": "VSORRESU",
            },
        )

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "002"],
                "VISIT": ["Visit 1", "Visit 1"],
                "ORRES_HEIGHT": [170, 165],
                "ORRESU_HEIGHT": ["cm", "cm"],
                "ORRES_WEIGHT": [70, 60],
                "ORRESU_WEIGHT": ["kg", "kg"],
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert result.applied
        assert len(result.data) == 4  # 2 subjects × 2 tests

        # Check structure
        assert "STUDYID" in result.data.columns
        assert "DOMAIN" in result.data.columns
        assert "USUBJID" in result.data.columns
        assert "VSTESTCD" in result.data.columns
        assert "VSTEST" in result.data.columns
        assert "VSORRES" in result.data.columns
        assert "VSORRESU" in result.data.columns

        # Check values
        assert all(result.data["STUDYID"] == "STUDY001")
        assert all(result.data["DOMAIN"] == "VS")
        assert set(result.data["USUBJID"]) == {"001", "002"}
        assert set(result.data["VSTESTCD"]) == {"HEIGHT", "WEIGHT"}

    def test_transform_empty_data(self):
        """Test transformation with no valid data."""
        patterns = [TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres")]
        transformer = WideToLongTransformer(domain="VS", patterns=patterns)

        df = pd.DataFrame(
            {
                "USUBJID": [],
                "ORRES_HEIGHT": [],
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.applied
        assert len(result.data) == 0
        assert result.has_warnings

    def test_transform_skips_rows_without_usubjid(self):
        """Test that rows without USUBJID are skipped."""
        patterns = [TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres")]
        transformer = WideToLongTransformer(
            domain="VS",
            patterns=patterns,
            output_mapping={"TESTCD": "VSTESTCD", "TEST": "VSTEST", "ORRES": "VSORRES"},
        )

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "", "003"],
                "ORRES_HEIGHT": [170, 165, 175],
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert len(result.data) == 2  # Only 2 valid rows
        assert set(result.data["USUBJID"]) == {"001", "003"}

    def test_transform_skips_empty_results(self):
        """Test that tests with empty results are skipped."""
        patterns = [TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres")]
        transformer = WideToLongTransformer(
            domain="VS",
            patterns=patterns,
            output_mapping={"TESTCD": "VSTESTCD", "TEST": "VSTEST", "ORRES": "VSORRES"},
        )

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_HEIGHT": [170],
                "ORRES_WEIGHT": [None],  # Empty result
                "ORRES_TEMP": [""],  # Empty string
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert len(result.data) == 1  # Only HEIGHT with valid result
        assert result.data.iloc[0]["VSTESTCD"] == "HEIGHT"

    def test_transform_with_test_code_normalizer(self):
        """Test transformation with custom test code normalizer."""

        def normalizer(domain: str, test_code: str) -> str | None:
            """Normalize GLUCU to GLUC."""
            if test_code == "GLUCU":
                return "GLUC"
            return test_code

        patterns = [TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres")]
        transformer = WideToLongTransformer(
            domain="LB",
            patterns=patterns,
            output_mapping={"TESTCD": "LBTESTCD", "TEST": "LBTEST", "ORRES": "LBORRES"},
            test_code_normalizer=normalizer,
        )

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_GLUCU": [5.5],
            }
        )

        context = TransformationContext(domain="LB", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert result.data.iloc[0]["LBTESTCD"] == "GLUC"  # Normalized

    def test_transform_with_test_label_getter(self):
        """Test transformation with custom test label getter."""

        def label_getter(domain: str, test_code: str) -> str:
            """Get test labels."""
            labels = {
                "HEIGHT": "Height Measurement",
                "WEIGHT": "Body Weight",
            }
            return labels.get(test_code, test_code)

        patterns = [TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres")]
        transformer = WideToLongTransformer(
            domain="VS",
            patterns=patterns,
            output_mapping={"TESTCD": "VSTESTCD", "TEST": "VSTEST", "ORRES": "VSORRES"},
            test_label_getter=label_getter,
        )

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_HEIGHT": [170],
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert result.data.iloc[0]["VSTEST"] == "Height Measurement"

    def test_transform_wrong_domain_returns_not_applied(self):
        """Test that wrong domain returns not applied result."""
        patterns = [TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres")]
        transformer = WideToLongTransformer(domain="VS", patterns=patterns)

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_HEIGHT": [170],
            }
        )

        context = TransformationContext(domain="LB", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert not result.applied
        assert "does not apply" in result.message

    def test_transform_metadata_tracking(self):
        """Test that transformation tracks metadata correctly."""
        patterns = [TestColumnPattern(r"^ORRES_([A-Z]+)$", "orres")]
        transformer = WideToLongTransformer(
            domain="VS",
            patterns=patterns,
            output_mapping={"TESTCD": "VSTESTCD", "TEST": "VSTEST", "ORRES": "VSORRES"},
        )

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "002"],
                "ORRES_HEIGHT": [170, 165],
                "ORRES_WEIGHT": [70, 60],
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.metadata["input_rows"] == 2
        assert result.metadata["output_rows"] == 4
        assert result.metadata["tests_found"] == 2
        assert set(result.metadata["test_codes"]) == {"HEIGHT", "WEIGHT"}
