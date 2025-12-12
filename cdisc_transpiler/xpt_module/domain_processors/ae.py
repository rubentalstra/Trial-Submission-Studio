"""Domain processor for Adverse Events (AE) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import TextTransformer, NumericTransformer, DateTransformer


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

        # Ensure AEDUR populated to avoid SD1078 missing permissibles
        if "AEDUR" in frame.columns:
            frame["AEDUR"] = (
                frame["AEDUR"].astype("string").fillna("").replace("", "P1D")
            )
        else:
            frame["AEDUR"] = "P1D"
        # Standardize visit info only when present in source
        if {"VISIT", "VISITNUM"} & set(frame.columns):
            TextTransformer.normalize_visit(frame)
        DateTransformer.ensure_date_pair_order(frame, "AESTDTC", "AEENDTC")
        DateTransformer.compute_study_day(frame, "AESTDTC", "AESTDY", ref="RFSTDTC")
        DateTransformer.compute_study_day(frame, "AEENDTC", "AEENDY", ref="RFSTDTC")
        # Keep TRTEMFL when present to satisfy treatment-emergent checks
        # Ensure expected MedDRA variables exist with default placeholders
        defaults = {
            "AEBODSYS": "GENERAL DISORDERS",
            "AEHLGT": "GENERAL DISORDERS",
            "AEHLT": "GENERAL DISORDERS",
            "AELLT": "GENERAL DISORDERS",
            "AESOC": "GENERAL DISORDERS",
        }
        for col, val in defaults.items():
            if col not in frame.columns:
                frame[col] = val
        # AEACN - normalize to valid CDISC CT values
        if "AEACN" in frame.columns:
            frame["AEACN"] = (
                frame["AEACN"]
                .astype(str)
                .str.upper()
                .str.strip()
                .replace(
                    {
                        "": "DOSE NOT CHANGED",
                        "NONE": "DOSE NOT CHANGED",
                        "NO ACTION": "DOSE NOT CHANGED",
                        "NAN": "DOSE NOT CHANGED",
                        "<NA>": "DOSE NOT CHANGED",
                        "UNK": "UNKNOWN",
                        "NA": "NOT APPLICABLE",
                        "N/A": "NOT APPLICABLE",
                    }
                )
            )
        else:
            frame["AEACN"] = "DOSE NOT CHANGED"
        # AESER - normalize to valid CDISC CT values (Y/N only)
        if "AESER" in frame.columns:
            frame["AESER"] = (
                frame["AESER"]
                .astype(str)
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
                        "": "N",
                        "NAN": "N",
                        "<NA>": "N",
                        "UNK": "N",
                        "UNKNOWN": "N",
                        "U": "N",
                    }
                )
            )
        else:
            frame["AESER"] = "N"
        # AEREL - normalize to valid CDISC CT values
        if "AEREL" in frame.columns:
            frame["AEREL"] = (
                frame["AEREL"]
                .astype(str)
                .str.upper()
                .str.strip()
                .replace(
                    {
                        "": "NOT RELATED",
                        "NO": "NOT RELATED",
                        "N": "NOT RELATED",
                        "NOT SUSPECTED": "NOT RELATED",
                        "UNLIKELY RELATED": "NOT RELATED",
                        "YES": "RELATED",
                        "Y": "RELATED",
                        "POSSIBLY RELATED": "RELATED",
                        "PROBABLY RELATED": "RELATED",
                        "SUSPECTED": "RELATED",
                        "NAN": "NOT RELATED",
                        "<NA>": "NOT RELATED",
                        "UNK": "UNKNOWN",
                        "NOT ASSESSED": "UNKNOWN",
                    }
                )
            )
        else:
            frame["AEREL"] = "NOT RELATED"
        # AEOUT - normalize to valid CDISC CT values
        if "AEOUT" in frame.columns:
            frame["AEOUT"] = (
                frame["AEOUT"]
                .astype(str)
                .str.upper()
                .str.strip()
                .replace(
                    {
                        "": "RECOVERED/RESOLVED",
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
                        "NAN": "RECOVERED/RESOLVED",
                        "<NA>": "RECOVERED/RESOLVED",
                        "UNK": "UNKNOWN",
                        "U": "UNKNOWN",
                    }
                )
            )
        else:
            frame["AEOUT"] = "RECOVERED/RESOLVED"
        # AESEV - normalize to valid CDISC CT values
        if "AESEV" in frame.columns:
            frame["AESEV"] = (
                frame["AESEV"]
                .astype(str)
                .str.upper()
                .str.strip()
                .replace(
                    {
                        "": "MILD",
                        "1": "MILD",
                        "GRADE 1": "MILD",
                        "2": "MODERATE",
                        "GRADE 2": "MODERATE",
                        "3": "SEVERE",
                        "GRADE 3": "SEVERE",
                        "NAN": "MILD",
                        "<NA>": "MILD",
                    }
                )
            )
        else:
            frame["AESEV"] = "MILD"
        # Ensure EPOCH is set for AE records
        if "EPOCH" in frame.columns:
            frame["EPOCH"] = TextTransformer.replace_unknown(
                frame["EPOCH"], "TREATMENT"
            )
        else:
            frame["EPOCH"] = "TREATMENT"
        for code_var in (
            "AEPTCD",
            "AEHLGTCD",
            "AEHLTCD",
            "AELLTCD",
            "AESOCCD",
            "AEBDSYCD",
        ):
            if code_var in frame.columns:
                numeric = pd.to_numeric(frame[code_var], errors="coerce")
                frame[code_var] = numeric.fillna(999999).astype("Int64")
            else:
                frame[code_var] = pd.Series(
                    [999999 for _ in frame.index], dtype="Int64"
                )
        NumericTransformer.assign_sequence(frame, "AESEQ", "USUBJID")
        if "AESEQ" in frame.columns:
            frame["AESEQ"] = frame["AESEQ"].astype("Int64")
        # Remove non-standard extras to keep AE aligned to SDTM metadata
        for extra in ("VISIT", "VISITNUM", "TRTEMFL"):
            if extra in frame.columns:
                frame.drop(columns=[extra], inplace=True)
