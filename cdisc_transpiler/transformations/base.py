"""Base interface for data transformers.

This module defines the abstract interface that all data transformers must implement,
providing a consistent contract for transformation operations across the codebase.

Example:
    Implementing a simple transformer:

    >>> from transformations.base import TransformerPort, TransformationContext, TransformationResult
    >>> import pandas as pd
    >>>
    >>> class UppercaseTransformer:
    ...     def can_transform(self, df: pd.DataFrame, domain: str) -> bool:
    ...         return domain == "DM"  # Only transform Demographics
    ...
    ...     def transform(self, df: pd.DataFrame, context: TransformationContext) -> TransformationResult:
    ...         transformed_df = df.copy()
    ...         for col in transformed_df.columns:
    ...             if transformed_df[col].dtype == 'object':
    ...                 transformed_df[col] = transformed_df[col].str.upper()
    ...         return TransformationResult(
    ...             data=transformed_df,
    ...             applied=True,
    ...             message="Uppercased all text columns"
    ...         )
"""

from dataclasses import dataclass, field
from typing import Protocol

import pandas as pd


def _empty_str_list() -> list[str]:
    return []


def _empty_metadata() -> dict[str, object]:
    return {}


@dataclass
class TransformationContext:
    """Context information for transformation operations.

    This dataclass carries metadata and configuration needed by transformers
    to make informed decisions about how to transform data.

    Attributes:
        domain: SDTM domain code (e.g., 'DM', 'AE', 'LB')
        study_id: Study identifier
        source_file: Path to source data file (optional)
        metadata: Additional metadata dictionary for transformer-specific data

    Example:
        >>> context = TransformationContext(
        ...     domain="VS",
        ...     study_id="STUDY001",
        ...     source_file="vital_signs.csv",
        ...     metadata={"unit_conversion": "metric"}
        ... )
    """

    domain: str
    study_id: str | None = None
    source_file: str | None = None
    metadata: dict[str, object] = field(default_factory=_empty_metadata)

    def with_metadata(self, **kwargs: object) -> TransformationContext:
        """Create a new context with additional metadata.

        Args:
            **kwargs: Key-value pairs to add to metadata

        Returns:
            New TransformationContext with merged metadata

        Example:
            >>> context = TransformationContext(domain="LB")
            >>> new_context = context.with_metadata(test_type="chemistry")
        """
        new_metadata: dict[str, object] = {**self.metadata, **kwargs}
        return TransformationContext(
            domain=self.domain,
            study_id=self.study_id,
            source_file=self.source_file,
            metadata=new_metadata,
        )


@dataclass
class TransformationResult:
    """Result of a transformation operation.

    This dataclass provides rich feedback about transformation results,
    including the transformed data, success status, and diagnostic messages.

    Attributes:
        data: Transformed DataFrame
        applied: Whether transformation was applied
        message: Human-readable description of what was done
        warnings: List of warning messages (non-fatal issues)
        errors: List of error messages (issues that may affect quality)
        metadata: Additional result metadata (e.g., row counts, columns modified)

    Example:
        >>> result = TransformationResult(
        ...     data=transformed_df,
        ...     applied=True,
        ...     message="Converted wide format to long format",
        ...     metadata={"rows_before": 100, "rows_after": 500}
        ... )
    """

    data: pd.DataFrame
    applied: bool = True
    message: str = ""
    warnings: list[str] = field(default_factory=_empty_str_list)
    errors: list[str] = field(default_factory=_empty_str_list)
    metadata: dict[str, object] = field(default_factory=_empty_metadata)

    @property
    def success(self) -> bool:
        """Whether transformation completed without errors."""
        return self.applied and len(self.errors) == 0

    @property
    def has_warnings(self) -> bool:
        """Whether transformation generated any warnings."""
        return len(self.warnings) > 0

    def add_warning(self, warning: str) -> None:
        """Add a warning message.

        Args:
            warning: Warning message to add
        """
        self.warnings.append(warning)

    def add_error(self, error: str) -> None:
        """Add an error message.

        Args:
            error: Error message to add
        """
        self.errors.append(error)

    def summary(self) -> str:
        """Generate a summary string of the transformation result.

        Returns:
            Human-readable summary with message, warnings, and errors

        Example:
            >>> print(result.summary())
            Transformation applied: Converted wide to long format
            Warnings (1): Column 'AGE' has missing values
        """
        lines: list[str] = []
        if self.message:
            status = "applied" if self.applied else "skipped"
            lines.append(f"Transformation {status}: {self.message}")

        if self.warnings:
            lines.append(f"Warnings ({len(self.warnings)}): {', '.join(self.warnings)}")

        if self.errors:
            lines.append(f"Errors ({len(self.errors)}): {', '.join(self.errors)}")

        return "\n".join(lines) if lines else "No transformation applied"


