"""Domain processor for Disposition (DS) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, NumericTransformer


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

            # If the coded value is invalid but the verbatim term normalizes to
            # a valid code, prefer that derived code.
            if "DSTERM" in frame.columns:
                dsterm_raw = frame["DSTERM"].astype("string").fillna("").str.strip()
                dsterm_code = (
                    dsterm_raw.apply(ct_dsdecod.normalize).astype("string").fillna("")
                )
                dsterm_code = (
                    dsterm_code.astype("string").fillna("").str.upper().str.strip()
                )
                valid_from_term = dsterm_code.isin(ct_dsdecod.submission_values)
                invalid_from_decod = ~canonical.isin(ct_dsdecod.submission_values)
                use_term_code = valid_from_term & invalid_from_decod
                if bool(use_term_code.any()):
                    frame.loc[use_term_code, "DSDECOD"] = dsterm_code.loc[use_term_code]

        if "DSSTDTC" in frame.columns:
            frame.loc[:, "DSSTDTC"] = frame["DSSTDTC"].apply(
                DateTransformer.coerce_iso8601
            )
        if "DSDTC" in frame.columns:
            frame.loc[:, "DSDTC"] = frame["DSDTC"].apply(DateTransformer.coerce_iso8601)

        if "DSSTDTC" in frame.columns and "DSSTDY" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "DSSTDTC",
                "DSSTDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )

        if "DSDTC" in frame.columns and "DSDY" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "DSDTC",
                "DSDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )

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
