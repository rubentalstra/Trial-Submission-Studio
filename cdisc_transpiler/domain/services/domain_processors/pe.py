"""Domain processor for Physical Examination (PE) domain."""

from __future__ import annotations

import pandas as pd

from ..transformers import DateTransformer, NumericTransformer
from .base import BaseDomainProcessor


class PEProcessor(BaseDomainProcessor):
    """Physical Examination domain processor.

    Handles domain-specific processing for the PE domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process PE domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        self._drop_placeholder_rows(frame)

        # Always regenerate PESEQ - source values may not be unique (SD0005)
        NumericTransformer.assign_sequence(frame, "PESEQ", "USUBJID")

        # Normalize PESTAT to CDISC CT 'Not Done'
        if "PESTAT" in frame.columns:
            stat_map = {
                "NOT DONE": "NOT DONE",
                "ND": "NOT DONE",
                "DONE": "",
                "COMPLETED": "",
                "": "",
                "nan": "",
            }
            frame.loc[:, "PESTAT"] = (
                frame["PESTAT"]
                .astype("string")
                .fillna("")
                .str.strip()
                .str.upper()
                .map(stat_map)
                .fillna("")
            )

        # PESTRESC should be derived from PEORRES when both are present.
        if {"PEORRES", "PESTRESC"}.issubset(frame.columns):
            orres = frame["PEORRES"].astype("string").fillna("").str.strip()
            stresc = frame["PESTRESC"].astype("string").fillna("").str.strip()
            needs = (stresc == "") & (orres != "")
            if bool(needs.any()):
                frame.loc[needs, "PESTRESC"] = orres.loc[needs]

        if "PEDTC" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "PEDTC",
                "PEDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )

        if "EPOCH" in frame.columns:
            frame.loc[:, "EPOCH"] = (
                frame["EPOCH"].astype("string").fillna("").str.strip()
            )
