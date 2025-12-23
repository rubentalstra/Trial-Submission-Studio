"""Date, time, and duration transformation utilities for SDTM domains."""

from collections.abc import Sequence
from typing import Any, cast

import pandas as pd

from ....pandas_utils import ensure_series
from ...entities.sdtm_domain import SDTMVariable
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
                source = ensure_series(frame[var.name], index=frame.index).astype(
                    "string"
                )
                normalized = ensure_series(source.map(normalize_iso8601)).astype(
                    "string"
                )
                frame.loc[:, var.name] = normalized

    @staticmethod
    def normalize_durations(
        frame: pd.DataFrame, domain_variables: Sequence[SDTMVariable]
    ) -> None:
        for var in domain_variables:
            if var.type == "Char" and "DUR" in var.name and var.name in frame.columns:
                source = ensure_series(frame[var.name], index=frame.index).astype(
                    "string"
                )
                normalized = ensure_series(
                    source.map(normalize_iso8601_duration)
                ).astype("string")
                frame.loc[:, var.name] = normalized

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
                    usubjids = ensure_series(frame["USUBJID"], index=frame.index)
                    dtcs = ensure_series(frame[dtc_col], index=frame.index)
                    dy_values: list[int | None] = []
                    for usubjid, dtc in zip(usubjids, dtcs):
                        dy_values.append(
                            DateTransformer._compute_dy_for_row(
                                str(usubjid) if pd.notna(usubjid) else "",
                                str(dtc) if pd.notna(dtc) else None,
                                reference_starts,
                            )
                        )
                    # Use a plain numeric series (float with NaN for missing) to avoid
                    # dtype-churn warnings *and* keep downstream writers (e.g., XPT)
                    # compatible with the produced dtypes.
                    frame.loc[:, var.name] = pd.to_numeric(dy_values, errors="coerce")

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
            # SDTMIG v3.4 4.4.4: compare the *date portion* of DTC to the
            # date portion of RFSTDTC (reference start) when calculating study day.
            delta = (obs_date.date() - start_date.date()).days
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
        if dtc_var not in frame.columns:
            return

        # Some metadata extracts omit DY variables (e.g., AESTDY/CMSTDY).
        # Pinnacle 21 expects them when the corresponding DTC is present.
        if dy_var not in frame.columns:
            frame.loc[:, dy_var] = pd.NA

        # SDTMIG v3.4 4.4.4: study day is calculated using the date portion.
        dates = pd.to_datetime(frame[dtc_var], errors="coerce").dt.normalize()

        baseline: pd.Series | None = None
        if reference_starts and "USUBJID" in frame.columns:
            baseline_series = ensure_series(frame["USUBJID"])
            baseline = baseline_series.map(reference_starts.get)
            baseline = pd.to_datetime(baseline, errors="coerce").dt.normalize()

        if baseline is None:
            if ref and ref in frame.columns:
                baseline = pd.to_datetime(frame[ref], errors="coerce").dt.normalize()
            else:
                return
        if baseline.isna().all():
            if ref and ref in frame.columns:
                baseline = pd.to_datetime(frame[ref], errors="coerce").dt.normalize()
            else:
                return

        baseline = ensure_series(baseline.bfill().ffill())
        # dates - baseline results in a Series of Timedeltas. .dt.days extracts the integer days.
        diff = dates - baseline
        # Cast to Any to access .dt.days which pyright misses on the result of subtraction
        deltas = ensure_series(cast("Any", diff).dt.days)

        def adjust_study_day(x: Any) -> Any:
            if pd.isna(x):
                return x
            # SDTM rule: no day 0. If delta >= 0, it's day 1+. If delta < 0, it's day -1-.
            return x + 1 if x >= 0 else x

        study_days = ensure_series(deltas.map(adjust_study_day))
        frame.loc[:, dy_var] = pd.to_numeric(study_days, errors="coerce")

    @staticmethod
    def ensure_date_pair_order(
        frame: pd.DataFrame, start_var: str, end_var: str | None
    ) -> None:
        if start_var not in frame.columns:
            return
        start = ensure_series(frame[start_var].map(DateTransformer.coerce_iso8601))
        frame.loc[:, start_var] = start
        if end_var and end_var in frame.columns:
            end = ensure_series(frame[end_var].map(DateTransformer.coerce_iso8601))
            needs_swap = (end == "") | (end < start)
            frame.loc[:, end_var] = end.where(~needs_swap, start)

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
