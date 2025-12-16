"""Date, time, and duration transformation utilities for SDTM domains."""

from __future__ import annotations

from typing import Any, Sequence

import pandas as pd

from ...entities.sdtm_domain import SDTMVariable
from ....pandas_utils import ensure_series
from .iso8601 import normalize_iso8601, normalize_iso8601_duration


class DateTransformer:
    """Transforms date, time, and duration values for SDTM compliance."""

    @staticmethod
    def normalize_dates(
        frame: pd.DataFrame, domain_variables: Sequence[SDTMVariable]
    ) -> None:
        for var in domain_variables:
            if (
                var.type in ("Char", "Num")
                and "DTC" in var.name
                and var.name in frame.columns
            ):
                frame[var.name] = frame[var.name].apply(normalize_iso8601)

    @staticmethod
    def normalize_durations(
        frame: pd.DataFrame, domain_variables: Sequence[SDTMVariable]
    ) -> None:
        for var in domain_variables:
            if var.type == "Char" and "DUR" in var.name and var.name in frame.columns:
                frame[var.name] = frame[var.name].apply(normalize_iso8601_duration)

    @staticmethod
    def calculate_dy(
        frame: pd.DataFrame,
        domain_variables: Sequence[SDTMVariable],
        reference_starts: dict[str, str],
    ) -> None:
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
                return
        if baseline.isna().all():
            if ref and ref in frame.columns:
                baseline = pd.to_datetime(frame[ref], errors="coerce")
            else:
                return

        baseline = baseline.bfill().ffill()
        deltas = (dates - baseline).dt.days  # type: ignore[attr-defined]

        study_days = deltas.where(
            deltas.isna(),
            deltas.apply(lambda x: x + 1 if x >= 0 else x),
        )

        frame[dy_var] = pd.to_numeric(study_days, errors="coerce")

    @staticmethod
    def ensure_date_pair_order(
        frame: pd.DataFrame, start_var: str, end_var: str | None
    ) -> None:
        if start_var not in frame.columns:
            return
        start = frame[start_var].apply(DateTransformer.coerce_iso8601)
        frame[start_var] = start
        if end_var and end_var in frame.columns:
            end = frame[end_var].apply(DateTransformer.coerce_iso8601)
            needs_swap = (end == "") | (end < start)
            frame[end_var] = end.where(~needs_swap, start)

    @staticmethod
    def coerce_iso8601(raw_value: Any) -> str:
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
