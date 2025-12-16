"""Transformation pipeline for composing and executing multiple transformers.

This module provides a pipeline for orchestrating multiple transformers in sequence,
with support for error handling, metadata collection, and conditional execution.
"""

from __future__ import annotations

from typing import Any

import pandas as pd

from .base import TransformationContext, TransformationResult, TransformerPort


class TransformationPipeline:
    """Pipeline for composing and executing transformers in sequence.

    The pipeline executes registered transformers in order, allowing each transformer
    to process the output of the previous one. It provides:
    - Sequential execution of transformers
    - Conditional execution (transformers can be skipped if not applicable)
    - Error handling with optional fail-safe mode
    - Metadata collection for debugging and monitoring

    Example:
        >>> from cdisc_transpiler.transformations.findings import VSTransformer
        >>> from cdisc_transpiler.transformations import TransformationContext
        >>>
        >>> pipeline = TransformationPipeline()
        >>> pipeline.add_transformer(VSTransformer())
        >>>
        >>> context = TransformationContext(domain="VS", study_id="STUDY001")
        >>> result = pipeline.execute(df, context)
        >>>
        >>> if result.success:
        ...     transformed_data = result.data
        ...     print(f"Applied {len(result.metadata['applied_transformers'])} transformers")
    """

    def __init__(self, fail_safe: bool = False):
        """Initialize the transformation pipeline.

        Args:
            fail_safe: If True, continue pipeline execution even if a transformer fails.
                      If False, stop on first error. Default is False.
        """
        self.transformers: list[TransformerPort] = []
        self.fail_safe = fail_safe

    def add_transformer(self, transformer: TransformerPort) -> TransformationPipeline:
        """Add a transformer to the pipeline.

        Transformers are executed in the order they are added.

        Args:
            transformer: Transformer to add (must implement TransformerPort protocol)

        Returns:
            Self for method chaining

        Example:
            >>> pipeline = TransformationPipeline()
            >>> pipeline.add_transformer(VSTransformer()).add_transformer(DateTransformer())
        """
        self.transformers.append(transformer)
        return self

    def execute(
        self, df: pd.DataFrame, context: TransformationContext
    ) -> TransformationResult:
        """Execute the pipeline on the input data.

        Each transformer is executed in order. Transformers that return
        can_transform=False are skipped. The output of each transformer
        becomes the input to the next.

        Args:
            df: Input DataFrame
            context: Transformation context

        Returns:
            TransformationResult with final transformed data and metadata

        Example:
            >>> context = TransformationContext(domain="VS", study_id="STUDY001")
            >>> result = pipeline.execute(df, context)
            >>>
            >>> if result.success:
            ...     print(f"Transformed {result.metadata['input_rows']} → {result.metadata['output_rows']} rows")
            ...     print(f"Applied: {result.metadata['applied_transformers']}")
            ...     print(f"Skipped: {result.metadata['skipped_transformers']}")
        """
        if not self.transformers:
            return TransformationResult(
                data=df,
                applied=False,
                message="Pipeline is empty (no transformers registered)",
            )

        current_data = df
        applied_transformers: list[dict[str, Any]] = []
        skipped_transformers: list[str] = []
        all_warnings: list[str] = []
        all_errors: list[str] = []

        input_rows = len(df)

        for i, transformer in enumerate(self.transformers):
            transformer_name = transformer.__class__.__name__

            # Check if transformer applies
            if not transformer.can_transform(current_data, context.domain):
                skipped_transformers.append(transformer_name)
                continue

            # Execute transformer
            try:
                result = transformer.transform(current_data, context)

                if result.applied:
                    # Record transformation
                    applied_transformers.append(
                        {
                            "name": transformer_name,
                            "input_rows": len(current_data),
                            "output_rows": len(result.data),
                            "message": result.message,
                            "metadata": result.metadata,
                        }
                    )

                    # Collect warnings and errors
                    if result.warnings:
                        all_warnings.extend(
                            [f"{transformer_name}: {w}" for w in result.warnings]
                        )
                    if result.errors:
                        all_errors.extend(
                            [f"{transformer_name}: {e}" for e in result.errors]
                        )

                    # Check for errors
                    if not result.success:
                        if self.fail_safe:
                            # Continue with current data (don't use failed transformation)
                            all_warnings.append(
                                f"{transformer_name}: Transformation failed but continuing (fail-safe mode)"
                            )
                        else:
                            # Stop pipeline on error
                            return TransformationResult(
                                data=current_data,
                                applied=True,
                                message=f"Pipeline stopped: {transformer_name} failed",
                                warnings=all_warnings,
                                errors=all_errors,
                                metadata={
                                    "input_rows": input_rows,
                                    "output_rows": len(current_data),
                                    "applied_transformers": applied_transformers,
                                    "skipped_transformers": skipped_transformers,
                                    "stopped_at": transformer_name,
                                },
                            )
                    else:
                        # Use transformed data for next transformer
                        current_data = result.data
                else:
                    skipped_transformers.append(transformer_name)

            except Exception as e:
                error_msg = f"{transformer_name}: Unexpected error: {str(e)}"
                all_errors.append(error_msg)

                if self.fail_safe:
                    all_warnings.append(
                        f"{transformer_name}: Caught exception, continuing (fail-safe mode)"
                    )
                else:
                    return TransformationResult(
                        data=current_data,
                        applied=True,
                        message=f"Pipeline stopped: {transformer_name} raised exception",
                        warnings=all_warnings,
                        errors=all_errors,
                        metadata={
                            "input_rows": input_rows,
                            "output_rows": len(current_data),
                            "applied_transformers": applied_transformers,
                            "skipped_transformers": skipped_transformers,
                            "stopped_at": transformer_name,
                        },
                    )

        # Pipeline completed successfully
        output_rows = len(current_data)

        if not applied_transformers:
            message = "No transformers were applicable"
        elif len(applied_transformers) == 1:
            message = f"Applied 1 transformer: {applied_transformers[0]['name']}"
        else:
            names = [t["name"] for t in applied_transformers]
            message = (
                f"Applied {len(applied_transformers)} transformers: {', '.join(names)}"
            )

        return TransformationResult(
            data=current_data,
            applied=len(applied_transformers) > 0,
            message=message,
            warnings=all_warnings,
            errors=all_errors,
            metadata={
                "input_rows": input_rows,
                "output_rows": output_rows,
                "applied_transformers": applied_transformers,
                "skipped_transformers": skipped_transformers,
                "transformers_count": len(self.transformers),
            },
        )

    def clear(self) -> None:
        """Clear all transformers from the pipeline."""
        self.transformers.clear()

    def __len__(self) -> int:
        """Return the number of transformers in the pipeline."""
        return len(self.transformers)

    def __repr__(self) -> str:
        """Return string representation of the pipeline."""
        transformer_names = [t.__class__.__name__ for t in self.transformers]
        return (
            f"TransformationPipeline({transformer_names}, fail_safe={self.fail_safe})"
        )
