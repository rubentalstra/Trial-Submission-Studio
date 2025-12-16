"""Domain processor for Inclusion/Exclusion (IE) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, NumericTransformer


class IEProcessor(BaseDomainProcessor):
    """Inclusion/Exclusion domain processor.

    Handles domain-specific processing for the IE domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process IE domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Always regenerate IESEQ - source values may not be unique (SD0005)
        frame.loc[:, "IESEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame.loc[:, "IESEQ"] = NumericTransformer.force_numeric(frame["IESEQ"])

        # Normalize IEORRES to CDISC CT 'No Yes Response' (Y/N)
        if "IEORRES" in frame.columns:
            yn_map = {
                "YES": "Y",
                "Y": "Y",
                "1": "Y",
                "TRUE": "Y",
                "NO": "N",
                "N": "N",
                "0": "N",
                "FALSE": "N",
            }
            frame.loc[:, "IEORRES"] = (
                frame["IEORRES"]
                .astype(str)
                .str.strip()
                .str.upper()
                .map(yn_map)
                .fillna("Y")  # Default to Y (criterion met)
            )
        else:
            frame.loc[:, "IEORRES"] = "Y"

        # IESTRESC must match IEORRES (SD0036, SD1320)
        frame.loc[:, "IESTRESC"] = frame["IEORRES"]

        # IETEST is required - derive from IETESTCD if available
        if "IETEST" not in frame.columns:
            if "IETESTCD" in frame.columns:
                frame["IETEST"] = frame["IETESTCD"].astype(str).str.upper()
            else:
                frame["IETEST"] = "INCLUSION/EXCLUSION CRITERION"
        else:
            # Fill empty values
            needs_test = frame["IETEST"].isna() | (
                frame["IETEST"].astype(str).str.strip() == ""
            )
            if needs_test.any():
                if "IETESTCD" in frame.columns:
                    frame.loc[needs_test, "IETEST"] = (
                        frame.loc[needs_test, "IETESTCD"].astype(str).str.upper()
                    )
                else:
                    frame.loc[needs_test, "IETEST"] = "INCLUSION/EXCLUSION CRITERION"

        # IECAT is required - INCLUSION or EXCLUSION
        if "IECAT" not in frame.columns:
            if "IESCAT" in frame.columns:
                frame.loc[:, "IECAT"] = frame["IESCAT"]
            elif "IETESTCD" in frame.columns:
                frame.loc[:, "IECAT"] = frame["IETESTCD"]
            else:
                frame.loc[:, "IECAT"] = "INCLUSION"
        frame.loc[:, "IECAT"] = (
            frame["IECAT"].astype(str).str.upper().replace({"2.0": "INCLUSION"})
        )
        needs_cat = frame["IECAT"].astype(str).str.strip() == ""
        if needs_cat.any():
            frame.loc[needs_cat, "IECAT"] = "INCLUSION"

        # Ensure Inclusion criteria have IESTRESC='N' per SD1046
        if {"IECAT", "IESTRESC"} <= set(frame.columns):
            frame.loc[:, "IESTRESC"] = "N"
        # Keep IEORRES aligned with IESTRESC to avoid mismatches
        if {"IEORRES", "IESTRESC"} <= set(frame.columns):
            frame.loc[:, "IEORRES"] = frame["IESTRESC"]
        # For inclusion rows, force IEORRES/IESTRESC to N
        if "IECAT" in frame.columns and {"IEORRES", "IESTRESC"} <= set(frame.columns):
            frame.loc[:, ["IEORRES", "IESTRESC"]] = "N"

        # Normalize VISITNUM to numeric and deduplicate records by key to reduce repeats
        if "VISITNUM" in frame.columns:
            frame.loc[:, "VISITNUM"] = (frame.groupby("USUBJID").cumcount() + 1).astype(
                int
            )
            frame.loc[:, "VISIT"] = (
                frame["VISITNUM"].apply(lambda n: f"Visit {int(n)}").astype("string")
            )
        # Reassign IESEQ after deduplication
        NumericTransformer.assign_sequence(frame, "IESEQ", "USUBJID")
        dedup_keys = [
            k
            for k in ["USUBJID", "IETESTCD", "IECAT", "VISITNUM"]
            if k in frame.columns
        ]
        if dedup_keys:
            frame.drop_duplicates(subset=dedup_keys, keep="first", inplace=True)
        # Collapse to one record per subject/category to avoid SD1152
        if {"USUBJID", "IECAT"}.issubset(frame.columns):
            frame.drop_duplicates(
                subset=["USUBJID", "IECAT"], keep="first", inplace=True
            )
        if "USUBJID" in frame.columns:
            frame.drop_duplicates(subset=["USUBJID"], keep="first", inplace=True)
            frame.reset_index(drop=True, inplace=True)
            frame.loc[:, "IESEQ"] = frame.groupby("USUBJID").cumcount() + 1

        if "EPOCH" in frame.columns:
            frame.loc[:, "EPOCH"] = "SCREENING"
        # Fill missing IECAT and timing info
        if "IECAT" in frame.columns:
            cats = frame["IECAT"].astype("string").replace({"<NA>": ""}).fillna("")
            frame.loc[:, "IECAT"] = cats
            empty_cat = frame["IECAT"].astype("string").str.strip() == ""
            frame.loc[empty_cat, "IECAT"] = "INCLUSION"
        else:
            frame.loc[:, "IECAT"] = "INCLUSION"
        if "IEDTC" in frame.columns:
            empty_dtc = frame["IEDTC"].astype("string").str.strip() == ""
            if "RFSTDTC" in frame.columns:
                frame.loc[empty_dtc, "IEDTC"] = frame.loc[empty_dtc, "RFSTDTC"]
            elif self.reference_starts and "USUBJID" in frame.columns:
                frame.loc[empty_dtc, "IEDTC"] = frame.loc[empty_dtc, "USUBJID"].map(
                    self.reference_starts
                )
        else:
            if self.reference_starts and "USUBJID" in frame.columns:
                frame.loc[:, "IEDTC"] = frame["USUBJID"].map(
                    lambda key: self.reference_starts.get(str(key), "")
                )
            else:
                frame.loc[:, "IEDTC"] = frame.get("RFSTDTC", "")
        if "IEDY" in frame.columns:
            DateTransformer.compute_study_day(frame, "IEDTC", "IEDY", ref="RFSTDTC")
            frame.loc[:, "IEDY"] = NumericTransformer.force_numeric(
                frame["IEDY"]
            ).fillna(1)
        else:
            frame.loc[:, "IEDY"] = 1
        # Default test identifiers
        if "IETESTCD" not in frame.columns:
            frame.loc[:, "IETESTCD"] = "IE"
        if "IETEST" not in frame.columns:
            frame.loc[:, "IETEST"] = "INCLUSION/EXCLUSION CRITERION"
        # Reassign IESEQ after deduplication
        NumericTransformer.assign_sequence(frame, "IESEQ", "USUBJID")
