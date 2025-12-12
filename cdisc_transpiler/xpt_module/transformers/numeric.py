"""Numeric transformation utilities for SDTM domains.

This module provides specialized transformation logic for numeric values,
including type coercion, sequence assignment, and STRESC population.
"""

from __future__ import annotations

import pandas as pd


class NumericTransformer:
    """Transforms numeric values for SDTM compliance.
    
    This class provides static methods for:
    - Populating STRESC from ORRES
    - Forcing numeric type coercion
    - Assigning sequence numbers
    """

    @staticmethod
    def populate_stresc_from_orres(
        frame: pd.DataFrame,
        domain_code: str,
    ) -> None:
        """Populate --STRESC from --ORRES when STRESC is empty.

        Per SDTM standards, if ORRES has a value and STRESC is missing,
        STRESC should be populated with ORRES (or a standardized version).
        This applies to Findings domains (LB, VS, DA, IE, PE, QS, etc.)
        
        Args:
            frame: DataFrame to modify in-place
            domain_code: SDTM domain code (e.g., "LB", "VS")
        """
        orres_col = f"{domain_code}ORRES"
        stresc_col = f"{domain_code}STRESC"

        if orres_col in frame.columns and stresc_col in frame.columns:
            # Where STRESC is null/empty and ORRES has a value, copy ORRES to STRESC
            # Convert ORRES to string first to handle numeric values
            orres_str = (
                frame[orres_col]
                .astype(str)
                .replace({"nan": "", "None": "", "<NA>": ""})
            )
            stresc_str = (
                frame[stresc_col]
                .astype(str)
                .replace({"nan": "", "None": "", "<NA>": ""})
            )

            mask = (stresc_str.str.strip() == "") & (orres_str.str.strip() != "")
            frame.loc[mask, stresc_col] = orres_str.loc[mask]

    @staticmethod
    def force_numeric(series: pd.Series) -> pd.Series:
        """Coerce a series to numeric type, replacing invalid values with NaN.
        
        Args:
            series: Series to convert to numeric
            
        Returns:
            Numeric series with invalid values as NaN
        """
        return pd.to_numeric(series, errors="coerce")

    @staticmethod
    def assign_sequence(
        frame: pd.DataFrame,
        seq_var: str,
        group_by: str,
    ) -> None:
        """Assign sequence numbers within groups.
        
        This assigns sequential integers (1, 2, 3, ...) within each group,
        which is used for domain-specific sequence variables like AESEQ, CMSEQ.
        
        Args:
            frame: DataFrame to modify in-place
            seq_var: Name of sequence variable to populate
            group_by: Name of grouping variable (typically USUBJID)
        """
        if seq_var not in frame.columns or group_by not in frame.columns:
            return
        frame[seq_var] = frame.groupby(group_by).cumcount() + 1
