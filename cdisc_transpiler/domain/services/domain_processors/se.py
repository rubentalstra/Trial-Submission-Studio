"""Domain processor for Subject Elements (SE) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, NumericTransformer
from ....constants import Defaults


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
        study_id = (getattr(self.config, "study_id", None) or "").strip()
        if not study_id and len(frame) > 0 and "STUDYID" in frame.columns:
            study_id = str(frame["STUDYID"].iloc[0]).strip()
        if not study_id:
            study_id = "STUDY"

        subjects_series = frame.get("USUBJID", pd.Series(dtype=str))
        subjects = (
            list(self.reference_starts.keys())
            if self.reference_starts
            else subjects_series.tolist()
        )
        subjects = [
            str(s).strip()
            for s in subjects
            if str(s).strip().upper() not in {"", "NAN", "<NA>", "NONE", "NULL"}
        ]
        if not subjects:
            subjects = [Defaults.SUBJECT_ID]
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
                    "SESTDTC": start or Defaults.DATE,
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
                    "SESTDTC": start or Defaults.DATE,
                    "SEENDTC": start or "2023-01-02",
                }
            )
        new_frame = pd.DataFrame(records)
        self._replace_frame_preserving_schema(frame, new_frame)
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
            if "SESTDY" not in frame.columns:
                frame.loc[:, "SESTDY"] = 1
            else:
                frame.loc[:, "SESTDY"] = (
                    pd.to_numeric(frame["SESTDY"], errors="coerce")
                    .fillna(1)
                    .astype(int)
                )

            frame.loc[:, "SEENDY"] = (
                pd.to_numeric(delta, errors="coerce").fillna(1).astype(int)
            )
