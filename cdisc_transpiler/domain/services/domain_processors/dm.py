"""Domain processor for Demographics (DM) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer


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
            if "AGE" in frame.columns:
                needs_years = (ageu == "") & frame["AGE"].notna()
                if bool(needs_years.any()):
                    ageu.loc[needs_years] = "YEARS"
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
            sex = sex.replace({"": "U"})
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

        if (
            "DMDY" in frame.columns
            and "DMDTC" in frame.columns
            and "RFSTDTC" in frame.columns
        ):
            DateTransformer.compute_study_day(frame, "DMDTC", "DMDY", ref="RFSTDTC")
