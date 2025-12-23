"""Domain processor for Trial Arms (TA) domain."""

from typing import override

import pandas as pd

from .base import BaseDomainProcessor


class TAProcessor(BaseDomainProcessor):
    """Trial Arms domain processor.

    Handles domain-specific processing for the TA domain.
    """

    @override
    def process(self, frame: pd.DataFrame) -> None:
        """Process TA domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        self._drop_placeholder_rows(frame)

        # TA has no USUBJID, so base placeholder-row logic may not remove
        # blank template rows. Drop rows where all key identifiers are empty.
        key_cols = [c for c in ("EPOCH", "ARMCD", "ARM", "ETCD") if c in frame.columns]
        if key_cols and len(frame) > 0:
            is_blank = pd.Series(True, index=frame.index)
            for col in key_cols:
                is_blank &= frame[col].astype("string").fillna("").str.strip().eq("")
            if bool(is_blank.any()):
                frame.drop(index=frame.index[is_blank].to_list(), inplace=True)
                frame.reset_index(drop=True, inplace=True)

        # Do not synthesize trial arms or default identifiers; only normalize existing rows.
        for col in ("EPOCH", "ARMCD", "ARM", "ETCD", "STUDYID", "DOMAIN"):
            if col in frame.columns:
                frame.loc[:, col] = frame[col].astype("string").fillna("").str.strip()

        if "TAETORD" in frame.columns:
            frame.loc[:, "TAETORD"] = pd.to_numeric(frame["TAETORD"], errors="coerce")

        # Keep a stable, predictable row order.
        if "TAETORD" in frame.columns:
            frame.sort_values(by=["TAETORD"], kind="stable", inplace=True)
            frame.reset_index(drop=True, inplace=True)
