"""Domain processor for Questionnaires (QS) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, NumericTransformer
from ....pandas_utils import ensure_series


class QSProcessor(BaseDomainProcessor):
    """Questionnaires domain processor.

    Handles domain-specific processing for the QS domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process QS domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Always regenerate QSSEQ - source values may not be unique (SD0005)
        frame.loc[:, "QSSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame.loc[:, "QSSEQ"] = NumericTransformer.force_numeric(frame["QSSEQ"])

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

        # If the source provides a PGA score field, populate results and (only if
        # missing) minimal identifying metadata for that instrument.
        source_score = None
        if "QSPGARS" in frame.columns:
            source_score = ensure_series(frame["QSPGARS"], index=frame.index)
        elif "QSPGARSCD" in frame.columns:
            source_score = ensure_series(frame["QSPGARSCD"], index=frame.index)

        if source_score is not None:
            if "QSORRES" in frame.columns:
                empty_orres = (
                    frame["QSORRES"].astype("string").fillna("").str.strip() == ""
                )
                frame.loc[empty_orres, "QSORRES"] = source_score.astype(
                    "string"
                ).fillna("")
            else:
                frame.loc[:, "QSORRES"] = source_score.astype("string").fillna("")

            if "QSTESTCD" in frame.columns:
                empty_testcd = (
                    frame["QSTESTCD"].astype("string").fillna("").str.strip() == ""
                )
                frame.loc[empty_testcd, "QSTESTCD"] = "PGAS"
            if "QSTEST" in frame.columns:
                empty_test = (
                    frame["QSTEST"].astype("string").fillna("").str.strip() == ""
                )
                frame.loc[empty_test, "QSTEST"] = "PHYSICIAN GLOBAL ASSESSMENT"
            if "QSCAT" in frame.columns:
                empty_cat = frame["QSCAT"].astype("string").fillna("").str.strip() == ""
                frame.loc[empty_cat, "QSCAT"] = "PGI"

        if "QSSTRESC" in frame.columns and "QSORRES" in frame.columns:
            empty_stresc = (
                frame["QSSTRESC"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_stresc, "QSSTRESC"] = (
                frame.loc[empty_stresc, "QSORRES"].astype("string").fillna("")
            )

        if "QSLOBXFL" in frame.columns:
            frame.loc[:, "QSLOBXFL"] = (
                frame["QSLOBXFL"].astype("string").fillna("").replace("N", "")
            )

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

        # If timing support variables are absent, clear QSTPTREF to avoid
        # inconsistent partial timing specification.
        if "QSTPTREF" in frame.columns and {"QSELTM", "QSTPTNUM", "QSTPT"}.isdisjoint(
            frame.columns
        ):
            frame.loc[:, "QSTPTREF"] = ""
