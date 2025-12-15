"""Codelist mapping transformer for applying controlled terminology mappings.

This module provides a transformer that maps coded values to their text equivalents
using codelist definitions from study metadata or CDISC Controlled Terminology.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

import pandas as pd

from ..base import TransformationContext, TransformationResult

if TYPE_CHECKING:
    from ...domain.entities.study_metadata import StudyMetadata


class CodelistMapperTransformer:
    """Transformer for mapping coded values to text using codelists.

    This transformer applies codelist transformations to convert codes to their
    text equivalents (e.g., "F" → "Female", "1" → "Asian"). It handles:

    - Code-to-text mapping using study metadata codelists
    - Missing codelist fallback (returns values unchanged)
    - Unmapped term handling (preserves original values)
    - Case-insensitive matching

    The transformer is typically used for mapping *CD (code) columns to *TERM
    columns, or for enriching data with decoded values.

    Example:
        >>> from cdisc_transpiler.domain.entities import StudyMetadata, CodeList, CodeListValue
        >>> from cdisc_transpiler.transformations import TransformationContext
        >>>
        >>> # Create metadata with SEX codelist
        >>> sex_codelist = CodeList(
        ...     format_name="SEX",
        ...     values=[
        ...         CodeListValue(code_value="M", code_text="Male", data_type="text"),
        ...         CodeListValue(code_value="F", code_text="Female", data_type="text"),
        ...     ]
        ... )
        >>> metadata = StudyMetadata(codelists=[sex_codelist])
        >>>
        >>> # Create transformer
        >>> transformer = CodelistMapperTransformer(metadata=metadata)
        >>>
        >>> # Apply transformation
        >>> df = pd.DataFrame({"SEXCD": ["M", "F", "M"]})
        >>> context = TransformationContext(domain="DM", study_id="STUDY001")
        >>> context = context.with_metadata(
        ...     codelist_mappings={"SEXCD": "SEX"}
        ... )
        >>> result = transformer.transform(df, context)
        >>>
        >>> # Result contains decoded values
        >>> print(result.data["SEXCD"])
        0    Male
        1  Female
        2    Male
    """

    def __init__(self, metadata: StudyMetadata | None = None):
        """Initialize the codelist mapper transformer.

        Args:
            metadata: Optional study metadata containing codelist definitions.
                     If None, transformer will pass through values unchanged.
        """
        self.metadata = metadata

    def can_transform(self, df: pd.DataFrame, domain: str) -> bool:
        """Check if this transformer can be applied.

        The transformer can be applied if:
        - We have metadata with codelists
        - The DataFrame has columns that might need codelist mapping

        Args:
            df: DataFrame to check
            domain: SDTM domain code

        Returns:
            True if transformation is possible, False otherwise
        """
        if not self.metadata or not self.metadata.codelists:
            return False

        # Check if any columns might benefit from codelist mapping
        # Look for common patterns like *CD, *TERM, *DECOD columns
        for col in df.columns:
            col_upper = col.upper()
            if any(suffix in col_upper for suffix in ["CD", "TERM", "DECOD"]):
                return True

        return False

    def transform(
        self, df: pd.DataFrame, context: TransformationContext
    ) -> TransformationResult:
        """Apply codelist transformations to the DataFrame.

        This method applies codelist mappings specified in the context metadata.
        The mappings are provided as a dictionary where keys are column names
        and values are codelist names to use for mapping.

        Args:
            df: DataFrame to transform
            context: Transformation context containing:
                    - codelist_mappings: dict[str, str] mapping column names to codelist names
                    - code_columns: dict[str, str] mapping target columns to source code columns (optional)

        Returns:
            TransformationResult with transformed data and metadata
        """
        if not self.metadata:
            return TransformationResult(
                data=df.copy(),
                applied=False,
                message="No metadata available, skipping codelist mapping",
            )

        # Extract mapping configuration from context
        codelist_mappings = context.metadata.get("codelist_mappings", {})
        code_columns = context.metadata.get("code_columns", {})

        if not codelist_mappings:
            return TransformationResult(
                data=df.copy(),
                applied=False,
                message="No codelist mappings specified in context",
            )

        result_df = df.copy()
        applied_mappings = []
        skipped_mappings = []
        warnings = []

        for column, codelist_name in codelist_mappings.items():
            if column not in result_df.columns:
                skipped_mappings.append(f"{column} (column not found)")
                continue

            # Get the codelist
            codelist = self.metadata.get_codelist(codelist_name)
            if not codelist:
                warnings.append(
                    f"Codelist '{codelist_name}' not found for column '{column}'"
                )
                skipped_mappings.append(f"{column} (codelist not found)")
                continue

            # Check if we should use a separate code column for lookup
            code_column = code_columns.get(column)
            if code_column and code_column in result_df.columns:
                # Use code column for lookup, write to target column
                result_df[column] = self._map_column_with_code_source(
                    result_df[code_column], codelist
                )
                applied_mappings.append(
                    f"{code_column} → {column} using {codelist_name}"
                )
            else:
                # Transform the column in place
                result_df[column] = self._map_column(result_df[column], codelist)
                applied_mappings.append(f"{column} using {codelist_name}")

        if not applied_mappings:
            return TransformationResult(
                data=result_df,
                applied=False,
                message=f"No codelist mappings applied. Skipped: {', '.join(skipped_mappings)}",
                warnings=warnings,
                metadata={
                    "applied_mappings": applied_mappings,
                    "skipped_mappings": skipped_mappings,
                    "input_rows": len(df),
                    "output_rows": len(result_df),
                },
            )

        return TransformationResult(
            data=result_df,
            applied=True,
            message=f"Applied {len(applied_mappings)} codelist mapping(s)",
            warnings=warnings,
            metadata={
                "applied_mappings": applied_mappings,
                "skipped_mappings": skipped_mappings,
                "input_rows": len(df),
                "output_rows": len(result_df),
            },
        )

    def _map_column(self, series: pd.Series, codelist: Any) -> pd.Series:
        """Map a series using a codelist.

        Args:
            series: Source series with code values
            codelist: CodeList object with mapping definitions

        Returns:
            Transformed series with text values
        """

        def transform_value(val: Any) -> Any:
            if pd.isna(val):
                return val
            # Try to get text for the code
            text = codelist.get_text(val)
            if text is not None:
                return text
            # If not found in codelist, return as-is
            return val

        return series.apply(transform_value)

    def _map_column_with_code_source(
        self, code_series: pd.Series, codelist: Any
    ) -> pd.Series:
        """Map a series using codes from another column.

        Args:
            code_series: Series containing code values
            codelist: CodeList object with mapping definitions

        Returns:
            Series with text values
        """

        def transform_code(code_val: Any) -> Any:
            if pd.isna(code_val):
                return None
            text = codelist.get_text(code_val)
            return text if text is not None else str(code_val)

        return code_series.apply(transform_code)
