"""Unit tests for transformation pipeline.

Tests for TransformationPipeline class.
"""

import pandas as pd
import pytest

from cdisc_transpiler.transformations import (
    TransformationPipeline,
    TransformationContext,
    TransformationResult,
)


class MockTransformer:
    """Mock transformer for testing."""

    def __init__(
        self,
        name: str,
        applies: bool = True,
        success: bool = True,
        multiply_rows: int = 1,
    ):
        """Initialize mock transformer.

        Args:
            name: Transformer name
            applies: Whether can_transform returns True
            success: Whether transformation succeeds
            multiply_rows: Factor to multiply row count by (for testing data changes)
        """
        self.name = name
        self.applies = applies
        self.success = success
        self.multiply_rows = multiply_rows
        self.call_count = 0

    def can_transform(self, df: pd.DataFrame, domain: str) -> bool:
        """Check if transformer applies."""
        return self.applies

    def transform(
        self, df: pd.DataFrame, context: TransformationContext
    ) -> TransformationResult:
        """Transform data."""
        self.call_count += 1

        if not self.success:
            return TransformationResult(
                data=df,
                applied=True,
                message=f"{self.name} failed",
                errors=[f"{self.name} error"],
            )

        # Simulate transformation by duplicating rows
        if self.multiply_rows > 1:
            new_df = pd.concat([df] * self.multiply_rows, ignore_index=True)
        else:
            new_df = df.copy()

        return TransformationResult(
            data=new_df,
            applied=True,
            message=f"{self.name} applied",
            metadata={
                "transformer": self.name,
                "input_rows": len(df),
                "output_rows": len(new_df),
            },
        )


