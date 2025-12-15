"""Unit tests for transformation base interfaces.

Tests for TransformationContext, TransformationResult, and TransformerPort protocol.
"""

import pandas as pd
import pytest

from cdisc_transpiler.transformations.base import (
    TransformationContext,
    TransformationResult,
    TransformerPort,
    is_transformer,
)


class TestTransformationContext:
    """Tests for TransformationContext dataclass."""

    def test_create_minimal_context(self):
        """Test creating context with minimal required fields."""
        context = TransformationContext(domain="DM")

        assert context.domain == "DM"
        assert context.study_id is None
        assert context.source_file is None
        assert context.metadata == {}

    def test_create_full_context(self):
        """Test creating context with all fields."""
        context = TransformationContext(
            domain="AE",
            study_id="STUDY001",
            source_file="adverse_events.csv",
            metadata={"key": "value"},
        )

        assert context.domain == "AE"
        assert context.study_id == "STUDY001"
        assert context.source_file == "adverse_events.csv"
        assert context.metadata == {"key": "value"}

    def test_with_metadata(self):
        """Test adding metadata to existing context."""
        context = TransformationContext(domain="LB", metadata={"existing": "data"})

        new_context = context.with_metadata(new_key="new_value", another="one")

        # Original context unchanged
        assert context.metadata == {"existing": "data"}

        # New context has merged metadata
        assert new_context.metadata == {
            "existing": "data",
            "new_key": "new_value",
            "another": "one",
        }
        assert new_context.domain == "LB"

    def test_with_metadata_overwrites(self):
        """Test that with_metadata overwrites existing keys."""
        context = TransformationContext(domain="VS", metadata={"key": "old"})

        new_context = context.with_metadata(key="new")

        assert new_context.metadata == {"key": "new"}


class TestTransformationResult:
    """Tests for TransformationResult dataclass."""

    def test_create_minimal_result(self):
        """Test creating result with minimal required fields."""
        df = pd.DataFrame({"A": [1, 2, 3]})
        result = TransformationResult(data=df)

        assert result.applied is True
        assert result.message == ""
        assert result.warnings == []
        assert result.errors == []
        assert result.metadata == {}
        assert len(result.data) == 3

    def test_create_full_result(self):
        """Test creating result with all fields."""
        df = pd.DataFrame({"A": [1, 2, 3]})
        result = TransformationResult(
            data=df,
            applied=True,
            message="Transformation successful",
            warnings=["Warning 1"],
            errors=["Error 1"],
            metadata={"rows_changed": 3},
        )

        assert result.applied is True
        assert result.message == "Transformation successful"
        assert result.warnings == ["Warning 1"]
        assert result.errors == ["Error 1"]
        assert result.metadata == {"rows_changed": 3}

    def test_success_property(self):
        """Test success property based on applied and errors."""
        df = pd.DataFrame()

        # Success: applied and no errors
        result1 = TransformationResult(data=df, applied=True)
        assert result1.success is True

        # Not success: not applied
        result2 = TransformationResult(data=df, applied=False)
        assert result2.success is False

        # Not success: has errors
        result3 = TransformationResult(data=df, applied=True, errors=["Error"])
        assert result3.success is False

        # Not success: not applied and has errors
        result4 = TransformationResult(data=df, applied=False, errors=["Error"])
        assert result4.success is False

    def test_has_warnings_property(self):
        """Test has_warnings property."""
        df = pd.DataFrame()

        result1 = TransformationResult(data=df)
        assert result1.has_warnings is False

        result2 = TransformationResult(data=df, warnings=["Warning"])
        assert result2.has_warnings is True

    def test_add_warning(self):
        """Test adding warnings to result."""
        df = pd.DataFrame()
        result = TransformationResult(data=df)

        assert result.warnings == []

        result.add_warning("First warning")
        assert result.warnings == ["First warning"]

        result.add_warning("Second warning")
        assert result.warnings == ["First warning", "Second warning"]

    def test_add_error(self):
        """Test adding errors to result."""
        df = pd.DataFrame()
        result = TransformationResult(data=df, applied=True)

        assert result.errors == []
        assert result.success is True

        result.add_error("First error")
        assert result.errors == ["First error"]
        assert result.success is False  # No longer successful

        result.add_error("Second error")
        assert result.errors == ["First error", "Second error"]

    def test_summary_success(self):
        """Test summary for successful transformation."""
        df = pd.DataFrame()
        result = TransformationResult(
            data=df,
            applied=True,
            message="Converted wide to long format",
        )

        summary = result.summary()
        assert "Transformation applied" in summary
        assert "Converted wide to long format" in summary

    def test_summary_with_warnings(self):
        """Test summary includes warnings."""
        df = pd.DataFrame()
        result = TransformationResult(
            data=df,
            applied=True,
            message="Transformation done",
            warnings=["Missing data in column X", "Column Y has nulls"],
        )

        summary = result.summary()
        assert "Warnings (2)" in summary
        assert "Missing data" in summary or "Column Y" in summary

    def test_summary_with_errors(self):
        """Test summary includes errors."""
        df = pd.DataFrame()
        result = TransformationResult(
            data=df,
            applied=True,
            message="Transformation attempted",
            errors=["Invalid data type", "Column not found"],
        )

        summary = result.summary()
        assert "Errors (2)" in summary
        assert "Invalid data" in summary or "Column not found" in summary

    def test_summary_not_applied(self):
        """Test summary when transformation not applied."""
        df = pd.DataFrame()
        result = TransformationResult(
            data=df,
            applied=False,
            message="Transformation skipped",
        )

        summary = result.summary()
        assert "skipped" in summary.lower()


