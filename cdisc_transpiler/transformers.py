"""Optimized data transformers for SDTM domains.

This module provides vectorized pandas operations for common
data transformations.
"""

from __future__ import annotations

from datetime import datetime
from typing import TYPE_CHECKING

import pandas as pd
import numpy as np

if TYPE_CHECKING:
    pass


class DateTransformer:
    """Optimized date/time transformations."""
    
    @staticmethod
    def normalize_iso8601(series: pd.Series) -> pd.Series:
        """Normalize ISO 8601 dates (vectorized).
        
        Args:
            series: Series with date/time values
            
        Returns:
            Normalized series
        """
        # Convert to string and strip
        result = series.astype(str).str.strip()
        
        # Replace common invalid values
        result = result.replace({
            "nan": "",
            "NaT": "",
            "None": "",
            "NA": "",
        })
        
        # Validate and normalize format
        # This is a simplified version - full ISO 8601 validation is complex
        mask = result.str.match(r"^\d{4}(-\d{2}(-\d{2}(T\d{2}:\d{2}(:\d{2}(\.\d+)?)?)?)?)?(Z|[+-]\d{2}:\d{2})?$")
        result.loc[~mask & (result != "")] = ""
        
        return result
    
    @staticmethod
    def calculate_study_days(
        dtc_series: pd.Series,
        reference_dates: dict[str, str],
        usubjid_series: pd.Series,
    ) -> pd.Series:
        """Calculate study days (vectorized where possible).
        
        Args:
            dtc_series: Date/time series
            reference_dates: USUBJID -> RFSTDTC mapping
            usubjid_series: Subject ID series
            
        Returns:
            Study day series
        """
        result = pd.Series(pd.NA, index=dtc_series.index, dtype="Int64")
        
        # Convert dates to datetime
        dtc_dates = pd.to_datetime(dtc_series, errors="coerce")
        
        # Process by subject
        for usubjid in usubjid_series.dropna().unique():
            if usubjid not in reference_dates:
                continue
            
            ref_date = pd.to_datetime(reference_dates[usubjid], errors="coerce")
            if pd.isna(ref_date):
                continue
            
            # Get mask for this subject
            mask = usubjid_series == usubjid
            subject_dates = dtc_dates[mask]
            
            # Calculate days
            delta = (subject_dates - ref_date).dt.days
            
            # Apply study day rules (no day 0)
            study_days = delta.where(delta < 0, delta + 1)
            
            result.loc[mask] = study_days
        
        return result
    
    @staticmethod
    def validate_date_pairs(
        start_series: pd.Series,
        end_series: pd.Series,
    ) -> tuple[pd.Series, int]:
        """Validate start/end date pairs.
        
        Args:
            start_series: Start dates
            end_series: End dates
            
        Returns:
            Tuple of (mask of invalid pairs, count of violations)
        """
        start_dates = pd.to_datetime(start_series, errors="coerce")
        end_dates = pd.to_datetime(end_series, errors="coerce")
        
        # Find where both are present and start > end
        both_present = start_dates.notna() & end_dates.notna()
        invalid = both_present & (start_dates > end_dates)
        
        return invalid, invalid.sum()


class NumericTransformer:
    """Optimized numeric transformations."""
    
    @staticmethod
    def force_numeric(series: pd.Series, *, fill_invalid: bool = False) -> pd.Series:
        """Force series to numeric type (vectorized).
        
        Args:
            series: Input series
            fill_invalid: Whether to fill invalid values with NA
            
        Returns:
            Numeric series
        """
        result = pd.to_numeric(series, errors="coerce")
        
        if not fill_invalid:
            # Keep original string values where coercion failed
            mask = result.isna() & series.notna()
            result = result.astype(object)
            result.loc[mask] = series.loc[mask]
        
        return result
    
    @staticmethod
    def validate_non_negative(series: pd.Series) -> tuple[pd.Series, int]:
        """Validate non-negative values.
        
        Args:
            series: Numeric series
            
        Returns:
            Tuple of (mask of negative values, count)
        """
        numeric = pd.to_numeric(series, errors="coerce")
        negative = numeric.notna() & (numeric < 0)
        return negative, negative.sum()
    
    @staticmethod
    def validate_range(
        series: pd.Series,
        low_series: pd.Series,
        high_series: pd.Series,
    ) -> tuple[pd.Series, int]:
        """Validate values are within range.
        
        Args:
            series: Value series
            low_series: Lower bound series
            high_series: Upper bound series
            
        Returns:
            Tuple of (mask of out-of-range values, count)
        """
        values = pd.to_numeric(series, errors="coerce")
        low = pd.to_numeric(low_series, errors="coerce")
        high = pd.to_numeric(high_series, errors="coerce")
        
        all_present = values.notna() & low.notna() & high.notna()
        out_of_range = all_present & ((values < low) | (values > high))
        
        return out_of_range, out_of_range.sum()


