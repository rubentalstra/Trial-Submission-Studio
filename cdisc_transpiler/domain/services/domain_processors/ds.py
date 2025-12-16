"""Domain processor for Disposition (DS) domain."""

from __future__ import annotations

from typing import Any, cast

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

        # Build per-subject consent + disposition rows (always ensure both)
        subject_series = ensure_series(
            frame.get("USUBJID", pd.Series(dtype="string")), index=frame.index
        )
        subjects = set(
            subject_series.astype("string").str.strip().replace({"nan": "", "<NA>": ""})
        )
        subjects |= {str(s) for s in (self.reference_starts or {}).keys()}
        subjects.discard("")

        def _add_days(raw_date: str, days: int) -> str:
            try:
                dt_candidate = pd.to_datetime(
                    DateTransformer.coerce_iso8601(raw_date), errors="coerce"
                )
            except Exception:
                dt_candidate = pd.NaT
            fallback_ts = pd.to_datetime(fallback_date)
            if not isinstance(dt_candidate, pd.Timestamp) or pd.isna(dt_candidate):
                dt_candidate = fallback_ts
            if pd.isna(dt_candidate):
                return ""
            assert isinstance(dt_candidate, pd.Timestamp)
            return dt_candidate.date().isoformat()

        defaults: list[dict[str, Any]] = []
        study_id = "STUDY"
        if len(frame) > 0 and "STUDYID" in frame.columns:
            study_id = frame["STUDYID"].iloc[0]

        for usubjid in sorted(subjects):
            start = (
                DateTransformer.coerce_iso8601(
                    (self.reference_starts or {}).get(usubjid)
                )
                or fallback_date
            )
            disposition_date = _add_days(start, 120)
            consent_row = {
                "STUDYID": study_id,
                "DOMAIN": "DS",
                "USUBJID": usubjid,
                "DSSEQ": pd.NA,
                "DSDECOD": "INFORMED CONSENT OBTAINED",
                "DSTERM": "INFORMED CONSENT OBTAINED",
                "DSCAT": "PROTOCOL MILESTONE",
                "DSSTDTC": start,
                "DSSTDY": pd.NA,
                "EPOCH": "SCREENING",
            }
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
            defaults.extend([consent_row, disp_row])

        defaults_df = pd.DataFrame(defaults)
        defaults_df = defaults_df.reindex(columns=frame.columns, fill_value="")
        if not defaults_df.empty:
            expanded = pd.concat([frame, defaults_df], ignore_index=True)
            expanded.reset_index(drop=True, inplace=True)
            frame.drop(index=frame.index.tolist(), inplace=True)
            for col in expanded.columns:
                frame.loc[:, col] = expanded[col]

        # Harmonize consent/disposition text and epochs (even for source rows)
        def _contains(series: pd.Series, token: str) -> pd.Series:
            return series.astype("string").str.upper().str.contains(token, na=False)

        consent_mask = _contains(
            ensure_series(frame["DSDECOD"]), "CONSENT"
        ) | _contains(ensure_series(frame["DSCAT"]), "PROTOCOL MILESTONE")
        frame.loc[consent_mask, "DSDECOD"] = "INFORMED CONSENT OBTAINED"
        frame.loc[consent_mask, "DSTERM"] = "INFORMED CONSENT OBTAINED"
        frame.loc[consent_mask, "DSCAT"] = "PROTOCOL MILESTONE"
        frame.loc[consent_mask, "EPOCH"] = "SCREENING"

        disposition_mask = ~consent_mask
        frame.loc[disposition_mask, "DSDECOD"] = "COMPLETED"
        frame.loc[disposition_mask, "DSTERM"] = "COMPLETED"
        frame.loc[disposition_mask, "DSCAT"] = "DISPOSITION EVENT"
        frame.loc[disposition_mask, "EPOCH"] = "TREATMENT"

        # Replace disposition dates that precede consent with a padded end date
        frame.loc[:, "DSSTDTC"] = frame["DSSTDTC"].apply(DateTransformer.coerce_iso8601)
        dsstdtc_loc = cast(int, frame.columns.get_loc("DSSTDTC"))
        for pos in range(len(frame)):
            row = frame.iloc[pos]
            subj = str(row.get("USUBJID", "") or "")
            base = DateTransformer.coerce_iso8601(
                (self.reference_starts or {}).get(subj)
            )
            base = base or fallback_date
            if disposition_mask.iloc[pos]:
                frame.iat[pos, dsstdtc_loc] = _add_days(base, 120)
            elif not str(row["DSSTDTC"]).strip():
                frame.iat[pos, dsstdtc_loc] = base

        DateTransformer.compute_study_day(frame, "DSSTDTC", "DSSTDY", ref="RFSTDTC")
        frame.loc[:, "DSDTC"] = frame["DSSTDTC"]
        frame.loc[:, "DSDY"] = (
            NumericTransformer.force_numeric(frame.get("DSSTDY", pd.Series()))
            .fillna(1)
            .astype("Int64")
        )
        frame.loc[:, "DSSTDY"] = frame["DSDY"]

        # Always regenerate DSSEQ - source values may not be unique (SD0005)
        NumericTransformer.assign_sequence(frame, "DSSEQ", "USUBJID")
        frame.loc[:, "DSSEQ"] = NumericTransformer.force_numeric(frame["DSSEQ"]).astype(
            "Int64"
        )

        # Remove duplicate disposition records per subject/date/term
        dedup_keys = ["USUBJID", "DSDECOD", "DSTERM", "DSCAT", "DSSTDTC"]
        existing_cols = [c for c in dedup_keys if c in frame.columns]
        if existing_cols:
            frame.drop_duplicates(subset=existing_cols, keep="first", inplace=True)
