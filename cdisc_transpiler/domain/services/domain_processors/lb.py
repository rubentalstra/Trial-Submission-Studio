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

        # Clean up literal string placeholders that commonly appear after mapping.
        for col in ("LBORRESU", "LBSTRESU"):
            if col in frame.columns:
                frame.loc[:, col] = (
                    frame[col]
                    .astype("string")
                    .replace({"<NA>": "", "nan": "", "None": ""})
                    .fillna("")
                    .str.strip()
                )

        # Normalize LBTESTCD using CT when available; do not drop rows or default values.
        if "LBTESTCD" in frame.columns:
            testcd = (
                frame["LBTESTCD"].astype("string").fillna("").str.upper().str.strip()
            )
            ct_lbtestcd = self._get_controlled_terminology(variable="LBTESTCD")
            if ct_lbtestcd:
                canonical = testcd.apply(ct_lbtestcd.normalize)
                valid = canonical.isin(ct_lbtestcd.submission_values)
                testcd = canonical.where(valid, "")
            frame.loc[:, "LBTESTCD"] = testcd

        # Keep LBTEST aligned with LBTESTCD when LBTEST exists and is blank.
        if {"LBTEST", "LBTESTCD"}.issubset(frame.columns):
            lbtest = frame["LBTEST"].astype("string").fillna("").str.strip()
            testcd = frame["LBTESTCD"].astype("string").fillna("").str.strip()
            needs = (lbtest == "") & (testcd != "")
            if bool(needs.any()):
                frame.loc[needs, "LBTEST"] = testcd.loc[needs]

        # Compute study days when dates are present.
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

        # Normalize qualitative results conservatively.
        if "LBSTRESC" in frame.columns:
            stresc = frame["LBSTRESC"].astype("string").fillna("").str.strip()
            frame.loc[:, "LBSTRESC"] = stresc.replace(
                {"Positive": "POSITIVE", "Negative": "NEGATIVE"}
            )

        # Ensure LBSTRESC mirrors LBORRES when both columns exist.
        if {"LBORRES", "LBSTRESC"}.issubset(frame.columns):
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

        # Keep LBSTRESN numeric when present.
        if "LBSTRESN" in frame.columns and "LBSTRESC" in frame.columns:
            numeric = ensure_numeric_series(frame["LBSTRESC"], frame.index).astype(
                "float64"
            )
            frame.loc[:, "LBSTRESN"] = numeric

        # Normalize LBCLSIG to Y/N when present.
        if "LBCLSIG" in frame.columns:
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

        # Controlled terminology normalization for units (blank invalid values; no defaults).
        ct_lb_units = self._get_controlled_terminology(variable="LBORRESU")
        if ct_lb_units:
            for col in ("LBORRESU", "LBSTRESU"):
                if col in frame.columns:
                    units = frame[col].astype("string").fillna("").str.strip()
                    normalized = units.apply(ct_lb_units.normalize)
                    normalized = normalized.where(
                        normalized.isin(ct_lb_units.submission_values), ""
                    )
                    frame[col] = normalized

        # Clear units when there is no corresponding result.
        if {"LBORRES", "LBORRESU"}.issubset(frame.columns):
            empty_orres = frame["LBORRES"].astype("string").fillna("").str.strip() == ""
            frame.loc[empty_orres, "LBORRESU"] = ""
        if {"LBSTRESC", "LBSTRESU"}.issubset(frame.columns):
            empty_stresc = (
                frame["LBSTRESC"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_stresc, "LBSTRESU"] = ""

        # Always regenerate LBSEQ - source values may not be unique (SD0005)
        NumericTransformer.assign_sequence(frame, "LBSEQ", "USUBJID")
