"""Domain processor for Adverse Events (AE) domain."""

from __future__ import annotations

import pandas as pd

from ....pandas_utils import ensure_numeric_series
from ..transformers import DateTransformer, NumericTransformer
from .base import BaseDomainProcessor


class AEProcessor(BaseDomainProcessor):
    """Adverse Events domain processor.

    Handles domain-specific processing for the AE domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process AE domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Do not default/guess durations.
        if "AEDUR" in frame.columns:
            frame.loc[:, "AEDUR"] = (
                frame["AEDUR"].astype("string").fillna("").str.strip()
            )
        # Do not synthesize visit numbering; only normalize whitespace.
        for col in ("VISIT", "VISITNUM"):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()
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
        # Avoid emitting TEAE (non-model) in AE.
        if "TEAE" in frame.columns:
            frame.drop(columns=["TEAE"], inplace=True)

        # Treatment-emergent info (e.g., TRTEMFL) is represented via SUPPAE
        # supplemental qualifiers, not as a non-model AE variable.
        # Do not add placeholder MedDRA hierarchy values.
        # AEACN - normalize known synonyms; do not default missing values.
        if "AEACN" in frame.columns:
            frame.loc[:, "AEACN"] = (
                frame["AEACN"]
                .astype("string")
                .fillna("")
                .str.upper()
                .str.strip()
                .replace(
                    {
                        "NONE": "DOSE NOT CHANGED",
                        "NO ACTION": "DOSE NOT CHANGED",
                        "UNK": "UNKNOWN",
                        "NA": "NOT APPLICABLE",
                        "N/A": "NOT APPLICABLE",
                    }
                )
            )
        # AESER - normalize to Y/N when possible; otherwise blank.
        if "AESER" in frame.columns:
            frame.loc[:, "AESER"] = (
                frame["AESER"]
                .astype("string")
                .fillna("")
                .str.upper()
                .str.strip()
                .replace(
                    {
                        "YES": "Y",
                        "NO": "N",
                        "1": "Y",
                        "0": "N",
                        "TRUE": "Y",
                        "FALSE": "N",
                    }
                )
            )
        # AEREL - normalize known synonyms; do not default missing values.
        if "AEREL" in frame.columns:
            frame.loc[:, "AEREL"] = (
                frame["AEREL"]
                .astype("string")
                .fillna("")
                .str.upper()
                .str.strip()
                .replace(
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
                    }
                )
            )
        # AEOUT - normalize known synonyms; do not default missing values.
        if "AEOUT" in frame.columns:
            frame.loc[:, "AEOUT"] = (
                frame["AEOUT"]
                .astype("string")
                .fillna("")
                .str.upper()
                .str.strip()
                .replace(
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
                    }
                )
            )
        # AESEV - normalize known severity encodings; do not default missing values.
        if "AESEV" in frame.columns:
            frame.loc[:, "AESEV"] = (
                frame["AESEV"]
                .astype("string")
                .fillna("")
                .str.upper()
                .str.strip()
                .replace(
                    {
                        "1": "MILD",
                        "GRADE 1": "MILD",
                        "2": "MODERATE",
                        "GRADE 2": "MODERATE",
                        "3": "SEVERE",
                        "GRADE 3": "SEVERE",
                    }
                )
            )

        # Do not default/guess EPOCH.
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
                frame.isetitem(frame.columns.get_loc(code_var), numeric.astype("Int64"))
            else:
                frame[code_var] = pd.Series([pd.NA] * len(frame), dtype="Int64")
        NumericTransformer.assign_sequence(frame, "AESEQ", "USUBJID")
        if "AESEQ" in frame.columns:
            frame.loc[:, "AESEQ"] = frame["AESEQ"].astype("Int64")

        # AESINTV is a Yes/No Response field (C66742). When mapped from non-YN
        # source columns (e.g., design version numbers), blank it rather than
        # emitting CT-invalid values.
        if "AESINTV" in frame.columns:
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
        # Remove non-standard extras to keep AE aligned to SDTM metadata.
        # Treatment-emergent information is provided via SUPPAE (e.g., TRTEMFL)
        # rather than adding non-model variables to AE.
        for extra in ("VISIT", "VISITNUM"):
            if extra in frame.columns:
                frame.drop(columns=[extra], inplace=True)
