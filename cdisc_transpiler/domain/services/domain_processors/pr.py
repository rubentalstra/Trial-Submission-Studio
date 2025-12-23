"""Domain processor for Procedures (PR) domain."""

from typing import TYPE_CHECKING, override

if TYPE_CHECKING:
    import pandas as pd

from ....pandas_utils import ensure_numeric_series
from ..transformers.date import DateTransformer
from ..transformers.numeric import NumericTransformer
from .base import BaseDomainProcessor


class PRProcessor(BaseDomainProcessor):
    """Procedures domain processor.

    Handles domain-specific processing for the PR domain.
    """

    @override
    def process(self, frame: pd.DataFrame) -> None:
        """Process PR domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        self._drop_placeholder_rows(frame)
        self._assign_sequence(frame)
        self._normalize_visit_fields(frame)
        self._compute_study_days(frame)
        self._normalize_prdur(frame)
        self._normalize_reference_fields(frame)
        self._normalize_prdecod(frame)
        self._normalize_epoch(frame)
        self._apply_timing_defaults(frame)
        self._normalize_visitnum(frame)

    @staticmethod
    def _assign_sequence(frame: pd.DataFrame) -> None:
        NumericTransformer.assign_sequence(frame, "PRSEQ", "USUBJID")

    @staticmethod
    def _normalize_visit_fields(frame: pd.DataFrame) -> None:
        for col in ("VISIT", "VISITNUM"):
            if col in frame.columns:
                frame[col] = frame[col].astype("string").fillna("").str.strip()

    def _compute_study_days(self, frame: pd.DataFrame) -> None:
        if "PRSTDTC" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "PRSTDTC",
                "PRSTDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
        if "PRENDTC" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "PRENDTC",
                "PRENDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )

    @staticmethod
    def _normalize_prdur(frame: pd.DataFrame) -> None:
        if "PRDUR" in frame.columns:
            frame["PRDUR"] = frame["PRDUR"].astype("string").fillna("").str.strip()

    @staticmethod
    def _normalize_reference_fields(frame: pd.DataFrame) -> None:
        if "PRRFTDTC" in frame.columns:
            frame["PRRFTDTC"] = (
                frame["PRRFTDTC"].astype("string").fillna("").str.strip()
            )
        for col in ("PRTPTREF", "PRTPT", "PRTPTNUM", "PRELTM"):
            if col in frame.columns:
                frame[col] = frame[col].astype("string").fillna("").str.strip()

    def _normalize_prdecod(self, frame: pd.DataFrame) -> None:
        if "PRDECOD" in frame.columns:
            prdecod = frame["PRDECOD"].astype("string").fillna("").str.strip()
            prdecod_upper = prdecod.str.upper()
            for site_col in ("SiteCode", "Site code"):
                if site_col in frame.columns:
                    site = (
                        frame[site_col]
                        .astype("string")
                        .fillna("")
                        .str.strip()
                        .str.upper()
                    )
                    prdecod_upper = prdecod_upper.where(prdecod_upper != site, "")
            if "USUBJID" in frame.columns:
                prefix = (
                    frame["USUBJID"]
                    .astype("string")
                    .fillna("")
                    .str.split("-", n=1)
                    .str[0]
                    .str.upper()
                    .str.strip()
                )
                prdecod_upper = prdecod_upper.where(prdecod_upper != prefix, "")
            frame["PRDECOD"] = prdecod_upper

        ct_prdecod = self._get_controlled_terminology(variable="PRDECOD")
        if ct_prdecod:
            if "PRDECOD" not in frame.columns:
                frame["PRDECOD"] = ""
            else:
                decod = (
                    frame["PRDECOD"].astype("string").fillna("").str.strip().str.upper()
                )
                decod = decod.apply(ct_prdecod.normalize)
                decod = decod.where(decod.isin(ct_prdecod.submission_values), "")
                frame["PRDECOD"] = decod

    @staticmethod
    def _normalize_epoch(frame: pd.DataFrame) -> None:
        if "EPOCH" in frame.columns:
            frame["EPOCH"] = frame["EPOCH"].astype("string").fillna("").str.strip()

    @staticmethod
    def _apply_timing_defaults(frame: pd.DataFrame) -> None:
        timing_defaults = {
            "PRTPTREF": "VISIT",
            "PRTPT": "VISIT",
            "PRTPTNUM": 1,
            "PRELTM": "PT0H",
        }
        for col, default in timing_defaults.items():
            if col not in frame.columns:
                frame[col] = default
            else:
                series = frame[col].astype("string").fillna("")
                if col == "PRTPTNUM":
                    numeric = ensure_numeric_series(series, frame.index).fillna(default)
                    frame[col] = numeric.astype(int)
                else:
                    frame[col] = series.replace("", default)

    @staticmethod
    def _normalize_visitnum(frame: pd.DataFrame) -> None:
        if "VISITNUM" not in frame.columns:
            return
        frame["VISITNUM"] = (
            NumericTransformer.force_numeric(frame["VISITNUM"]).fillna(1).astype(int)
        )
        frame["VISIT"] = frame["VISITNUM"].map(_visit_label).astype("string")


def _visit_label(value: object) -> str:
    try:
        return f"Visit {int(float(str(value)))}"
    except (TypeError, ValueError):
        return "Visit 1"
