"""Domain processor for Exposure (EX) domain."""

import re
from typing import override

import pandas as pd

from ..transformers import DateTransformer, NumericTransformer
from .base import BaseDomainProcessor


class EXProcessor(BaseDomainProcessor):
    """Exposure domain processor.

    Handles domain-specific processing for the EX domain.
    """

    @override
    def process(self, frame: pd.DataFrame) -> None:
        """Process EX domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        if "EXTRT" in frame.columns:
            frame.loc[:, "EXTRT"] = (
                frame["EXTRT"].astype("string").fillna("").str.strip()
            )

        # Do not default/guess EPOCH.

        # Some sources/mappings mistakenly place a treatment label into EXELTM
        # (elapsed time). If EXTRT is missing and EXELTM contains text, treat it
        # as the topic and clear EXELTM.
        if {"EXTRT", "EXELTM"}.issubset(frame.columns):
            extrt = frame["EXTRT"].astype("string").fillna("").str.strip()
            exeltm = frame["EXELTM"].astype("string").fillna("").str.strip()
            has_letters = exeltm.str.contains(r"[A-Za-z]", regex=True, na=False)
            fill_mask = (extrt == "") & (exeltm != "") & has_letters
            if bool(fill_mask.any()):
                frame.loc[fill_mask, "EXTRT"] = exeltm.loc[fill_mask]
                frame.loc[fill_mask, "EXELTM"] = ""

        # Do not drop records solely because EXTRT is missing. Missing topic values
        # should be reported via conformance checks, but the underlying exposure
        # record should be preserved.

        # Do not default/guess EXCAT.

        for date_col in ("EXSTDTC", "EXENDTC"):
            if date_col in frame.columns:
                frame.loc[:, date_col] = frame[date_col].apply(
                    DateTransformer.coerce_iso8601
                )

        if "EXSTDTC" in frame.columns and "EXSTDY" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "EXSTDTC",
                "EXSTDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
        if "EXENDTC" in frame.columns and "EXENDY" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "EXENDTC",
                "EXENDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )

        # EXDOSE is numeric in SDTM. Coerce to numeric to avoid type mismatches.
        if "EXDOSE" in frame.columns:
            frame.loc[:, "EXDOSE"] = pd.to_numeric(frame["EXDOSE"], errors="coerce")

        # Derive EXDOSU from metadata when dose is collected with an explicit unit
        # in the source label (e.g., "Dose administered (mg)").
        if {
            "EXDOSE",
            "EXDOSU",
        }.issubset(frame.columns) and self.metadata is not None:
            exdose = frame["EXDOSE"]
            exdosu = frame["EXDOSU"].astype("string").fillna("").str.strip()
            needs_u = exdose.notna() & (exdosu == "")
            if bool(needs_u.any()):
                col = self.metadata.get_column("EXDOSE")
                label = (col.label or "") if col else ""
                # Extract unit within parentheses.
                match = re.search(r"\(([^)]+)\)", str(label))
                if match:
                    unit = match.group(1).strip()
                    if unit:
                        frame.loc[needs_u, "EXDOSU"] = unit
        for col in (
            "EXDOSFRM",
            "EXDOSU",
            "EXDOSFRQ",
            "EXDUR",
            "EXSCAT",
            "EXCAT",
            "EPOCH",
            "EXELTM",
            "EXTPTREF",
            "EXRFTDTC",
        ):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()

        NumericTransformer.assign_sequence(frame, "EXSEQ", "USUBJID")
        for dy in ("EXSTDY", "EXENDY"):
            if dy in frame.columns:
                frame.loc[:, dy] = NumericTransformer.force_numeric(frame[dy])
