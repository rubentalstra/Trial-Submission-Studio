"""Domain processor for Subject Elements (SE) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, NumericTransformer


class SEProcessor(BaseDomainProcessor):
    """Subject Elements domain processor.

    Handles domain-specific processing for the SE domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process SE domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        self._drop_placeholder_rows(frame)

        # Do not synthesize SE records. Only normalize and derive deterministic values.
        for col in (
            "STUDYID",
            "DOMAIN",
            "USUBJID",
            "ETCD",
            "ELEMENT",
            "EPOCH",
            "SESTDTC",
            "SEENDTC",
        ):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()

        DateTransformer.ensure_date_pair_order(frame, "SESTDTC", "SEENDTC")
        if "SESTDTC" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "SESTDTC",
                "SESTDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
        if "SEENDTC" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "SEENDTC",
                "SEENDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )

        NumericTransformer.assign_sequence(frame, "SESEQ", "USUBJID")
