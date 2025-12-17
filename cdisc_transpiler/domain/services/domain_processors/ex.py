"""Domain processor for Exposure (EX) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, NumericTransformer


class EXProcessor(BaseDomainProcessor):
    """Exposure domain processor.

    Handles domain-specific processing for the EX domain.
    """

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

        if "EXTRT" in frame.columns:
            missing_topic = frame["EXTRT"].astype("string").fillna("").str.strip() == ""
            if bool(missing_topic.any()):
                frame.drop(index=frame.index[missing_topic], inplace=True)
                frame.reset_index(drop=True, inplace=True)

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
