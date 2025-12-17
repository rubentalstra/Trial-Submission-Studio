"""Domain processor for Procedures (PR) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, NumericTransformer, TextTransformer
from ....pandas_utils import ensure_numeric_series


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
        frame.loc[:, "PRSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame.loc[:, "PRSEQ"] = NumericTransformer.force_numeric(frame["PRSEQ"])
        # Normalize visit info
        frame.loc[:, "VISITNUM"] = (frame.groupby("USUBJID").cumcount() + 1).astype(int)
        frame.loc[:, "VISIT"] = (
            frame["VISITNUM"].apply(lambda n: f"Visit {int(n)}").astype("string")
        )
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
        if "PRDUR" not in frame.columns:
            frame.loc[:, "PRDUR"] = "P1D"
        else:
            frame.loc[:, "PRDUR"] = TextTransformer.replace_unknown(
                frame["PRDUR"], "P1D"
            ).astype("string")
        if "PRRFTDTC" not in frame.columns:
            frame.loc[:, "PRRFTDTC"] = frame.get("RFSTDTC", "")
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
        frame.loc[:, "PRTPTREF"] = "VISIT"
        frame.loc[:, "PRTPT"] = frame.get("PRTPT", "VISIT")
        frame.loc[:, "PRTPTNUM"] = frame.get("PRTPTNUM", 1)
        frame.loc[:, "PRELTM"] = frame.get("PRELTM", "PT0H")
        # PRDECOD should use CT value. If we can't map confidently, leave it blank
        # rather than inventing a (potentially wrong) CT default.
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
            # Also treat USUBJID prefix (e.g., KIEM-01) as contamination.
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
            frame.loc[:, "PRDECOD"] = prdecod_upper
        ct_prdecod = self._get_controlled_terminology(variable="PRDECOD")
        if ct_prdecod:
            if "PRDECOD" not in frame.columns:
                frame.loc[:, "PRDECOD"] = ""
            else:
                decod = (
                    frame["PRDECOD"].astype("string").fillna("").str.strip().str.upper()
                )
                decod = decod.apply(ct_prdecod.normalize)
                decod = decod.where(decod.isin(ct_prdecod.submission_values), "")
                frame.loc[:, "PRDECOD"] = decod
        if "EPOCH" in frame.columns:
            frame.loc[:, "EPOCH"] = "TREATMENT"
        # Ensure timing reference fields are populated to satisfy SD1282
        timing_defaults = {
            "PRTPTREF": "VISIT",
            "PRTPT": "VISIT",
            "PRTPTNUM": 1,
            "PRELTM": "PT0H",
        }
        for col, default in timing_defaults.items():
            if col not in frame.columns:
                frame.loc[:, col] = default
            else:
                series = frame[col].astype("string").fillna("")
                if col == "PRTPTNUM":
                    numeric = ensure_numeric_series(series, frame.index).fillna(default)
                    frame.loc[:, col] = numeric.astype(int)
                else:
                    frame.loc[:, col] = series.replace("", default)
        # Ensure VISITNUM numeric
        if "VISITNUM" in frame.columns:
            frame.loc[:, "VISITNUM"] = (
                NumericTransformer.force_numeric(frame["VISITNUM"])
                .fillna(1)
                .astype(int)
            )
            frame.loc[:, "VISIT"] = (
                frame["VISITNUM"].apply(lambda n: f"Visit {int(n)}").astype("string")
            )
