"""Validation and column management utilities for SDTM domains.

This module provides validation logic to ensure XPT files comply with
SDTM requirements, including required values, field lengths, and column management.
"""

from __future__ import annotations

import pandas as pd


class XPTValidator:
    """Validates SDTM DataFrames for XPT compliance.
    
    This class provides methods for:
    - Enforcing required value presence
    - Enforcing field length constraints
    - Dropping empty optional columns
    - Reordering columns to match domain specification
    """

    @staticmethod
    def enforce_required_values(
        frame: pd.DataFrame,
        domain_variables: list,
        lenient: bool = False,
    ) -> None:
        """Enforce that required variables have non-missing values.
        
        Args:
            frame: DataFrame to validate
            domain_variables: List of SDTMVariable objects defining domain structure
            lenient: If True, skip validation (useful for Dataset-XML generation)
            
        Raises:
            ValueError: If required variables have missing values (when not lenient)
        """
        if lenient:
            return
            
        for var in domain_variables:
            if (var.core or "").strip().lower() == "req" and var.name in frame.columns:
                # Use pd.isna() for robust check across dtypes
                if frame[var.name].isna().any():
                    raise ValueError(
                        f"Required variable {var.name} has missing values"
                    )

    @staticmethod
    def enforce_lengths(
        frame: pd.DataFrame,
        domain_variables: list,
    ) -> None:
        """Truncate character values to maximum length specified in domain.
        
        Per SDTM standards, character fields have maximum lengths that must
        be enforced before writing to XPT format.
        
        Args:
            frame: DataFrame to modify in-place
            domain_variables: List of SDTMVariable objects defining domain structure
        """
        for var in domain_variables:
            if var.type == "Char" and var.length and var.name in frame.columns:
                # Ensure a string-capable dtype before using .str accessor and
                # handle missing values safely to avoid pandas errors like
                # "Can only use .str accessor with string values".
                col = frame[var.name].astype("string")
                col = col.fillna("")
                frame[var.name] = col.str.slice(0, var.length)

    @staticmethod
    def drop_empty_optional_columns(
        frame: pd.DataFrame,
        domain_variables: list,
    ) -> None:
        """Remove permissible columns that contain no data.
        
        This removes PERM (permissible) columns that are completely empty,
        while keeping required and expected columns even if empty.
        
        Args:
            frame: DataFrame to modify in-place
            domain_variables: List of SDTMVariable objects defining domain structure
        """
        drop_cols: list[str] = []
        missing_tokens = {"", "NAN", "<NA>", "NA", "N/A", "NONE"}

        for var in domain_variables:
            if var.name not in frame.columns:
                continue
            core = (getattr(var, "core", None) or "").upper()
            
            # Drop fully empty PERM variables; keep Req/Exp (e.g., ARMNRS) even when empty
            if core != "PERM":
                continue
            if var.name in {"ARMNRS"}:
                continue
            # Keep date/time/duration columns even if empty
            if any(token in var.name for token in ("DTC", "DY", "DUR")):
                continue
                
            series = frame[var.name]
            if series.dtype.kind in "biufc":
                # Numeric columns - check for all NaN
                if series.isna().all():
                    drop_cols.append(var.name)
            else:
                # Character columns - check for all empty/missing
                normalized = series.astype("string").fillna("")
                stripped = normalized.str.strip().str.upper()
                if stripped.isin(missing_tokens).all():
                    drop_cols.append(var.name)

        if drop_cols:
            frame.drop(columns=drop_cols, inplace=True)

    @staticmethod
    def reorder_columns(
        frame: pd.DataFrame,
        domain_variables: list,
    ) -> None:
        """Align columns to domain metadata order.
        
        This ensures columns appear in the order specified by the domain
        definition, with any extra columns appended at the end.
        
        Args:
            frame: DataFrame to modify in-place
            domain_variables: List of SDTMVariable objects defining domain structure
        """
        ordering = [
            var.name for var in domain_variables if var.name in frame.columns
        ]
        extras = [col for col in frame.columns if col not in ordering]
        frame_reordered = frame.reindex(columns=ordering + extras)
        
        # Update frame in-place
        frame.drop(columns=list(frame.columns), inplace=True)
        for col in frame_reordered.columns:
            frame[col] = frame_reordered[col]

    @staticmethod
    def validate_required_values(
        frame: pd.DataFrame,
        domain_variables: list,
    ) -> list[str]:
        """Check for missing required values and return list of problematic variables.
        
        This is a non-raising validation that returns a list of variables
        with missing required values, useful for reporting.
        
        Args:
            frame: DataFrame to validate
            domain_variables: List of SDTMVariable objects defining domain structure
            
        Returns:
            List of variable names with missing required values
        """
        missing: list[str] = []
        
        for variable in domain_variables:
            if (variable.core or "").strip().lower() != "req":
                continue
            if variable.name not in frame.columns:
                continue
                
            series = frame[variable.name]
            if series.dtype.kind in "biufc":
                # Numeric types
                is_empty = series.isna()
            else:
                # Character types
                is_empty = series.astype(str).str.strip().isin(["", "nan"])
                
            if is_empty.any():
                missing.append(variable.name)
                
        return missing