class TextTransformer:
    """Optimized text transformations."""
    
    @staticmethod
    def normalize_whitespace(series: pd.Series) -> pd.Series:
        """Normalize whitespace (vectorized).
        
        Args:
            series: Text series
            
        Returns:
            Normalized series
        """
        result = series.astype(str).str.strip()
        result = result.str.replace(r"\s+", " ", regex=True)
        result = result.replace("nan", "")
        return result
    
    @staticmethod
    def truncate(series: pd.Series, max_length: int) -> pd.Series:
        """Truncate strings to maximum length (vectorized).
        
        Args:
            series: Text series
            max_length: Maximum length
            
        Returns:
            Truncated series
        """
        return series.astype(str).str[:max_length]
    
    @staticmethod
    def replace_unknown(series: pd.Series, replacement: str = "") -> pd.Series:
        """Replace unknown/missing indicators (vectorized).
        
        Args:
            series: Text series
            replacement: Replacement value
            
        Returns:
            Cleaned series
        """
        result = series.astype(str).str.upper()
        
        unknown_values = {
            "UNKNOWN",
            "UK",
            "N/A",
            "NA",
            "NOT APPLICABLE",
            "NOT AVAILABLE",
            "MISSING",
            ".",
            "",
        }
        
        mask = result.isin(unknown_values)
        result = series.copy()
        result.loc[mask] = replacement
        
        return result


class CodelistTransformer:
    """Optimized codelist transformations."""
    
    @staticmethod
    def apply_codelist(
        series: pd.Series,
        codelist_mapping: dict[str, str],
        *,
        case_insensitive: bool = True,
    ) -> pd.Series:
        """Apply codelist transformation (vectorized).
        
        Args:
            series: Input series
            codelist_mapping: Value -> canonical mapping
            case_insensitive: Whether to match case-insensitively
            
        Returns:
            Transformed series
        """
        result = series.copy()
        
        if case_insensitive:
            # Build case-insensitive mapping
            upper_mapping = {k.upper(): v for k, v in codelist_mapping.items()}
            upper_series = series.astype(str).str.upper()
            
            # Apply mapping
            mapped = upper_series.map(upper_mapping)
            result = mapped.where(mapped.notna(), result)
        else:
            # Direct mapping
            mapped = series.map(codelist_mapping)
            result = mapped.where(mapped.notna(), result)
        
        return result
    
    @staticmethod
    def validate_codelist(
        series: pd.Series,
        valid_values: set[str],
        *,
        case_insensitive: bool = True,
    ) -> tuple[pd.Series, list[str]]:
        """Validate values against codelist.
        
        Args:
            series: Input series
            valid_values: Set of valid values
            case_insensitive: Whether to match case-insensitively
            
        Returns:
            Tuple of (mask of invalid values, list of invalid unique values)
        """
        if case_insensitive:
            valid_upper = {v.upper() for v in valid_values}
            series_upper = series.astype(str).str.strip().str.upper()
            invalid = ~series_upper.isin(valid_upper) & (series_upper != "")
        else:
            series_clean = series.astype(str).str.strip()
            invalid = ~series_clean.isin(valid_values) & (series_clean != "")
        
        invalid_values = series[invalid].unique().tolist()
        
        return invalid, invalid_values
