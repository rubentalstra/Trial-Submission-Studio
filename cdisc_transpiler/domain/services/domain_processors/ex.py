"""Domain processor for Exposure (EX) domain."""

from __future__ import annotations

import pandas as pd

from cdisc_transpiler.constants import Defaults

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, NumericTransformer, TextTransformer


class EXProcessor(BaseDomainProcessor):
    """Exposure domain processor.

    Handles domain-specific processing for the EX domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process EX domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        frame.loc[:, "EXTRT"] = TextTransformer.replace_unknown(
            frame.get("EXTRT", pd.Series([""] * len(frame))), "TREATMENT"
        ).astype("string")
        # Always regenerate EXSEQ - source values may not be unique (SD0005)
        frame.loc[:, "EXSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame.loc[:, "EXSEQ"] = NumericTransformer.force_numeric(frame["EXSEQ"])
        frame.loc[:, "EXSTDTC"] = TextTransformer.replace_unknown(
            frame.get("EXSTDTC", pd.Series([""] * len(frame))), Defaults.DATE
        )
        end_series = frame.get("EXENDTC", pd.Series([""] * len(frame)))
        end_series = TextTransformer.replace_unknown(end_series, "2023-12-31")
        frame.loc[:, "EXENDTC"] = end_series.where(
            end_series.astype(str).str.strip() != "", frame["EXSTDTC"]
        )
        DateTransformer.ensure_date_pair_order(frame, "EXSTDTC", "EXENDTC")
        DateTransformer.compute_study_day(frame, "EXSTDTC", "EXSTDY", ref="RFSTDTC")
        DateTransformer.compute_study_day(frame, "EXENDTC", "EXENDY", ref="RFSTDTC")
        frame.loc[:, "EXDOSFRM"] = TextTransformer.replace_unknown(
            frame["EXDOSFRM"], "TABLET"
        ).astype("string")

        # EXDOSU is required when EXDOSE/EXDOSTXT/EXDOSTOT is provided (SD0035)
        if "EXDOSU" not in frame.columns:
            frame.loc[:, "EXDOSU"] = "mg"
        else:
            needs_unit = frame["EXDOSU"].isna() | (
                frame["EXDOSU"].astype(str).str.strip() == ""
            )
            if needs_unit.any():
                # Check if dose is provided
                has_dose = (
                    ("EXDOSE" in frame.columns and frame["EXDOSE"].notna())
                    | (
                        "EXDOSTXT" in frame.columns
                        and frame["EXDOSTXT"].astype(str).str.strip() != ""
                    )
                    | ("EXDOSTOT" in frame.columns and frame["EXDOSTOT"].notna())
                )
                frame.loc[needs_unit & has_dose, "EXDOSU"] = "mg"

        frame.loc[:, "EXDOSFRQ"] = TextTransformer.replace_unknown(
            frame.get("EXDOSFRQ", pd.Series(["" for _ in frame.index])), "QD"
        ).astype("string")
        # EXDUR permissibility - provide basic duration
        frame.loc[:, "EXDUR"] = TextTransformer.replace_unknown(
            frame.get("EXDUR", pd.Series([""] * len(frame))), "P1D"
        ).astype("string")
        # Align EXSCAT/EXCAT to a controlled value with sane length
        frame.loc[:, "EXSCAT"] = ""
        frame.loc[:, "EXCAT"] = "INVESTIGATIONAL PRODUCT"

        # EXCAT is required when EXSCAT is provided (SD1098)
        if "EXSCAT" in frame.columns:
            if "EXCAT" not in frame.columns:
                frame.loc[:, "EXCAT"] = "INVESTIGATIONAL PRODUCT"
            else:
                needs_cat = frame["EXCAT"].isna() | (
                    frame["EXCAT"].astype(str).str.strip() == ""
                )
                if needs_cat.any():
                    frame.loc[needs_cat, "EXCAT"] = "INVESTIGATIONAL PRODUCT"

        # EPOCH is required when EXSTDTC is provided (SD1339)
        if "EPOCH" in frame.columns:
            frame.loc[:, "EPOCH"] = "TREATMENT"
        elif "EXSTDTC" in frame.columns:
            frame.loc[:, "EPOCH"] = "TREATMENT"
        # Clear non-ISO EXELTM values and ensure EXTPTREF exists
        if "EXELTM" in frame.columns:
            frame.loc[:, "EXELTM"] = "PT0H"
        if "EXTPTREF" not in frame.columns:
            frame.loc[:, "EXTPTREF"] = "VISIT"
        else:
            frame.loc[:, "EXTPTREF"] = (
                frame["EXTPTREF"].astype("string").fillna("").replace("", "VISIT")
            )
        NumericTransformer.assign_sequence(frame, "EXSEQ", "USUBJID")
        # Recompute dates/study days for any appended defaults
        DateTransformer.ensure_date_pair_order(frame, "EXSTDTC", "EXENDTC")
        DateTransformer.compute_study_day(frame, "EXSTDTC", "EXSTDY", ref="RFSTDTC")
        DateTransformer.compute_study_day(frame, "EXENDTC", "EXENDY", ref="RFSTDTC")
        for dy in ("EXSTDY", "EXENDY"):
            if dy in frame.columns:
                frame.loc[:, dy] = NumericTransformer.force_numeric(frame[dy]).fillna(1)
        # Ensure timing reference present when EXRFTDTC populated
        if "EXTPTREF" in frame.columns:
            frame.loc[:, "EXTPTREF"] = (
                frame["EXTPTREF"].astype("string").fillna("").replace("", "VISIT")
            )
        # Reference start date on EX records
        if "EXRFTDTC" not in frame.columns:
            frame.loc[:, "EXRFTDTC"] = frame.get(
                "RFSTDTC", pd.Series([""] * len(frame))
            )
        if (
            self.reference_starts
            and "EXRFTDTC" in frame.columns
            and "USUBJID" in frame.columns
        ):
            empty_ref = frame["EXRFTDTC"].astype("string").fillna("").str.strip() == ""
            frame.loc[empty_ref, "EXRFTDTC"] = frame.loc[empty_ref, "USUBJID"].map(
                self.reference_starts
            )
        elif "EXRFTDTC" in frame.columns:
            frame.loc[:, "EXRFTDTC"] = frame["EXRFTDTC"].replace(
                "", frame.get("RFSTDTC", "")
            )
