"""Domain processor for Demographics (DM) domain."""

from __future__ import annotations

import pandas as pd

from ..transformers import DateTransformer
from .base import BaseDomainProcessor


class DMProcessor(BaseDomainProcessor):
    """Demographics domain processor.

    Handles domain-specific processing for the DM domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process DM domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        self._drop_placeholder_rows(frame)

        if "AGE" in frame.columns:
            frame.loc[:, "AGE"] = pd.to_numeric(frame["AGE"], errors="coerce")

        if "AGEU" in frame.columns:
            ageu = (
                frame["AGEU"]
                .astype("string")
                .fillna("")
                .str.upper()
                .str.strip()
                .replace(
                    {
                        "YEAR": "YEARS",
                        "YRS": "YEARS",
                        "Y": "YEARS",
                    }
                )
            )
            frame.loc[:, "AGEU"] = ageu

        if "COUNTRY" in frame.columns:
            frame.loc[:, "COUNTRY"] = (
                frame["COUNTRY"].astype("string").fillna("").str.strip()
            )

        if "ETHNIC" in frame.columns:
            frame.loc[:, "ETHNIC"] = (
                frame["ETHNIC"]
                .astype("string")
                .fillna("")
                .str.upper()
                .str.strip()
                .replace({"UNK": "UNKNOWN"})
            )

        if "RACE" in frame.columns:
            frame.loc[:, "RACE"] = (
                frame["RACE"]
                .astype("string")
                .fillna("")
                .str.upper()
                .str.strip()
                .replace(
                    {
                        "WHITE, CAUCASIAN, OR ARABIC": "WHITE",
                        "CAUCASIAN": "WHITE",
                        "BLACK": "BLACK OR AFRICAN AMERICAN",
                        "AFRICAN AMERICAN": "BLACK OR AFRICAN AMERICAN",
                        "UNK": "UNKNOWN",
                    }
                )
            )

        if "SEX" in frame.columns:
            sex = (
                frame["SEX"]
                .astype("string")
                .fillna("")
                .str.upper()
                .str.strip()
                .replace(
                    {
                        "FEMALE": "F",
                        "MALE": "M",
                        "UNKNOWN": "U",
                        "UNK": "U",
                    }
                )
            )
            frame.loc[:, "SEX"] = sex

        for date_col in (
            "RFICDTC",
            "RFSTDTC",
            "RFENDTC",
            "RFXSTDTC",
            "RFXENDTC",
            "DMDTC",
        ):
            if date_col in frame.columns:
                frame.loc[:, date_col] = (
                    frame[date_col]
                    .astype("string")
                    .fillna("")
                    .str.strip()
                    .str.split("T")
                    .str[0]
                )

        # SDTMIG v3.4 notes RFSTDTC is sponsor-defined and may be defined as an
        # enrollment date (not necessarily first dose) for certain study designs.
        # When RFSTDTC is missing but RFICDTC is present, use RFICDTC as a
        # deterministic fallback so study day calculations have a visible anchor.
        if "RFSTDTC" in frame.columns and "RFICDTC" in frame.columns:
            rfstdtc = frame["RFSTDTC"].astype("string").fillna("").str.strip()
            rficdtc = frame["RFICDTC"].astype("string").fillna("").str.strip()
            needs_rfstdtc = (rfstdtc == "") & (rficdtc != "")
            if bool(needs_rfstdtc.any()):
                frame.loc[needs_rfstdtc, "RFSTDTC"] = rficdtc.loc[needs_rfstdtc]

        if (
            "DMDY" in frame.columns
            and "DMDTC" in frame.columns
            and "RFSTDTC" in frame.columns
        ):
            DateTransformer.compute_study_day(frame, "DMDTC", "DMDY", ref="RFSTDTC")
