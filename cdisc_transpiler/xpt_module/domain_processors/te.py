"""Domain processor for Trial Elements (TE) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor


class TEProcessor(BaseDomainProcessor):
    """Trial Elements domain processor.

    Handles domain-specific processing for the TE domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process TE domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Rebuild TE to align with SE/TA elements
        study_id = "STUDY"
        if len(frame) > 0 and "STUDYID" in frame.columns:
            study_id = frame["STUDYID"].iloc[0]
        elements = [
            {
                "ETCD": "SCRN",
                "ELEMENT": "SCREENING",
                "TESTRL": "START",
                "TEENRL": "END",
            },
            {
                "ETCD": "TRT",
                "ELEMENT": "TREATMENT",
                "TESTRL": "START",
                "TEENRL": "END",
            },
        ]
        te_df = pd.DataFrame(elements)
        te_df["STUDYID"] = study_id
        te_df["DOMAIN"] = "TE"
        frame.drop(frame.index, inplace=True)
        frame.drop(columns=list(frame.columns), inplace=True)
        for col in te_df.columns:
            frame[col] = te_df[col].values
