"""Domain processor for Trial Arms (TA) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor


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

        # TA is required for many exports; ensure we always have at least the
        # minimal screening + treatment structure when the synthesized source is empty.
        if frame.empty:
            base: dict[str, object] = {col: "" for col in frame.columns}
            base.update(
                {
                    "EPOCH": "TREATMENT",
                    "ETCD": "TRT",
                    # Use 1-based ordering to avoid downstream XPT readers
                    # misinterpreting numeric 0 as a tiny float.
                    "TAETORD": 2,
                    "ARMCD": "ARM1",
                    "ARM": "Treatment Arm",
                }
            )
            frame.loc[0] = base
            screening = base.copy()
            screening.update({"EPOCH": "SCREENING", "ETCD": "SCRN", "TAETORD": 1})
            frame.loc[1] = screening
            frame.reset_index(drop=True, inplace=True)

        # Ensure TA includes both SCREENING and TREATMENT epochs
        if len(frame) == 1:
            # If only one record, duplicate it for SCREENING epoch
            first_row = frame.iloc[0].to_dict()
            screening_row = first_row.copy()
            screening_row["EPOCH"] = "SCREENING"
            screening_row["ETCD"] = "SCRN"
            screening_row["TAETORD"] = 1
            frame.loc[len(frame)] = screening_row
        if "EPOCH" in frame.columns:
            frame.loc[:, "EPOCH"] = (
                frame["EPOCH"].astype("string").fillna("").replace("", "TREATMENT")
            )
        if "ARMCD" in frame.columns:
            frame.loc[:, "ARMCD"] = (
                frame["ARMCD"].astype("string").fillna("").replace("", "ARM1")
            )
        if "ARM" in frame.columns:
            frame.loc[:, "ARM"] = (
                frame["ARM"].astype("string").fillna("").replace("", "Treatment Arm")
            )
        if "ETCD" in frame.columns:
            frame.loc[frame["EPOCH"] == "TREATMENT", "ETCD"] = "TRT"
            frame.loc[frame["EPOCH"] == "SCREENING", "ETCD"] = "SCRN"

            missing_etcd = frame["ETCD"].astype("string").fillna("").str.strip() == ""
            if missing_etcd.any():
                frame.loc[missing_etcd & (frame["EPOCH"] == "TREATMENT"), "ETCD"] = (
                    "TRT"
                )
                frame.loc[missing_etcd & (frame["EPOCH"] == "SCREENING"), "ETCD"] = (
                    "SCRN"
                )

        # Ensure ordering is deterministic and avoids numeric zero.
        if "TAETORD" in frame.columns and "EPOCH" in frame.columns:
            frame.loc[frame["EPOCH"] == "TREATMENT", "TAETORD"] = 2
            frame.loc[frame["EPOCH"] == "SCREENING", "TAETORD"] = 1

        if "TAETORD" in frame.columns:
            frame.loc[:, "TAETORD"] = pd.to_numeric(
                frame["TAETORD"], errors="coerce"
            ).fillna(1)
