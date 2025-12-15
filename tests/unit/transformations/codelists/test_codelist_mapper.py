"""Tests for CodelistMapperTransformer."""

import pandas as pd
import pytest

from cdisc_transpiler.domain.entities.study_metadata import (
    CodeList,
    CodeListValue,
    StudyMetadata,
)
from cdisc_transpiler.transformations import TransformationContext
from cdisc_transpiler.transformations.codelists import CodelistMapperTransformer


@pytest.fixture
def sex_codelist():
    """Create a SEX codelist for testing."""
    return CodeList(
        format_name="SEX",
        values=[
            CodeListValue(code_value="M", code_text="Male", data_type="text"),
            CodeListValue(code_value="F", code_text="Female", data_type="text"),
            CodeListValue(code_value="U", code_text="Unknown", data_type="text"),
        ],
    )


@pytest.fixture
def race_codelist():
    """Create a RACE codelist for testing."""
    return CodeList(
        format_name="RACE",
        values=[
            CodeListValue(code_value="1", code_text="White", data_type="text"),
            CodeListValue(
                code_value="2", code_text="Black or African American", data_type="text"
            ),
            CodeListValue(code_value="3", code_text="Asian", data_type="text"),
        ],
    )


@pytest.fixture
def metadata_with_codelists(sex_codelist, race_codelist):
    """Create metadata with multiple codelists."""
    return StudyMetadata(codelists={"SEX": sex_codelist, "RACE": race_codelist})


