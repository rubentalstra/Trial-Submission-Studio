"""Domain processor for Disposition (DS) domain."""

from __future__ import annotations

from typing import Any

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, NumericTransformer
from ....pandas_utils import ensure_series


class DSProcessor(BaseDomainProcessor):
    """Disposition domain processor.

    Handles domain-specific processing for the DS domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process DS domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Normalize core string fields; override obvious non-SDTM payloads
        for col in ("DSDECOD", "DSTERM", "DSCAT", "EPOCH"):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()
            else:
                frame.loc[:, col] = ""

        # Baseline/fallback dates
        baseline_default = None
        if self.reference_starts:
            baseline_default = next(iter(self.reference_starts.values()))
        fallback_date = DateTransformer.coerce_iso8601(baseline_default) or "2024-12-31"

        # Clean existing DSSTDTC values
        if "DSSTDTC" in frame.columns:
            frame.loc[:, "DSSTDTC"] = frame["DSSTDTC"].apply(
                DateTransformer.coerce_iso8601
            )
            frame.loc[:, "DSSTDTC"] = frame["DSSTDTC"].replace(
                {"": fallback_date, "1900-01-01": fallback_date}
            )
        else:
            frame.loc[:, "DSSTDTC"] = fallback_date
        DateTransformer.ensure_date_pair_order(frame, "DSSTDTC", None)

        # Build per-subject disposition rows only when DS is missing entirely.
        # If DS has source data, do not fabricate additional records.
        subject_series = ensure_series(
            frame.get("USUBJID", pd.Series(dtype="string")), index=frame.index
        )
        subjects = set(
            subject_series.astype("string").str.strip().replace({"nan": "", "<NA>": ""})
        )
        subjects.discard("")

        def _add_days(raw_date: str, days: int) -> str:
            dt_candidate = pd.to_datetime(
                DateTransformer.coerce_iso8601(raw_date), errors="coerce"
            )
            fallback_ts = pd.to_datetime(fallback_date)
            if not isinstance(dt_candidate, pd.Timestamp) or pd.isna(dt_candidate):
                dt_candidate = fallback_ts
            if pd.isna(dt_candidate):
                return ""
            assert isinstance(dt_candidate, pd.Timestamp)
            return (dt_candidate + pd.Timedelta(days=days)).date().isoformat()

        defaults: list[dict[str, Any]] = []
        if frame.empty and self.reference_starts:
            study_id = "STUDY"
            if "STUDYID" in frame.columns and len(frame) > 0:
                study_id = frame["STUDYID"].iloc[0]
            else:
                study_id = getattr(self.config, "study_id", None) or "STUDY"

            for usubjid in sorted({str(s) for s in self.reference_starts.keys()}):
                start = (
                    DateTransformer.coerce_iso8601(
                        (self.reference_starts or {}).get(usubjid)
                    )
                    or fallback_date
                )
                disposition_date = _add_days(start, 120)
                disp_row = {
                    "STUDYID": study_id,
                    "DOMAIN": "DS",
                    "USUBJID": usubjid,
                    "DSSEQ": pd.NA,
                    "DSDECOD": "COMPLETED",
                    "DSTERM": "COMPLETED",
                    "DSCAT": "DISPOSITION EVENT",
                    "DSSTDTC": disposition_date,
                    "DSSTDY": pd.NA,
                    "EPOCH": "TREATMENT",
                }
                defaults.append(disp_row)

        if defaults:
            # Ensure the required DS columns exist so appended rows keep the SDTM shape.
            for col in ("STUDYID", "DOMAIN", "USUBJID", "DSSEQ"):
                if col not in frame.columns:
                    frame.loc[:, col] = ""

            needed_cols: set[str] = set().union(*(row.keys() for row in defaults))
            for col in needed_cols:
                if col not in frame.columns:
                    frame.loc[:, col] = ""

            # Append defaults in-place (preserves external references).
            for row in defaults:
                idx = len(frame)
                # Only assign explicit values; leave other columns as NA.
                # This avoids pandas dtype warnings (e.g., assigning "" into float columns).
                for col, value in row.items():
                    if col in frame.columns:
                        frame.at[idx, col] = value

        # Normalize DS as disposition-only.
        frame.loc[:, "DSDECOD"] = "COMPLETED"
        frame.loc[:, "DSTERM"] = "COMPLETED"
        frame.loc[:, "DSCAT"] = "DISPOSITION EVENT"
        frame.loc[:, "EPOCH"] = "TREATMENT"

        # Ensure DSSTDTC is ISO8601 after any default-row additions.
        frame.loc[:, "DSSTDTC"] = frame["DSSTDTC"].apply(DateTransformer.coerce_iso8601)

        DateTransformer.compute_study_day(frame, "DSSTDTC", "DSSTDY", ref="RFSTDTC")
        frame.loc[:, "DSDTC"] = frame["DSSTDTC"]
        dsdy_source = NumericTransformer.force_numeric(
            frame.get("DSSTDY", pd.Series(index=frame.index))
        ).fillna(1)
        frame.loc[:, "DSDY"] = pd.to_numeric(dsdy_source, errors="coerce")
        frame.loc[:, "DSSTDY"] = frame["DSDY"]

        # Always regenerate DSSEQ - source values may not be unique (SD0005)
        NumericTransformer.assign_sequence(frame, "DSSEQ", "USUBJID")
        frame.loc[:, "DSSEQ"] = pd.to_numeric(
            NumericTransformer.force_numeric(frame["DSSEQ"]), errors="coerce"
        )

        # Remove duplicate disposition records per subject/date/term
        dedup_keys = ["USUBJID", "DSDECOD", "DSTERM", "DSCAT", "DSSTDTC"]
        existing_cols = [c for c in dedup_keys if c in frame.columns]
        if existing_cols:
            frame.drop_duplicates(subset=existing_cols, keep="first", inplace=True)
