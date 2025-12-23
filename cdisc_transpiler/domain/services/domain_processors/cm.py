"""Domain processor for Concomitant Medications (CM) domain."""

from __future__ import annotations

import pandas as pd

from ..transformers import DateTransformer, NumericTransformer, TextTransformer
from .base import BaseDomainProcessor


class CMProcessor(BaseDomainProcessor):
    """Concomitant Medications domain processor.

    Handles domain-specific processing for the CM domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process CM domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Do not default/guess units or durations. If present, normalize casing/whitespace.
        if "CMDOSU" in frame.columns:
            frame.loc[:, "CMDOSU"] = (
                frame["CMDOSU"].astype("string").fillna("").str.strip().str.lower()
            )
        if "CMDUR" in frame.columns:
            frame.loc[:, "CMDUR"] = (
                frame["CMDUR"].astype("string").fillna("").str.strip()
            )
        # Remove duplicate records based on common key fields
        key_cols = [
            c for c in ("USUBJID", "CMTRT", "CMSTDTC", "CMENDTC") if c in frame.columns
        ]
        if key_cols:
            frame.drop_duplicates(subset=key_cols, keep="first", inplace=True)
        else:
            frame.drop_duplicates(inplace=True)
        # Always regenerate CMSEQ - source values may not be unique (SD0005)
        frame.loc[:, "CMSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame.loc[:, "CMSEQ"] = NumericTransformer.force_numeric(frame["CMSEQ"])
        # Normalize CMDOSTXT to non-numeric descriptive text
        if "CMDOSTXT" in frame.columns:

            def _normalize_dostxt(val: object) -> str:
                text = str(val).strip()
                if text.replace(".", "", 1).isdigit():
                    return f"DOSE {text}"
                return text

            frame.loc[:, "CMDOSTXT"] = frame["CMDOSTXT"].apply(_normalize_dostxt)

        # Normalize CMSTAT to CDISC CT 'Not Done'
        if "CMSTAT" in frame.columns:
            stat_map = {
                "NOT DONE": "NOT DONE",
                "ND": "NOT DONE",
                "": "",
                "nan": "",
            }
            frame.loc[:, "CMSTAT"] = (
                frame["CMSTAT"]
                .astype(str)
                .str.strip()
                .str.upper()
                .map(stat_map)
                .fillna("")  # Clear invalid values
            )

        # Normalize CMDOSFRQ to CDISC CT 'Frequency' codelist
        if "CMDOSFRQ" in frame.columns:
            freq_map = {
                "ONCE": "ONCE",
                "QD": "QD",
                "BID": "BID",
                "TID": "TID",
                "QID": "QID",
                "QOD": "QOD",
                "QW": "QW",
                "QM": "QM",
                "PRN": "PRN",
                "DAILY": "QD",
                "TWICE DAILY": "BID",
                "TWICE PER DAY": "BID",
                "THREE TIMES DAILY": "TID",
                "ONCE DAILY": "QD",
                "AS NEEDED": "PRN",
                "": "",
                "nan": "",
            }
            upper_freq = frame["CMDOSFRQ"].astype(str).str.strip().str.upper()
            frame.loc[:, "CMDOSFRQ"] = upper_freq.map(freq_map).fillna(upper_freq)

        # Normalize CMROUTE to CDISC CT 'Route of Administration Response'
        if "CMROUTE" in frame.columns:
            route_map = {
                "ORAL": "ORAL",
                "PO": "ORAL",
                "INTRAVENOUS": "INTRAVENOUS",
                "IV": "INTRAVENOUS",
                "INTRAMUSCULAR": "INTRAMUSCULAR",
                "IM": "INTRAMUSCULAR",
                "SUBCUTANEOUS": "SUBCUTANEOUS",
                "SC": "SUBCUTANEOUS",
                "SUBQ": "SUBCUTANEOUS",
                "TOPICAL": "TOPICAL",
                "TRANSDERMAL": "TRANSDERMAL",
                "INHALATION": "INHALATION",
                "INHALED": "INHALATION",
                "RECTAL": "RECTAL",
                "VAGINAL": "VAGINAL",
                "OPHTHALMIC": "OPHTHALMIC",
                "OTIC": "OTIC",
                "NASAL": "NASAL",
                "": "",
                "nan": "",
            }
            upper_route = frame["CMROUTE"].astype(str).str.strip().str.upper()
            frame.loc[:, "CMROUTE"] = upper_route.map(route_map).fillna(upper_route)
        # Units - blank unrecognized placeholders rather than defaulting.
        if "CMDOSU" in frame.columns:
            frame.loc[:, "CMDOSU"] = TextTransformer.replace_unknown(
                frame["CMDOSU"], ""
            )

        if "CMSTDTC" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "CMSTDTC",
                "CMSTDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
        if "CMENDTC" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "CMENDTC",
                "CMENDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
        # Do not default/guess EPOCH.
        # Final pass to remove any exact duplicate rows and realign sequence
        frame.drop_duplicates(inplace=True)
        NumericTransformer.assign_sequence(frame, "CMSEQ", "USUBJID")
