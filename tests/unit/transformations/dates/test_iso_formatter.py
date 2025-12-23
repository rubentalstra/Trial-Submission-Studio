"""Unit tests for ISO date formatter transformer.

Tests for ISODateFormatter class.
"""

import pandas as pd

from cdisc_transpiler.transformations import TransformationContext
from cdisc_transpiler.transformations.dates import ISODateFormatter


class TestISODateFormatter:
    """Tests for ISODateFormatter class."""

    def test_initialization(self):
        """Test formatter initialization."""
        formatter = ISODateFormatter()
        assert formatter is not None

    def test_can_transform_with_dtc_columns(self):
        """Test can_transform returns True for data with DTC columns."""
        formatter = ISODateFormatter()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "AESTDTC": ["2023-09-15"],
                "AEENDTC": ["2023-09-20"],
            }
        )

        assert formatter.can_transform(df, "AE") is True

    def test_can_transform_with_dur_columns(self):
        """Test can_transform returns True for data with DUR columns."""
        formatter = ISODateFormatter()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "AEDUR": ["2 hours"],
            }
        )

        assert formatter.can_transform(df, "AE") is True

    def test_can_transform_without_date_columns(self):
        """Test can_transform returns False for data without date columns."""
        formatter = ISODateFormatter()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "SEX": ["M"],
            }
        )

        assert formatter.can_transform(df, "DM") is False

    def test_transform_normalizes_dtc_columns(self):
        """Test that DTC columns are normalized to ISO 8601."""
        formatter = ISODateFormatter()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "002"],
                "AESTDTC": ["09/15/2023", "2023-09-16"],
                "AEENDTC": ["09/20/2023", "2023-09-21"],
            }
        )

        context = TransformationContext(domain="AE", study_id="STUDY001")
        result = formatter.transform(df, context)

        assert result.success
        assert result.applied
        assert "AESTDTC" in result.metadata["dtc_columns_processed"]
        assert "AEENDTC" in result.metadata["dtc_columns_processed"]

        # Check that dates are in ISO 8601 format
        assert "2023-09-15" in result.data["AESTDTC"].values[0]
        assert "2023-09-16" in result.data["AESTDTC"].values[1]

    def test_transform_handles_partial_dates(self):
        """Test handling of partial dates (YYYY, YYYY-MM)."""
        formatter = ISODateFormatter()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "002", "003"],
                "BRTHDTC": ["2023", "2023-09", "2023-09-15"],
            }
        )

        context = TransformationContext(domain="DM", study_id="STUDY001")
        result = formatter.transform(df, context)

        assert result.success
        # Partial dates are converted to full timestamps by pandas
        assert "2023-01-01" in result.data["BRTHDTC"].values[0]  # Year only → Jan 1
        assert (
            "2023-09-01" in result.data["BRTHDTC"].values[1]
        )  # Year-month → 1st of month
        assert "2023-09-15" in result.data["BRTHDTC"].values[2]  # Full date preserved

    def test_transform_handles_unknown_components(self):
        """Test handling of unknown date components (NK, UN, UNK)."""
        formatter = ISODateFormatter()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "002", "003"],
                "BRTHDTC": ["2023-10-NK", "2023-NK-NK", "2023-UN-15"],
            }
        )

        context = TransformationContext(domain="DM", study_id="STUDY001")
        result = formatter.transform(df, context)

        assert result.success
        # Unknown components should be omitted
        assert result.data["BRTHDTC"].values[0] == "2023-10"  # Day unknown
        assert result.data["BRTHDTC"].values[1] == "2023"  # Month and day unknown
        assert result.data["BRTHDTC"].values[2] == "2023-15"  # Cleaned up

    def test_transform_handles_missing_values(self):
        """Test handling of missing values (None, NaN, empty string)."""
        formatter = ISODateFormatter()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "002", "003", "004"],
                "AESTDTC": ["2023-09-15", None, "", pd.NA],
            }
        )

        context = TransformationContext(domain="AE", study_id="STUDY001")
        result = formatter.transform(df, context)

        assert result.success
        # Missing values should become empty strings
        assert "2023-09-15" in result.data["AESTDTC"].values[0]
        assert result.data["AESTDTC"].values[1] == ""
        assert result.data["AESTDTC"].values[2] == ""
        assert result.data["AESTDTC"].values[3] == ""

    def test_transform_normalizes_dur_columns(self):
        """Test that DUR columns are normalized to ISO 8601 duration format."""
        formatter = ISODateFormatter()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "002", "003"],
                "AEDUR": ["2 hours", "30 minutes", "1 hour 30 minutes"],
            }
        )

        context = TransformationContext(domain="AE", study_id="STUDY001")
        result = formatter.transform(df, context)

        assert result.success
        assert "AEDUR" in result.metadata["dur_columns_processed"]

        # Check ISO 8601 duration format
        assert result.data["AEDUR"].values[0] == "PT2H"
        assert result.data["AEDUR"].values[1] == "PT30M"
        assert result.data["AEDUR"].values[2] == "PT1H30M"

    def test_transform_handles_multiple_dtc_columns(self):
        """Test handling of multiple DTC columns."""
        formatter = ISODateFormatter()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "AESTDTC": ["2023-09-15"],
                "AEENDTC": ["2023-09-20"],
                "RFSTDTC": ["2023-01-01"],
            }
        )

        context = TransformationContext(domain="AE", study_id="STUDY001")
        result = formatter.transform(df, context)

        assert result.success
        assert len(result.metadata["dtc_columns_processed"]) == 3
        assert all(
            col in result.metadata["dtc_columns_processed"]
            for col in ["AESTDTC", "AEENDTC", "RFSTDTC"]
        )

    def test_transform_handles_both_dtc_and_dur(self):
        """Test handling of both DTC and DUR columns."""
        formatter = ISODateFormatter()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "AESTDTC": ["2023-09-15"],
                "AEDUR": ["2 hours"],
            }
        )

        context = TransformationContext(domain="AE", study_id="STUDY001")
        result = formatter.transform(df, context)

        assert result.success
        assert len(result.metadata["dtc_columns_processed"]) == 1
        assert len(result.metadata["dur_columns_processed"]) == 1
        assert "date/time columns" in result.message
        assert "duration columns" in result.message

    def test_transform_with_no_date_columns(self):
        """Test transformation when no date columns present."""
        formatter = ISODateFormatter()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "SEX": ["M"],
            }
        )

        context = TransformationContext(domain="DM", study_id="STUDY001")
        result = formatter.transform(df, context)

        assert not result.applied
        assert "No date/time or duration columns found" in result.message

    def test_transform_preserves_other_columns(self):
        """Test that non-date columns are preserved unchanged."""
        formatter = ISODateFormatter()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "002"],
                "SEX": ["M", "F"],
                "AGE": [30, 25],
                "AESTDTC": ["2023-09-15", "2023-09-16"],
            }
        )

        context = TransformationContext(domain="AE", study_id="STUDY001")
        result = formatter.transform(df, context)

        assert result.success
        # Check that non-date columns are unchanged
        assert list(result.data["USUBJID"]) == ["001", "002"]
        assert list(result.data["SEX"]) == ["M", "F"]
        assert list(result.data["AGE"]) == [30, 25]

    def test_transform_metadata(self):
        """Test that transformation metadata is populated correctly."""
        formatter = ISODateFormatter()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "002", "003"],
                "AESTDTC": ["2023-09-15", "2023-09-16", "2023-09-17"],
                "AEDUR": ["2H", "30M", "1H30M"],
            }
        )

        context = TransformationContext(domain="AE", study_id="STUDY001")
        result = formatter.transform(df, context)

        assert result.metadata["input_rows"] == 3
        assert result.metadata["output_rows"] == 3
        assert "AESTDTC" in result.metadata["dtc_columns_processed"]
        assert "AEDUR" in result.metadata["dur_columns_processed"]