class TestTransformationPipeline:
    """Tests for TransformationPipeline class."""

    def test_initialization(self):
        """Test pipeline initialization."""
        pipeline = TransformationPipeline()

        assert len(pipeline) == 0
        assert pipeline.fail_safe is False

    def test_initialization_with_fail_safe(self):
        """Test pipeline initialization with fail-safe mode."""
        pipeline = TransformationPipeline(fail_safe=True)

        assert pipeline.fail_safe is True

    def test_add_transformer(self):
        """Test adding a transformer."""
        pipeline = TransformationPipeline()
        transformer = MockTransformer("T1")

        result = pipeline.add_transformer(transformer)

        assert result is pipeline  # Method chaining
        assert len(pipeline) == 1

    def test_add_multiple_transformers(self):
        """Test adding multiple transformers."""
        pipeline = TransformationPipeline()

        pipeline.add_transformer(MockTransformer("T1"))
        pipeline.add_transformer(MockTransformer("T2"))
        pipeline.add_transformer(MockTransformer("T3"))

        assert len(pipeline) == 3

    def test_method_chaining(self):
        """Test method chaining when adding transformers."""
        pipeline = TransformationPipeline()

        pipeline.add_transformer(MockTransformer("T1")).add_transformer(
            MockTransformer("T2")
        )

        assert len(pipeline) == 2

    def test_execute_empty_pipeline(self):
        """Test executing an empty pipeline."""
        pipeline = TransformationPipeline()
        df = pd.DataFrame({"A": [1, 2, 3]})
        context = TransformationContext(domain="TEST", study_id="STUDY001")

        result = pipeline.execute(df, context)

        assert not result.applied
        assert "empty" in result.message.lower()
        assert result.data.equals(df)

    def test_execute_single_transformer(self):
        """Test executing pipeline with single transformer."""
        pipeline = TransformationPipeline()
        transformer = MockTransformer("T1", multiply_rows=2)
        pipeline.add_transformer(transformer)

        df = pd.DataFrame({"A": [1, 2, 3]})
        context = TransformationContext(domain="TEST", study_id="STUDY001")

        result = pipeline.execute(df, context)

        assert result.success
        assert result.applied
        assert len(result.data) == 6  # 3 rows * 2
        assert transformer.call_count == 1
        assert "Applied 1 transformer" in result.message
        assert result.metadata["applied_transformers"][0]["name"] == "MockTransformer"

    def test_execute_multiple_transformers(self):
        """Test executing pipeline with multiple transformers."""
        pipeline = TransformationPipeline()
        t1 = MockTransformer("T1", multiply_rows=2)
        t2 = MockTransformer("T2", multiply_rows=3)
        pipeline.add_transformer(t1).add_transformer(t2)

        df = pd.DataFrame({"A": [1, 2]})  # 2 rows
        context = TransformationContext(domain="TEST", study_id="STUDY001")

        result = pipeline.execute(df, context)

        assert result.success
        assert result.applied
        assert len(result.data) == 12  # 2 * 2 * 3
        assert t1.call_count == 1
        assert t2.call_count == 1
        assert "Applied 2 transformers" in result.message
        assert len(result.metadata["applied_transformers"]) == 2

    def test_execute_skips_non_applicable_transformer(self):
        """Test that non-applicable transformers are skipped."""
        pipeline = TransformationPipeline()
        t1 = MockTransformer("T1", applies=True)
        t2 = MockTransformer("T2", applies=False)  # Will be skipped
        t3 = MockTransformer("T3", applies=True)
        pipeline.add_transformer(t1).add_transformer(t2).add_transformer(t3)

        df = pd.DataFrame({"A": [1, 2]})
        context = TransformationContext(domain="TEST", study_id="STUDY001")

        result = pipeline.execute(df, context)

        assert result.success
        assert t1.call_count == 1
        assert t2.call_count == 0  # Skipped
        assert t3.call_count == 1
        assert len(result.metadata["applied_transformers"]) == 2
        assert "MockTransformer" in result.metadata["skipped_transformers"]

    def test_execute_stops_on_error(self):
        """Test that pipeline stops on error when fail_safe=False."""
        pipeline = TransformationPipeline(fail_safe=False)
        t1 = MockTransformer("T1", success=True)
        t2 = MockTransformer("T2", success=False)  # Will fail
        t3 = MockTransformer("T3", success=True)  # Should not run
        pipeline.add_transformer(t1).add_transformer(t2).add_transformer(t3)

        df = pd.DataFrame({"A": [1, 2]})
        context = TransformationContext(domain="TEST", study_id="STUDY001")

        result = pipeline.execute(df, context)

        assert not result.success
        assert "stopped" in result.message.lower()
        assert t1.call_count == 1
        assert t2.call_count == 1
        assert t3.call_count == 0  # Not executed
        assert len(result.metadata["applied_transformers"]) == 2
        assert result.metadata["stopped_at"] == "MockTransformer"
        assert len(result.errors) > 0

    def test_execute_continues_on_error_with_fail_safe(self):
        """Test that pipeline continues on error when fail_safe=True."""
        pipeline = TransformationPipeline(fail_safe=True)
        t1 = MockTransformer("T1", success=True, multiply_rows=2)
        t2 = MockTransformer("T2", success=False)  # Will fail but continue
        t3 = MockTransformer("T3", success=True, multiply_rows=3)
        pipeline.add_transformer(t1).add_transformer(t2).add_transformer(t3)

        df = pd.DataFrame({"A": [1]})  # 1 row
        context = TransformationContext(domain="TEST", study_id="STUDY001")

        result = pipeline.execute(df, context)

        # Pipeline completes but has errors (success=False because errors exist)
        assert not result.success  # Has errors from T2
        assert result.applied  # Pipeline was applied
        assert t1.call_count == 1
        assert t2.call_count == 1
        assert t3.call_count == 1
        # T1 transforms: 1 -> 2 rows
        # T2 fails: stays at 2 rows
        # T3 transforms: 2 -> 6 rows
        assert len(result.data) == 6
        assert len(result.metadata["applied_transformers"]) == 3
        assert len(result.warnings) > 0  # Warning about T2 failure
        assert len(result.errors) > 0  # Error from T2

    def test_execute_handles_exception(self):
        """Test that pipeline handles exceptions properly."""

        class ExceptionTransformer:
            def can_transform(self, df, domain):
                return True

            def transform(self, df, context):
                raise ValueError("Test exception")

        pipeline = TransformationPipeline(fail_safe=False)
        pipeline.add_transformer(ExceptionTransformer())

        df = pd.DataFrame({"A": [1, 2]})
        context = TransformationContext(domain="TEST", study_id="STUDY001")

        result = pipeline.execute(df, context)

        assert not result.success
        assert "exception" in result.message.lower()
        assert len(result.errors) > 0
        assert "Test exception" in result.errors[0]

    def test_execute_handles_exception_with_fail_safe(self):
        """Test that pipeline handles exceptions with fail-safe mode."""

        class ExceptionTransformer:
            def can_transform(self, df, domain):
                return True

            def transform(self, df, context):
                raise ValueError("Test exception")

        pipeline = TransformationPipeline(fail_safe=True)
        t1 = MockTransformer("T1", success=True)
        pipeline.add_transformer(t1)
        pipeline.add_transformer(ExceptionTransformer())
        t2 = MockTransformer("T2", success=True)
        pipeline.add_transformer(t2)

        df = pd.DataFrame({"A": [1, 2]})
        context = TransformationContext(domain="TEST", study_id="STUDY001")

        result = pipeline.execute(df, context)

        # Pipeline completes but has errors (success=False because errors exist)
        assert not result.success  # Has errors from exception
        assert result.applied  # Pipeline was applied
        assert t1.call_count == 1
        assert t2.call_count == 1
        assert len(result.warnings) > 0  # Warning about exception
        assert len(result.errors) > 0  # Error recorded

    def test_execute_collects_metadata(self):
        """Test that pipeline collects transformation metadata."""
        pipeline = TransformationPipeline()
        t1 = MockTransformer("T1", multiply_rows=2)
        t2 = MockTransformer("T2", multiply_rows=3)
        pipeline.add_transformer(t1).add_transformer(t2)

        df = pd.DataFrame({"A": [1]})
        context = TransformationContext(domain="TEST", study_id="STUDY001")

        result = pipeline.execute(df, context)

        assert result.metadata["input_rows"] == 1
        assert result.metadata["output_rows"] == 6  # 1 * 2 * 3
        assert result.metadata["transformers_count"] == 2
        assert len(result.metadata["applied_transformers"]) == 2

        # Check first transformer metadata
        t1_meta = result.metadata["applied_transformers"][0]
        assert t1_meta["name"] == "MockTransformer"
        assert t1_meta["input_rows"] == 1
        assert t1_meta["output_rows"] == 2

        # Check second transformer metadata
        t2_meta = result.metadata["applied_transformers"][1]
        assert t2_meta["input_rows"] == 2
        assert t2_meta["output_rows"] == 6

    def test_execute_with_no_applicable_transformers(self):
        """Test execution when no transformers are applicable."""
        pipeline = TransformationPipeline()
        t1 = MockTransformer("T1", applies=False)
        t2 = MockTransformer("T2", applies=False)
        pipeline.add_transformer(t1).add_transformer(t2)

        df = pd.DataFrame({"A": [1, 2]})
        context = TransformationContext(domain="TEST", study_id="STUDY001")

        result = pipeline.execute(df, context)

        assert not result.applied
        assert "No transformers were applicable" in result.message
        assert t1.call_count == 0
        assert t2.call_count == 0
        assert len(result.metadata["skipped_transformers"]) == 2

    def test_clear(self):
        """Test clearing the pipeline."""
        pipeline = TransformationPipeline()
        pipeline.add_transformer(MockTransformer("T1"))
        pipeline.add_transformer(MockTransformer("T2"))

        assert len(pipeline) == 2

        pipeline.clear()

        assert len(pipeline) == 0

    def test_repr(self):
        """Test string representation."""
        pipeline = TransformationPipeline()
        t1 = MockTransformer("T1")
        t2 = MockTransformer("T2")
        pipeline.add_transformer(t1).add_transformer(t2)

        repr_str = repr(pipeline)

        assert "TransformationPipeline" in repr_str
        assert "MockTransformer" in repr_str
        assert "fail_safe=False" in repr_str

    def test_integration_with_real_transformers(self):
        """Test integration with real transformer structure."""
        from cdisc_transpiler.transformations.dates import ISODateFormatter

        pipeline = TransformationPipeline()
        pipeline.add_transformer(ISODateFormatter())

        # Create test data
        df = pd.DataFrame(
            {
                "USUBJID": ["001"],
                "AESTDTC": ["2025-01-02"],
            }
        )

        context = TransformationContext(domain="AE", study_id="TEST001")
        result = pipeline.execute(df, context)

        assert result.success
        assert result.applied
        assert len(result.metadata["applied_transformers"]) == 1
        assert result.metadata["applied_transformers"][0]["name"] == "ISODateFormatter"
