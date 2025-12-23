"""Domain processor for Procedures (PR) domain."""

from __future__ import annotations

import pandas as pd

from ....pandas_utils import ensure_numeric_series
from ..transformers import DateTransformer, NumericTransformer
from .base import BaseDomainProcessor


class PRProcessor(BaseDomainProcessor):
    """Procedures domain processor.

    Handles domain-specific processing for the PR domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process PR domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        self._drop_placeholder_rows(frame)

        # Always regenerate PRSEQ - source values may not be unique (SD0005)
        NumericTransformer.assign_sequence(frame, "PRSEQ", "USUBJID")

        # Do not synthesize visit variables; only normalize when present.
        for col in ("VISIT", "VISITNUM"):
            if col in frame.columns:
                frame.isetitem(
                    frame.columns.get_loc(col),
                    frame[col].astype("string").fillna("").str.strip(),
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
        if "PRDUR" in frame.columns:
            frame.isetitem(
                frame.columns.get_loc("PRDUR"),
                frame["PRDUR"].astype("string").fillna("").str.strip(),
            )

        if "PRRFTDTC" in frame.columns:
            frame.isetitem(
                frame.columns.get_loc("PRRFTDTC"),
                frame["PRRFTDTC"].astype("string").fillna("").str.strip(),
            )

        for col in ("PRTPTREF", "PRTPT", "PRTPTNUM", "PRELTM"):
            if col in frame.columns:
                frame.isetitem(
                    frame.columns.get_loc(col),
                    frame[col].astype("string").fillna("").str.strip(),
                )
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
            frame.isetitem(frame.columns.get_loc("PRDECOD"), prdecod_upper)
        ct_prdecod = self._get_controlled_terminology(variable="PRDECOD")
        if ct_prdecod:
            if "PRDECOD" not in frame.columns:
                frame.isetitem(frame.columns.get_loc("PRDECOD"), "")
            else:
                decod = (
                    frame["PRDECOD"].astype("string").fillna("").str.strip().str.upper()
                )
                decod = decod.apply(ct_prdecod.normalize)
                decod = decod.where(decod.isin(ct_prdecod.submission_values), "")
                frame.isetitem(frame.columns.get_loc("PRDECOD"), decod)
        if "EPOCH" in frame.columns:
            frame.isetitem(
                frame.columns.get_loc("EPOCH"),
                frame["EPOCH"].astype("string").fillna("").str.strip(),
            )
        # Ensure timing reference fields are populated to satisfy SD1282
        timing_defaults = {
            "PRTPTREF": "VISIT",
            "PRTPT": "VISIT",
            "PRTPTNUM": 1,
            "PRELTM": "PT0H",
        }
        for col, default in timing_defaults.items():
            if col not in frame.columns:
                frame.isetitem(frame.columns.get_loc(col), default)
            else:
                series = frame[col].astype("string").fillna("")
                if col == "PRTPTNUM":
                    numeric = ensure_numeric_series(series, frame.index).fillna(default)
                    frame.isetitem(frame.columns.get_loc(col), numeric.astype(int))
                else:
                    frame.isetitem(
                        frame.columns.get_loc(col), series.replace("", default)
                    )
        # Ensure VISITNUM numeric
        if "VISITNUM" in frame.columns:
            frame.isetitem(
                frame.columns.get_loc("VISITNUM"),
                NumericTransformer.force_numeric(frame["VISITNUM"])
                .fillna(1)
                .astype(int),
            )
            frame.isetitem(
                frame.columns.get_loc("VISIT"),
                frame["VISITNUM"].apply(lambda n: f"Visit {int(n)}").astype("string"),
            )
