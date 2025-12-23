"""ISO 8601 date formatter transformer.

This module provides a transformer for normalizing date/time values to ISO 8601 format
as required by SDTM standards.

SDTM IG v3.4 Requirements:
- Date/time variables (--DTC) must conform to ISO 8601
- Unknown date components should be omitted (not replaced with letters)
- Partial dates are supported (YYYY, YYYY-MM, YYYY-MM-DD)
"""

from __future__ import annotations

import pandas as pd

from ...domain.services.transformers import (
    normalize_iso8601,
    normalize_iso8601_duration,
)
from ..base import TransformationContext, TransformationResult


class ISODateFormatter:
    """Transformer for normalizing dates/times to ISO 8601 format.

    This transformer identifies and normalizes all date/time columns (--DTC)
    and duration columns (--DUR) to ISO 8601 format per SDTM standards.

    Features:
    - Full date normalization (YYYY-MM-DD)
    - Partial date support (YYYY, YYYY-MM)
    - Duration normalization (PnYnMnDTnHnMnS)
    - Unknown component handling (NK, UN, UNK → omitted)

    Example:
        >>> from cdisc_transpiler.transformations.dates import ISODateFormatter
        >>> from cdisc_transpiler.transformations import TransformationContext
        >>>
        >>> formatter = ISODateFormatter()
        >>> context = TransformationContext(domain="DM", study_id="STUDY001")
        >>> result = formatter.transform(df, context)
        >>>
        >>> if result.success:
        ...     print(f"Normalized {result.metadata['dtc_columns_processed']} date columns")
        ...     formatted_data = result.data
    """

    def __init__(self):
        """Initialize the ISO date formatter transformer."""

    def can_transform(self, df: pd.DataFrame, domain: str) -> bool:
        """Check if transformer applies to this data.

        The formatter applies if there are any --DTC or --DUR columns.

        Args:
            df: Input DataFrame
            domain: SDTM domain code

        Returns:
            True if there are date/time or duration columns to normalize
        """
        _ = domain
        return any("DTC" in col or "DUR" in col for col in df.columns)

    def transform(
        self, df: pd.DataFrame, context: TransformationContext
    ) -> TransformationResult:
        """Normalize date/time and duration columns to ISO 8601 format.

        Args:
            df: Input DataFrame
            context: Transformation context

        Returns:
            TransformationResult with normalized dates
        """
        _ = context
        transformed_df = df.copy()

        dtc_columns = []
        dur_columns = []

        # Find and normalize DTC columns
        for col in transformed_df.columns:
            if "DTC" in col:
                dtc_columns.append(col)
                transformed_df.loc[:, col] = transformed_df[col].apply(
                    normalize_iso8601
                )

        # Find and normalize DUR columns
        for col in transformed_df.columns:
            if "DUR" in col:
                dur_columns.append(col)
                transformed_df.loc[:, col] = transformed_df[col].apply(
                    normalize_iso8601_duration
                )

        if not dtc_columns and not dur_columns:
            return TransformationResult(
                data=transformed_df,
                applied=False,
                message="No date/time or duration columns found",
            )

        message_parts = []
        if dtc_columns:
            message_parts.append(f"{len(dtc_columns)} date/time columns")
        if dur_columns:
            message_parts.append(f"{len(dur_columns)} duration columns")

        message = f"Normalized {' and '.join(message_parts)} to ISO 8601 format"

        return TransformationResult(
            data=transformed_df,
            applied=True,
            message=message,
            metadata={
                "dtc_columns_processed": dtc_columns,
                "dur_columns_processed": dur_columns,
                "input_rows": len(df),
                "output_rows": len(transformed_df),
            },
        )
