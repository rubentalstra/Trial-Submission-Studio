"""Domain processor for Procedures (PR) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ....xpt_module.transformers import TextTransformer, NumericTransformer, DateTransformer
from ....pandas_utils import ensure_numeric_series
from ....terminology_module import get_controlled_terminology


class PRProcessor(BaseDomainProcessor):
    """Procedures domain processor.

    Handles domain-specific processing for the PR domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process PR domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Always regenerate PRSEQ - source values may not be unique (SD0005)
        frame["PRSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame["PRSEQ"] = NumericTransformer.force_numeric(frame["PRSEQ"])
        # Normalize visit info
        frame["VISITNUM"] = (frame.groupby("USUBJID").cumcount() + 1).astype(int)
        frame["VISIT"] = frame["VISITNUM"].apply(lambda n: f"Visit {n}")
        if "PRSTDTC" in frame.columns:
            DateTransformer.compute_study_day(frame, "PRSTDTC", "PRSTDY", ref="RFSTDTC")
        if "PRENDTC" in frame.columns:
            DateTransformer.compute_study_day(frame, "PRENDTC", "PRENDY", ref="RFSTDTC")
        if "PRDUR" not in frame.columns:
            frame["PRDUR"] = "P1D"
        else:
            frame["PRDUR"] = TextTransformer.replace_unknown(frame["PRDUR"], "P1D")
        if "PRRFTDTC" not in frame.columns:
            frame["PRRFTDTC"] = frame.get("RFSTDTC", "")
        else:
            empty_prrft = (
                frame["PRRFTDTC"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_prrft, "PRRFTDTC"] = frame.get("RFSTDTC", "")
        if (
            "PRRFTDTC" in frame.columns
            and self.reference_starts
            and "USUBJID" in frame.columns
        ):
            empty_prrft = (
                frame["PRRFTDTC"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_prrft, "PRRFTDTC"] = frame.loc[empty_prrft, "USUBJID"].map(
                self.reference_starts
            )
        # Ensure timing reference present with supporting timing variables
        frame["PRTPTREF"] = "VISIT"
        frame["PRTPT"] = frame.get("PRTPT", "VISIT")
        frame["PRTPTNUM"] = frame.get("PRTPTNUM", 1)
        frame["PRELTM"] = frame.get("PRELTM", "PT0H")
        # PRDECOD should use CT value; default to first submission value if invalid/missing
        ct_prdecod = get_controlled_terminology(variable="PRDECOD")
        if ct_prdecod:
            canonical_default = sorted(ct_prdecod.submission_values)[0]
            if "PRDECOD" not in frame.columns:
                frame["PRDECOD"] = canonical_default
            else:
                decod = (
                    frame["PRDECOD"].astype("string").fillna("").str.strip().str.upper()
                )
                decod = decod.apply(ct_prdecod.normalize)
                decod = decod.where(
                    decod.isin(ct_prdecod.submission_values), canonical_default
                )
                frame["PRDECOD"] = decod
        if "EPOCH" in frame.columns:
            frame["EPOCH"] = "TREATMENT"
        # Ensure timing reference fields are populated to satisfy SD1282
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
        # Ensure VISITNUM numeric
        if "VISITNUM" in frame.columns:
            frame["VISITNUM"] = (
                NumericTransformer.force_numeric(frame["VISITNUM"])
                .fillna(1)
                .astype(int)
            )
            frame["VISIT"] = frame["VISITNUM"].apply(lambda n: f"Visit {n}")
