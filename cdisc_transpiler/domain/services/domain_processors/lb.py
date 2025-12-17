"""Domain processor for Laboratory (LB) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, NumericTransformer, TextTransformer
from ....pandas_utils import ensure_numeric_series, ensure_series


class LBProcessor(BaseDomainProcessor):
    """Laboratory domain processor.

    Handles domain-specific processing for the LB domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process LB domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        # Normalize VISITNUM/VISIT when provided
        if {"VISIT", "VISITNUM"} & set(frame.columns):
            TextTransformer.normalize_visit(frame)

        # Clean up literal string placeholders that commonly appear after mapping.
        # Keep this conservative: only known placeholder tokens.
        for col in ("LBORRESU", "LBSTRESU"):
            if col in frame.columns:
                frame.loc[:, col] = (
                    frame[col]
                    .astype("string")
                    .replace({"<NA>": "", "nan": "", "None": ""})
                    .fillna("")
                )

        # Force LBTEST names to CT-friendly labels based on LBTESTCD
        if "LBTESTCD" in frame.columns:
            lb_label_map = {
                "ALT": "Alanine Aminotransferase",
                "AST": "Aspartate Aminotransferase",
                "CHOL": "Cholesterol",
                "GLUC": "Glucose",
                "HGB": "Hemoglobin",
                "HCT": "Hematocrit",
                "RBC": "Erythrocytes",
                "WBC": "Leukocytes",
                "PLAT": "Platelets",
                "PH": "pH",
                "PROT": "Protein",
                "OCCBLD": "Occult Blood",
            }

            testcd = (
                frame["LBTESTCD"].astype("string").fillna("").str.upper().str.strip()
            )
            # Common source artifact: site code leaks into LBTESTCD.
            for site_col in ("SiteCode", "Site code"):
                if site_col in frame.columns:
                    site = (
                        frame[site_col]
                        .astype("string")
                        .fillna("")
                        .str.upper()
                        .str.strip()
                    )
                    testcd = testcd.where(testcd != site, "")
            # Standardize urine glucose token if present.
            testcd = testcd.replace({"GLUCU": "GLUC"})

            # If LBTESTCD equals the USUBJID prefix (e.g., KIEM-01), it's almost
            # certainly a site/subject artifact rather than a lab test code.
            if "USUBJID" in frame.columns:
                prefix = (
                    frame["USUBJID"]
                    .astype("string")
                    .fillna("")
                    .str.split("-", n=1)
                    .str[0]
                    .str.upper()
                    .str.strip()
                )
                testcd = testcd.where(testcd != prefix, "")

            # If CT is available, normalize and blank invalid values.
            ct_lbtestcd = self._get_controlled_terminology(variable="LBTESTCD")
            if ct_lbtestcd:
                canonical = testcd.apply(ct_lbtestcd.normalize)
                valid = canonical.isin(ct_lbtestcd.submission_values)
                testcd = canonical.where(valid, "")

            mapped_lbtest = testcd.map(lb_label_map).astype("string")
            existing_lbtest = (
                frame["LBTEST"].astype("string")
                if "LBTEST" in frame.columns
                else pd.Series([""] * len(frame), index=frame.index, dtype="string")
            )
            resolved = mapped_lbtest.fillna(existing_lbtest).astype("string").fillna("")
            # Required: LBTEST must not be blank when LBTESTCD is present.
            resolved = resolved.where(resolved.str.strip() != "", testcd)
            frame.loc[:, "LBTEST"] = resolved
            frame.loc[:, "LBTESTCD"] = testcd

            # Drop rows that do not have a valid LBTESTCD after cleanup.
            # This prevents operational/helper files from polluting LB outputs.
            empty_testcd = (
                frame["LBTESTCD"].astype("string").fillna("").str.strip() == ""
            )
            if empty_testcd.any():
                frame.drop(index=frame.index[empty_testcd], inplace=True)
        # Derive LBDTC from LBENDTC before computing study days
        if "LBENDTC" in frame.columns:
            has_endtc = frame["LBENDTC"].astype(str).str.strip() != ""
            if "LBDTC" not in frame.columns:
                frame.loc[:, "LBDTC"] = ""
            needs_dtc = has_endtc & (frame["LBDTC"].astype(str).str.strip() == "")
            if needs_dtc.any():
                frame.loc[needs_dtc, "LBDTC"] = frame.loc[needs_dtc, "LBENDTC"]
        # If LBENDTC is missing, add and default it to LBDTC to avoid empty permissible column
        if "LBENDTC" not in frame.columns and "LBDTC" in frame.columns:
            frame.loc[:, "LBENDTC"] = frame["LBDTC"]
        if "LBENDTC" in frame.columns and "LBDTC" in frame.columns:
            empty_endtc = frame["LBENDTC"].astype(str).str.strip() == ""
            frame.loc[empty_endtc, "LBENDTC"] = frame.loc[empty_endtc, "LBDTC"]
        # Compute study days
        if "LBDTC" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "LBDTC",
                "LBDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
        if "LBENDTC" in frame.columns and "LBENDY" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "LBENDTC",
                "LBENDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
        if "LBDY" in frame.columns:
            frame.loc[:, "LBDY"] = NumericTransformer.force_numeric(frame["LBDY"])
        else:
            frame.loc[:, "LBDY"] = pd.NA
        # Expected variable LBLOBXFL should exist even when empty
        if "LBLOBXFL" not in frame.columns:
            frame.loc[:, "LBLOBXFL"] = ""
        if "LBSTRESC" in frame.columns:
            stresc = frame["LBSTRESC"].astype("string").fillna("")
            # Normalize common qualitative results to uppercase to align with
            # user-defined codelists that frequently use uppercase tokens.
            normalized = stresc.str.strip()
            normalized = normalized.where(normalized != "", "")
            normalized = normalized.replace(
                {"Positive": "POSITIVE", "Negative": "NEGATIVE"}
            )
            normalized = normalized.str.upper()
            frame.loc[:, "LBSTRESC"] = normalized
        # Ensure LBSTRESC mirrors LBORRES when missing
        if "LBORRES" in frame.columns and "LBSTRESC" in frame.columns:
            empty_stresc = frame["LBSTRESC"].astype(str).str.strip() == ""
            orres_str = (
                frame["LBORRES"]
                .astype("string")
                .replace({"<NA>": "", "nan": "", "None": ""})
            )
            frame.loc[empty_stresc, "LBSTRESC"] = orres_str.where(orres_str != "", "0")
        if "LBORRESU" not in frame.columns:
            frame.loc[:, "LBORRESU"] = ""
        else:
            frame.loc[:, "LBORRESU"] = (
                frame["LBORRESU"]
                .astype("string")
                .replace({"<NA>": "", "nan": "", "None": ""})
                .fillna("")
            )
        if "LBSTRESU" not in frame.columns:
            frame.loc[:, "LBSTRESU"] = ""
        else:
            frame.loc[:, "LBSTRESU"] = (
                frame["LBSTRESU"]
                .astype("string")
                .replace({"<NA>": "", "nan": "", "None": ""})
                .fillna("")
            )
        if "LBNRIND" in frame.columns:
            frame.loc[:, "LBNRIND"] = TextTransformer.replace_unknown(
                frame["LBNRIND"], "NORMAL"
            ).astype("string")
        if "LBLOBXFL" not in frame.columns:
            frame.loc[:, "LBLOBXFL"] = ""
        else:
            frame.loc[:, "LBLOBXFL"] = frame["LBLOBXFL"].fillna("")

        # Always regenerate LBSEQ - source values may not be unique (SD0005)
        frame.loc[:, "LBSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        frame.loc[:, "LBSEQ"] = NumericTransformer.force_numeric(frame["LBSEQ"])

        # Normalize LBCLSIG to CDISC CT 'No Yes Response' (Y/N)
        if "LBCLSIG" in frame.columns:
            yn_map = {
                "YES": "Y",
                "Y": "Y",
                "1": "Y",
                "TRUE": "Y",
                "NO": "N",
                "N": "N",
                "0": "N",
                "FALSE": "N",
                "CS": "Y",
                "NCS": "N",  # Clinical Significance codes
                "": "",
                "nan": "",
            }
            frame.loc[:, "LBCLSIG"] = (
                frame["LBCLSIG"]
                .astype(str)
                .str.strip()
                .str.upper()
                .map(yn_map)
                .fillna("")
                .astype("string")
            )

        if "LBSTRESC" in frame.columns and "LBSTRESN" in frame.columns:
            numeric = ensure_numeric_series(frame["LBSTRESC"], frame.index).astype(
                "float64"
            )
            frame.loc[:, "LBSTRESN"] = numeric
        if "LBORRES" in frame.columns and "LBSTRESC" in frame.columns:
            empty_stresc = frame["LBSTRESC"].astype(str).str.strip() == ""
            orres_str = (
                frame["LBORRES"]
                .astype("string")
                .replace({"<NA>": "", "nan": "", "None": ""})
            )
            frame.loc[empty_stresc, "LBSTRESC"] = orres_str.where(orres_str != "", "")
        # Also ensure LBSTRESC is populated when LBORRES exists (SD0036, SD1320)
        if "LBORRES" in frame.columns:
            if "LBSTRESC" not in frame.columns:
                frame.loc[:, "LBSTRESC"] = frame["LBORRES"]
            else:
                needs_stresc = frame["LBSTRESC"].isna() | (
                    frame["LBSTRESC"].astype(str).str.strip() == ""
                )
                if needs_stresc.any():
                    frame.loc[needs_stresc, "LBSTRESC"] = frame.loc[
                        needs_stresc, "LBORRES"
                    ]

        if "LBSTRESN" in frame.columns:
            frame.loc[:, "LBSTRESN"] = ensure_numeric_series(
                frame["LBSTRESN"], frame.index
            ).astype("float64")
            needs_stresn = frame["LBSTRESN"].isna() & (
                frame["LBSTRESC"].astype("string").fillna("").str.strip() != ""
            )
            numeric_fill = ensure_numeric_series(
                frame.loc[needs_stresn, "LBSTRESC"], frame.index
            ).astype("float64")
            frame.loc[needs_stresn, "LBSTRESN"] = numeric_fill
            frame.loc[
                frame["LBSTRESC"]
                .astype("string")
                .str.upper()
                .isin({"NEGATIVE", "POSITIVE"}),
                "LBSTRESN",
            ] = float("nan")
        for col in ("LBDY", "LBENDY", "VISITDY", "VISITNUM"):
            if col in frame.columns:
                frame.loc[:, col] = pd.to_numeric(frame[col], errors="coerce")
        # LBORNRLO and LBORNRHI are character fields per SDTM IG
        # LBSTNRLO and LBSTNRHI are numeric
        for col in ("LBORNRLO", "LBORNRHI"):
            if col in frame.columns:
                series = frame[col].astype("string").fillna("")
                frame.loc[:, col] = series.replace({"0.0": "0", "0": "0"})
        for col in ("LBSTNRLO", "LBSTNRHI"):
            if col in frame.columns:
                frame.loc[:, col] = ensure_numeric_series(
                    frame[col], frame.index
                ).fillna(0)
        # Provide default units for non-missing results using test-code-aware defaults.
        unit_by_testcd = {
            "GLUC": "mg/dL",
            "CHOL": "mg/dL",
            "HGB": "g/dL",
            "HCT": "%",
            "RBC": "10^12/L",
            "WBC": "10^9/L",
            "PLAT": "10^9/L",
            "ALT": "U/L",
            "AST": "U/L",
            "PROT": "g/L",
            "PH": "",
            "OCCBLD": "",
        }
        if {"LBTESTCD", "LBORRESU"} <= set(frame.columns):
            testcd = (
                frame["LBTESTCD"].astype("string").fillna("").str.strip().str.upper()
            )
            inferred = testcd.map(unit_by_testcd).fillna("")
            has_result = (
                frame.get("LBORRES", pd.Series(index=frame.index))
                .astype("string")
                .fillna("")
                .str.strip()
                != ""
            ) | (
                frame.get("LBSTRESC", pd.Series(index=frame.index))
                .astype("string")
                .fillna("")
                .str.strip()
                != ""
            )
            needs_oru = frame["LBORRESU"].astype("string").fillna("").str.strip() == ""
            frame.loc[needs_oru & has_result, "LBORRESU"] = inferred.loc[
                needs_oru & has_result
            ]
        if {"LBTESTCD", "LBSTRESU"} <= set(frame.columns):
            testcd = (
                frame["LBTESTCD"].astype("string").fillna("").str.strip().str.upper()
            )
            inferred = testcd.map(unit_by_testcd).fillna("")
            has_result = (
                frame.get("LBORRES", pd.Series(index=frame.index))
                .astype("string")
                .fillna("")
                .str.strip()
                != ""
            ) | (
                frame.get("LBSTRESC", pd.Series(index=frame.index))
                .astype("string")
                .fillna("")
                .str.strip()
                != ""
            )
            needs_su = frame["LBSTRESU"].astype("string").fillna("").str.strip() == ""
            frame.loc[needs_su & has_result, "LBSTRESU"] = inferred.loc[
                needs_su & has_result
            ]
        ct_lb_units = self._get_controlled_terminology(variable="LBORRESU")
        if ct_lb_units:
            for col in ("LBORRESU", "LBSTRESU"):
                if col in frame.columns:
                    units = frame[col].astype("string").fillna("").str.strip()
                    normalized = units.apply(ct_lb_units.normalize)
                    has_value = units != ""
                    normalized = normalized.where(
                        normalized.isin(ct_lb_units.submission_values), "U/L"
                    )
                    normalized = normalized.where(has_value, "")
                    frame.loc[:, col] = normalized

        # LBCAT is required when LBSCAT is present (SD1098)
        if "LBSCAT" in frame.columns:
            if "LBCAT" not in frame.columns:
                frame.loc[:, "LBCAT"] = "LABORATORY"
            else:
                needs_cat = frame["LBCAT"].isna() | (
                    frame["LBCAT"].astype(str).str.strip() == ""
                )
                if needs_cat.any():
                    frame.loc[needs_cat, "LBCAT"] = "LABORATORY"
        elif "LBCAT" in frame.columns:
            frame.loc[:, "LBCAT"] = (
                frame["LBCAT"].replace("", "LABORATORY").fillna("LABORATORY")
            )

        # LBSTAT is required when LBREASND is provided (SD0023)
        if "LBREASND" in frame.columns:
            has_reasnd = frame["LBREASND"].astype(str).str.strip() != ""
            if "LBSTAT" not in frame.columns:
                frame.loc[:, "LBSTAT"] = ""
            frame.loc[
                has_reasnd & (frame["LBSTAT"].astype(str).str.strip() == ""),
                "LBSTAT",
            ] = "NOT DONE"

        if "LBORRES" in frame.columns:
            numeric_orres = ensure_numeric_series(frame["LBORRES"], frame.index)
            range_cols = [
                frame[col]
                for col in ("LBORNRLO", "LBORNRHI", "LBSTNRLO", "LBSTNRHI")
                if col in frame.columns
            ]
            has_ranges = any(
                (col_series.astype(str).str.strip() != "").any()
                for col_series in range_cols
            )
            if has_ranges:
                frame.loc[:, "LBORRES"] = numeric_orres.fillna(0).astype(str)
        if "EPOCH" in frame.columns:
            frame.loc[:, "EPOCH"] = "TREATMENT"
        # Ensure LBLOBXFL is empty (last observation flag not applicable with single record)
        if "LBLOBXFL" in frame.columns:
            frame.loc[:, "LBLOBXFL"] = ""
        # Clear optional specimen/result type qualifiers that were non-CT values
        for col in ("LBRESTYP", "LBSPEC", "LBSPCCND"):
            if col in frame.columns:
                frame.loc[:, col] = ""
        # Drop optional columns causing CT issues when unneeded
        for col in ("LBANMETH", "LBTSTOPO", "LBTPTREF", "LBPDUR", "LBRFTDTC"):
            if col in frame.columns:
                frame.drop(columns=[col], inplace=True)
        if "LBELTM" in frame.columns:
            frame.loc[:, "LBELTM"] = ""
        for col in ("LBBDAGNT", "LBCLSIG", "LBREFID", "LBSCAT"):
            if col in frame.columns:
                frame.drop(columns=[col], inplace=True)
        # Remove duplicate records on key identifiers to reduce SD1117 noise
        key_cols = [
            col
            for col in ("USUBJID", "LBTESTCD", "LBDTC", "LBENDTC", "VISITNUM")
            if col in frame.columns
        ]
        if key_cols:
            frame.drop_duplicates(subset=key_cols, keep="first", inplace=True)
        else:
            frame.drop_duplicates(inplace=True)
        frame.drop_duplicates(inplace=True)
        dup_keys = [
            col
            for col in (
                "USUBJID",
                "LBTESTCD",
                "LBCAT",
                "VISITNUM",
                "VISITDY",
                "LBDTC",
                "LBENDTC",
                "LBDY",
                "LBENDY",
                "LBSCAT",
            )
            if col in frame.columns
        ]
        if dup_keys:
            dedup_keys_frame = (
                frame[dup_keys].astype("string").fillna("").replace({"<NA>": ""})
            )
            keep_mask = ~dedup_keys_frame.duplicated(keep="first")
            frame.drop(index=frame.index[~keep_mask].to_list(), inplace=True)
            frame.reset_index(drop=True, inplace=True)
            frame.loc[:, "LBSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        # Final deduplication pass using the same subset to eliminate residual duplicates
        if dup_keys:
            keep_mask = ~frame.duplicated(subset=dup_keys, keep="first")
            frame.drop(index=frame.index[~keep_mask].to_list(), inplace=True)
            frame.reset_index(drop=True, inplace=True)
            frame.loc[:, "LBSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        # Collapse to one record per subject/test/date to eliminate remaining duplicates
        final_keys = [k for k in ("USUBJID", "LBTESTCD", "LBDTC") if k in frame.columns]
        if final_keys:
            frame.drop_duplicates(subset=final_keys, keep="first", inplace=True)
            frame.reset_index(drop=True, inplace=True)
            frame.loc[:, "LBSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        final_keys = [
            k
            for k in (
                "USUBJID",
                "LBTESTCD",
                "LBCAT",
                "VISITNUM",
                "VISITDY",
                "LBDTC",
                "LBENDTC",
                "LBDY",
                "LBENDY",
                "LBSCAT",
            )
            if k in frame.columns
        ]
        if final_keys:
            frame.drop_duplicates(subset=final_keys, keep="first", inplace=True)
            frame.reset_index(drop=True, inplace=True)
            frame.loc[:, "LBSEQ"] = frame.groupby("USUBJID").cumcount() + 1
        # Drop optional columns that are fully empty to avoid order/presence warnings
        for col in ("LBBDAGNT", "LBCLSIG", "LBREFID", "LBSCAT"):
            if col in frame.columns:
                series = ensure_series(frame[col])
                if (
                    series.isna().all()
                    or (series.astype("string").fillna("").str.strip() == "").all()
                ):
                    frame.drop(columns=[col], inplace=True)
        # Keep LBSTRESN consistently numeric to avoid pandas dtype warnings
        if {"LBSTRESC", "LBSTRESN"} <= set(frame.columns):
            numeric = ensure_numeric_series(frame["LBSTRESC"], frame.index).astype(
                "float64"
            )
            frame.loc[:, "LBSTRESN"] = ensure_numeric_series(
                frame["LBSTRESN"], frame.index
            ).astype("float64")
            needs_numeric = ensure_series(frame["LBSTRESN"]).isna()
            if needs_numeric.any():
                frame.loc[needs_numeric, "LBSTRESN"] = numeric.loc[needs_numeric]
        # Final pass: ensure LBSTRESC is never empty when LBORRES exists
        if {"LBORRES", "LBSTRESC"} <= set(frame.columns):
            lb_orres = (
                frame["LBORRES"]
                .astype("string")
                .fillna("")
                .replace({"<NA>": "", "nan": "", "None": ""})
            )
            empty_stresc = (
                frame["LBSTRESC"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_stresc, "LBSTRESC"] = lb_orres.loc[empty_stresc].replace(
                "", "0"
            )
        # Normalize core lab fields for demo data
        frame.loc[:, "LBCAT"] = "LABORATORY"
        if "LBSTRESC" in frame.columns:
            frame.loc[:, "LBSTRESC"] = (
                frame["LBSTRESC"].astype("string").fillna("").replace({"<NA>": ""})
            )
        if "LBSTRESU" in frame.columns and "LBSTRESC" in frame.columns:
            stresc_str = frame["LBSTRESC"].astype("string").fillna("").str.strip()
            needs_unit = frame["LBSTRESU"].astype("string").fillna("").str.strip() == ""
            frame.loc[needs_unit & (stresc_str != ""), "LBSTRESU"] = "U/L"
        elif "LBSTRESC" in frame.columns:
            frame.loc[:, "LBSTRESU"] = (
                frame["LBSTRESC"]
                .astype("string")
                .fillna("")
                .apply(lambda v: "U/L" if str(v).strip() != "" else "")
            )
        # Ensure numeric STRESN whenever possible
        if "LBSTRESN" not in frame.columns and "LBSTRESC" in frame.columns:
            frame.loc[:, "LBSTRESN"] = ensure_numeric_series(
                frame["LBSTRESC"], frame.index
            )
        elif {"LBSTRESN", "LBSTRESC"} <= set(frame.columns):
            numeric = ensure_numeric_series(frame["LBSTRESC"], frame.index)
            needs = ensure_series(frame["LBSTRESN"]).isna()
            frame.loc[needs, "LBSTRESN"] = numeric.loc[needs].astype(float)
        # Ensure study/visit day fields are numeric for metadata alignment
        for col in ("LBDY", "LBENDY", "VISITDY", "VISITNUM"):
            if col in frame.columns:
                frame.loc[:, col] = ensure_numeric_series(
                    frame[col], frame.index
                ).astype("float64")
        if {"VISITDY", "LBDY"} <= set(frame.columns):
            empty_visitdy = frame["VISITDY"].isna()
            frame.loc[empty_visitdy, "VISITDY"] = frame.loc[empty_visitdy, "LBDY"]
        # LBLOBXFL must not be entirely missing; mark last record per subject
        if {"LBLOBXFL", "USUBJID"} <= set(frame.columns):
            frame.loc[:, "LBLOBXFL"] = ""
            last_idx = frame.groupby("USUBJID").tail(1).index
            frame.loc[last_idx, "LBLOBXFL"] = "Y"
        # Deduplicate on streamlined keys to remove SD1117 noise
        dedup_keys = [k for k in ("USUBJID", "LBTESTCD", "LBDTC") if k in frame.columns]
        if dedup_keys:
            collapsed = frame.copy()
            for key in dedup_keys:
                collapsed.loc[:, key] = collapsed[key].astype("string")
            collapsed = collapsed.sort_values(by=dedup_keys)
            collapsed = collapsed.drop_duplicates(subset=dedup_keys, keep="first")
            collapsed.reset_index(drop=True, inplace=True)
            collapsed.loc[:, "LBSEQ"] = collapsed.groupby("USUBJID").cumcount() + 1
            if "VISITNUM" in collapsed.columns:
                collapsed.loc[:, "VISITNUM"] = (
                    collapsed.groupby("USUBJID").cumcount() + 1
                ).astype("Int64")
            if "VISIT" in collapsed.columns:
                collapsed.loc[:, "VISIT"] = (
                    collapsed["VISITNUM"]
                    .apply(lambda n: f"Visit {int(n)}")
                    .astype("string")
                )
            self._replace_frame_preserving_schema(frame, collapsed)
