"""Domain processor for Trial Arms (TA) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import TextTransformer, NumericTransformer, DateTransformer


class TAProcessor(BaseDomainProcessor):
    """Trial Arms domain processor.
    
    Handles domain-specific processing for the TA domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process TA domain DataFrame.
        
        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)
        
        # Ensure TA includes both SCREENING and TREATMENT epochs
        if len(frame) == 1:
            # If only one record, duplicate it for SCREENING epoch
            first_row = frame.iloc[0].to_dict()
            screening_row = first_row.copy()
            screening_row["EPOCH"] = "SCREENING"
            screening_row["ETCD"] = "SCRN"
            screening_row["TAETORD"] = 0
            frame.loc[len(frame)] = screening_row

        if "TAETORD" in frame.columns:
            frame.loc[frame["EPOCH"] == "TREATMENT", "TAETORD"] = 1
            frame.loc[frame["EPOCH"] == "SCREENING", "TAETORD"] = 0
        if "EPOCH" in frame.columns:
            frame["EPOCH"] = frame["EPOCH"].replace("", "TREATMENT")
        if "ARMCD" in frame.columns:
            frame["ARMCD"] = frame["ARMCD"].replace("", "ARM1")
        if "ARM" in frame.columns:
            frame["ARM"] = frame["ARM"].replace("", "Treatment Arm")
        if "ETCD" in frame.columns:
            frame.loc[frame["EPOCH"] == "TREATMENT", "ETCD"] = "TRT"
            frame.loc[frame["EPOCH"] == "SCREENING", "ETCD"] = "SCRN"

