"""Domain processor for Disposition (DS) domain."""

from typing import override

import pandas as pd

from ..transformers import DateTransformer, NumericTransformer
from .base import BaseDomainProcessor


class DSProcessor(BaseDomainProcessor):
    """Disposition domain processor.

    Handles domain-specific processing for the DS domain.
    """

    @override
    def process(self, frame: pd.DataFrame) -> None:
        """Process DS domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Normalize core string fields.
        for col in ("DSDECOD", "DSTERM", "DSCAT", "EPOCH"):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()

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
                # encode as the canonical CT term when we can.
                frame.loc[screen_failure, "DSDECOD"] = "SCREEN FAILURE"
                frame.loc[screen_failure, "DSTERM"] = "SCREEN FAILURE"
                # Do not default DSCAT/EPOCH.

            junk_site_rows = is_site_payload & ~screen_failure
            if bool(junk_site_rows.any()):
                # Don't drop rows: mapping heuristics can temporarily land the site
                # code into DSDECOD/DSTERM. Preserve the record and clear the junk
                # payload so later normalization (including DSTERM->DSDECOD) can
                # still salvage the row.
                frame.loc[junk_site_rows, "DSDECOD"] = ""
                frame.loc[junk_site_rows & (dsterm_upper == site_upper), "DSTERM"] = ""

        # Fix common mis-mapping where a discontinuation reason is placed into DSCAT
        # and the site name leaks into DSTERM.
        if {"DSCAT", "DSTERM"}.issubset(frame.columns):
            dscat_raw = frame["DSCAT"].astype("string").fillna("").str.strip()
            dsterm_raw = frame["DSTERM"].astype("string").fillna("").str.strip()
            dsterm_upper = dsterm_raw.str.upper().str.strip()
            looks_like_site = dsterm_upper.str.contains(
                r"\bSITE\b", regex=True, na=False
            )

            ct_dscat = self._get_controlled_terminology(variable="DSCAT")
            if ct_dscat:
                dscat_norm = (
                    dscat_raw.apply(ct_dscat.normalize).astype("string").fillna("")
                )
                dscat_norm = dscat_norm.astype("string").str.upper().str.strip()
                is_valid = dscat_norm.isin(ct_dscat.submission_values)
                invalid = (dscat_raw != "") & ~is_valid
            else:
                # Fallback: treat free-text as invalid.
                invalid = dscat_raw != ""

            if bool(invalid.any()):
                move_reason = invalid & ((dsterm_raw == "") | looks_like_site)
                if bool(move_reason.any()):
                    frame.loc[move_reason, "DSTERM"] = dscat_raw.loc[move_reason]
                # Do not default DSCAT; clear invalid values.
                frame.loc[invalid, "DSCAT"] = ""

        # Ensure required DSDECOD is populated when possible.
        if {"DSDECOD", "DSTERM"}.issubset(frame.columns):
            dsdecod_raw = frame["DSDECOD"].astype("string").fillna("").str.strip()
            missing = dsdecod_raw == ""
            if bool(missing.any()):
                term_upper = (
                    frame["DSTERM"].astype("string").fillna("").str.upper().str.strip()
                )
                screen_failure = term_upper.str.contains(
                    r"SCREEN\s+FAILURE|FAILURE\s+TO\s+MEET",
                    regex=True,
                    na=False,
                )
                withdrawal_consent = term_upper.str.contains(
                    r"WITHDRAW.*CONSENT|WITHDRAWAL\s+OF\s+CONSENT",
                    regex=True,
                    na=False,
                )
                withdrawal_subject = term_upper.str.contains(
                    r"WITHDRAW.*SUBJECT|SUBJECT\s+WITHDRAW",
                    regex=True,
                    na=False,
                )
                lost_follow = term_upper.str.contains(
                    r"LOST\s+TO\s+FOLLOW",
                    regex=True,
                    na=False,
                )

                frame.loc[missing & screen_failure, "DSDECOD"] = "SCREEN FAILURE"
                frame.loc[missing & withdrawal_consent, "DSDECOD"] = (
                    "WITHDRAWAL OF CONSENT"
                )
                frame.loc[missing & withdrawal_subject, "DSDECOD"] = (
                    "WITHDRAWAL BY SUBJECT"
                )
                frame.loc[missing & lost_follow, "DSDECOD"] = "LOST TO FOLLOW-UP"

                # Do not default DSDECOD when it cannot be inferred.

        # If DSTERM is obviously a site label, prefer the coded disposition.
        if {"DSTERM", "DSDECOD"}.issubset(frame.columns):
            dsterm = frame["DSTERM"].astype("string").fillna("").str.strip()
            dsdecod = frame["DSDECOD"].astype("string").fillna("").str.strip()
            looks_like_site = dsterm.str.contains(r"\bSITE\b", regex=True, na=False)
            replace = looks_like_site & (dsdecod != "")
            if bool(replace.any()):
                frame.loc[replace, "DSTERM"] = dsdecod.loc[replace]

        # Do not default/guess EPOCH.

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
                # Generic non-completion signal: choose a valid CT umbrella term.
                "N": "SCREENING NOT COMPLETED",
                "NO": "SCREENING NOT COMPLETED",
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
