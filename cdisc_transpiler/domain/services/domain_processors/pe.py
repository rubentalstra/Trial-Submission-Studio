from typing import TYPE_CHECKING, override

if TYPE_CHECKING:
    import pandas as pd
from ..transformers.date import DateTransformer
from ..transformers.numeric import NumericTransformer
from .base import BaseDomainProcessor


class PEProcessor(BaseDomainProcessor):
    pass

    @override
    def process(self, frame: pd.DataFrame) -> None:
        self._drop_placeholder_rows(frame)
        NumericTransformer.assign_sequence(frame, "PESEQ", "USUBJID")
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
