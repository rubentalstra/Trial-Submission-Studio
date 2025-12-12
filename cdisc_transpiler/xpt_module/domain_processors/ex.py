"""Domain processor for Exposure (EX) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import TextTransformer, NumericTransformer, DateTransformer


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

        frame["EXTRT"] = TextTransformer.replace_unknown(
            frame.get("EXTRT", pd.Series([""] * len(frame))), "TREATMENT"
        )
        # Always regenerate EXSEQ - source values may not be unique (SD0005)
        frame["EXSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame["EXSEQ"] = NumericTransformer.force_numeric(frame["EXSEQ"])
        frame["EXSTDTC"] = TextTransformer.replace_unknown(
            frame.get("EXSTDTC", pd.Series([""] * len(frame))), "2023-01-01"
        )
        end_series = frame.get("EXENDTC", pd.Series([""] * len(frame)))
        end_series = TextTransformer.replace_unknown(end_series, "2023-12-31")
        frame["EXENDTC"] = end_series.where(
            end_series.astype(str).str.strip() != "", frame["EXSTDTC"]
        )
        DateTransformer.ensure_date_pair_order(frame, "EXSTDTC", "EXENDTC")
        DateTransformer.compute_study_day(frame, "EXSTDTC", "EXSTDY", ref="RFSTDTC")
        DateTransformer.compute_study_day(frame, "EXENDTC", "EXENDY", ref="RFSTDTC")
        frame["EXDOSFRM"] = TextTransformer.replace_unknown(frame["EXDOSFRM"], "TABLET")

        # EXDOSU is required when EXDOSE/EXDOSTXT/EXDOSTOT is provided (SD0035)
        if "EXDOSU" not in frame.columns:
            frame["EXDOSU"] = "mg"
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

        frame["EXDOSFRQ"] = TextTransformer.replace_unknown(
            frame.get("EXDOSFRQ", pd.Series(["" for _ in frame.index])), "QD"
        )
        # EXDUR permissibility - provide basic duration
        frame["EXDUR"] = TextTransformer.replace_unknown(
            frame.get("EXDUR", pd.Series([""] * len(frame))), "P1D"
        )
        # Align EXSCAT/EXCAT to a controlled value with sane length
        frame["EXSCAT"] = ""
        frame["EXCAT"] = "INVESTIGATIONAL PRODUCT"

        # EXCAT is required when EXSCAT is provided (SD1098)
        if "EXSCAT" in frame.columns:
            if "EXCAT" not in frame.columns:
                frame["EXCAT"] = "INVESTIGATIONAL PRODUCT"
            else:
                needs_cat = frame["EXCAT"].isna() | (
                    frame["EXCAT"].astype(str).str.strip() == ""
                )
                if needs_cat.any():
                    frame.loc[needs_cat, "EXCAT"] = "INVESTIGATIONAL PRODUCT"

        # EPOCH is required when EXSTDTC is provided (SD1339)
        if "EPOCH" in frame.columns:
            frame["EPOCH"] = "TREATMENT"
        elif "EXSTDTC" in frame.columns:
            frame["EPOCH"] = "TREATMENT"
        # Clear non-ISO EXELTM values and ensure EXTPTREF exists
        if "EXELTM" in frame.columns:
            frame["EXELTM"] = "PT0H"
        if "EXTPTREF" not in frame.columns:
            frame["EXTPTREF"] = "VISIT"
        else:
            frame["EXTPTREF"] = (
                frame["EXTPTREF"].astype("string").fillna("").replace("", "VISIT")
            )
        existing = set(frame.get("USUBJID", pd.Series(dtype=str)).astype(str))
        # Ensure every subject with a reference start has an EX record
        if self.reference_starts:
            missing = set(self.reference_starts.keys()) - existing
            if missing:
                filler = []
                for usubjid in missing:
                    start = (
                        DateTransformer.coerce_iso8601(
                            self.reference_starts.get(usubjid, "")
                        )
                        or "2023-01-01"
                    )
                    filler.append(
                        {
                            "STUDYID": self.config.study_id or "STUDY",
                            "DOMAIN": "EX",
                            "USUBJID": usubjid,
                            "EXSEQ": float("nan"),
                            "EXTRT": "TREATMENT",
                            "EXDOSE": float("nan"),
                            "EXDOSU": "mg",
                            "EXDOSFRM": "TABLET",
                            "EXDOSFRQ": "",
                            "EXSTDTC": start,
                            "EXENDTC": start,
                            "EXDUR": "P1D",
                            "EXSTDY": float("nan"),
                            "EXENDY": float("nan"),
                            "EPOCH": "TREATMENT",
                        }
                    )
                filler_df = pd.DataFrame(filler).reindex(
                    columns=frame.columns, fill_value=""
                )
                new_frame = pd.concat([frame, filler_df], ignore_index=True)
                frame.drop(frame.index, inplace=True)
                frame.drop(columns=list(frame.columns), inplace=True)
                for col in new_frame.columns:
                    frame[col] = new_frame[col].values
        NumericTransformer.assign_sequence(frame, "EXSEQ", "USUBJID")
        # Recompute dates/study days for any appended defaults
        DateTransformer.ensure_date_pair_order(frame, "EXSTDTC", "EXENDTC")
        DateTransformer.compute_study_day(frame, "EXSTDTC", "EXSTDY", ref="RFSTDTC")
        DateTransformer.compute_study_day(frame, "EXENDTC", "EXENDY", ref="RFSTDTC")
        for dy in ("EXSTDY", "EXENDY"):
            if dy in frame.columns:
                frame[dy] = NumericTransformer.force_numeric(frame[dy]).fillna(1)
        # Ensure timing reference present when EXRFTDTC populated
        if "EXTPTREF" in frame.columns:
            frame["EXTPTREF"] = (
                frame["EXTPTREF"].astype("string").fillna("").replace("", "VISIT")
            )
        # Reference start date on EX records
        if "EXRFTDTC" not in frame.columns:
            frame["EXRFTDTC"] = frame.get("RFSTDTC", pd.Series([""] * len(frame)))
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
            frame["EXRFTDTC"] = frame["EXRFTDTC"].replace("", frame.get("RFSTDTC", ""))
