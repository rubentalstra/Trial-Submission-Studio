"""Domain processor for Drug Accountability (DA) domain."""

from typing import override

import pandas as pd

from ....pandas_utils import ensure_numeric_series
from ..transformers.date import DateTransformer
from ..transformers.numeric import NumericTransformer
from .base import BaseDomainProcessor


class DAProcessor(BaseDomainProcessor):
    """Drug Accountability domain processor.

    Handles domain-specific processing for the DA domain.
    """

    @override
    def process(self, frame: pd.DataFrame) -> None:
        """Process DA domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        self._drop_placeholder_rows(frame)

        # Always assign unique DASEQ per subject (SD0005 compliance)
        NumericTransformer.assign_sequence(frame, "DASEQ", "USUBJID")

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
            frame.loc[:, "DASTAT"] = (
                frame["DASTAT"]
                .astype("string")
                .fillna("")
                .str.strip()
                .str.upper()
                .map(stat_map)
                .fillna("")
            )

        # Keep result/unit fields consistent without defaulting values.
        if "DAORRESU" in frame.columns and "DAORRES" in frame.columns:
            cleaned_orres = frame["DAORRES"].astype("string").fillna("").str.strip()
            has_orres = cleaned_orres != ""
            frame.loc[~has_orres, "DAORRESU"] = ""

        # DASTRESC should be derived from DAORRES when both are present.
        if {"DAORRES", "DASTRESC"}.issubset(frame.columns):
            orres = frame["DAORRES"].astype("string").fillna("").str.strip()
            stresc = frame["DASTRESC"].astype("string").fillna("").str.strip()
            needs = (stresc == "") & (orres != "")
            if bool(needs.any()):
                frame.loc[needs, "DASTRESC"] = orres.loc[needs]

        # Align DASTRESN with numeric interpretation of DASTRESC when available.
        if {"DASTRESC", "DASTRESN"}.issubset(frame.columns):
            numeric_stresc = ensure_numeric_series(
                frame["DASTRESC"], frame.index
            ).astype("float64")
            coerced = ensure_numeric_series(frame["DASTRESN"], frame.index).astype(
                "float64"
            )
            needs_numeric = coerced.isna() & numeric_stresc.notna()
            frame.loc[:, "DASTRESN"] = coerced
            if bool(needs_numeric.any()):
                frame.loc[needs_numeric, "DASTRESN"] = numeric_stresc.loc[needs_numeric]

            # If the standardized result is non-numeric (e.g., Yes/No), DASTRESN must be blank.
            stresc = frame["DASTRESC"].astype("string").fillna("").str.strip()
            as_num = pd.to_numeric(stresc, errors="coerce")
            non_numeric = (stresc != "") & as_num.isna()
            if bool(non_numeric.any()):
                frame.loc[non_numeric, "DASTRESN"] = pd.NA

        # Prefer an actual event/collection date from the source when present.
        if "DADTC" in frame.columns:
            for raw_date_col in ("EventDate", "EVENTDATE", "DADATE", "DATE"):
                if raw_date_col in frame.columns:
                    needs_dadtc = (
                        frame["DADTC"].astype("string").fillna("").str.strip() == ""
                    )
                    if bool(needs_dadtc.any()):
                        frame.loc[needs_dadtc, "DADTC"] = frame.loc[
                            needs_dadtc, raw_date_col
                        ].apply(DateTransformer.coerce_iso8601)
                    break

            DateTransformer.compute_study_day(
                frame,
                "DADTC",
                "DADY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
