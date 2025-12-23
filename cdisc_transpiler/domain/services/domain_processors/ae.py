"""Domain processor for Adverse Events (AE) domain."""

from typing import override

import pandas as pd

from ....pandas_utils import ensure_numeric_series
from ..transformers.date import DateTransformer
from ..transformers.numeric import NumericTransformer
from .base import BaseDomainProcessor


class AEProcessor(BaseDomainProcessor):
    """Adverse Events domain processor.

    Handles domain-specific processing for the AE domain.
    """

    @override
    def process(self, frame: pd.DataFrame) -> None:
        """Process AE domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        self._drop_placeholder_rows(frame)
        self._normalize_duration(frame)
        self._normalize_visits(frame)
        self._apply_date_fields(frame)
        self._drop_nonmodel_columns(frame)
        self._normalize_core_terms(frame)
        self._normalize_code_columns(frame)
        self._assign_sequence(frame)
        self._normalize_yes_no(frame)
        self._drop_visit_extras(frame)

    @staticmethod
    def _normalize_duration(frame: pd.DataFrame) -> None:
        if "AEDUR" not in frame.columns:
            return
        frame.loc[:, "AEDUR"] = frame["AEDUR"].astype("string").fillna("").str.strip()

    @staticmethod
    def _normalize_visits(frame: pd.DataFrame) -> None:
        for col in ("VISIT", "VISITNUM"):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()

    def _apply_date_fields(self, frame: pd.DataFrame) -> None:
        DateTransformer.ensure_date_pair_order(frame, "AESTDTC", "AEENDTC")
        DateTransformer.compute_study_day(
            frame,
            "AESTDTC",
            "AESTDY",
            reference_starts=self.reference_starts,
            ref="RFSTDTC",
        )
        DateTransformer.compute_study_day(
            frame,
            "AEENDTC",
            "AEENDY",
            reference_starts=self.reference_starts,
            ref="RFSTDTC",
        )

    @staticmethod
    def _drop_nonmodel_columns(frame: pd.DataFrame) -> None:
        if "TEAE" in frame.columns:
            frame.drop(columns=["TEAE"], inplace=True)

    def _normalize_core_terms(self, frame: pd.DataFrame) -> None:
        self._normalize_with_map(
            frame,
            "AEACN",
            {
                "NONE": "DOSE NOT CHANGED",
                "NO ACTION": "DOSE NOT CHANGED",
                "UNK": "UNKNOWN",
                "NA": "NOT APPLICABLE",
                "N/A": "NOT APPLICABLE",
            },
        )
        self._normalize_with_map(
            frame,
            "AESER",
            {
                "YES": "Y",
                "NO": "N",
                "1": "Y",
                "0": "N",
                "TRUE": "Y",
                "FALSE": "N",
            },
        )
        self._normalize_with_map(
            frame,
            "AEREL",
            {
                "NO": "NOT RELATED",
                "N": "NOT RELATED",
                "NOT SUSPECTED": "NOT RELATED",
                "UNLIKELY RELATED": "NOT RELATED",
                "YES": "RELATED",
                "Y": "RELATED",
                "POSSIBLY RELATED": "RELATED",
                "PROBABLY RELATED": "RELATED",
                "SUSPECTED": "RELATED",
                "UNK": "UNKNOWN",
                "NOT ASSESSED": "UNKNOWN",
            },
        )
        self._normalize_with_map(
            frame,
            "AEOUT",
            {
                "RECOVERED": "RECOVERED/RESOLVED",
                "RESOLVED": "RECOVERED/RESOLVED",
                "RECOVERED OR RESOLVED": "RECOVERED/RESOLVED",
                "RECOVERING": "RECOVERING/RESOLVING",
                "RESOLVING": "RECOVERING/RESOLVING",
                "NOT RECOVERED": "NOT RECOVERED/NOT RESOLVED",
                "NOT RESOLVED": "NOT RECOVERED/NOT RESOLVED",
                "UNRESOLVED": "NOT RECOVERED/NOT RESOLVED",
                "RECOVERED WITH SEQUELAE": "RECOVERED/RESOLVED WITH SEQUELAE",
                "RESOLVED WITH SEQUELAE": "RECOVERED/RESOLVED WITH SEQUELAE",
                "DEATH": "FATAL",
                "5": "FATAL",
                "GRADE 5": "FATAL",
                "UNK": "UNKNOWN",
                "U": "UNKNOWN",
            },
        )
        self._normalize_with_map(
            frame,
            "AESEV",
            {
                "1": "MILD",
                "GRADE 1": "MILD",
                "2": "MODERATE",
                "GRADE 2": "MODERATE",
                "3": "SEVERE",
                "GRADE 3": "SEVERE",
            },
        )

    @staticmethod
    def _normalize_with_map(
        frame: pd.DataFrame,
        column: str,
        mapping: dict[str, str],
    ) -> None:
        if column not in frame.columns:
            return
        frame.loc[:, column] = (
            frame[column]
            .astype("string")
            .fillna("")
            .str.upper()
            .str.strip()
            .replace(mapping)
        )

    @staticmethod
    def _normalize_code_columns(frame: pd.DataFrame) -> None:
        for code_var in (
            "AEPTCD",
            "AEHLGTCD",
            "AEHLTCD",
            "AELLTCD",
            "AESOCCD",
            "AEBDSYCD",
        ):
            if code_var in frame.columns:
                numeric = ensure_numeric_series(frame[code_var], frame.index)
                frame[code_var] = numeric.astype("Int64")
            else:
                frame[code_var] = pd.Series([pd.NA] * len(frame), dtype="Int64")

    @staticmethod
    def _assign_sequence(frame: pd.DataFrame) -> None:
        NumericTransformer.assign_sequence(frame, "AESEQ", "USUBJID")
        if "AESEQ" in frame.columns:
            frame.loc[:, "AESEQ"] = frame["AESEQ"].astype("Int64")

    @staticmethod
    def _normalize_yes_no(frame: pd.DataFrame) -> None:
        if "AESINTV" not in frame.columns:
            return
        yn_map = {
            "Y": "Y",
            "YES": "Y",
            "1": "Y",
            "TRUE": "Y",
            "N": "N",
            "NO": "N",
            "0": "N",
            "FALSE": "N",
            "": "",
            "NAN": "",
            "<NA>": "",
        }
        raw = frame["AESINTV"].astype("string").fillna("").str.strip().str.upper()
        frame.loc[:, "AESINTV"] = raw.map(yn_map).fillna("")

    @staticmethod
    def _drop_visit_extras(frame: pd.DataFrame) -> None:
        for extra in ("VISIT", "VISITNUM"):
            if extra in frame.columns:
                frame.drop(columns=[extra], inplace=True)
