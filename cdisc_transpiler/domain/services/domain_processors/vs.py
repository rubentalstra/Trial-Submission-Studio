"""Domain processor for Vital Signs (VS) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, NumericTransformer
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
        self._drop_placeholder_rows(frame)

        # Study day is a deterministic derivation from VSDTC and RFSTDTC/reference starts.
        if "VSDTC" in frame.columns:
            DateTransformer.compute_study_day(
                frame,
                "VSDTC",
                "VSDY",
                reference_starts=self.reference_starts,
                ref="RFSTDTC",
            )
        if "VSDY" in frame.columns:
            frame.loc[:, "VSDY"] = NumericTransformer.force_numeric(frame["VSDY"])

        # Keep standardized results aligned with original results without defaulting.
        if {"VSORRES", "VSSTRESC"}.issubset(frame.columns):
            orres = frame["VSORRES"].astype("string").fillna("").str.strip()
            stresc = frame["VSSTRESC"].astype("string").fillna("").str.strip()
            needs = (stresc == "") & (orres != "")
            if bool(needs.any()):
                frame.loc[needs, "VSSTRESC"] = orres.loc[needs]

        if {"VSORRESU", "VSSTRESU"}.issubset(frame.columns):
            oru = frame["VSORRESU"].astype("string").fillna("").str.strip()
            stu = frame["VSSTRESU"].astype("string").fillna("").str.strip()
            needs = (stu == "") & (oru != "")
            if bool(needs.any()):
                frame.loc[needs, "VSSTRESU"] = oru.loc[needs]

        # When results missing, clear units to avoid CT issues.
        if {"VSORRES", "VSORRESU"}.issubset(frame.columns):
            empty_orres = frame["VSORRES"].astype("string").fillna("").str.strip() == ""
            frame.loc[empty_orres, "VSORRESU"] = ""
        if {"VSSTRESC", "VSSTRESU"}.issubset(frame.columns):
            empty_stresc = (
                frame["VSSTRESC"].astype("string").fillna("").str.strip() == ""
            )
            frame.loc[empty_stresc, "VSSTRESU"] = ""

        # Controlled terminology normalization (blank invalid values; no defaults).
        ct_units = self._get_controlled_terminology(variable="VSORRESU")
        if ct_units:
            for col in ("VSORRESU", "VSSTRESU"):
                if col in frame.columns:
                    units = frame[col].astype("string").fillna("").str.strip()
                    normalized = units.apply(ct_units.normalize)
                    normalized = normalized.where(
                        normalized.isin(ct_units.submission_values), ""
                    )
                    frame.loc[:, col] = normalized

        ct_vstestcd = self._get_controlled_terminology(variable="VSTESTCD")
        if ct_vstestcd and "VSTESTCD" in frame.columns:
            raw = frame["VSTESTCD"].astype("string").fillna("").str.strip()
            canonical = raw.apply(ct_vstestcd.normalize)
            valid = canonical.isin(ct_vstestcd.submission_values)
            frame.loc[:, "VSTESTCD"] = canonical.where(valid, "")

        ct_vstest = self._get_controlled_terminology(variable="VSTEST")
        if ct_vstest and "VSTEST" in frame.columns:
            raw = frame["VSTEST"].astype("string").fillna("").str.strip()
            canonical = raw.apply(ct_vstest.normalize)
            valid = canonical.isin(ct_vstest.submission_values)
            frame.loc[:, "VSTEST"] = canonical.where(valid, "")

        if {"VSORRES", "VSSTRESN"}.issubset(frame.columns):
            numeric = pd.to_numeric(frame["VSORRES"], errors="coerce")
            frame.loc[:, "VSSTRESN"] = ensure_numeric_series(
                numeric, frame.index
            ).astype("float64")

        NumericTransformer.assign_sequence(frame, "VSSEQ", "USUBJID")

        if "VSLOBXFL" in frame.columns and {"USUBJID", "VSTESTCD"}.issubset(
            frame.columns
        ):
            frame.loc[:, "VSLOBXFL"] = (
                ensure_series(frame["VSLOBXFL"]).astype("string").fillna("")
            )
            group_cols = ["USUBJID", "VSTESTCD"]
            if "VSPOS" in frame.columns:
                group_cols.append("VSPOS")
            frame.loc[:, "VSLOBXFL"] = ""
            last_idx = frame.groupby(group_cols).tail(1).index
            frame.loc[last_idx, "VSLOBXFL"] = "Y"

        # Clear non-ISO collection times that trigger format errors.
        if "VSELTM" in frame.columns:
            raw = frame["VSELTM"].astype("string").fillna("").str.strip()
            # Accept HH:MM or HH:MM:SS; otherwise blank.
            valid = raw.str.match(r"^\d{2}:\d{2}(:\d{2})?$", na=False)
            frame.loc[:, "VSELTM"] = raw.where(valid, "")