class TransformerPort(Protocol):
    """Protocol defining the interface for data transformers.

    All transformers must implement this protocol to ensure consistent behavior
    across the transformation pipeline. The protocol uses structural subtyping,
    so any class with these methods is considered a valid transformer.

    Methods:
        can_transform: Check if this transformer applies to the given data/domain
        transform: Perform the transformation and return results

    Example:
        >>> class MyTransformer:
        ...     def can_transform(self, df: pd.DataFrame, domain: str) -> bool:
        ...         return "TEST_" in df.columns and domain == "LB"
        ...
        ...     def transform(self, df: pd.DataFrame, context: TransformationContext) -> TransformationResult:
        ...         # Transformation logic here
        ...         return TransformationResult(data=df, applied=True)

    Note:
        This is a Protocol, not an abstract base class. Classes don't need to
        explicitly inherit from TransformerPort to be considered valid transformers.
    """

    def can_transform(self, df: pd.DataFrame, domain: str) -> bool:
        """Check if this transformer can/should transform the given data.

        This method allows transformers to inspect the data and domain to decide
        if their transformation logic applies. It enables selective transformation
        in a pipeline where multiple transformers may be available.

        Args:
            df: Input DataFrame to potentially transform
            domain: SDTM domain code (e.g., 'DM', 'AE', 'LB')

        Returns:
            True if this transformer should be applied, False otherwise

        Example:
            >>> transformer = VSTransformer()
            >>> if transformer.can_transform(df, "VS"):
            ...     result = transformer.transform(df, context)
        """
        ...

    def transform(
        self, df: pd.DataFrame, context: TransformationContext
    ) -> TransformationResult:
        """Transform the input DataFrame according to transformer logic.

        This method performs the actual transformation work. It receives the
        input data and context, applies transformation logic, and returns
        a rich result object with the transformed data and metadata.

        Args:
            df: Input DataFrame to transform
            context: Transformation context with metadata and configuration

        Returns:
            TransformationResult containing transformed data and metadata

        Raises:
            Should not raise exceptions for data issues; instead capture
            issues in TransformationResult.errors

        Example:
            >>> context = TransformationContext(domain="VS", study_id="STUDY001")
            >>> result = transformer.transform(df, context)
            >>> if result.success:
            ...     print(f"Transformed {len(result.data)} rows")
            ... else:
            ...     print(f"Errors: {result.errors}")
        """
        ...


def is_transformer(obj: object) -> bool:
    """Check if an object implements the TransformerPort protocol.

    This function performs runtime checking to verify that an object
    has the required methods to be considered a valid transformer.

    Args:
        obj: Object to check

    Returns:
        True if object implements TransformerPort protocol, False otherwise

    Example:
        >>> class MyTransformer:
        ...     def can_transform(self, df, domain): return True
        ...     def transform(self, df, context): return TransformationResult(df)
        >>>
        >>> is_transformer(MyTransformer())
        True
        >>> is_transformer("not a transformer")
        False
    """
    can_transform = getattr(obj, "can_transform", None)
    transform = getattr(obj, "transform", None)
    return callable(can_transform) and callable(transform)
