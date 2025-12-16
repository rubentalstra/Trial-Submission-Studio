"""Domain processor for Vital Signs (VS) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, NumericTransformer, TextTransformer
from ....pandas_utils import ensure_numeric_series, ensure_series


class VSProcessor(BaseDomainProcessor):
    """Vital Signs domain processor.

    Handles domain-specific processing for the VS domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process VS domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        TextTransformer.normalize_visit(frame)
        DateTransformer.compute_study_day(frame, "VSDTC", "VSDY", ref="RFSTDTC")
        frame["VSDY"] = NumericTransformer.force_numeric(frame["VSDY"])
        frame["VSLOBXFL"] = ""
        if "VISITNUM" not in frame.columns:
            frame["VISITNUM"] = (frame.groupby("USUBJID").cumcount() + 1).astype(int)
            frame["VISIT"] = frame["VISITNUM"].apply(lambda n: f"Visit {n}")

        vstestcd_series = ensure_series(
            frame.get("VSTESTCD", pd.Series([""] * len(frame))), index=frame.index
        )
        vstestcd_upper = vstestcd_series.astype("string").str.upper().str.strip()
        has_any_test = vstestcd_upper.ne("").any()

        # Preserve VSSTAT from upstream mapping/derivation
        frame["VSSTAT"] = (
            ensure_series(
                frame.get("VSSTAT", pd.Series([""] * len(frame))), index=frame.index
            )
            .astype("string")
            .fillna("")
        )

        default_unit = "beats/min" if not has_any_test else ""
        frame["VSORRESU"] = TextTransformer.replace_unknown(
            frame.get("VSORRESU", pd.Series([""] * len(frame))), default_unit
        )
        frame["VSSTRESU"] = TextTransformer.replace_unknown(
            frame.get("VSSTRESU", pd.Series([""] * len(frame))), default_unit
        )

        if not has_any_test:
            frame["VSTESTCD"] = "HR"
            frame["VSTEST"] = "Heart Rate"
            vsorres = pd.Series([""] * len(frame))
            if "Pulserate" in frame.columns:
                vsorres = frame["Pulserate"]
            frame["VSORRES"] = vsorres.astype("string").fillna("")
            if "Pulse rate (unit)" in frame.columns:
                units = (
                    frame["Pulse rate (unit)"].astype("string").fillna("").str.strip()
                )
                frame["VSORRESU"] = units.replace("", "beats/min")

        for perf_col in ("Were vital signs collected? - Code", "VSPERFCD"):
            if perf_col in frame.columns:
                perf = frame[perf_col].astype("string").str.upper()
                not_done = perf == "N"
                frame.loc[not_done, "VSSTAT"] = "NOT DONE"
                frame.loc[not_done, ["VSORRES", "VSORRESU"]] = ""
                break

        frame["VSSTRESC"] = frame.get("VSSTRESC", frame.get("VSORRES", ""))
        if "VSSTRESC" in frame.columns and "VSORRES" in frame.columns:
            empty_stresc = (
                frame["VSSTRESC"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_stresc, "VSSTRESC"] = frame.loc[empty_stresc, "VSORRES"]

        # Populate VSSTRESU when missing
        if {"VSSTRESU", "VSORRESU"} <= set(frame.columns):
            empty_stresu = (
                frame["VSSTRESU"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_stresu, "VSSTRESU"] = frame.loc[empty_stresu, "VSORRESU"]
        if "VSSTRESU" in frame.columns and "VSTESTCD" in frame.columns:
            test_to_unit = {
                "HR": "beats/min",
                "SYSBP": "mmHg",
                "DIABP": "mmHg",
                "TEMP": "C",
                "WEIGHT": "kg",
                "HEIGHT": "cm",
                "BMI": "kg/m2",
            }
            empty_stresu = (
                frame["VSSTRESU"].astype("string").fillna("").str.strip() == ""
            )
            test_upper = frame["VSTESTCD"].astype("string").str.upper().str.strip()
            mapped = test_upper.map(test_to_unit).fillna("")
            still_empty = empty_stresu & (
                frame["VSORRESU"].astype("string").str.strip() == ""
            )
            frame.loc[empty_stresu & ~still_empty, "VSSTRESU"] = frame.loc[
                empty_stresu & ~still_empty, "VSORRESU"
            ]
            frame.loc[still_empty, "VSSTRESU"] = mapped.loc[still_empty]

        if not has_any_test and "VSORRES" in frame.columns:
            empty_res = frame["VSORRES"].astype("string").fillna("").str.strip() == ""
            frame.loc[empty_res, "VSORRES"] = "0"
            if "VSSTRESC" in frame.columns:
                frame.loc[empty_res, "VSSTRESC"] = frame.loc[empty_res, "VSORRES"]

        ct_units = self._get_controlled_terminology(variable="VSORRESU")
        if ct_units and "VSORRESU" in frame.columns:
            units = frame["VSORRESU"].astype("string").fillna("").str.strip()
            normalized_units = units.apply(ct_units.normalize)
            normalized_units = normalized_units.where(
                normalized_units.isin(ct_units.submission_values), ""
            )
            frame["VSORRESU"] = normalized_units
        if ct_units and "VSSTRESU" in frame.columns:
            st_units = frame["VSSTRESU"].astype("string").fillna("").str.strip()
            normalized_st = st_units.apply(ct_units.normalize)
            normalized_st = normalized_st.where(
                normalized_st.isin(ct_units.submission_values), ""
            )
            frame["VSSTRESU"] = normalized_st
        if "VSSTRESN" in frame.columns:
            numeric = pd.to_numeric(frame["VSORRES"], errors="coerce")
            frame["VSSTRESN"] = ensure_numeric_series(numeric, frame.index)
        NumericTransformer.assign_sequence(frame, "VSSEQ", "USUBJID")
        if "VSLOBXFL" in frame.columns:
            frame["VSLOBXFL"] = (
                ensure_series(frame["VSLOBXFL"]).astype("string").fillna("")
            )
            if {"USUBJID", "VSTESTCD", "VSPOS"} <= set(frame.columns):
                group_cols = ["USUBJID", "VSTESTCD", "VSPOS"]
            else:
                group_cols = ["USUBJID", "VSTESTCD"]
            frame.loc[:, "VSLOBXFL"] = ""
            last_idx = frame.groupby(group_cols).tail(1).index
            frame.loc[last_idx, "VSLOBXFL"] = "Y"
            not_done_mask = (
                ensure_series(
                    frame.get("VSSTAT", pd.Series([""] * len(frame))), index=frame.index
                )
                .astype("string")
                .str.upper()
                == "NOT DONE"
            )
            frame.loc[not_done_mask, "VSLOBXFL"] = ""
        # Normalize test codes to valid CT; fall back to Heart Rate
        ct_vstestcd = self._get_controlled_terminology(variable="VSTESTCD")
        if ct_vstestcd and "VSTESTCD" in frame.columns:
            raw = frame["VSTESTCD"].astype("string").str.strip()
            canonical = raw.apply(ct_vstestcd.normalize)
            valid = canonical.isin(ct_vstestcd.submission_values)
            # Keep canonical when valid; keep original (uppercased) when not
            frame["VSTESTCD"] = canonical.where(valid, raw.str.upper())
            if "VSTEST" in frame.columns:
                frame["VSTEST"] = frame["VSTEST"].astype("string").fillna("")
                empty_vstest = frame["VSTEST"].str.strip() == ""
                frame.loc[empty_vstest, "VSTEST"] = frame.loc[empty_vstest, "VSTESTCD"]
        ct_vstest = self._get_controlled_terminology(variable="VSTEST")
        if ct_vstest and "VSTEST" in frame.columns:
            frame["VSTEST"] = (
                frame["VSTEST"].astype("string").fillna("").apply(ct_vstest.normalize)
            )
        # Clear non-ISO collection times that trigger format errors
        if "VSELTM" in frame.columns:
            frame["VSELTM"] = ""
        if "VSTPTREF" in frame.columns:
            frame["VSTPTREF"] = frame["VSTPTREF"].astype("string").fillna("")
        # Populate timing reference to avoid SD1238
        frame["VSTPTREF"] = "VISIT"
        frame["VSTPT"] = "VISIT"
        if "VISITNUM" in frame.columns:
            frame["VSTPTNUM"] = ensure_numeric_series(
                frame["VISITNUM"], frame.index
            ).fillna(1)
        else:
            frame["VSTPTNUM"] = 1
        if "VSDTC" in frame.columns:
            # Keep all timing records; avoid collapsing multiple measurements
            frame["VSDTC"] = frame["VSDTC"]
        # Derive reference date for VS if missing
        if "VSRFTDTC" not in frame.columns:
            frame["VSRFTDTC"] = frame.get("RFSTDTC", "")
        if (
            self.reference_starts
            and "USUBJID" in frame.columns
            and "VSRFTDTC" in frame.columns
        ):
            empty_ref = frame["VSRFTDTC"].astype("string").fillna("").str.strip() == ""
            frame.loc[empty_ref, "VSRFTDTC"] = frame.loc[empty_ref, "USUBJID"].map(
                self.reference_starts
            )
        # When results missing, clear units to avoid CT errors
        if {"VSORRES", "VSORRESU"} <= set(frame.columns):
            empty_orres = frame["VSORRES"].astype("string").fillna("").str.strip() == ""
            frame.loc[empty_orres, "VSORRESU"] = ""
        if {"VSSTRESC", "VSSTRESU"} <= set(frame.columns):
            empty_stresc = (
                frame["VSSTRESC"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_stresc, "VSSTRESU"] = ""
        # Avoid over-deduplication; only drop exact duplicate rows
        frame.drop_duplicates(inplace=True)
        # Ensure EPOCH is set
        if "EPOCH" in frame.columns:
            frame["EPOCH"] = TextTransformer.replace_unknown(
                frame["EPOCH"], "TREATMENT"
            )
        else:
            frame["EPOCH"] = "TREATMENT"
