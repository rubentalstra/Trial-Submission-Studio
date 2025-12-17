"""Domain processor for Physical Examination (PE) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, NumericTransformer, TextTransformer


class PEProcessor(BaseDomainProcessor):
    """Physical Examination domain processor.

    Handles domain-specific processing for the PE domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process PE domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Always regenerate PESEQ - source values may not be unique (SD0005)
        frame.loc[:, "PESEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame.loc[:, "PESEQ"] = NumericTransformer.force_numeric(frame["PESEQ"])
        # Normalize visit numbering to align VISIT/VISITNUM
        TextTransformer.normalize_visit(frame)

        # Normalize PESTAT to CDISC CT 'Not Done'
        if "PESTAT" in frame.columns:
            stat_map = {
                "NOT DONE": "NOT DONE",
                "ND": "NOT DONE",
                "DONE": "",
                "COMPLETED": "",
                "": "",
                "nan": "",
            }
            frame.loc[:, "PESTAT"] = (
                frame["PESTAT"]
                .astype(str)
                .str.strip()
                .str.upper()
                .map(stat_map)
                .fillna("")  # Clear invalid values
            )

        # PETEST is required - derive from PETESTCD if available (SD0002)
        if "PETEST" not in frame.columns:
            if "PETESTCD" in frame.columns:
                frame.loc[:, "PETEST"] = frame["PETESTCD"].astype(str).str.upper()
            else:
                frame.loc[:, "PETEST"] = "PHYSICAL EXAMINATION"
        else:
            # Fill empty PETEST values
            needs_test = frame["PETEST"].isna() | (
                frame["PETEST"].astype(str).str.strip() == ""
            )
            if needs_test.any():
                if "PETESTCD" in frame.columns:
                    frame.loc[needs_test, "PETEST"] = (
                        frame.loc[needs_test, "PETESTCD"].astype(str).str.upper()
                    )
                else:
                    frame.loc[needs_test, "PETEST"] = "PHYSICAL EXAMINATION"
        # PESTRESC should be derived from PEORRES if available (SD0036)
        if "PEORRES" in frame.columns:
            if "PESTRESC" not in frame.columns:
                frame["PESTRESC"] = frame["PEORRES"]
            else:
                needs_stresc = frame["PESTRESC"].isna() | (
                    frame["PESTRESC"].astype(str).str.strip() == ""
                )
                if needs_stresc.any():
                    frame.loc[needs_stresc, "PESTRESC"] = frame.loc[
                        needs_stresc, "PEORRES"
                    ]
        if "PESTRESC" in frame.columns:
            empty = frame["PESTRESC"].astype("string").fillna("").str.strip() == ""
            frame.loc[empty, "PESTRESC"] = "NORMAL"
        if "PEORRES" in frame.columns:
            empty_orres = frame["PEORRES"].astype("string").fillna("").str.strip() == ""
            frame.loc[empty_orres, "PEORRES"] = "NORMAL"
        if "PEDTC" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "PEDTC",
                "PEDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
        if "EPOCH" in frame.columns:
            frame.loc[:, "EPOCH"] = TextTransformer.replace_unknown(
                frame["EPOCH"], "TREATMENT"
            )
        else:
            frame.loc[:, "EPOCH"] = "TREATMENT"
        dedup_keys = [k for k in ("USUBJID", "VISITNUM") if k in frame.columns]
        if dedup_keys:
            frame.drop_duplicates(subset=dedup_keys, keep="first", inplace=True)
            frame.reset_index(drop=True, inplace=True)
            frame.loc[:, "PESEQ"] = frame.groupby("USUBJID").cumcount() + 1
