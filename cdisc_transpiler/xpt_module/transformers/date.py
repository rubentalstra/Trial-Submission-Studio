"""Date, time, and duration transformation utilities for SDTM domains.

This module provides specialized transformation logic for date/time values,
durations, and study day calculations according to SDTM standards.
"""

from __future__ import annotations

from typing import Sequence

import pandas as pd

from ...domains_module import SDTMVariable
from .iso8601 import normalize_iso8601, normalize_iso8601_duration
from ...pandas_utils import ensure_series


class DateTransformer:
    """Transforms date, time, and duration values for SDTM compliance.

    This class provides static methods for:
    - ISO 8601 date/time normalization
    - ISO 8601 duration normalization
    - Study day calculations (per SDTM conventions)
    - Date pair validation and ordering
    """

    @staticmethod
    def normalize_dates(
        frame: pd.DataFrame,
        domain_variables: Sequence[SDTMVariable],
    ) -> None:
        """Normalize all date/datetime columns to ISO 8601 strings.

        Args:
            frame: DataFrame to modify in-place
            domain_variables: List of SDTMVariable objects defining domain structure
        """
        for var in domain_variables:
            if var.type in ("Char", "Num") and "DTC" in var.name:
                if var.name in frame.columns:
                    frame[var.name] = frame[var.name].apply(normalize_iso8601)

    @staticmethod
    def normalize_durations(
        frame: pd.DataFrame,
        domain_variables: Sequence[SDTMVariable],
    ) -> None:
        """Normalize all duration columns to ISO 8601 duration strings.

        Args:
            frame: DataFrame to modify in-place
            domain_variables: List of SDTMVariable objects defining domain structure
        """
        for var in domain_variables:
            if var.type == "Char" and "DUR" in var.name:
                if var.name in frame.columns:
                    frame[var.name] = frame[var.name].apply(normalize_iso8601_duration)

    @staticmethod
    def calculate_dy(
        frame: pd.DataFrame,
        domain_variables: Sequence[SDTMVariable],
        reference_starts: dict[str, str],
    ) -> None:
        """Calculate --DY variables if --DTC and RFSTDTC are present.

        Args:
            frame: DataFrame to modify in-place
            domain_variables: List of SDTMVariable objects defining domain structure
            reference_starts: Mapping of USUBJID -> RFSTDTC for study day calculations
        """
        if not reference_starts:
            return

        for var in domain_variables:
            if var.name.endswith("DY") and var.name[:-2] + "DTC" in frame.columns:
                dtc_col = var.name[:-2] + "DTC"
                if "USUBJID" in frame.columns:
                    frame[var.name] = frame.apply(
                        lambda row: DateTransformer._compute_dy_for_row(
                            row["USUBJID"],
                            row.get(dtc_col),
                            reference_starts,
                        ),
                        axis=1,
                    )

    @staticmethod
    def _compute_dy_for_row(
        usubjid: str,
        dtc: str | None,
        reference_starts: dict[str, str],
    ) -> int | None:
        """Compute the study day for a given date and subject.

        Args:
            usubjid: Subject identifier
            dtc: Date/time string in ISO 8601 format
            reference_starts: Mapping of USUBJID -> RFSTDTC

        Returns:
            Study day (integer) or None if cannot be computed
        """
        if not dtc or not usubjid or usubjid not in reference_starts:
            return None
        try:
            start_date = pd.to_datetime(reference_starts[usubjid], errors="coerce")
            obs_date = pd.to_datetime(dtc, errors="coerce")
            if pd.isna(start_date) or pd.isna(obs_date):
                return None
            delta = (obs_date - start_date).days
            return delta + 1 if delta >= 0 else delta
        except (ValueError, TypeError):
            return None

    @staticmethod
    def compute_study_day(
        frame: pd.DataFrame,
        dtc_var: str,
        dy_var: str,
        reference_starts: dict[str, str] | None = None,
        ref: str | None = None,
    ) -> None:
        """Compute study day per SDTM conventions.

        SDTM Study Day calculation:
        - If event_date >= RFSTDTC: study_day = (event_date - RFSTDTC).days + 1
        - If event_date < RFSTDTC: study_day = (event_date - RFSTDTC).days
        - There is NO Day 0 in SDTM
        - Missing dates should result in missing study day

        Args:
            frame: DataFrame to modify in-place
            dtc_var: Name of date/time column
            dy_var: Name of study day column to populate
            reference_starts: Optional mapping of USUBJID -> RFSTDTC
            ref: Optional reference date column name
        """
        if dtc_var not in frame.columns or dy_var not in frame.columns:
            return

        dates = pd.to_datetime(frame[dtc_var], errors="coerce")

        baseline: pd.Series | None = None
        if reference_starts and "USUBJID" in frame.columns:
            baseline_series = ensure_series(frame["USUBJID"])
            baseline = baseline_series.map(reference_starts.get)
            baseline = pd.to_datetime(baseline, errors="coerce")

        if baseline is None:
            if ref and ref in frame.columns:
                baseline = pd.to_datetime(frame[ref], errors="coerce")
            else:
                # If no reference date available, cannot compute study day
                return
        if baseline.isna().all():
            if ref and ref in frame.columns:
                baseline = pd.to_datetime(frame[ref], errors="coerce")
            else:
                return

        # Fill missing baselines from available values
        baseline = baseline.bfill().ffill()

        # Compute day difference
        deltas = (dates - baseline).dt.days

        # Per SDTM: add 1 for dates on or after reference start, no adjustment for dates before
        # This ensures there is no Day 0
        study_days = deltas.where(
            deltas.isna(),  # Keep NaN as NaN
            deltas.apply(lambda x: x + 1 if x >= 0 else x),
        )

        # Convert to numeric, keeping NaN for missing dates
        frame[dy_var] = pd.to_numeric(study_days, errors="coerce")

    @staticmethod
    def ensure_date_pair_order(
        frame: pd.DataFrame,
        start_var: str,
        end_var: str | None,
    ) -> None:
        """Ensure start date <= end date, swapping if needed.

        Args:
            frame: DataFrame to modify in-place
            start_var: Name of start date column
            end_var: Name of end date column (optional)
        """
        if start_var not in frame.columns:
            return
        start = frame[start_var].apply(DateTransformer.coerce_iso8601)
        frame[start_var] = start
        if end_var and end_var in frame.columns:
            end = frame[end_var].apply(DateTransformer.coerce_iso8601)
            needs_swap = (end == "") | (end < start)
            frame[end_var] = end.where(~needs_swap, start)

    @staticmethod
    def coerce_iso8601(raw_value) -> str:
        """Coerce a value to ISO 8601 date format, handling special cases.

        This method normalizes dates and handles special tokens like "NK" (unknown)
        by replacing them with valid date components.

        Args:
            raw_value: Value to coerce (string, datetime, etc.)

        Returns:
            ISO 8601 date string (YYYY-MM-DD) or empty string if invalid
        """
        normalized = normalize_iso8601(raw_value)
        fixed = normalized
        if "NK" in normalized.upper():
            fixed = normalized.upper().replace("NK", "01")
        try:
            parsed = pd.to_datetime(fixed, errors="coerce", utc=False)
        except (TypeError, ValueError, OverflowError):
            parsed = pd.NaT
        if not isinstance(parsed, pd.Timestamp):
            return ""
        return parsed.date().isoformat()
