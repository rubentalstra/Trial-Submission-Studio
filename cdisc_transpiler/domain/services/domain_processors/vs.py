"""Domain processor for Vital Signs (VS) domain."""

from typing import override

import pandas as pd

from ....pandas_utils import ensure_numeric_series, ensure_series
from ..transformers.date import DateTransformer
from ..transformers.numeric import NumericTransformer
from .base import BaseDomainProcessor


class VSProcessor(BaseDomainProcessor):
    """Vital Signs domain processor.

    Handles domain-specific processing for the VS domain.
    """

    @override
    def process(self, frame: pd.DataFrame) -> None:
        """Process VS domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        self._drop_placeholder_rows(frame)
        self._compute_study_day(frame)
        self._sync_stresc(frame)
        self._sync_units(frame)
        self._clear_units_when_missing(frame)
        self._normalize_units_ct(frame)
        self._normalize_test_codes(frame)
        self._sync_numeric_results(frame)
        NumericTransformer.assign_sequence(frame, "VSSEQ", "USUBJID")
        self._flag_last_observation(frame)
        self._normalize_collection_time(frame)

    def _compute_study_day(self, frame: pd.DataFrame) -> None:
        if "VSDTC" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "VSDTC",
                "VSDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
        if "VSDY" in frame.columns:
            frame.loc[:, "VSDY"] = NumericTransformer.force_numeric(frame["VSDY"])

    @staticmethod
    def _string_series(frame: pd.DataFrame, column: str) -> pd.Series:
        series = ensure_series(frame[column]).astype("string")
        return ensure_series(series.fillna("")).astype("string").str.strip()

    def _sync_stresc(self, frame: pd.DataFrame) -> None:
        if not {"VSORRES", "VSSTRESC"}.issubset(frame.columns):
            return
        orres = self._string_series(frame, "VSORRES")
        stresc = self._string_series(frame, "VSSTRESC")
        needs = (stresc == "") & (orres != "")
        if bool(needs.any()):
            frame.loc[needs, "VSSTRESC"] = orres.loc[needs]

    def _sync_units(self, frame: pd.DataFrame) -> None:
        if not {"VSORRESU", "VSSTRESU"}.issubset(frame.columns):
            return
        oru = self._string_series(frame, "VSORRESU")
        stu = self._string_series(frame, "VSSTRESU")
        needs = (stu == "") & (oru != "")
        if bool(needs.any()):
            frame.loc[needs, "VSSTRESU"] = oru.loc[needs]

    def _clear_units_when_missing(self, frame: pd.DataFrame) -> None:
        if {"VSORRES", "VSORRESU"}.issubset(frame.columns):
            empty_orres = self._string_series(frame, "VSORRES") == ""
            frame.loc[empty_orres, "VSORRESU"] = ""
        if {"VSSTRESC", "VSSTRESU"}.issubset(frame.columns):
            empty_stresc = self._string_series(frame, "VSSTRESC") == ""
            frame.loc[empty_stresc, "VSSTRESU"] = ""

    def _normalize_units_ct(self, frame: pd.DataFrame) -> None:
        ct_units = self._get_controlled_terminology(variable="VSORRESU")
        if not ct_units:
            return
        for col in ("VSORRESU", "VSSTRESU"):
            if col in frame.columns:
                units = self._string_series(frame, col)
                normalized = units.apply(ct_units.normalize)
                normalized = normalized.where(
                    normalized.isin(ct_units.submission_values), ""
                )
                frame.loc[:, col] = normalized

    def _normalize_test_codes(self, frame: pd.DataFrame) -> None:
        ct_vstestcd = self._get_controlled_terminology(variable="VSTESTCD")
        if ct_vstestcd and "VSTESTCD" in frame.columns:
            raw = self._string_series(frame, "VSTESTCD")
            canonical = raw.apply(ct_vstestcd.normalize)
            valid = canonical.isin(ct_vstestcd.submission_values)
            frame.loc[:, "VSTESTCD"] = canonical.where(valid, "")

        ct_vstest = self._get_controlled_terminology(variable="VSTEST")
        if ct_vstest and "VSTEST" in frame.columns:
            raw = self._string_series(frame, "VSTEST")
            canonical = raw.apply(ct_vstest.normalize)
            valid = canonical.isin(ct_vstest.submission_values)
            frame.loc[:, "VSTEST"] = canonical.where(valid, "")

    @staticmethod
    def _sync_numeric_results(frame: pd.DataFrame) -> None:
        if {"VSORRES", "VSSTRESN"}.issubset(frame.columns):
            numeric = pd.to_numeric(frame["VSORRES"], errors="coerce")
            frame.loc[:, "VSSTRESN"] = ensure_numeric_series(
                numeric, frame.index
            ).astype("float64")

    def _flag_last_observation(self, frame: pd.DataFrame) -> None:
        if "VSLOBXFL" not in frame.columns or not {"USUBJID", "VSTESTCD"}.issubset(
            frame.columns
        ):
            return
        frame.loc[:, "VSLOBXFL"] = self._string_series(frame, "VSLOBXFL")
        group_cols = ["USUBJID", "VSTESTCD"]
        if "VSPOS" in frame.columns:
            group_cols.append("VSPOS")
        frame.loc[:, "VSLOBXFL"] = ""
        last_idx = frame.groupby(group_cols).tail(1).index
        frame.loc[last_idx, "VSLOBXFL"] = "Y"

    def _normalize_collection_time(self, frame: pd.DataFrame) -> None:
        if "VSELTM" not in frame.columns:
            return
        raw = self._string_series(frame, "VSELTM")
        valid = raw.str.match(r"^\d{2}:\d{2}(:\d{2})?$", na=False)
        frame.loc[:, "VSELTM"] = raw.where(valid, "")
