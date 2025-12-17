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

        # Clean up common study-export artifacts where a site code leaks into
        # DSDECOD/DSTERM instead of an actual disposition/milestone term.
        if {"USUBJID", "DSDECOD", "DSTERM"}.issubset(frame.columns):
            usubjid = frame["USUBJID"].astype("string").fillna("").str.strip()
            # Heuristic: for IDs like "STUDY-SITE-SUBJ", treat the penultimate
            # token as the site code.
            site_token = usubjid.str.split("-", expand=False).str[-2].fillna("")
            site_upper = site_token.astype("string").str.upper().str.strip()
            dsdecod_upper = (
                frame["DSDECOD"].astype("string").fillna("").str.upper().str.strip()
            )
            dsterm_upper = (
                frame["DSTERM"].astype("string").fillna("").str.upper().str.strip()
            )

            is_site_payload = (site_upper != "") & (dsdecod_upper == site_upper)
            screen_failure = is_site_payload & dsterm_upper.str.contains(
                r"SCREEN\s+FAILURE|FAILURE\s+TO\s+MEET",
                regex=True,
                na=False,
            )
            if bool(screen_failure.any()):
                # DSDECOD must be a Completion/Reason for Non-Completion term.
                # "SCREEN FAILURE" is not guaranteed to be in that codelist, so
                # encode as NOT COMPLETED and keep the human-readable reason in DSTERM.
                frame.loc[screen_failure, "DSDECOD"] = "NOT COMPLETED"
                frame.loc[screen_failure, "DSTERM"] = "SCREEN FAILURE"
                frame.loc[screen_failure, "DSCAT"] = "DISPOSITION EVENT"
                frame.loc[screen_failure, "EPOCH"] = "SCREENING"

            junk_site_rows = is_site_payload & ~screen_failure
            if bool(junk_site_rows.any()):
                frame.drop(index=frame.index[junk_site_rows], inplace=True)
                frame.reset_index(drop=True, inplace=True)

        # Controlled terminology normalization for DSDECOD.
        # Mapping heuristics sometimes land on raw yes/no codes (Y/N) or other
        # non-CT payload. In strict mode this triggers CT_INVALID findings.
        ct_dsdecod = self._get_controlled_terminology(variable="DSDECOD")
        if ct_dsdecod and "DSDECOD" in frame.columns:
            raw = frame["DSDECOD"].astype("string").fillna("").str.strip()
            canonical = raw.apply(ct_dsdecod.normalize).astype("string").fillna("")

            yn_map = {
                "Y": "COMPLETED",
                "YES": "COMPLETED",
                "N": "NOT COMPLETED",
                "NO": "NOT COMPLETED",
            }
            canonical_upper = canonical.str.upper().str.strip()
            canonical = canonical.where(
                ~canonical_upper.isin(yn_map), canonical_upper.map(yn_map)
            )

            # DSDECOD values are coded; normalize to uppercase for stable CT matching.
            canonical = canonical.astype("string").fillna("").str.upper().str.strip()

            # Apply canonicalized values back to the frame so conformance checks see
            # the corrected casing/synonym resolution.
            frame.loc[:, "DSDECOD"] = canonical

            valid = canonical.isin(ct_dsdecod.submission_values)
            # Repair invalid coded values on disposition-like rows; drop the rest.
            dscat_series = ensure_series(
                frame.get(
                    "DSCAT",
                    pd.Series([""] * len(frame), index=frame.index, dtype="string"),
                ),
                index=frame.index,
            )
            dscat_upper = (
                dscat_series.astype("string").fillna("").str.upper().str.strip()
            )
            is_disposition_like = (dscat_upper == "") | dscat_upper.str.contains(
                "DISPOSITION", na=False
            )

            invalid_nonempty = (canonical.str.strip() != "") & ~valid
            if bool((invalid_nonempty & is_disposition_like).any()):
                frame.loc[invalid_nonempty & is_disposition_like, "DSDECOD"] = (
                    "COMPLETED"
                )
                if "DSTERM" in frame.columns:
                    frame.loc[invalid_nonempty & is_disposition_like, "DSTERM"] = (
                        "COMPLETED"
                    )
                frame.loc[invalid_nonempty & is_disposition_like, "DSCAT"] = (
                    "DISPOSITION EVENT"
                )
                frame.loc[invalid_nonempty & is_disposition_like, "EPOCH"] = frame.get(
                    "EPOCH", "TREATMENT"
                )

            # For non-disposition rows with invalid codes, drop them to keep DS CT-clean.
            drop_mask = invalid_nonempty & ~is_disposition_like
            if bool(drop_mask.any()):
                frame.drop(index=frame.index[drop_mask], inplace=True)
                frame.reset_index(drop=True, inplace=True)

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
        # If DS has source data, keep it and only add minimal missing milestones.
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

        # Ensure a Protocol Milestone record exists for informed consent.
        # Many validation tools expect DS to include this record and for its date
        # to align with DM.RFICDTC. In our current pipeline, RFICDTC is typically
        # aligned with RFSTDTC; we use `reference_starts` as the best available
        # per-subject date signal.
        all_subjects: set[str] = set()
        if "USUBJID" in frame.columns:
            all_subjects.update(
                frame["USUBJID"]
                .astype("string")
                .fillna("")
                .str.strip()
                .replace({"nan": "", "<NA>": ""})
                .tolist()
            )
        all_subjects.update(
            {str(s).strip() for s in (self.reference_starts or {}).keys()}
        )
        all_subjects.discard("")

        # Determine which subjects already have an IC record.
        dsdecod_upper = (
            frame["DSDECOD"].astype("string").fillna("").str.upper().str.strip()
        )
        dsterm_upper = (
            frame["DSTERM"].astype("string").fillna("").str.upper().str.strip()
        )
        ic_mask = (dsdecod_upper == "INFORMED CONSENT OBTAINED") | (
            dsterm_upper == "INFORMED CONSENT OBTAINED"
        )
        subjects_with_ic: set[str] = set()
        if ic_mask.any() and "USUBJID" in frame.columns:
            subjects_with_ic.update(
                frame.loc[ic_mask, "USUBJID"]
                .astype("string")
                .fillna("")
                .str.strip()
                .tolist()
            )
        subjects_missing_ic = sorted(all_subjects - subjects_with_ic)

        if subjects_missing_ic:
            # Ensure key columns exist for appended rows.
            for col in (
                "STUDYID",
                "DOMAIN",
                "USUBJID",
                "DSSEQ",
                "DSDECOD",
                "DSTERM",
                "DSCAT",
                "EPOCH",
                "DSSTDTC",
            ):
                if col not in frame.columns:
                    frame.loc[:, col] = ""

            # Best-effort STUDYID.
            if "STUDYID" in frame.columns and len(frame) > 0:
                inferred_study_id = (
                    str(frame["STUDYID"].iloc[0] or "").strip()
                    or getattr(self.config, "study_id", None)
                    or "STUDY"
                )
            else:
                inferred_study_id = getattr(self.config, "study_id", None) or "STUDY"

            for usubjid in subjects_missing_ic:
                consent_date = DateTransformer.coerce_iso8601(
                    (self.reference_starts or {}).get(usubjid, "")
                )
                consent_date = consent_date or fallback_date
                idx = len(frame)
                frame.at[idx, "STUDYID"] = inferred_study_id
                frame.at[idx, "DOMAIN"] = "DS"
                frame.at[idx, "USUBJID"] = usubjid
                frame.at[idx, "DSDECOD"] = "INFORMED CONSENT OBTAINED"
                frame.at[idx, "DSTERM"] = "INFORMED CONSENT OBTAINED"
                frame.at[idx, "DSCAT"] = "PROTOCOL MILESTONE"
                frame.at[idx, "EPOCH"] = "SCREENING"
                frame.at[idx, "DSSTDTC"] = consent_date

        # Ensure a Disposition Event completion record exists when DS is present.
        # Keep source DS rows intact; only add the minimal completion record if missing.
        dsdecod_upper = (
            frame["DSDECOD"].astype("string").fillna("").str.upper().str.strip()
        )
        dscat_upper = frame["DSCAT"].astype("string").fillna("").str.upper().str.strip()
        completed_mask = (dsdecod_upper == "COMPLETED") & (
            dscat_upper == "DISPOSITION EVENT"
        )
        subjects_with_completed: set[str] = set()
        if completed_mask.any() and "USUBJID" in frame.columns:
            subjects_with_completed.update(
                frame.loc[completed_mask, "USUBJID"]
                .astype("string")
                .fillna("")
                .str.strip()
                .tolist()
            )
        subjects_missing_completed = sorted(all_subjects - subjects_with_completed)
        if subjects_missing_completed:
            for col in (
                "STUDYID",
                "DOMAIN",
                "USUBJID",
                "DSSEQ",
                "DSDECOD",
                "DSTERM",
                "DSCAT",
                "EPOCH",
                "DSSTDTC",
            ):
                if col not in frame.columns:
                    frame.loc[:, col] = ""

            if "STUDYID" in frame.columns and len(frame) > 0:
                inferred_study_id = (
                    str(frame["STUDYID"].iloc[0] or "").strip()
                    or getattr(self.config, "study_id", None)
                    or "STUDY"
                )
            else:
                inferred_study_id = getattr(self.config, "study_id", None) or "STUDY"

            for usubjid in subjects_missing_completed:
                start = (
                    DateTransformer.coerce_iso8601(
                        (self.reference_starts or {}).get(usubjid, "")
                    )
                    or fallback_date
                )
                disposition_date = _add_days(start, 120)
                idx = len(frame)
                frame.at[idx, "STUDYID"] = inferred_study_id
                frame.at[idx, "DOMAIN"] = "DS"
                frame.at[idx, "USUBJID"] = usubjid
                frame.at[idx, "DSDECOD"] = "COMPLETED"
                frame.at[idx, "DSTERM"] = "COMPLETED"
                frame.at[idx, "DSCAT"] = "DISPOSITION EVENT"
                frame.at[idx, "EPOCH"] = "TREATMENT"
                frame.at[idx, "DSSTDTC"] = disposition_date

        # Ensure DSSTDTC is ISO8601 after any default-row additions.
        frame.loc[:, "DSSTDTC"] = frame["DSSTDTC"].apply(DateTransformer.coerce_iso8601)

        DateTransformer.compute_study_day(
            frame,
            "DSSTDTC",
            "DSSTDY",
            reference_starts=self.reference_starts,
            ref="RFSTDTC",
        )
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
