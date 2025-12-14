"""Study day calculator transformer.

This module provides a transformer for calculating study day (--DY) variables
based on event dates (--DTC) and reference start dates (RFSTDTC).

SDTM Study Day Calculation Rules:
- If event_date >= RFSTDTC: study_day = (event_date - RFSTDTC).days + 1
- If event_date < RFSTDTC: study_day = (event_date - RFSTDTC).days
- There is NO Day 0 in SDTM
- Missing dates result in missing study day
"""

from __future__ import annotations

import pandas as pd

from ..base import TransformationContext, TransformationResult


class StudyDayCalculator:
    """Transformer for calculating SDTM study day variables.
    
    This transformer calculates --DY (study day) variables from --DTC (date/time)
    variables using the reference start date (RFSTDTC) per SDTM conventions.
    
    The calculator requires:
    - USUBJID column (subject identifier)
    - Reference start dates mapping (passed via context metadata)
    - --DTC columns with corresponding --DY columns in the DataFrame
    
    Example:
        >>> from cdisc_transpiler.transformations.dates import StudyDayCalculator
        >>> from cdisc_transpiler.transformations import TransformationContext
        >>> 
        >>> calculator = StudyDayCalculator()
        >>> context = TransformationContext(
        ...     domain="AE",
        ...     study_id="STUDY001",
        ...     metadata={
        ...         "reference_starts": {
        ...             "001": "2023-01-15",
        ...             "002": "2023-01-20",
        ...         }
        ...     }
        ... )
        >>> result = calculator.transform(df, context)
        >>> 
        >>> if result.success:
        ...     print(f"Calculated study days for {result.metadata['dy_columns_calculated']} columns")
        ...     data_with_dy = result.data
    """
    
    def __init__(self):
        """Initialize the study day calculator transformer."""
        pass
    
    def can_transform(self, df: pd.DataFrame, domain: str) -> bool:
        """Check if transformer applies to this data.
        
        The calculator applies if there are any --DY columns and USUBJID.
        
        Args:
            df: Input DataFrame
            domain: SDTM domain code
            
        Returns:
            True if there are study day columns to calculate
        """
        has_dy_columns = any(col.endswith("DY") for col in df.columns)
        has_usubjid = "USUBJID" in df.columns
        return has_dy_columns and has_usubjid
    
    def transform(
        self, df: pd.DataFrame, context: TransformationContext
    ) -> TransformationResult:
        """Calculate study day variables based on reference start dates.
        
        Args:
            df: Input DataFrame
            context: Transformation context with reference_starts in metadata
            
        Returns:
            TransformationResult with calculated study days
        """
        transformed_df = df.copy()
        
        # Get reference start dates from context metadata
        reference_starts = context.metadata.get("reference_starts", {}) if context.metadata else {}
        
        if not reference_starts:
            return TransformationResult(
                data=transformed_df,
                applied=False,
                message="No reference start dates provided in context metadata",
                warnings=["Cannot calculate study days without reference_starts in context.metadata"],
            )
        
        if "USUBJID" not in transformed_df.columns:
            return TransformationResult(
                data=transformed_df,
                applied=False,
                message="USUBJID column not found",
                warnings=["Study day calculation requires USUBJID column"],
            )
        
        dy_columns_calculated = []
        
        # Find all --DY columns and calculate them
        for col in transformed_df.columns:
            if col.endswith("DY"):
                # Find corresponding --DTC column
                prefix = col[:-2]  # Remove "DY"
                dtc_col = prefix + "DTC"
                
                if dtc_col not in transformed_df.columns:
                    continue
                
                # Calculate study day for each row
                transformed_df[col] = transformed_df.apply(
                    lambda row: self._compute_dy_for_row(
                        row.get("USUBJID"),
                        row.get(dtc_col),
                        reference_starts,
                    ),
                    axis=1,
                )
                
                dy_columns_calculated.append(col)
        
        if not dy_columns_calculated:
            return TransformationResult(
                data=transformed_df,
                applied=False,
                message="No matching DTC/DY column pairs found",
            )
        
        message = f"Calculated study days for {len(dy_columns_calculated)} column(s): {', '.join(dy_columns_calculated)}"
        
        return TransformationResult(
            data=transformed_df,
            applied=True,
            message=message,
            metadata={
                "dy_columns_calculated": dy_columns_calculated,
                "input_rows": len(df),
                "output_rows": len(transformed_df),
                "subjects_with_reference": len(reference_starts),
            },
        )
    
    @staticmethod
    def _compute_dy_for_row(
        usubjid: str | None,
        dtc: str | None,
        reference_starts: dict[str, str],
    ) -> int | None:
        """Compute the study day for a given date and subject.
        
        SDTM Rules:
        - Day 1 is the first day on or after RFSTDTC
        - Days before RFSTDTC are negative (Day -1, Day -2, etc.)
        - There is no Day 0
        
        Args:
            usubjid: Subject identifier
            dtc: Date/time string in ISO 8601 format
            reference_starts: Mapping of USUBJID -> RFSTDTC
            
        Returns:
            Study day (integer) or None if cannot be computed
        """
        # Handle missing values
        if not usubjid or not dtc:
            return None
        
        # Check if subject has a reference start date
        if usubjid not in reference_starts:
            return None
        
        try:
            # Parse dates
            start_date = pd.to_datetime(reference_starts[usubjid], errors="coerce")
            obs_date = pd.to_datetime(dtc, errors="coerce")
            
            # Check for parsing failures
            if pd.isna(start_date) or pd.isna(obs_date):
                return None
            
            # Calculate day difference
            delta = (obs_date - start_date).days
            
            # Apply SDTM rule: add 1 for dates on or after reference, no adjustment for before
            # This ensures there is no Day 0
            return delta + 1 if delta >= 0 else delta
            
        except (ValueError, TypeError, AttributeError):
            return None
