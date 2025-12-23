"""Domain processor for Laboratory (LB) domain."""

from typing import TYPE_CHECKING, override

if TYPE_CHECKING:
    import pandas as pd

from ....pandas_utils import ensure_numeric_series
from ..transformers.date import DateTransformer
from ..transformers.numeric import NumericTransformer
from .base import BaseDomainProcessor


class LBProcessor(BaseDomainProcessor):
    """Laboratory domain processor.

    Handles domain-specific processing for the LB domain.
    """

    @override
    def process(self, frame: pd.DataFrame) -> None:
        """Process LB domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        self._drop_placeholder_rows(frame)
        self._clean_unit_placeholders(frame)
        self._normalize_lbtestcd(frame)
        self._ensure_lbtest(frame)
        self._compute_study_days(frame)
        self._normalize_lbstresc(frame)
        self._sync_lbstresc(frame)
        self._sync_lbstresn(frame)
        self._normalize_lbclsig(frame)
        self._normalize_units(frame)
        self._clear_units_without_results(frame)
        NumericTransformer.assign_sequence(frame, "LBSEQ", "USUBJID")

    @staticmethod
    def _clean_unit_placeholders(frame: pd.DataFrame) -> None:
        for col in ("LBORRESU", "LBSTRESU"):
            if col in frame.columns:
                frame.loc[:, col] = (
                    frame[col]
                    .astype("string")
                    .replace({"<NA>": "", "nan": "", "None": ""})
                    .fillna("")
                    .str.strip()
                )

    def _normalize_lbtestcd(self, frame: pd.DataFrame) -> None:
        if "LBTESTCD" not in frame.columns:
            return
        testcd = frame["LBTESTCD"].astype("string").fillna("").str.upper().str.strip()
        ct_lbtestcd = self._get_controlled_terminology(variable="LBTESTCD")
        if ct_lbtestcd:
            canonical = testcd.apply(ct_lbtestcd.normalize)
            valid = canonical.isin(ct_lbtestcd.submission_values)
            testcd = canonical.where(valid, "")
        frame.loc[:, "LBTESTCD"] = testcd

    @staticmethod
    def _ensure_lbtest(frame: pd.DataFrame) -> None:
        if not {"LBTEST", "LBTESTCD"}.issubset(frame.columns):
            return
        lbtest = frame["LBTEST"].astype("string").fillna("").str.strip()
        testcd = frame["LBTESTCD"].astype("string").fillna("").str.strip()
        needs = (lbtest == "") & (testcd != "")
        if bool(needs.any()):
            frame.loc[needs, "LBTEST"] = testcd.loc[needs]

    def _compute_study_days(self, frame: pd.DataFrame) -> None:
        if "LBDTC" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "LBDTC",
                "LBDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
        if "LBENDTC" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "LBENDTC",
                "LBENDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )

    @staticmethod
    def _normalize_lbstresc(frame: pd.DataFrame) -> None:
        if "LBSTRESC" not in frame.columns:
            return
        stresc = frame["LBSTRESC"].astype("string").fillna("").str.strip()
        frame.loc[:, "LBSTRESC"] = stresc.replace(
            {"Positive": "POSITIVE", "Negative": "NEGATIVE"}
        )

    @staticmethod
    def _sync_lbstresc(frame: pd.DataFrame) -> None:
        if not {"LBORRES", "LBSTRESC"}.issubset(frame.columns):
            return
        orres = (
            frame["LBORRES"]
            .astype("string")
            .fillna("")
            .replace({"<NA>": "", "nan": "", "None": ""})
            .str.strip()
        )
        stresc = frame["LBSTRESC"].astype("string").fillna("").str.strip()
        needs = (stresc == "") & (orres != "")
        if bool(needs.any()):
            frame.loc[needs, "LBSTRESC"] = orres.loc[needs]

    @staticmethod
    def _sync_lbstresn(frame: pd.DataFrame) -> None:
        if "LBSTRESN" in frame.columns and "LBSTRESC" in frame.columns:
            numeric = ensure_numeric_series(frame["LBSTRESC"], frame.index).astype(
                "float64"
            )
            frame.loc[:, "LBSTRESN"] = numeric

    @staticmethod
    def _normalize_lbclsig(frame: pd.DataFrame) -> None:
        if "LBCLSIG" not in frame.columns:
            return
        yn_map = {
            "YES": "Y",
            "Y": "Y",
            "1": "Y",
            "TRUE": "Y",
            "NO": "N",
            "N": "N",
            "0": "N",
            "FALSE": "N",
            "CS": "Y",
            "NCS": "N",
            "": "",
            "nan": "",
        }
        frame["LBCLSIG"] = (
            frame["LBCLSIG"]
            .astype("string")
            .fillna("")
            .str.strip()
            .str.upper()
            .map(yn_map)
            .fillna("")
        )

    def _normalize_units(self, frame: pd.DataFrame) -> None:
        ct_lb_units = self._get_controlled_terminology(variable="LBORRESU")
        if not ct_lb_units:
            return
        for col in ("LBORRESU", "LBSTRESU"):
            if col in frame.columns:
                units = frame[col].astype("string").fillna("").str.strip()
                normalized = units.apply(ct_lb_units.normalize)
                normalized = normalized.where(
                    normalized.isin(ct_lb_units.submission_values), ""
                )
                frame[col] = normalized

    @staticmethod
    def _clear_units_without_results(frame: pd.DataFrame) -> None:
        if {"LBORRES", "LBORRESU"}.issubset(frame.columns):
            empty_orres = frame["LBORRES"].astype("string").fillna("").str.strip() == ""
            frame.loc[empty_orres, "LBORRESU"] = ""
        if {"LBSTRESC", "LBSTRESU"}.issubset(frame.columns):
            empty_stresc = (
                frame["LBSTRESC"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_stresc, "LBSTRESU"] = ""
