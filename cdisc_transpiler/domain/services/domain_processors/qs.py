"""Domain processor for Questionnaires (QS) domain."""

from typing import override

import pandas as pd

from ....pandas_utils import ensure_series
from ..transformers.date import DateTransformer
from ..transformers.numeric import NumericTransformer
from .base import BaseDomainProcessor

USUBJID_SITE_PARTS_MIN = 2


class QSProcessor(BaseDomainProcessor):
    """Questionnaires domain processor.

    Handles domain-specific processing for the QS domain.
    """

    @override
    def process(self, frame: pd.DataFrame) -> None:
        """Process QS domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        self._drop_placeholder_rows(frame)
        self._assign_sequence(frame)
        self._normalize_string_columns(frame)
        source_score, score_from_qsgrpid = self._extract_pga_score(frame)
        if source_score is not None:
            self._apply_pga_defaults(
                frame,
                source_score=source_score,
                score_from_qsgrpid=score_from_qsgrpid,
            )
        self._sync_qsstresc(frame)
        self._normalize_qslobxfl(frame)
        self._normalize_dates(frame)
        self._clear_qstptref_without_timing(frame)

    @staticmethod
    def _assign_sequence(frame: pd.DataFrame) -> None:
        frame.loc[:, "QSSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame.loc[:, "QSSEQ"] = NumericTransformer.force_numeric(frame["QSSEQ"])

    @staticmethod
    def _normalize_string_columns(frame: pd.DataFrame) -> None:
        for col in (
            "QSTESTCD",
            "QSTEST",
            "QSCAT",
            "QSSCAT",
            "QSORRES",
            "QSSTRESC",
            "QSLOBXFL",
            "VISIT",
            "EPOCH",
        ):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()

    @staticmethod
    def _extract_pga_score(
        frame: pd.DataFrame,
    ) -> tuple[pd.Series | None, bool]:
        if "QSPGARS" in frame.columns:
            return ensure_series(frame["QSPGARS"], index=frame.index), False
        if "QSPGARSCD" in frame.columns:
            return ensure_series(frame["QSPGARSCD"], index=frame.index), False
        if "QSGRPID" in frame.columns and "QSORRES" in frame.columns:
            qsorres = frame["QSORRES"].astype("string").fillna("").str.strip()
            qsgrpid = frame["QSGRPID"].astype("string").fillna("").str.strip()
            if bool((qsorres == "").all()) and bool((qsgrpid != "").any()):
                return ensure_series(frame["QSGRPID"], index=frame.index), True
        return None, False

    def _apply_pga_defaults(
        self,
        frame: pd.DataFrame,
        *,
        source_score: pd.Series,
        score_from_qsgrpid: bool,
    ) -> None:
        if "QSORRES" in frame.columns:
            empty_orres = frame["QSORRES"].astype("string").fillna("").str.strip() == ""
            frame.loc[empty_orres, "QSORRES"] = source_score.astype("string").fillna("")
        else:
            frame.loc[:, "QSORRES"] = source_score.astype("string").fillna("")

        if score_from_qsgrpid and "QSGRPID" in frame.columns:
            frame.loc[:, "QSGRPID"] = ""

        self._ensure_pga_testcd(frame)
        self._ensure_pga_test(frame)
        self._ensure_pga_cat(frame)

    def _ensure_pga_testcd(self, frame: pd.DataFrame) -> None:
        if "QSTESTCD" not in frame.columns:
            return
        empty_testcd = frame["QSTESTCD"].astype("string").fillna("").str.strip() == ""
        mis_mapped = pd.Series([False] * len(frame), index=frame.index)
        if "USUBJID" in frame.columns:
            usubjid = frame["USUBJID"].astype("string").fillna("").str.strip()
            parts = usubjid.str.split("-", n=2, expand=True)
            if parts.shape[1] >= USUBJID_SITE_PARTS_MIN:
                site_part = parts[1].astype("string").fillna("").str.strip()
                testcd = frame["QSTESTCD"].astype("string").fillna("").str.strip()
                mis_mapped = (testcd != "") & (site_part != "") & (testcd == site_part)
        frame.loc[empty_testcd | mis_mapped, "QSTESTCD"] = "PGAS"

    @staticmethod
    def _ensure_pga_test(frame: pd.DataFrame) -> None:
        if "QSTEST" in frame.columns:
            empty_test = frame["QSTEST"].astype("string").fillna("").str.strip() == ""
            frame.loc[empty_test, "QSTEST"] = "PHYSICIAN GLOBAL ASSESSMENT"

    @staticmethod
    def _ensure_pga_cat(frame: pd.DataFrame) -> None:
        if "QSCAT" in frame.columns:
            empty_cat = frame["QSCAT"].astype("string").fillna("").str.strip() == ""
            frame.loc[empty_cat, "QSCAT"] = "PGI"

    @staticmethod
    def _sync_qsstresc(frame: pd.DataFrame) -> None:
        if "QSSTRESC" in frame.columns and "QSORRES" in frame.columns:
            empty_stresc = (
                frame["QSSTRESC"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_stresc, "QSSTRESC"] = (
                frame.loc[empty_stresc, "QSORRES"].astype("string").fillna("")
            )

    @staticmethod
    def _normalize_qslobxfl(frame: pd.DataFrame) -> None:
        if "QSLOBXFL" in frame.columns:
            frame.loc[:, "QSLOBXFL"] = (
                frame["QSLOBXFL"].astype("string").fillna("").replace("N", "")
            )

    def _normalize_dates(self, frame: pd.DataFrame) -> None:
        if "QSDTC" in frame.columns:
            frame.loc[:, "QSDTC"] = frame["QSDTC"].apply(DateTransformer.coerce_iso8601)
            if "QSDY" in frame.columns:
                DateTransformer.compute_study_day(
                    frame,
                    "QSDTC",
                    "QSDY",
                    reference_starts=self.reference_starts,
                    ref="RFSTDTC",
                )

    @staticmethod
    def _clear_qstptref_without_timing(frame: pd.DataFrame) -> None:
        if "QSTPTREF" in frame.columns and {"QSELTM", "QSTPTNUM", "QSTPT"}.isdisjoint(
            frame.columns
        ):
            frame.loc[:, "QSTPTREF"] = ""
