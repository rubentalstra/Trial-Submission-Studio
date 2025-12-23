"""Unit tests for study day calculator transformer.

Tests for StudyDayCalculator class.
"""

import pandas as pd

from cdisc_transpiler.transformations import TransformationContext
from cdisc_transpiler.transformations.dates import StudyDayCalculator


class TestStudyDayCalculator:
    """Tests for StudyDayCalculator class."""

    def test_initialization(self):
        """Test calculator initialization."""
        calculator = StudyDayCalculator()
        assert calculator is not None

    def test_can_transform_with_dy_columns(self):
        """Test can_transform returns True for data with DY columns and USUBJID."""
        calculator = StudyDayCalculator()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "AESTDTC": ["2023-09-15"],
                "AESTDY": [None],
            }
        )

        assert calculator.can_transform(df, "AE") is True

    def test_can_transform_without_dy_columns(self):
        """Test can_transform returns False for data without DY columns."""
        calculator = StudyDayCalculator()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "AESTDTC": ["2023-09-15"],
            }
        )

        assert calculator.can_transform(df, "AE") is False

    def test_can_transform_without_usubjid(self):
        """Test can_transform returns False for data without USUBJID."""
        calculator = StudyDayCalculator()

        df = pd.DataFrame(
            {
                "AESTDTC": ["2023-09-15"],
                "AESTDY": [None],
            }
        )

        assert calculator.can_transform(df, "AE") is False

    def test_transform_calculates_study_days(self):
        """Test basic study day calculation."""
        calculator = StudyDayCalculator()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "001", "001"],
                "AESTDTC": ["2023-01-15", "2023-01-16", "2023-01-14"],
                "AESTDY": [None, None, None],
            }
        )

        context = TransformationContext(
            domain="AE",
            study_id="STUDY001",
            metadata={"reference_starts": {"001": "2023-01-15"}},
        )

        result = calculator.transform(df, context)

        assert result.success
        assert result.applied
        assert "AESTDY" in result.metadata["dy_columns_calculated"]

        # Day 1 = same as reference date
        assert result.data["AESTDY"].values[0] == 1
        # Day 2 = one day after reference
        assert result.data["AESTDY"].values[1] == 2
        # Day -1 = one day before reference (no Day 0 in SDTM)
        assert result.data["AESTDY"].values[2] == -1

    def test_transform_no_day_zero(self):
        """Test that there is no Day 0 in SDTM."""
        calculator = StudyDayCalculator()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "001", "001"],
                "AESTDTC": ["2023-01-14", "2023-01-15", "2023-01-16"],
                "AESTDY": [None, None, None],
            }
        )

        context = TransformationContext(
            domain="AE",
            study_id="STUDY001",
            metadata={"reference_starts": {"001": "2023-01-15"}},
        )

        result = calculator.transform(df, context)

        assert result.success
        # -1, 1, 2 (no 0)
        days = result.data["AESTDY"].tolist()
        assert 0 not in days
        assert days == [-1, 1, 2]

    def test_transform_multiple_subjects(self):
        """Test study day calculation with multiple subjects."""
        calculator = StudyDayCalculator()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "001", "002", "002"],
                "AESTDTC": ["2023-01-15", "2023-01-16", "2023-02-01", "2023-02-05"],
                "AESTDY": [None, None, None, None],
            }
        )

        context = TransformationContext(
            domain="AE",
            study_id="STUDY001",
            metadata={
                "reference_starts": {
                    "001": "2023-01-15",
                    "002": "2023-02-01",
                }
            },
        )

        result = calculator.transform(df, context)

        assert result.success
        # Subject 001: Days 1, 2
        assert result.data.loc[0, "AESTDY"] == 1
        assert result.data.loc[1, "AESTDY"] == 2
        # Subject 002: Days 1, 5
        assert result.data.loc[2, "AESTDY"] == 1
        assert result.data.loc[3, "AESTDY"] == 5

    def test_transform_handles_missing_dates(self):
        """Test handling of missing date values."""
        calculator = StudyDayCalculator()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "001", "001"],
                "AESTDTC": ["2023-01-15", None, ""],
                "AESTDY": [None, None, None],
            }
        )

        context = TransformationContext(
            domain="AE",
            study_id="STUDY001",
            metadata={"reference_starts": {"001": "2023-01-15"}},
        )

        result = calculator.transform(df, context)

        assert result.success
        # First row has valid date
        assert result.data["AESTDY"].values[0] == 1
        # Missing dates should result in None/NaN
        assert pd.isna(result.data["AESTDY"].values[1])
        assert pd.isna(result.data["AESTDY"].values[2])

    def test_transform_handles_subject_without_reference(self):
        """Test handling of subjects without reference start dates."""
        calculator = StudyDayCalculator()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "002"],
                "AESTDTC": ["2023-01-15", "2023-02-01"],
                "AESTDY": [None, None],
            }
        )

        context = TransformationContext(
            domain="AE",
            study_id="STUDY001",
            metadata={
                "reference_starts": {"001": "2023-01-15"}
            },  # No reference for 002
        )

        result = calculator.transform(df, context)

        assert result.success
        # Subject 001 has reference
        assert result.data["AESTDY"].values[0] == 1
        # Subject 002 has no reference
        assert pd.isna(result.data["AESTDY"].values[1])

    def test_transform_multiple_dy_columns(self):
        """Test calculation of multiple DY columns."""
        calculator = StudyDayCalculator()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "AESTDTC": ["2023-01-20"],
                "AESTDY": [None],
                "AEENDTC": ["2023-01-25"],
                "AEENDY": [None],
            }
        )

        context = TransformationContext(
            domain="AE",
            study_id="STUDY001",
            metadata={"reference_starts": {"001": "2023-01-15"}},
        )

        result = calculator.transform(df, context)

        assert result.success
        assert len(result.metadata["dy_columns_calculated"]) == 2
        assert "AESTDY" in result.metadata["dy_columns_calculated"]
        assert "AEENDY" in result.metadata["dy_columns_calculated"]

        # Check calculations
        assert result.data["AESTDY"].values[0] == 6  # 5 days after + 1
        assert result.data["AEENDY"].values[0] == 11  # 10 days after + 1

    def test_transform_without_reference_starts(self):
        """Test transformation without reference start dates in context."""
        calculator = StudyDayCalculator()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "AESTDTC": ["2023-01-15"],
                "AESTDY": [None],
            }
        )

        context = TransformationContext(domain="AE", study_id="STUDY001")

        result = calculator.transform(df, context)

        assert not result.applied
        assert "No reference start dates" in result.message
        assert len(result.warnings) > 0

    def test_transform_without_matching_dtc_column(self):
        """Test when DY column exists but corresponding DTC column doesn't."""
        calculator = StudyDayCalculator()

        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "AESTDY": [None],  # No AESTDTC column
            }
        )

        context = TransformationContext(
            domain="AE",
            study_id="STUDY001",
            metadata={"reference_starts": {"001": "2023-01-15"}},
        )

        result = calculator.transform(df, context)

        assert not result.applied
        assert "No matching DTC/DY column pairs found" in result.message

    def test_transform_preserves_other_columns(self):
        """Test that non-DY columns are preserved unchanged."""
        calculator = StudyDayCalculator()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "002"],
                "SEX": ["M", "F"],
                "AGE": [30, 25],
                "AESTDTC": ["2023-01-15", "2023-01-20"],
                "AESTDY": [None, None],
            }
        )

        context = TransformationContext(
            domain="AE",
            study_id="STUDY001",
            metadata={
                "reference_starts": {
                    "001": "2023-01-15",
                    "002": "2023-01-15",
                }
            },
        )

        result = calculator.transform(df, context)

        assert result.success
        # Check that non-DY columns are unchanged
        assert list(result.data["USUBJID"]) == ["001", "002"]
        assert list(result.data["SEX"]) == ["M", "F"]
        assert list(result.data["AGE"]) == [30, 25]

    def test_transform_metadata(self):
        """Test that transformation metadata is populated correctly."""
        calculator = StudyDayCalculator()

        df = pd.DataFrame(
            {
                "USUBJID": ["001", "002", "003"],
                "AESTDTC": ["2023-01-15", "2023-01-16", "2023-01-17"],
                "AESTDY": [None, None, None],
            }
        )

        context = TransformationContext(
            domain="AE",
            study_id="STUDY001",
            metadata={
                "reference_starts": {
                    "001": "2023-01-15",
                    "002": "2023-01-15",
                }
            },
        )

        result = calculator.transform(df, context)

        assert result.metadata["input_rows"] == 3
        assert result.metadata["output_rows"] == 3
        assert result.metadata["subjects_with_reference"] == 2
        assert "AESTDY" in result.metadata["dy_columns_calculated"]

    def test_compute_dy_edge_cases(self):
        """Test edge cases in study day computation."""
        calculator = StudyDayCalculator()

        # Test with various date differences
        test_cases = [
            ("2023-01-15", "2023-01-15", 1),  # Same day = Day 1
            ("2023-01-15", "2023-01-16", 2),  # Next day = Day 2
            ("2023-01-15", "2023-01-14", -1),  # Previous day = Day -1
            ("2023-01-15", "2023-01-10", -5),  # 5 days before = Day -5
            ("2023-01-01", "2023-12-31", 365),  # Year later
        ]

        for ref_date, event_date, expected_day in test_cases:
            df = pd.DataFrame(
                {
                    "USUBJID": ["001"],
                    "AESTDTC": [event_date],
                    "AESTDY": [None],
                }
            )

            context = TransformationContext(
                domain="AE",
                study_id="TEST",
                metadata={"reference_starts": {"001": ref_date}},
            )

            result = calculator.transform(df, context)
            assert result.data["AESTDY"].values[0] == expected_day, (
                f"Expected day {expected_day} for {event_date} vs {ref_date}, got {result.data['AESTDY'].values[0]}"
            )
