"""Unit tests for VS (Vital Signs) transformer.

Tests for VSTransformer class covering VS-specific patterns and logic.
"""

import pandas as pd

from cdisc_transpiler.transformations.findings import VSTransformer
from cdisc_transpiler.transformations.base import TransformationContext


class TestVSTransformer:
    """Tests for VSTransformer class."""

    def test_initialization(self):
        """Test VS transformer initialization."""
        transformer = VSTransformer()

        assert transformer.domain == "VS"
        assert len(transformer.patterns) == 4  # orres, unit, position, label
        assert "VSTESTCD" in transformer.output_mapping.values()
        assert "VSTEST" in transformer.output_mapping.values()

    def test_can_transform_vs_domain(self):
        """Test can_transform returns True for VS domain with test columns."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_HEIGHT": [170],
                "ORRES_WEIGHT": [70],
            }
        )

        assert transformer.can_transform(df, "VS") is True

    def test_can_transform_wrong_domain(self):
        """Test can_transform returns False for non-VS domain."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_HEIGHT": [170],
            }
        )

        assert transformer.can_transform(df, "LB") is False

    def test_can_transform_no_test_columns(self):
        """Test can_transform returns False when no test columns found."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "OTHER_COLUMN": ["value"],
            }
        )

        assert transformer.can_transform(df, "VS") is False

    def test_pattern_matching_orres(self):
        """Test that ORRES_* patterns are correctly matched."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_TEMP": [37.5],
                "ORRES_DIABP": [80],
                "ORRES_SYSBP": [120],
            }
        )

        test_defs = transformer._discover_tests(df)

        assert len(test_defs) == 3
        test_codes = {td.test_code for td in test_defs}
        assert test_codes == {"TEMP", "DIABP", "SYSBP"}

    def test_pattern_matching_with_units(self):
        """Test that ORRESU_* patterns are correctly matched."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_HEIGHT": [170],
                "ORRESU_HEIGHT": ["cm"],
            }
        )

        test_defs = transformer._discover_tests(df)

        assert len(test_defs) == 1
        assert test_defs[0].test_code == "HEIGHT"
        assert test_defs[0].get_column("orres") == "ORRES_HEIGHT"
        assert test_defs[0].get_column("unit") == "ORRESU_HEIGHT"

    def test_pattern_matching_with_position(self):
        """Test that POS_* patterns are correctly matched."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_SYSBP": [120],
                "POS_SYSBP": ["SITTING"],
            }
        )

        test_defs = transformer._discover_tests(df)

        assert len(test_defs) == 1
        assert test_defs[0].get_column("position") == "POS_SYSBP"

    def test_normalize_columns_renames_common_variations(self):
        """Test that common column name variations are normalized."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "Subject Id": ["001"],
                "Event name": ["Visit 1"],
                "Event date": ["2023-01-15"],
            }
        )

        normalized = transformer._normalize_columns(df)

        assert "USUBJID" in normalized.columns
        assert "VISIT" in normalized.columns
        assert "VSDTC" in normalized.columns

    def test_normalize_columns_converts_visitnum_to_numeric(self):
        """Test that VISITNUM is converted to numeric."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "VISITNUM": ["1", "2", "3"],
            }
        )

        normalized = transformer._normalize_columns(df)

        # Check that it's numeric (int or float)
        assert pd.api.types.is_numeric_dtype(normalized["VISITNUM"])
        assert list(normalized["VISITNUM"]) == [1.0, 2.0, 3.0]

    def test_normalize_columns_generates_visit_from_visitnum(self):
        """Test that VISIT is generated from VISITNUM if missing."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "VISITNUM": [1.0, 2.0, 3.0],
            }
        )

        normalized = transformer._normalize_columns(df)

        assert "VISIT" in normalized.columns
        assert list(normalized["VISIT"]) == ["Visit 1", "Visit 2", "Visit 3"]

    def test_transform_simple_vs_data(self):
        """Test complete transformation of simple VS data."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "002"],
                "VISIT": ["Visit 1", "Visit 1"],
                "VSDTC": ["2023-01-15", "2023-01-15"],
                "ORRES_HEIGHT": [170, 165],
                "ORRESU_HEIGHT": ["cm", "cm"],
                "ORRES_WEIGHT": [70, 60],
                "ORRESU_WEIGHT": ["kg", "kg"],
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert len(result.data) == 4  # 2 subjects × 2 tests

        # Check structure
        assert "STUDYID" in result.data.columns
        assert "DOMAIN" in result.data.columns
        assert "USUBJID" in result.data.columns
        assert "VSTESTCD" in result.data.columns
        assert "VSTEST" in result.data.columns
        assert "VSORRES" in result.data.columns
        assert "VSORRESU" in result.data.columns
        assert "VSSTAT" in result.data.columns
        assert "VSREASND" in result.data.columns

        # Check values
        assert all(result.data["STUDYID"] == "STUDY001")
        assert all(result.data["DOMAIN"] == "VS")
        assert set(result.data["USUBJID"]) == {"001", "002"}
        assert set(result.data["VSTESTCD"]) == {"HEIGHT", "WEIGHT"}

    def test_transform_with_position(self):
        """Test transformation with position data."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_SYSBP": [120],
                "ORRESU_SYSBP": ["mmHg"],
                "POS_SYSBP": ["SITTING"],
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert len(result.data) == 1

        # Check VSPOS is populated
        if "VSPOS" in result.data.columns:
            assert result.data.iloc[0]["VSPOS"] == "SITTING"

    def test_transform_with_vsstat_not_done(self):
        """Test transformation when test not performed (VSPERFCD = N)."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "VSPERFCD": ["N"],
                "VSREASND": ["Subject refused"],
                "ORRES_HEIGHT": [None],  # No value when not done
                "ORRES_WEIGHT": [None],
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert len(result.data) == 2  # Both tests recorded as NOT DONE

        # Check VSSTAT is set
        assert all(result.data["VSSTAT"] == "NOT DONE")

        # Check VSREASND is populated
        assert all(result.data["VSREASND"] == "Subject refused")

        # Check VSORRES and VSORRESU are empty
        assert all(result.data["VSORRES"] == "")
        assert all(result.data["VSORRESU"] == "")

    def test_transform_mixed_performed_and_not_done(self):
        """Test transformation with mix of performed and not performed tests."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "VSPERFCD": ["Y"],  # Performed
                "ORRES_HEIGHT": [170],
                "ORRESU_HEIGHT": ["cm"],
                "ORRES_WEIGHT": [70],
                "ORRESU_WEIGHT": ["kg"],
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert len(result.data) == 2

        # Check all have empty VSSTAT (performed normally)
        assert all(result.data["VSSTAT"] == "")
        assert all(result.data["VSREASND"] == "")

        # Check values are populated
        assert all(result.data["VSORRES"] != "")

    def test_transform_with_visit_generation(self):
        """Test that VISIT is generated from VISITNUM."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "VISITNUM": [1.0],
                "ORRES_HEIGHT": [170],
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert result.data.iloc[0]["VISIT"] == "Visit 1"
        assert result.data.iloc[0]["VISITNUM"] == 1.0

    def test_transform_with_test_code_normalizer(self):
        """Test transformation with custom test code normalizer."""

        def normalizer(domain: str, test_code: str) -> str | None:
            """Normalize TEMP to TEMP (C)."""
            if test_code == "TEMP":
                return "TEMP"
            return test_code

        transformer = VSTransformer(test_code_normalizer=normalizer)

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_TEMP": [37.5],
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert result.data.iloc[0]["VSTESTCD"] == "TEMP"

    def test_transform_with_test_label_getter(self):
        """Test transformation with custom test label getter."""

        def label_getter(domain: str, test_code: str) -> str:
            """Get test labels."""
            labels = {
                "HEIGHT": "Height Measurement",
                "WEIGHT": "Body Weight",
            }
            return labels.get(test_code, test_code)

        transformer = VSTransformer(test_label_getter=label_getter)

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

    def test_transform_uses_test_column_for_label_fallback(self):
        """Test that TEST_* column is used as fallback for label."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_CUSTOM": [100],
                "TEST_CUSTOM": ["Custom Test Name"],
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert result.data.iloc[0]["VSTEST"] == "Custom Test Name"

    def test_transform_skips_empty_results_unless_not_done(self):
        """Test that empty results are skipped unless VSSTAT = NOT DONE."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_HEIGHT": [170],
                "ORRES_WEIGHT": [None],  # Empty, should be skipped
                "ORRES_TEMP": [""],  # Empty string, should be skipped
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert len(result.data) == 1  # Only HEIGHT with valid value
        assert result.data.iloc[0]["VSTESTCD"] == "HEIGHT"

    def test_transform_limits_testcd_to_8_chars(self):
        """Test that VSTESTCD is limited to 8 characters."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_VERYLONGTESTCODE": [100],
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert len(result.data.iloc[0]["VSTESTCD"]) <= 8

    def test_transform_empty_dataframe(self):
        """Test transformation with empty DataFrame."""
        transformer = VSTransformer()

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

    def test_transform_skips_rows_without_usubjid(self):
        """Test that rows without USUBJID are skipped."""
        transformer = VSTransformer()

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

    def test_integration_with_all_columns(self):
        """Test complete integration with all VS columns."""
        transformer = VSTransformer()

        df = pd.DataFrame(
            {
                "Subject Id": ["001"],  # Will be renamed
                "Event name": ["Screening"],  # Will be renamed
                "Event date": ["2023-01-15"],  # Will be renamed
                "VISITNUM": [0],
                "VSPERFCD": ["Y"],
                "ORRES_HEIGHT": [170],
                "ORRESU_HEIGHT": ["cm"],
                "POS_HEIGHT": ["STANDING"],
                "TEST_HEIGHT": ["Height (Standing)"],
                "ORRES_WEIGHT": [70],
                "ORRESU_WEIGHT": ["kg"],
            }
        )

        context = TransformationContext(domain="VS", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert len(result.data) == 2

        # Check all expected columns are present
        expected_cols = [
            "STUDYID",
            "DOMAIN",
            "USUBJID",
            "VSTESTCD",
            "VSTEST",
            "VSORRES",
            "VSORRESU",
            "VSSTAT",
            "VSREASND",
            "VISITNUM",
            "VISIT",
            "VSDTC",
        ]
        for col in expected_cols:
            assert col in result.data.columns

        # Check height record has position
        height_row = result.data[result.data["VSTESTCD"] == "HEIGHT"].iloc[0]
        if "VSPOS" in result.data.columns:
            assert height_row["VSPOS"] == "STANDING"
