"""Domain processor for Inclusion/Exclusion (IE) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, NumericTransformer


class IEProcessor(BaseDomainProcessor):
    """Inclusion/Exclusion domain processor.

    Handles domain-specific processing for the IE domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process IE domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Normalize common string columns when present.
        for col in (
            "IEORRES",
            "IESTRESC",
            "IETESTCD",
            "IETEST",
            "IECAT",
            "IESCAT",
            "EPOCH",
        ):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()

        # Normalize IEORRES to Y/N when possible; do not default missing.
        if "IEORRES" in frame.columns:
            yn_map = {
                "YES": "Y",
                "Y": "Y",
                "1": "Y",
                "TRUE": "Y",
                "NO": "N",
                "N": "N",
                "0": "N",
                "FALSE": "N",
                "": "",
            }
            raw = frame["IEORRES"].astype("string").fillna("").str.upper().str.strip()
            frame.loc[:, "IEORRES"] = raw.map(yn_map).fillna("")

        # IESTRESC should align with IEORRES when both are present.
        if {"IESTRESC", "IEORRES"}.issubset(frame.columns):
            stresc = frame["IESTRESC"].astype("string").fillna("").str.strip()
            orres = frame["IEORRES"].astype("string").fillna("").str.strip()
            needs = (stresc == "") & (orres != "")
            if bool(needs.any()):
                frame.loc[needs, "IESTRESC"] = orres.loc[needs]

        # Compute IEDY only when both IEDTC and IEDY are present.
        if "IEDTC" in frame.columns and "IEDY" in frame.columns:
            frame.loc[:, "IEDTC"] = (
                frame["IEDTC"].astype("string").fillna("").str.strip()
            )
            DateTransformer.compute_study_day(
                frame,
                "IEDTC",
                "IEDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
            frame.loc[:, "IEDY"] = NumericTransformer.force_numeric(frame["IEDY"])

        # Always regenerate IESEQ - source values may not be unique (SD0005)
        NumericTransformer.assign_sequence(frame, "IESEQ", "USUBJID")
