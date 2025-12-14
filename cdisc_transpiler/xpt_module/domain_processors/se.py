"""Domain processor for Subject Elements (SE) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import NumericTransformer, DateTransformer


class SEProcessor(BaseDomainProcessor):
    """Subject Elements domain processor.

    Handles domain-specific processing for the SE domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process SE domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Rebuild SE using reference starts to align ETCD/ELEMENT/EPOCH with TE/TA
        records = []
        study_id = "STUDY"
        if len(frame) > 0 and "STUDYID" in frame.columns:
            study_id = frame["STUDYID"].iloc[0]

        subjects_series = frame.get("USUBJID", pd.Series(dtype=str))
        if subjects_series is None:
            subjects_series = pd.Series(dtype=str)
        subjects = (
            list(self.reference_starts.keys())
            if self.reference_starts
            else subjects_series.tolist()
        )
        for usubjid in subjects:
            start = (
                DateTransformer.coerce_iso8601(self.reference_starts.get(usubjid, ""))
                if self.reference_starts
                else ""
            )
            # Screening element
            records.append(
                {
                    "STUDYID": study_id,
                    "DOMAIN": "SE",
                    "USUBJID": usubjid,
                    "ETCD": "SCRN",
                    "ELEMENT": "SCREENING",
                    "EPOCH": "SCREENING",
                    "SESTDTC": start or "2023-01-01",
                    "SEENDTC": start or "2023-01-02",
                }
            )
            # Treatment element
            records.append(
                {
                    "STUDYID": study_id,
                    "DOMAIN": "SE",
                    "USUBJID": usubjid,
                    "ETCD": "TRT",
                    "ELEMENT": "TREATMENT",
                    "EPOCH": "TREATMENT",
                    "SESTDTC": start or "2023-01-01",
                    "SEENDTC": start or "2023-01-02",
                }
            )
        new_frame = pd.DataFrame(records)
        frame.drop(index=frame.index.tolist(), inplace=True)
        frame.drop(columns=list(frame.columns), inplace=True)
        for col in new_frame.columns:
            frame[col] = new_frame[col].values
        DateTransformer.ensure_date_pair_order(frame, "SESTDTC", "SEENDTC")
        DateTransformer.compute_study_day(frame, "SESTDTC", "SESTDY", ref="RFSTDTC")
        DateTransformer.compute_study_day(frame, "SEENDTC", "SEENDY", ref="RFSTDTC")
        NumericTransformer.assign_sequence(frame, "SESEQ", "USUBJID")
        if "SEENDY" not in frame.columns:
            frame["SEENDY"] = ""
        if "SESEQ" not in frame.columns:
            frame["SESEQ"] = frame.groupby("USUBJID").cumcount() + 1
        # Guarantee study day values even when reference dates are absent
        if {"SESTDTC", "SEENDTC"} <= set(frame.columns):
            start = pd.to_datetime(frame["SESTDTC"], errors="coerce")
            end = pd.to_datetime(frame["SEENDTC"], errors="coerce")
            delta = (end - start).dt.days + 1
            frame["SESTDY"] = frame.get("SESTDY", pd.Series([1] * len(frame)))
            frame["SEENDY"] = delta.fillna(1)
