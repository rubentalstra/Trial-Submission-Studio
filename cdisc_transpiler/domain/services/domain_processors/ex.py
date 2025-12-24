import re
from typing import override

import pandas as pd

from ..transformers.date import DateTransformer
from ..transformers.numeric import NumericTransformer
from .base import BaseDomainProcessor


class EXProcessor(BaseDomainProcessor):
    pass

    @override
    def process(self, frame: pd.DataFrame) -> None:
        self._drop_placeholder_rows(frame)
        self._normalize_extrt(frame)
        self._reassign_exeltm_as_extrt(frame)
        self._normalize_dates(frame)
        self._normalize_dose(frame)
        self._derive_exdosu(frame)
        self._normalize_string_columns(frame)
        self._assign_sequence(frame)
        self._normalize_day_columns(frame)

    @staticmethod
    def _normalize_extrt(frame: pd.DataFrame) -> None:
        if "EXTRT" in frame.columns:
            frame.loc[:, "EXTRT"] = (
                frame["EXTRT"].astype("string").fillna("").str.strip()
            )

    @staticmethod
    def _reassign_exeltm_as_extrt(frame: pd.DataFrame) -> None:
        if not {"EXTRT", "EXELTM"}.issubset(frame.columns):
            return
        extrt = frame["EXTRT"].astype("string").fillna("").str.strip()
        exeltm = frame["EXELTM"].astype("string").fillna("").str.strip()
        has_letters = exeltm.str.contains("[A-Za-z]", regex=True, na=False)
        fill_mask = (extrt == "") & (exeltm != "") & has_letters
        if bool(fill_mask.any()):
            frame.loc[fill_mask, "EXTRT"] = exeltm.loc[fill_mask]
            frame.loc[fill_mask, "EXELTM"] = ""

    def _normalize_dates(self, frame: pd.DataFrame) -> None:
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

    @staticmethod
    def _normalize_dose(frame: pd.DataFrame) -> None:
        if "EXDOSE" in frame.columns:
            frame.loc[:, "EXDOSE"] = pd.to_numeric(frame["EXDOSE"], errors="coerce")

    def _derive_exdosu(self, frame: pd.DataFrame) -> None:
        if self.metadata is None or not {"EXDOSE", "EXDOSU"}.issubset(frame.columns):
            return
        exdose = frame["EXDOSE"]
        exdosu = frame["EXDOSU"].astype("string").fillna("").str.strip()
        needs_u = exdose.notna() & (exdosu == "")
        if not bool(needs_u.any()):
            return
        col = self.metadata.get_column("EXDOSE")
        label = col.label or "" if col else ""
        match = re.search("\\(([^)]+)\\)", str(label))
        if not match:
            return
        unit = match.group(1).strip()
        if unit:
            frame.loc[needs_u, "EXDOSU"] = unit

    @staticmethod
    def _normalize_string_columns(frame: pd.DataFrame) -> None:
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

    @staticmethod
    def _assign_sequence(frame: pd.DataFrame) -> None:
        NumericTransformer.assign_sequence(frame, "EXSEQ", "USUBJID")

    @staticmethod
    def _normalize_day_columns(frame: pd.DataFrame) -> None:
        for dy in ("EXSTDY", "EXENDY"):
            if dy in frame.columns:
                frame.loc[:, dy] = NumericTransformer.force_numeric(frame[dy])
