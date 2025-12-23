"""Domain processor for Disposition (DS) domain."""

from typing import override

import pandas as pd

from ..transformers.date import DateTransformer
from ..transformers.numeric import NumericTransformer
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
        self._drop_placeholder_rows(frame)
        self._normalize_core_fields(frame)
        self._cleanup_site_payload(frame)
        self._fix_mismapped_dscat(frame)
        self._ensure_dsdecod(frame)
        self._prefer_dsdecod_for_site_dsterm(frame)
        self._normalize_dsdecod_ct(frame)
        self._normalize_date_fields(frame)
        self._assign_sequence(frame)
        self._deduplicate(frame)

    @staticmethod
    def _normalize_core_fields(frame: pd.DataFrame) -> None:
        for col in ("DSDECOD", "DSTERM", "DSCAT", "EPOCH"):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()

    @staticmethod
    def _cleanup_site_payload(frame: pd.DataFrame) -> None:
        if not {"USUBJID", "DSDECOD", "DSTERM"}.issubset(frame.columns):
            return

        usubjid = frame["USUBJID"].astype("string").fillna("").str.strip()
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
            frame.loc[screen_failure, "DSDECOD"] = "SCREEN FAILURE"
            frame.loc[screen_failure, "DSTERM"] = "SCREEN FAILURE"

        junk_site_rows = is_site_payload & ~screen_failure
        if bool(junk_site_rows.any()):
            frame.loc[junk_site_rows, "DSDECOD"] = ""
            frame.loc[junk_site_rows & (dsterm_upper == site_upper), "DSTERM"] = ""

    def _fix_mismapped_dscat(self, frame: pd.DataFrame) -> None:
        if not {"DSCAT", "DSTERM"}.issubset(frame.columns):
            return

        dscat_raw = frame["DSCAT"].astype("string").fillna("").str.strip()
        dsterm_raw = frame["DSTERM"].astype("string").fillna("").str.strip()
        dsterm_upper = dsterm_raw.str.upper().str.strip()
        looks_like_site = dsterm_upper.str.contains(r"\bSITE\b", regex=True, na=False)

        ct_dscat = self._get_controlled_terminology(variable="DSCAT")
        if ct_dscat:
            dscat_norm = dscat_raw.apply(ct_dscat.normalize).astype("string").fillna("")
            dscat_norm = dscat_norm.astype("string").str.upper().str.strip()
            is_valid = dscat_norm.isin(ct_dscat.submission_values)
            invalid = (dscat_raw != "") & ~is_valid
        else:
            invalid = dscat_raw != ""

        if not bool(invalid.any()):
            return

        move_reason = invalid & ((dsterm_raw == "") | looks_like_site)
        if bool(move_reason.any()):
            frame.loc[move_reason, "DSTERM"] = dscat_raw.loc[move_reason]
        frame.loc[invalid, "DSCAT"] = ""

    @staticmethod
    def _ensure_dsdecod(frame: pd.DataFrame) -> None:
        if not {"DSDECOD", "DSTERM"}.issubset(frame.columns):
            return
        dsdecod_raw = frame["DSDECOD"].astype("string").fillna("").str.strip()
        missing = dsdecod_raw == ""
        if not bool(missing.any()):
            return

        term_upper = frame["DSTERM"].astype("string").fillna("").str.upper().str.strip()
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
        frame.loc[missing & withdrawal_consent, "DSDECOD"] = "WITHDRAWAL OF CONSENT"
        frame.loc[missing & withdrawal_subject, "DSDECOD"] = "WITHDRAWAL BY SUBJECT"
        frame.loc[missing & lost_follow, "DSDECOD"] = "LOST TO FOLLOW-UP"

    @staticmethod
    def _prefer_dsdecod_for_site_dsterm(frame: pd.DataFrame) -> None:
        if not {"DSTERM", "DSDECOD"}.issubset(frame.columns):
            return
        dsterm = frame["DSTERM"].astype("string").fillna("").str.strip()
        dsdecod = frame["DSDECOD"].astype("string").fillna("").str.strip()
        looks_like_site = dsterm.str.contains(r"\bSITE\b", regex=True, na=False)
        replace = looks_like_site & (dsdecod != "")
        if bool(replace.any()):
            frame.loc[replace, "DSTERM"] = dsdecod.loc[replace]

    def _normalize_dsdecod_ct(self, frame: pd.DataFrame) -> None:
        ct_dsdecod = self._get_controlled_terminology(variable="DSDECOD")
        if not ct_dsdecod or "DSDECOD" not in frame.columns:
            return

        raw = frame["DSDECOD"].astype("string").fillna("").str.strip()
        canonical = raw.apply(ct_dsdecod.normalize).astype("string").fillna("")
        yn_map = {
            "Y": "COMPLETED",
            "YES": "COMPLETED",
            "N": "SCREENING NOT COMPLETED",
            "NO": "SCREENING NOT COMPLETED",
        }
        canonical_upper = canonical.str.upper().str.strip()
        canonical = canonical.where(
            ~canonical_upper.isin(yn_map), canonical_upper.map(yn_map)
        )
        canonical = canonical.astype("string").fillna("").str.upper().str.strip()
        frame.loc[:, "DSDECOD"] = canonical

        if "DSTERM" not in frame.columns:
            return

        dsterm_raw = frame["DSTERM"].astype("string").fillna("").str.strip()
        dsterm_code = dsterm_raw.apply(ct_dsdecod.normalize).astype("string").fillna("")
        dsterm_code = dsterm_code.astype("string").fillna("").str.upper().str.strip()
        valid_from_term = dsterm_code.isin(ct_dsdecod.submission_values)
        invalid_from_decod = ~canonical.isin(ct_dsdecod.submission_values)
        use_term_code = valid_from_term & invalid_from_decod
        if bool(use_term_code.any()):
            frame.loc[use_term_code, "DSDECOD"] = dsterm_code.loc[use_term_code]

    def _normalize_date_fields(self, frame: pd.DataFrame) -> None:
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

    @staticmethod
    def _assign_sequence(frame: pd.DataFrame) -> None:
        NumericTransformer.assign_sequence(frame, "DSSEQ", "USUBJID")
        frame.loc[:, "DSSEQ"] = pd.to_numeric(
            NumericTransformer.force_numeric(frame["DSSEQ"]), errors="coerce"
        )

    @staticmethod
    def _deduplicate(frame: pd.DataFrame) -> None:
        dedup_keys = ["USUBJID", "DSDECOD", "DSTERM", "DSCAT", "DSSTDTC"]
        existing_cols = [c for c in dedup_keys if c in frame.columns]
        if existing_cols:
            frame.drop_duplicates(subset=existing_cols, keep="first", inplace=True)
