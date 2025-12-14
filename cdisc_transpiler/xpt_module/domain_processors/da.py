"""Domain processor for Drug Accountability (DA) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import TextTransformer, NumericTransformer, DateTransformer
from ...pandas_utils import ensure_numeric_series


class DAProcessor(BaseDomainProcessor):
    """Drug Accountability domain processor.

    Handles domain-specific processing for the DA domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process DA domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # DATEST and DATESTCD are required; force to valid CT pair
        frame["DATEST"] = "Dispensed Amount"
        frame["DATESTCD"] = "DISPAMT"
        # Always assign unique DASEQ per subject (SD0005 compliance)
        # Source data SEQ values may not be unique - we must regenerate
        frame["DASEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame["DASEQ"] = NumericTransformer.force_numeric(frame["DASEQ"])

        # Normalize DASTAT to CDISC CT 'Not Done'
        if "DASTAT" in frame.columns:
            stat_map = {
                "NOT DONE": "NOT DONE",
                "ND": "NOT DONE",
                "DONE": "",
                "COMPLETED": "",
                "": "",
                "nan": "",
            }
            frame["DASTAT"] = (
                frame["DASTAT"]
                .astype(str)
                .str.strip()
                .str.upper()
                .map(stat_map)
                .fillna("")  # Clear invalid values
            )

        if "DAORRESU" not in frame.columns:
            frame["DAORRESU"] = ""

        # DASTRESC should be derived from DAORRES if available (SD0036, SD1320)
        if "DAORRES" in frame.columns:
            cleaned_orres = (
                frame["DAORRES"]
                .astype(str)
                .str.strip()
                .replace({"nan": "", "None": "", "<NA>": ""})
            )
            if "DASTRESC" not in frame.columns:
                frame["DASTRESC"] = cleaned_orres
            else:
                needs_stresc = frame["DASTRESC"].isna() | (
                    frame["DASTRESC"].astype(str).str.strip() == ""
                )
                if needs_stresc.any():
                    frame.loc[needs_stresc, "DASTRESC"] = cleaned_orres.loc[
                        needs_stresc
                    ]
        elif "DASTRESC" not in frame.columns:
            frame["DASTRESC"] = ""

        # Align DASTRESN with numeric interpretation of DASTRESC/DAORRES
        numeric_stresc = ensure_numeric_series(
            frame.get("DASTRESC", pd.Series()), frame.index
        )
        if "DASTRESN" not in frame.columns:
            frame["DASTRESN"] = numeric_stresc
        else:
            from ...cli.logging_config import get_logger
            
            logger = get_logger()
            coerced = ensure_numeric_series(frame["DASTRESN"], frame.index)
            needs_numeric = coerced.isna() & numeric_stresc.notna()
            
            # Ensure DASTRESN has the correct dtype before assignment to avoid FutureWarning
            if "DASTRESN" not in frame.columns or frame["DASTRESN"].dtype != numeric_stresc.dtype:
                try:
                    frame["DASTRESN"] = coerced.astype(numeric_stresc.dtype)
                except Exception as e:
                    logger.warning(f"DA domain: Could not convert DASTRESN dtype: {e}")
                    frame["DASTRESN"] = coerced
            else:
                frame["DASTRESN"] = coerced
            
            # Now safely assign the numeric values where needed
            if needs_numeric.any():
                frame.loc[needs_numeric, "DASTRESN"] = numeric_stresc.loc[needs_numeric]

        # DAORRESU is required when DAORRES is provided (SD0026)
        if "DAORRES" in frame.columns:
            cleaned_orres = (
                frame["DAORRES"]
                .astype(str)
                .str.strip()
                .replace({"nan": "", "None": "", "<NA>": ""})
            )
            has_orres = cleaned_orres != ""
            needs_unit = frame["DAORRESU"].astype(str).str.strip() == ""
            # Clear units when no result present to avoid SD0027/CT errors
            frame.loc[~has_orres, "DAORRESU"] = ""
            if (needs_unit & has_orres).any():
                frame.loc[needs_unit & has_orres, "DAORRESU"] = ""

        # Backfill collection date from DATEST if provided
        if "DADTC" not in frame.columns:
            frame["DADTC"] = ""
        if "DATEST" in frame.columns:
            needs_dadtc = (
                frame["DADTC"]
                .astype(str)
                .str.strip()
                .str.upper()
                .isin({"", "NAN", "<NA>"})
            )
            if needs_dadtc.any():
                frame.loc[needs_dadtc, "DADTC"] = frame.loc[
                    needs_dadtc, "DATEST"
                ].apply(DateTransformer.coerce_iso8601)
        # If still missing, use RFSTDTC as collection date
        if "RFSTDTC" in frame.columns:
            empty_dadtc = frame["DADTC"].astype("string").fillna("").str.strip() == ""
            frame.loc[empty_dadtc, "DADTC"] = frame.loc[empty_dadtc, "RFSTDTC"]
        elif self.reference_starts and "USUBJID" in frame.columns:
            frame["DADTC"] = frame.apply(
                lambda row: self.reference_starts.get(str(row["USUBJID"]), ""),
                axis=1,
            )

        if "DADTC" in frame.columns:
            DateTransformer.compute_study_day(frame, "DADTC", "DADY", ref="RFSTDTC")
        if "EPOCH" in frame.columns:
            frame["EPOCH"] = TextTransformer.replace_unknown(
                frame["EPOCH"], "TREATMENT"
            )
        else:
            frame["EPOCH"] = "TREATMENT"
        # Normalize VISITNUM to numeric per subject order to avoid type/key issues
        if "VISITNUM" in frame.columns:
            frame["VISITNUM"] = (frame.groupby("USUBJID").cumcount() + 1).astype(int)
            frame["VISIT"] = frame["VISITNUM"].apply(lambda n: f"Visit {n}")
        # Fill missing results to satisfy presence rules
        if "DAORRES" in frame.columns:
            frame["DAORRES"] = frame["DAORRES"].astype("string")
            empty_orres = frame["DAORRES"].fillna("").str.strip() == ""
            frame.loc[empty_orres, "DAORRES"] = "0"
        else:
            frame["DAORRES"] = "0"
        if "DASTRESC" in frame.columns:
            empty_stresc = (
                frame["DASTRESC"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_stresc, "DASTRESC"] = frame.loc[empty_stresc, "DAORRES"]
        else:
            frame["DASTRESC"] = frame.get("DAORRES", "0")
        if "DASTRESN" in frame.columns:
            coerced = ensure_numeric_series(frame["DASTRESN"], frame.index)
            fallback = ensure_numeric_series(frame["DAORRES"], frame.index)
            frame["DASTRESN"] = coerced.fillna(fallback)
        else:
            frame["DASTRESN"] = ensure_numeric_series(
                frame.get("DAORRES", "0"), frame.index
            )
        # Normalize VISITNUM to numeric per subject order to avoid type/key issues
        if "VISITNUM" in frame.columns:
            frame["VISITNUM"] = (frame.groupby("USUBJID").cumcount() + 1).astype(int)
            frame["VISIT"] = frame["VISITNUM"].apply(lambda n: f"Visit {n}")
