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

        # TA requires STUDYID/DOMAIN to be populated for XPT/Define-XML.
        study_id_default = getattr(self.config, "study_id", None) or "STUDY"
        if "DOMAIN" in frame.columns:
            frame.loc[:, "DOMAIN"] = (
                frame["DOMAIN"].astype("string").fillna("").replace("", "TA")
            )
        if "STUDYID" in frame.columns:
            current = frame["STUDYID"].astype("string").fillna("").str.strip()
            first_non_blank = next((v for v in current.tolist() if v), "")
            fill_value = first_non_blank or study_id_default
            frame.loc[current.eq(""), "STUDYID"] = fill_value

        # TA is required for many exports; ensure we always have at least the
        # minimal screening + treatment structure when the synthesized source is empty.
        if frame.empty:
            base: dict[str, str | int] = {col: "" for col in frame.columns}
            base.update(
                {
                    "STUDYID": study_id_default,
                    "DOMAIN": "TA",
                    "EPOCH": "TREATMENT",
                    "ETCD": "TRT",
                    # Use 1-based ordering to avoid downstream XPT readers
                    # misinterpreting numeric 0 as a tiny float.
                    "TAETORD": 2,
                    "ARMCD": "ARM1",
                    "ARM": "Treatment Arm",
                }
            )
            screening = base.copy()
            screening.update({"EPOCH": "SCREENING", "ETCD": "SCRN", "TAETORD": 1})

            cols = list(frame.columns)
            frame.loc[0, cols] = [base.get(c, "") for c in cols]
            frame.loc[1, cols] = [screening.get(c, "") for c in cols]
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

        # Keep a stable, predictable row order.
        if "TAETORD" in frame.columns:
            frame.sort_values(by=["TAETORD"], kind="stable", inplace=True)
            frame.reset_index(drop=True, inplace=True)
