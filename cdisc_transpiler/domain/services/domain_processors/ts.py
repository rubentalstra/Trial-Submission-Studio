"""Domain processor for Trial Summary (TS) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor


class TSProcessor(BaseDomainProcessor):
    """Trial Summary domain processor.

    Handles domain-specific processing for the TS domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process TS domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        self._drop_placeholder_rows(frame)

        if frame.empty:
            return

        # Conservative cleanup only: strip strings and blank invalid CT values.
        for col in (
            "STUDYID",
            "DOMAIN",
            "TSPARMCD",
            "TSPARM",
            "TSVAL",
            "TSVALCD",
            "TSVCDREF",
            "TSVCDVER",
            "TSGRPID",
            "TSVALNF",
        ):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()

        ct_parmcd = self._get_controlled_terminology(variable="TSPARMCD")
        if ct_parmcd and "TSPARMCD" in frame.columns:
            raw = frame["TSPARMCD"].astype("string").fillna("").str.strip()
            canonical = raw.apply(ct_parmcd.normalize)
            valid = canonical.isin(ct_parmcd.submission_values)
            frame.loc[:, "TSPARMCD"] = canonical.where(valid, "")

        ct_parm = self._get_controlled_terminology(variable="TSPARM")
        if ct_parm and "TSPARM" in frame.columns:
            raw = frame["TSPARM"].astype("string").fillna("").str.strip()
            canonical = raw.apply(ct_parm.normalize)
            valid = canonical.isin(ct_parm.submission_values)
            frame.loc[:, "TSPARM"] = canonical.where(valid, "")

        ct_dict = self._get_controlled_terminology(variable="TSVCDREF")
        if ct_dict and "TSVCDREF" in frame.columns:
            raw = frame["TSVCDREF"].astype("string").fillna("").str.strip()
            canonical = raw.apply(ct_dict.normalize)
            valid = canonical.isin(ct_dict.submission_values)
            frame.loc[:, "TSVCDREF"] = canonical.where(valid, "")