class TestTransformerPort:
    """Tests for TransformerPort protocol."""

    def test_valid_transformer(self):
        """Test that a class with required methods is a valid transformer."""

        class ValidTransformer:
            def can_transform(self, df: pd.DataFrame, domain: str) -> bool:
                return True

            def transform(
                self, df: pd.DataFrame, context: TransformationContext
            ) -> TransformationResult:
                return TransformationResult(data=df)

        transformer = ValidTransformer()
        assert is_transformer(transformer)

    def test_invalid_transformer_missing_can_transform(self):
        """Test that a class missing can_transform is not valid."""

        class InvalidTransformer:
            def transform(
                self, df: pd.DataFrame, context: TransformationContext
            ) -> TransformationResult:
                return TransformationResult(data=df)

        transformer = InvalidTransformer()
        assert not is_transformer(transformer)

    def test_invalid_transformer_missing_transform(self):
        """Test that a class missing transform is not valid."""

        class InvalidTransformer:
            def can_transform(self, df: pd.DataFrame, domain: str) -> bool:
                return True

        transformer = InvalidTransformer()
        assert not is_transformer(transformer)

    def test_invalid_transformer_not_callable(self):
        """Test that attributes must be callable methods."""

        class InvalidTransformer:
            can_transform = "not a method"
            transform = "also not a method"

        transformer = InvalidTransformer()
        assert not is_transformer(transformer)

    def test_non_object_is_not_transformer(self):
        """Test that non-objects are not transformers."""
        assert not is_transformer(None)
        assert not is_transformer("string")
        assert not is_transformer(123)
        assert not is_transformer([])


class TestTransformerIntegration:
    """Integration tests showing typical transformer usage."""

    def test_simple_transformer_workflow(self):
        """Test a complete transformer workflow."""

        class UppercaseTransformer:
            """Transformer that uppercases string columns."""

            def can_transform(self, df: pd.DataFrame, domain: str) -> bool:
                # Only transform DM domain
                return domain == "DM"

            def transform(
                self, df: pd.DataFrame, context: TransformationContext
            ) -> TransformationResult:
                if not self.can_transform(df, context.domain):
                    return TransformationResult(
                        data=df,
                        applied=False,
                        message="Transformer does not apply to this domain",
                    )

                result_df = df.copy()
                modified_cols = []

                for col in result_df.columns:
                    if result_df[col].dtype == object:
                        result_df[col] = result_df[col].str.upper()
                        modified_cols.append(col)

                return TransformationResult(
                    data=result_df,
                    applied=True,
                    message=f"Uppercased {len(modified_cols)} columns",
                    metadata={"columns_modified": modified_cols},
                )

        # Setup
        transformer = UppercaseTransformer()
        df = pd.DataFrame(
            {
                "NAME": ["alice", "bob"],
                "AGE": [30, 25],
            }
        )
        context = TransformationContext(domain="DM", study_id="TEST001")

        # Execute
        assert transformer.can_transform(df, "DM")
        result = transformer.transform(df, context)

        # Verify
        assert result.success
        assert result.applied
        assert result.data["NAME"].tolist() == ["ALICE", "BOB"]
        assert result.data["AGE"].tolist() == [30, 25]  # Unchanged
        assert "columns_modified" in result.metadata

    def test_transformer_skip_workflow(self):
        """Test transformer that skips based on domain."""

        class VSOnlyTransformer:
            """Transformer that only applies to VS domain."""

            def can_transform(self, df: pd.DataFrame, domain: str) -> bool:
                return domain == "VS"

            def transform(
                self, df: pd.DataFrame, context: TransformationContext
            ) -> TransformationResult:
                if not self.can_transform(df, context.domain):
                    return TransformationResult(
                        data=df,
                        applied=False,
                        message=f"Skipped: only applies to VS, got {context.domain}",
                    )

                # Transform logic here
                return TransformationResult(data=df, applied=True)

        # Setup
        transformer = VSOnlyTransformer()
        df = pd.DataFrame({"A": [1, 2]})
        context = TransformationContext(domain="DM")  # Wrong domain

        # Execute
        assert not transformer.can_transform(df, "DM")
        result = transformer.transform(df, context)

        # Verify
        assert not result.applied
        assert "Skipped" in result.message