class TestCodelistMapperTransformer:
    """Tests for CodelistMapperTransformer."""

    def test_initialization_with_metadata(self, metadata_with_codelists):
        """Test transformer initialization with metadata."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        assert transformer.metadata == metadata_with_codelists

    def test_initialization_without_metadata(self):
        """Test transformer initialization without metadata."""
        transformer = CodelistMapperTransformer()
        assert transformer.metadata is None

    def test_can_transform_with_metadata_and_cd_column(self, metadata_with_codelists):
        """Test can_transform returns True when metadata exists and DF has *CD column."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        df = pd.DataFrame({"SEXCD": ["M", "F"]})
        assert transformer.can_transform(df, "DM") is True

    def test_can_transform_with_metadata_and_term_column(self, metadata_with_codelists):
        """Test can_transform returns True when metadata exists and DF has *TERM column."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        df = pd.DataFrame({"AETERM": ["Headache", "Nausea"]})
        assert transformer.can_transform(df, "AE") is True

    def test_can_transform_with_metadata_and_decod_column(
        self, metadata_with_codelists
    ):
        """Test can_transform returns True when metadata exists and DF has *DECOD column."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        df = pd.DataFrame({"AEDECOD": ["Headache", "Nausea"]})
        assert transformer.can_transform(df, "AE") is True

    def test_can_transform_without_metadata(self):
        """Test can_transform returns False without metadata."""
        transformer = CodelistMapperTransformer()
        df = pd.DataFrame({"SEXCD": ["M", "F"]})
        assert transformer.can_transform(df, "DM") is False

    def test_can_transform_with_empty_codelists(self):
        """Test can_transform returns False with empty codelists."""
        metadata = StudyMetadata(codelists={})
        transformer = CodelistMapperTransformer(metadata=metadata)
        df = pd.DataFrame({"SEXCD": ["M", "F"]})
        assert transformer.can_transform(df, "DM") is False

    def test_can_transform_without_relevant_columns(self, metadata_with_codelists):
        """Test can_transform returns False without relevant columns."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        df = pd.DataFrame({"AGE": [25, 30], "WEIGHT": [70, 80]})
        assert transformer.can_transform(df, "DM") is False

    def test_transform_without_metadata(self):
        """Test transform without metadata returns unchanged data."""
        transformer = CodelistMapperTransformer()
        df = pd.DataFrame({"SEXCD": ["M", "F"]})
        context = TransformationContext(domain="DM", study_id="STUDY001")

        result = transformer.transform(df, context)

        assert result.applied is False
        assert "No metadata" in result.message
        pd.testing.assert_frame_equal(result.data, df)

    def test_transform_without_mappings_in_context(self, metadata_with_codelists):
        """Test transform without codelist_mappings in context."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        df = pd.DataFrame({"SEXCD": ["M", "F"]})
        context = TransformationContext(domain="DM", study_id="STUDY001")

        result = transformer.transform(df, context)

        assert result.applied is False
        assert "No codelist mappings specified" in result.message
        pd.testing.assert_frame_equal(result.data, df)

    def test_transform_successful_mapping(self, metadata_with_codelists):
        """Test successful code-to-text mapping."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        df = pd.DataFrame({"SEXCD": ["M", "F", "U"]})
        context = TransformationContext(
            domain="DM",
            study_id="STUDY001",
            metadata={"codelist_mappings": {"SEXCD": "SEX"}},
        )

        result = transformer.transform(df, context)

        assert result.applied is True
        assert "Applied 1 codelist mapping" in result.message
        assert result.data["SEXCD"].tolist() == ["Male", "Female", "Unknown"]
        assert result.metadata["applied_mappings"] == ["SEXCD using SEX"]
        assert result.metadata["input_rows"] == 3
        assert result.metadata["output_rows"] == 3

    def test_transform_multiple_columns(self, metadata_with_codelists):
        """Test mapping multiple columns."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        df = pd.DataFrame({"SEXCD": ["M", "F"], "RACECD": ["1", "2"]})
        context = TransformationContext(
            domain="DM",
            study_id="STUDY001",
            metadata={"codelist_mappings": {"SEXCD": "SEX", "RACECD": "RACE"}},
        )

        result = transformer.transform(df, context)

        assert result.applied is True
        assert "Applied 2 codelist mapping" in result.message
        assert result.data["SEXCD"].tolist() == ["Male", "Female"]
        assert result.data["RACECD"].tolist() == ["White", "Black or African American"]
        assert len(result.metadata["applied_mappings"]) == 2

    def test_transform_with_missing_codelist(self, metadata_with_codelists):
        """Test transform with non-existent codelist."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        df = pd.DataFrame({"COUNTRY": ["USA", "CAN"]})
        context = TransformationContext(
            domain="DM",
            study_id="STUDY001",
            metadata={"codelist_mappings": {"COUNTRY": "COUNTRY_CODES"}},
        )

        result = transformer.transform(df, context)

        assert result.applied is False
        assert "No codelist mappings applied" in result.message
        assert len(result.warnings) == 1
        assert "Codelist 'COUNTRY_CODES' not found" in result.warnings[0]
        # Data should remain unchanged
        pd.testing.assert_frame_equal(result.data, df)

    def test_transform_with_missing_column(self, metadata_with_codelists):
        """Test transform when target column doesn't exist."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        df = pd.DataFrame({"AGE": [25, 30]})
        context = TransformationContext(
            domain="DM",
            study_id="STUDY001",
            metadata={"codelist_mappings": {"SEXCD": "SEX"}},
        )

        result = transformer.transform(df, context)

        assert result.applied is False
        assert "SEXCD (column not found)" in result.metadata["skipped_mappings"]

    def test_transform_with_unmapped_values(self, metadata_with_codelists):
        """Test transform with values not in codelist."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        df = pd.DataFrame({"SEXCD": ["M", "X", "F"]})  # "X" not in codelist
        context = TransformationContext(
            domain="DM",
            study_id="STUDY001",
            metadata={"codelist_mappings": {"SEXCD": "SEX"}},
        )

        result = transformer.transform(df, context)

        assert result.applied is True
        # Unmapped values should be preserved as-is
        assert result.data["SEXCD"].tolist() == ["Male", "X", "Female"]

    def test_transform_with_na_values(self, metadata_with_codelists):
        """Test transform handles NA values correctly."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        df = pd.DataFrame({"SEXCD": ["M", None, "F", pd.NA]})
        context = TransformationContext(
            domain="DM",
            study_id="STUDY001",
            metadata={"codelist_mappings": {"SEXCD": "SEX"}},
        )

        result = transformer.transform(df, context)

        assert result.applied is True
        assert result.data["SEXCD"].tolist()[0] == "Male"
        assert pd.isna(result.data["SEXCD"].tolist()[1])
        assert result.data["SEXCD"].tolist()[2] == "Female"
        assert pd.isna(result.data["SEXCD"].tolist()[3])

    def test_transform_case_insensitive_matching(self, metadata_with_codelists):
        """Test that codelist matching is case-insensitive."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        df = pd.DataFrame({"SEXCD": ["m", "F", "u"]})  # lowercase codes
        context = TransformationContext(
            domain="DM",
            study_id="STUDY001",
            metadata={"codelist_mappings": {"SEXCD": "SEX"}},
        )

        result = transformer.transform(df, context)

        assert result.applied is True
        assert result.data["SEXCD"].tolist() == ["Male", "Female", "Unknown"]

    def test_transform_with_code_column(self, metadata_with_codelists):
        """Test mapping using separate code column."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        df = pd.DataFrame(
            {
                "SEXCD": ["M", "F"],
                "SEX": ["", ""],  # Empty target column
            }
        )
        context = TransformationContext(
            domain="DM",
            study_id="STUDY001",
            metadata={
                "codelist_mappings": {"SEX": "SEX"},
                "code_columns": {"SEX": "SEXCD"},
            },
        )

        result = transformer.transform(df, context)

        assert result.applied is True
        # SEX column should be populated from SEXCD codes
        assert result.data["SEX"].tolist() == ["Male", "Female"]
        # SEXCD should remain unchanged
        assert result.data["SEXCD"].tolist() == ["M", "F"]
        assert "SEXCD → SEX using SEX" in result.metadata["applied_mappings"][0]

    def test_transform_with_whitespace_in_values(self, sex_codelist):
        """Test transform handles whitespace in code values."""
        metadata = StudyMetadata(codelists={"SEX": sex_codelist})
        transformer = CodelistMapperTransformer(metadata=metadata)
        df = pd.DataFrame({"SEXCD": [" M ", " F", "U "]})  # Values with whitespace
        context = TransformationContext(
            domain="DM",
            study_id="STUDY001",
            metadata={"codelist_mappings": {"SEXCD": "SEX"}},
        )

        result = transformer.transform(df, context)

        assert result.applied is True
        assert result.data["SEXCD"].tolist() == ["Male", "Female", "Unknown"]

    def test_transform_preserves_other_columns(self, metadata_with_codelists):
        """Test that transform doesn't affect unmapped columns."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        df = pd.DataFrame(
            {"USUBJID": ["001", "002"], "SEXCD": ["M", "F"], "AGE": [25, 30]}
        )
        context = TransformationContext(
            domain="DM",
            study_id="STUDY001",
            metadata={"codelist_mappings": {"SEXCD": "SEX"}},
        )

        result = transformer.transform(df, context)

        assert result.applied is True
        # Other columns should remain unchanged
        assert result.data["USUBJID"].tolist() == ["001", "002"]
        assert result.data["AGE"].tolist() == [25, 30]

    def test_transform_empty_dataframe(self, metadata_with_codelists):
        """Test transform with empty DataFrame."""
        transformer = CodelistMapperTransformer(metadata=metadata_with_codelists)
        df = pd.DataFrame({"SEXCD": []})
        context = TransformationContext(
            domain="DM",
            study_id="STUDY001",
            metadata={"codelist_mappings": {"SEXCD": "SEX"}},
        )

        result = transformer.transform(df, context)

        assert result.applied is True
        assert len(result.data) == 0
        assert result.metadata["input_rows"] == 0
        assert result.metadata["output_rows"] == 0

    def test_map_column_internal_method(self, sex_codelist):
        """Test the internal _map_column method."""
        metadata = StudyMetadata(codelists={"SEX": sex_codelist})
        transformer = CodelistMapperTransformer(metadata=metadata)

        series = pd.Series(["M", "F", "U", "X", None])
        result = transformer._map_column(series, sex_codelist)

        assert result.tolist()[0] == "Male"
        assert result.tolist()[1] == "Female"
        assert result.tolist()[2] == "Unknown"
        assert result.tolist()[3] == "X"  # Unmapped preserved
        assert pd.isna(result.tolist()[4])  # NA preserved

    def test_map_column_with_code_source_internal_method(self, sex_codelist):
        """Test the internal _map_column_with_code_source method."""
        metadata = StudyMetadata(codelists={"SEX": sex_codelist})
        transformer = CodelistMapperTransformer(metadata=metadata)

        code_series = pd.Series(["M", "F", None, "X"])
        result = transformer._map_column_with_code_source(code_series, sex_codelist)

        assert result.tolist()[0] == "Male"
        assert result.tolist()[1] == "Female"
        assert result.tolist()[2] is None
        assert result.tolist()[3] == "X"  # Unmapped returns string representation
