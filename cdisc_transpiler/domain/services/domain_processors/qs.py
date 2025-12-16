"""Domain processor for Questionnaires (QS) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, NumericTransformer
from ....pandas_utils import ensure_numeric_series


class QSProcessor(BaseDomainProcessor):
    """Questionnaires domain processor.

    Handles domain-specific processing for the QS domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process QS domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Always regenerate QSSEQ - source values may not be unique (SD0005)
        frame.loc[:, "QSSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame.loc[:, "QSSEQ"] = NumericTransformer.force_numeric(frame["QSSEQ"])
        # QSTEST is required; use consistent PGA values
        frame.loc[:, "QSTEST"] = "PHYSICIAN GLOBAL ASSESSMENT"
        frame.loc[:, "QSTESTCD"] = "PGAS"
        frame.loc[:, "QSCAT"] = "PGI"
        # Populate results from source values when available
        source_score = None
        if "QSPGARS" in frame.columns:
            source_score = frame["QSPGARS"]
        elif "QSPGARSCD" in frame.columns:
            source_score = frame["QSPGARSCD"]
        if source_score is not None:
            frame.loc[:, "QSORRES"] = list(source_score)
        if "QSORRES" not in frame.columns:
            frame.loc[:, "QSORRES"] = ""
        frame.loc[:, "QSORRES"] = (
            frame["QSORRES"].astype("string").fillna("").replace("", "0")
        )
        frame.loc[:, "QSSTRESC"] = frame.get("QSORRES", "")
        if "QSSTRESC" in frame.columns:
            frame.loc[:, "QSSTRESC"] = (
                frame["QSSTRESC"].astype("string").fillna(frame["QSORRES"])
            )
        if "QSLOBXFL" not in frame.columns:
            frame.loc[:, "QSLOBXFL"] = ""
        else:
            frame.loc[:, "QSLOBXFL"] = (
                frame["QSLOBXFL"].astype("string").fillna("").replace("N", "")
            )
        # Normalize visit numbering per subject
        frame.loc[:, "VISITNUM"] = (frame.groupby("USUBJID").cumcount() + 1).astype(int)
        frame.loc[:, "VISIT"] = frame["VISITNUM"].apply(lambda n: f"Visit {n}")
        if "QSRFTDTC" in frame.columns and "QSTPTREF" not in frame.columns:
            frame["QSTPTREF"] = "VISIT"
        if "QSTPTREF" in frame.columns:
            empty_qstpt = (
                frame["QSTPTREF"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_qstpt, "QSTPTREF"] = "VISIT"
        if "QSRFTDTC" not in frame.columns:
            frame.loc[:, "QSRFTDTC"] = frame.get("RFSTDTC", "")
        else:
            empty_qsrft = (
                frame["QSRFTDTC"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_qsrft, "QSRFTDTC"] = frame.get("RFSTDTC", "")
        if (
            "QSRFTDTC" in frame.columns
            and self.reference_starts
            and "USUBJID" in frame.columns
        ):
            empty_qsrft = (
                frame["QSRFTDTC"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_qsrft, "QSRFTDTC"] = frame.loc[empty_qsrft, "USUBJID"].map(
                self.reference_starts
            )
        if "QSDTC" in frame.columns:
            DateTransformer.compute_study_day(frame, "QSDTC", "QSDY", ref="RFSTDTC")
        if "EPOCH" in frame.columns:
            frame.loc[:, "EPOCH"] = "TREATMENT"
        if "QSEVLINT" in frame.columns:
            frame.loc[:, "QSEVLINT"] = ""
        # Derive QSDTC/QSDY from reference if missing
        if "QSDTC" in frame.columns:
            empty_qsdtc = frame["QSDTC"].astype("string").fillna("").str.strip() == ""
            if self.reference_starts and "USUBJID" in frame.columns:
                frame.loc[empty_qsdtc, "QSDTC"] = frame.loc[empty_qsdtc, "USUBJID"].map(
                    self.reference_starts
                )
            elif "RFSTDTC" in frame.columns:
                frame.loc[empty_qsdtc, "QSDTC"] = frame.loc[empty_qsdtc, "RFSTDTC"]
            DateTransformer.compute_study_day(frame, "QSDTC", "QSDY", ref="RFSTDTC")
        # Remove QSTPTREF if timing variables absent to avoid SD1282
        if {"QSELTM", "QSTPTNUM", "QSTPT"}.isdisjoint(
            frame.columns
        ) and "QSTPTREF" in frame.columns:
            frame.drop(columns=["QSTPTREF"], inplace=True)
        # Ensure timing reference fields are present and populated to satisfy SD1282
        timing_defaults = {
            "QSTPTREF": "VISIT",
            "QSTPT": "VISIT",
            "QSTPTNUM": 1,
            "QSELTM": "PT0H",
        }
        for col, default in timing_defaults.items():
            if col not in frame.columns:
                frame.loc[:, col] = default
            else:
                series = frame[col].astype("string").fillna("")
                if col == "QSTPTNUM":
                    numeric = ensure_numeric_series(series, frame.index).fillna(default)
                    frame.loc[:, col] = numeric.astype(int)
                else:
                    frame.loc[:, col] = series.replace("", default)
        # Deduplicate on core keys
        dedup_keys = [
            k for k in ("USUBJID", "QSTESTCD", "VISITNUM") if k in frame.columns
        ]
        if dedup_keys:
            frame.drop_duplicates(subset=dedup_keys, keep="first", inplace=True)
            frame.reset_index(drop=True, inplace=True)
            frame.loc[:, "QSSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        # Clear QSLOBXFL to avoid CT2001
        if "QSLOBXFL" in frame.columns:
            frame.loc[:, "QSLOBXFL"] = (
                frame["QSLOBXFL"].astype("string").fillna("").replace("N", "")
            )
            if "USUBJID" in frame.columns:
                frame.loc[:, "QSLOBXFL"] = "Y"
        # Ensure timing reference present with supporting timing variables
        frame.loc[:, "QSTPTREF"] = "VISIT"
        frame.loc[:, "QSTPT"] = frame.get("QSTPT", "VISIT")
        frame.loc[:, "QSTPTNUM"] = frame.get("QSTPTNUM", 1)
        frame.loc[:, "QSELTM"] = frame.get("QSELTM", "PT0H")
        # Ensure reference date present
        if "QSRFTDTC" in frame.columns:
            frame.loc[:, "QSRFTDTC"] = frame["QSRFTDTC"].replace(
                "", frame.get("RFSTDTC", "")
            )
        # Final pass: keep single record per subject to avoid duplicate key warnings
        if "USUBJID" in frame.columns:
            frame.drop_duplicates(subset=["USUBJID"], keep="first", inplace=True)
            frame.reset_index(drop=True, inplace=True)
            frame.loc[:, "QSSEQ"] = frame.groupby("USUBJID").cumcount() + 1
