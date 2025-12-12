"""Domain processor for Medical History (MH) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import TextTransformer, NumericTransformer, DateTransformer


class MHProcessor(BaseDomainProcessor):
    """Medical History domain processor.

    Handles domain-specific processing for the MH domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process MH domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Ensure USUBJID is populated - derive from source if present
        if "USUBJID" in frame.columns:
            usub = frame["USUBJID"].astype("string").str.strip()
            missing_usubjid = usub.str.lower().isin({"", "nan", "<na>", "none", "null"})
            if missing_usubjid.any():
                frame = frame.loc[~missing_usubjid].copy()
        if "MHSEQ" not in frame.columns:
            frame["MHSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        else:
            # Always regenerate MHSEQ - source values may not be unique (SD0005)
            frame["MHSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame["MHSEQ"] = NumericTransformer.force_numeric(frame["MHSEQ"])
        # MHTERM is required - derive from MHDECOD or source data if available
        if (
            "MHTERM" not in frame.columns
            or frame["MHTERM"].astype("string").fillna("").str.strip().eq("").all()
        ):
            if (
                "MHDECOD" in frame.columns
                and not frame["MHDECOD"].astype(str).str.strip().eq("").all()
            ):
                frame["MHTERM"] = frame["MHDECOD"]
            else:
                frame["MHTERM"] = "MEDICAL HISTORY"
        else:
            # Fill empty MHTERM values with MHDECOD or default
            empty_mhterm = frame["MHTERM"].astype("string").fillna("").str.strip() == ""
            if empty_mhterm.any():
                if "MHDECOD" in frame.columns:
                    frame.loc[empty_mhterm, "MHTERM"] = frame.loc[
                        empty_mhterm, "MHDECOD"
                    ]
                else:
                    frame.loc[empty_mhterm, "MHTERM"] = "MEDICAL HISTORY"
        # Set EPOCH for screening
        if "EPOCH" in frame.columns:
            frame["EPOCH"] = "SCREENING"
        else:
            frame["EPOCH"] = "SCREENING"

        # Remove problematic relation-to-reference variables when not populated correctly
        for col in ("MHENRF",):
            if col in frame.columns:
                frame.drop(columns=[col], inplace=True)

        # SD0021/SD0022 - Set default time-point values if missing
        # Only fill values for columns that exist in the domain
        if "MHSTTPT" in frame.columns:
            empty_sttpt = frame["MHSTTPT"].astype(str).str.strip() == ""
            frame.loc[empty_sttpt, "MHSTTPT"] = "BEFORE"
        if "MHSTRTPT" in frame.columns:
            empty_strtpt = frame["MHSTRTPT"].astype(str).str.strip() == ""
            frame.loc[empty_strtpt, "MHSTRTPT"] = "SCREENING"
        if "MHENTPT" in frame.columns:
            empty_entpt = frame["MHENTPT"].astype(str).str.strip() == ""
            frame.loc[empty_entpt, "MHENTPT"] = "ONGOING"
        if "MHENRTPT" in frame.columns:
            empty_enrtpt = frame["MHENRTPT"].astype(str).str.strip() == ""
            frame.loc[empty_enrtpt, "MHENRTPT"] = "SCREENING"
        # Ensure MHDTC exists, using MHSTDTC when available
        if "MHDTC" not in frame.columns:
            frame["MHDTC"] = frame.get("MHSTDTC", "")
        else:
            empty_mhdtc = frame["MHDTC"].astype("string").fillna("").str.strip() == ""
            frame.loc[empty_mhdtc, "MHDTC"] = frame.get("MHSTDTC", "")
        for col in ("MHSTDTC", "MHENDTC", "MHDTC"):
            if col in frame.columns:
                frame[col] = frame[col].apply(self._coerce_iso8601)
        # Fill missing end dates from reference end if available
        if "MHENDTC" in frame.columns:
            empty_end = frame["MHENDTC"].astype("string").fillna("").str.strip() == ""
            if "RFENDTC" in frame.columns:
                frame.loc[empty_end, "MHENDTC"] = frame.loc[empty_end, "RFENDTC"]
            elif self.reference_starts and "USUBJID" in frame.columns:
                frame.loc[empty_end, "MHENDTC"] = frame.loc[empty_end, "USUBJID"].map(
                    self.reference_starts
                )
        else:
            frame["MHENDTC"] = frame.get("MHSTDTC", "")
        # Compute study day for MHSTDTC into MHDY to keep numeric type
        if {"MHSTDTC", "MHDY"} <= set(frame.columns):
            DateTransformer.compute_study_day(frame, "MHSTDTC", "MHDY", "RFSTDTC")
        elif "MHSTDTC" in frame.columns:
            frame["MHDY"] = pd.NA
            DateTransformer.compute_study_day(frame, "MHSTDTC", "MHDY", "RFSTDTC")
        if "MHDY" in frame.columns:
            frame["MHDY"] = pd.to_numeric(frame["MHDY"], errors="coerce").astype(
                "Int64"
            )
        dedup_keys = [k for k in ("USUBJID", "MHTERM") if k in frame.columns]
        if dedup_keys:
            frame.drop_duplicates(subset=dedup_keys, keep="first", inplace=True)
            frame.reset_index(drop=True, inplace=True)
            frame["MHSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        if "USUBJID" in frame.columns:
            frame.drop_duplicates(subset=["USUBJID"], keep="first", inplace=True)
            frame.reset_index(drop=True, inplace=True)
            frame["MHSEQ"] = frame.groupby("USUBJID").cumcount() + 1
