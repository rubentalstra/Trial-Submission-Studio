"""Unit tests for LB (Laboratory Test Results) transformer.

Tests for LBTransformer class covering LB-specific patterns and logic.
"""

import pandas as pd
import pytest

from cdisc_transpiler.transformations.findings import LBTransformer
from cdisc_transpiler.transformations.base import TransformationContext


class TestLBTransformer:
    """Tests for LBTransformer class."""

    def test_initialization(self):
        """Test LB transformer initialization."""
        transformer = LBTransformer()

        assert transformer.domain == "LB"
        assert len(transformer.patterns) == 10  # Multiple patterns for LB
        assert "LBTESTCD" in transformer.output_mapping.values()
        assert "LBTEST" in transformer.output_mapping.values()

    def test_can_transform_lb_domain(self):
        """Test can_transform returns True for LB domain with test columns."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_GLUC": [5.5],
                "ORRES_HGB": [14.2],
            }
        )

        assert transformer.can_transform(df, "LB") is True

    def test_can_transform_wrong_domain(self):
        """Test can_transform returns False for non-LB domain."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_GLUC": [5.5],
            }
        )

        assert transformer.can_transform(df, "VS") is False

    def test_pattern_matching_orres_prefix(self):
        """Test that ORRES_* patterns are correctly matched."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_GLUC": [5.5],
                "ORRES_HGB": [14.2],
                "ORRES_WBC": [7.5],
            }
        )

        test_defs = transformer._discover_tests(df)

        assert len(test_defs) == 3
        test_codes = {td.test_code for td in test_defs}
        assert test_codes == {"GLUC", "HGB", "WBC"}

    def test_pattern_matching_orres_suffix(self):
        """Test that *ORRES patterns are correctly matched."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "GLUCORRES": [5.5],
                "HGBORRES": [14.2],
            }
        )

        test_defs = transformer._discover_tests(df)

        assert len(test_defs) == 2
        test_codes = {td.test_code for td in test_defs}
        assert test_codes == {"GLUC", "HGB"}

    def test_pattern_matching_long_format(self):
        """Test that long format patterns are correctly matched."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "Glucose result or finding in original units": [5.5],
                "Hemoglobin result or finding in original units": [14.2],
            }
        )

        test_defs = transformer._discover_tests(df)

        assert len(test_defs) == 2
        test_codes = {td.test_code for td in test_defs}
        assert test_codes == {"GLUCOSE", "HEMOGLOBIN"}

    def test_pattern_matching_with_units(self):
        """Test that unit patterns are correctly matched."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_GLUC": [5.5],
                "ORRESU_GLUC": ["mmol/L"],
                "Hemoglobin unit": ["g/dL"],
                "HemoglobinORRES": [14.2],
            }
        )

        test_defs = transformer._discover_tests(df)

        # Check GLUC has both orres and unit
        gluc_def = [td for td in test_defs if td.test_code == "GLUC"][0]
        assert gluc_def.get_column("orres") == "ORRES_GLUC"
        assert gluc_def.get_column("unit") == "ORRESU_GLUC"

        # Check HEMOGLOBIN has both
        hgb_def = [td for td in test_defs if td.test_code == "HEMOGLOBIN"][0]
        assert hgb_def.get_column("unit") == "Hemoglobin unit"

    def test_pattern_matching_with_normal_ranges(self):
        """Test that normal range patterns are correctly matched."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_GLUC": [5.5],
                "ORNR_GLUC_Lower": [3.9],
                "ORNR_GLUC_Upper": [6.1],
                "Hemoglobin range (lower limit)": [12.0],
                "Hemoglobin range (upper limit)": [16.0],
                "HemoglobinORRES": [14.2],
            }
        )

        test_defs = transformer._discover_tests(df)

        # Check GLUC has normal ranges
        gluc_def = [td for td in test_defs if td.test_code == "GLUC"][0]
        assert gluc_def.get_column("nrlo") == "ORNR_GLUC_Lower"
        assert gluc_def.get_column("nrhi") == "ORNR_GLUC_Upper"

        # Check HEMOGLOBIN has normal ranges
        hgb_def = [td for td in test_defs if td.test_code == "HEMOGLOBIN"][0]
        assert hgb_def.get_column("nrlo") == "Hemoglobin range (lower limit)"
        assert hgb_def.get_column("nrhi") == "Hemoglobin range (upper limit)"

    def test_normalize_columns_renames_common_variations(self):
        """Test that common column name variations are normalized."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "Subject Id": ["001"],
                "Date of blood sample": ["2023-01-15"],
            }
        )

        normalized = transformer._normalize_columns(df)

        assert "USUBJID" in normalized.columns
        assert "LBDTC" in normalized.columns

    def test_transform_simple_lb_data(self):
        """Test complete transformation of simple LB data."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "002"],
                "LBDTC": ["2023-01-15", "2023-01-15"],
                "ORRES_GLUC": [5.5, 6.2],
                "ORRESU_GLUC": ["mmol/L", "mmol/L"],
                "ORRES_HGB": [14.2, 13.5],
                "ORRESU_HGB": ["g/dL", "g/dL"],
            }
        )

        context = TransformationContext(domain="LB", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert len(result.data) == 4  # 2 subjects × 2 tests

        # Check structure
        assert "STUDYID" in result.data.columns
        assert "DOMAIN" in result.data.columns
        assert "USUBJID" in result.data.columns
        assert "LBTESTCD" in result.data.columns
        assert "LBTEST" in result.data.columns
        assert "LBORRES" in result.data.columns
        assert "LBORRESU" in result.data.columns

        # Check values
        assert all(result.data["STUDYID"] == "STUDY001")
        assert all(result.data["DOMAIN"] == "LB")
        assert set(result.data["USUBJID"]) == {"001", "002"}
        assert set(result.data["LBTESTCD"]) == {"GLUC", "HGB"}

    def test_transform_with_normal_ranges(self):
        """Test transformation with normal range data."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_GLUC": [5.5],
                "ORRESU_GLUC": ["mmol/L"],
                "ORNR_GLUC_Lower": [3.9],
                "ORNR_GLUC_Upper": [6.1],
            }
        )

        context = TransformationContext(domain="LB", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert len(result.data) == 1

        # Check normal ranges are populated
        assert "LBORNRLO" in result.data.columns
        assert "LBORNRHI" in result.data.columns
        assert result.data.iloc[0]["LBORNRLO"] == "3.9"
        assert result.data.iloc[0]["LBORNRHI"] == "6.1"

    def test_transform_glucu_to_gluc_normalization(self):
        """Test that GLUCU is normalized to GLUC."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_GLUCU": [5.5],
            }
        )

        context = TransformationContext(domain="LB", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert result.data.iloc[0]["LBTESTCD"] == "GLUC"

    def test_transform_with_test_code_normalizer(self):
        """Test transformation with custom test code normalizer."""

        def normalizer(domain: str, test_code: str) -> str | None:
            """Normalize test codes."""
            if test_code == "CUSTOM":
                return "NORMALIZED"
            return test_code

        transformer = LBTransformer(test_code_normalizer=normalizer)

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_CUSTOM": [100],
            }
        )

        context = TransformationContext(domain="LB", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert result.data.iloc[0]["LBTESTCD"] == "NORMALIZ"  # Truncated to 8 chars

    def test_transform_with_test_label_getter(self):
        """Test transformation with custom test label getter."""

        def label_getter(domain: str, test_code: str) -> str:
            """Get test labels."""
            labels = {
                "GLUC": "Glucose",
                "HGB": "Hemoglobin",
            }
            return labels.get(test_code, test_code)

        transformer = LBTransformer(test_label_getter=label_getter)

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_GLUC": [5.5],
            }
        )

        context = TransformationContext(domain="LB", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert result.data.iloc[0]["LBTEST"] == "Glucose"

    def test_transform_skips_header_rows(self):
        """Test that header rows starting with ORRES are skipped."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "002"],
                "ORRES_GLUC": [5.5, "ORRES_GLUC"],  # Second row is header
            }
        )

        context = TransformationContext(domain="LB", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert len(result.data) == 1  # Only first row with valid data
        assert result.data.iloc[0]["USUBJID"] == "001"

    def test_transform_skips_empty_results(self):
        """Test that empty results are skipped."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "ORRES_GLUC": [5.5],
                "ORRES_HGB": [None],  # Empty
                "ORRES_WBC": [""],  # Empty string
            }
        )

        context = TransformationContext(domain="LB", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert len(result.data) == 1  # Only GLUC with valid value
        assert result.data.iloc[0]["LBTESTCD"] == "GLUC"

    def test_transform_handles_date_variations(self):
        """Test that various date column names are handled."""
        transformer = LBTransformer()

        # Test with Event date
        df1 = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "Event date": ["2023-01-15"],
                "ORRES_GLUC": [5.5],
            }
        )

        context = TransformationContext(domain="LB", study_id="STUDY001")
        result1 = transformer.transform(df1, context)
        assert result1.success
        assert result1.data.iloc[0]["LBDTC"] == "2023-01-15"

        # Test with Date of blood sample
        df2 = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "Date of blood sample": ["2023-01-16"],
                "ORRES_GLUC": [5.5],
            }
        )

        result2 = transformer.transform(df2, context)
        assert result2.success
        assert result2.data.iloc[0]["LBDTC"] == "2023-01-16"

    def test_transform_empty_dataframe(self):
        """Test transformation with empty DataFrame."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": [],
                "ORRES_GLUC": [],
            }
        )

        context = TransformationContext(domain="LB", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.applied
        assert len(result.data) == 0

    def test_transform_skips_rows_without_usubjid(self):
        """Test that rows without USUBJID are skipped."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "", "003", "subjectid"],  # Empty and header values
                "ORRES_GLUC": [5.5, 6.0, 5.8, 6.2],
            }
        )

        context = TransformationContext(domain="LB", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert len(result.data) == 2  # Only 2 valid rows (001 and 003)
        assert set(result.data["USUBJID"]) == {"001", "003"}

    def test_integration_with_all_columns(self):
        """Test complete integration with all LB columns."""
        transformer = LBTransformer()

        df = pd.DataFrame(
            {
                "Subject Id": ["001"],  # Will be renamed
                "Date of blood sample": ["2023-01-15"],  # Will be renamed
                "ORRES_GLUC": [5.5],
                "ORRESU_GLUC": ["mmol/L"],
                "ORNR_GLUC_Lower": [3.9],
                "ORNR_GLUC_Upper": [6.1],
                "TEST_GLUC": ["Glucose Test"],
            }
        )

        context = TransformationContext(domain="LB", study_id="STUDY001")
        result = transformer.transform(df, context)

        assert result.success
        assert len(result.data) == 1

        # Check all expected columns are present
        expected_cols = [
            "STUDYID",
            "DOMAIN",
            "USUBJID",
            "LBTESTCD",
            "LBTEST",
            "LBORRES",
            "LBORRESU",
            "LBORNRLO",
            "LBORNRHI",
            "LBDTC",
        ]
        for col in expected_cols:
            assert col in result.data.columns

        # Check values
        row = result.data.iloc[0]
        assert row["USUBJID"] == "001"
        assert row["LBDTC"] == "2023-01-15"
        assert row["LBTESTCD"] == "GLUC"
        assert row["LBTEST"] == "Glucose Test"
        assert row["LBORRES"] == "5.5"
        assert row["LBORRESU"] == "mmol/L"
        assert row["LBORNRLO"] == "3.9"
        assert row["LBORNRHI"] == "6.1"
