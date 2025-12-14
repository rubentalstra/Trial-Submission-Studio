"""Text transformation utilities for SDTM domains.

This module provides specialized transformation logic for text values,
including normalization, visit handling, and unknown value replacement.
"""

from __future__ import annotations

from typing import Any

import pandas as pd

from ...pandas_utils import ensure_numeric_series, ensure_series


class TextTransformer:
    """Transforms text values for SDTM compliance.

    This class provides static methods for:
    - Replacing unknown/missing value markers with defaults
    - Normalizing visit information
    """

    @staticmethod
    def replace_unknown(series: Any, default: str) -> pd.Series:
        """Replace empty/unknown markers with a controlled default.

        This method normalizes various representations of missing/unknown values
        (empty string, "UNK", "UNKNOWN", "NA", etc.) to a standard default value.

        Args:
            series: Series to transform
            default: Default value to use for unknown/missing values

        Returns:
            Transformed series with unknown values replaced
        """
        normalized = ensure_series(series).astype("string").fillna("")
        upper = normalized.str.upper()
        missing_tokens = {"", "UNK", "UNKNOWN", "NA", "N/A", "NONE", "NAN", "<NA>"}
        normalized.loc[upper.isin(missing_tokens)] = default
        normalized = normalized.fillna(default)
        return normalized.astype(str)

    @staticmethod
    def normalize_visit(frame: pd.DataFrame) -> None:
        """Ensure VISITNUM is numeric and VISIT matches VISITNUM.

        This method standardizes visit information by:
        1. If VISITNUM exists: ensure it's numeric and derive VISIT text
        2. If only VISIT exists: extract VISITNUM from text and standardize format

        Args:
            frame: DataFrame to modify in-place
        """
        if "VISITNUM" in frame.columns:
            frame["VISITNUM"] = (
                ensure_numeric_series(frame["VISITNUM"], frame.index)
                .fillna(1)
                .astype(int)
            )
            frame["VISIT"] = frame["VISITNUM"].apply(lambda n: f"Visit {int(n)}")
        elif "VISIT" in frame.columns:
            # Derive VISITNUM if VISIT text exists but VISITNUM missing
            visit_text = frame["VISIT"].astype("string").str.extract(r"(\d+)")[0]
            frame["VISITNUM"] = (
                ensure_numeric_series(visit_text, frame.index).fillna(1).astype(int)
            )
            frame["VISIT"] = frame["VISITNUM"].apply(lambda n: f"Visit {int(n)}")
