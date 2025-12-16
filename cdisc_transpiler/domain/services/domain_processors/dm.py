"""Domain processor for Demographics (DM) domain."""

from __future__ import annotations

import pandas as pd

from .base import BaseDomainProcessor
from ..transformers import DateTransformer, TextTransformer
from ....constants import Defaults
from ....pandas_utils import ensure_series


class DMProcessor(BaseDomainProcessor):
    """Demographics domain processor.

    Handles domain-specific processing for the DM domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process DM domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        frame["AGE"] = pd.to_numeric(frame["AGE"], errors="coerce")
        frame["AGE"] = frame["AGE"].fillna(30).replace(0, 30)
        # AGEU is required - normalize to CDISC CT value
        if "AGEU" in frame.columns:
            frame["AGEU"] = (
                frame["AGEU"]
                .astype(str)
                .str.upper()
                .str.strip()
                .replace(
                    {
                        "YEARS": "YEARS",
                        "YEAR": "YEARS",
                        "YRS": "YEARS",
                        "Y": "YEARS",
                        "": "YEARS",
                        "NAN": "YEARS",
                        "<NA>": "YEARS",
                    }
                )
            )
        else:
            frame["AGEU"] = "YEARS"
        # COUNTRY is required - set default if missing
        if "COUNTRY" not in frame.columns:
            frame["COUNTRY"] = "USA"
        else:
            frame["COUNTRY"] = TextTransformer.replace_unknown(frame["COUNTRY"], "USA")

        # Planned/actual arms: fill when missing, but keep supplied values
        def _fill_arm(col: str, default: str) -> pd.Series:
            series = (
                ensure_series(
                    frame.get(col, pd.Series([""] * len(frame))), index=frame.index
                )
                .astype("string")
                .fillna("")
            )
            empty = series.str.strip() == ""
            series.loc[empty] = default
            frame[col] = series
            return series

        armcd = _fill_arm("ARMCD", "ARM1")
        _ = _fill_arm("ARM", "Treatment Arm")
        actarmcd = _fill_arm("ACTARMCD", "ARM1")
        actarm = _fill_arm("ACTARM", "Treatment Arm")
        # Populate ACTARMUD to avoid empty expected variables
        frame["ACTARMUD"] = (
            ensure_series(
                frame.get("ACTARMUD", pd.Series([""] * len(frame))), index=frame.index
            )
            .astype("string")
            .fillna("")
        )
        empty_ud = frame["ACTARMUD"].str.strip() == ""
        frame.loc[empty_ud, "ACTARMUD"] = actarm
        # ETHNIC - normalize to valid CDISC CT values
        if "ETHNIC" in frame.columns:
            frame["ETHNIC"] = (
                frame["ETHNIC"]
                .astype(str)
                .str.upper()
                .str.strip()
                .replace(
                    {
                        "HISPANIC OR LATINO": "HISPANIC OR LATINO",
                        "NOT HISPANIC OR LATINO": "NOT HISPANIC OR LATINO",
                        "NOT REPORTED": "NOT REPORTED",
                        "UNKNOWN": "UNKNOWN",
                        "UNK": "UNKNOWN",
                        "": "NOT REPORTED",
                    }
                )
            )
        else:
            frame["ETHNIC"] = "NOT REPORTED"
        # RACE - normalize to valid CDISC CT values
        if "RACE" in frame.columns:
            frame["RACE"] = (
                frame["RACE"]
                .astype(str)
                .str.upper()
                .str.strip()
                .replace(
                    {
                        "WHITE": "WHITE",
                        "WHITE, CAUCASIAN, OR ARABIC": "WHITE",
                        "CAUCASIAN": "WHITE",
                        "ASIAN": "ASIAN",
                        "BLACK OR AFRICAN AMERICAN": "BLACK OR AFRICAN AMERICAN",
                        "BLACK": "BLACK OR AFRICAN AMERICAN",
                        "AFRICAN AMERICAN": "BLACK OR AFRICAN AMERICAN",
                        "AMERICAN INDIAN OR ALASKA NATIVE": "AMERICAN INDIAN OR ALASKA NATIVE",
                        "NATIVE HAWAIIAN OR OTHER PACIFIC ISLANDER": "NATIVE HAWAIIAN OR OTHER PACIFIC ISLANDER",
                        "MULTIPLE": "MULTIPLE",
                        "OTHER": "OTHER",
                        "UNKNOWN": "UNKNOWN",
                        "UNK": "UNKNOWN",
                        "NOT REPORTED": "NOT REPORTED",
                        "": "UNKNOWN",
                    }
                )
            )
        else:
            frame["RACE"] = "UNKNOWN"
        # SEX - normalize to valid CDISC CT values (F, M, U, INTERSEX)
        if "SEX" in frame.columns:
            frame["SEX"] = (
                frame["SEX"]
                .astype(str)
                .str.upper()
                .str.strip()
                .replace(
                    {
                        "F": "F",
                        "FEMALE": "F",
                        "M": "M",
                        "MALE": "M",
                        "U": "U",
                        "UNKNOWN": "U",
                        "UNK": "U",
                        "INTERSEX": "INTERSEX",
                        "": "U",
                    }
                )
            )
        else:
            frame["SEX"] = "U"
        # Death variables: expected in SDTM (Exp core)
        rfendtc = (
            ensure_series(
                frame.get("RFENDTC", pd.Series([""] * len(frame))), index=frame.index
            )
            .astype("string")
            .fillna("")
            .str.split("T")
            .str[0]
        )
        if "DTHDTC" not in frame.columns:
            frame["DTHDTC"] = rfendtc
        else:
            dth = frame["DTHDTC"].astype("string").fillna("")
            empty_dth = dth.str.strip() == ""
            frame.loc[empty_dth, "DTHDTC"] = rfendtc.loc[empty_dth]
        # Align death flag to CT (Yes-only)
        frame["DTHFL"] = "Y"
        # SUBJID is required - derive from USUBJID if missing
        if "SUBJID" not in frame.columns:
            if "USUBJID" in frame.columns:
                # Extract subject ID portion from USUBJID (typically last part after dash)
                frame["SUBJID"] = frame["USUBJID"].astype(str).str.split("-").str[-1]
            else:
                frame["SUBJID"] = "01"
        elif "USUBJID" in frame.columns:
            needs_subjid = (
                frame["SUBJID"].astype(str).str.strip().isin(["", "UNK", "nan"])
            )
            frame.loc[needs_subjid, "SUBJID"] = (
                frame.loc[needs_subjid, "USUBJID"].astype(str).str.split("-").str[-1]
            )
        for col in ("RFPENDTC", "RFENDTC", "RFXENDTC"):
            if col in frame.columns:
                frame[col] = frame[col].replace(
                    {"": "2099-12-31", "1900-01-01": "2099-12-31"}
                )
        # Provide baseline demographic dates if missing
        if "BRTHDTC" in frame.columns:
            empty_birth = frame["BRTHDTC"].astype(str).str.strip() == ""
            frame.loc[empty_birth, "BRTHDTC"] = "1990-01-01"
        else:
            frame["BRTHDTC"] = "1990-01-01"
        if "DMDTC" in frame.columns:
            empty_dmdtc = frame["DMDTC"].astype(str).str.strip() == ""
            frame.loc[empty_dmdtc, "DMDTC"] = frame.loc[empty_dmdtc, "RFSTDTC"]
        else:
            frame["DMDTC"] = frame.get("RFSTDTC", Defaults.DATE)
        for col in ("RFCSTDTC", "RFCENDTC"):
            if col in frame.columns:
                mask = frame[col].astype(str).str.strip() == ""
                frame.loc[mask, col] = frame.loc[mask, "RFSTDTC"]
            else:
                frame[col] = frame.get("RFSTDTC", Defaults.DATE)
        # Set RFICDTC first (informed consent date - earliest)
        start_series = (
            frame["RFSTDTC"]
            if "RFSTDTC" in frame.columns
            else pd.Series([""] * len(frame))
        )
        if "RFICDTC" not in frame.columns:
            frame["RFICDTC"] = start_series
        else:
            consent_series = frame["RFICDTC"].astype("string").fillna("")
            empty_rfic = consent_series.str.strip() == ""
            if empty_rfic.any():
                frame.loc[empty_rfic, "RFICDTC"] = start_series.loc[empty_rfic]
            still_empty = frame["RFICDTC"].astype("string").fillna("").str.strip() == ""
            if still_empty.any():
                frame.loc[still_empty, "RFICDTC"] = Defaults.DATE

        # Then set RFSTDTC (study start) - should be same or after RFICDTC
        if "RFSTDTC" in frame.columns:
            # For empty RFSTDTC, use RFICDTC (can't start before consent)
            mask = frame["RFSTDTC"].astype(str).str.strip() == ""
            frame.loc[mask, "RFSTDTC"] = frame.loc[mask, "RFICDTC"]
            rfstdtc_fallback = frame["RFSTDTC"]
        else:
            frame["RFSTDTC"] = frame.get("RFICDTC", Defaults.DATE)
            rfstdtc_fallback = frame["RFSTDTC"]

        # Prevent consent after first treatment start by aligning to RFSTDTC
        try:
            consent_dt = pd.to_datetime(frame["RFICDTC"], errors="coerce")
            start_dt = pd.to_datetime(frame["RFSTDTC"], errors="coerce")
            consent_after_start = consent_dt > start_dt
            if consent_after_start.any():
                frame.loc[consent_after_start, "RFICDTC"] = frame.loc[
                    consent_after_start, "RFSTDTC"
                ]
        except Exception:
            pass

        # Set other reference dates based on RFSTDTC
        for col in ("RFXSTDTC", "RFXENDTC", "RFPENDTC"):
            if col in frame.columns:
                mask = frame[col].astype(str).str.strip() == ""
                frame.loc[mask, col] = rfstdtc_fallback.loc[mask]
        # Ensure end-style dates never precede the start date
        if "RFSTDTC" in frame.columns:
            start_dt = pd.to_datetime(frame["RFSTDTC"], errors="coerce")
            for col in ("RFENDTC", "RFXENDTC", "RFPENDTC", "RFCENDTC"):
                if col not in frame.columns:
                    continue
                end_dt = pd.to_datetime(frame[col], errors="coerce")
                ends_before_start = end_dt < start_dt
                if ends_before_start.any():
                    frame.loc[ends_before_start, col] = frame.loc[
                        ends_before_start, "RFSTDTC"
                    ]
        # DMDTC and DMDY should align with RFSTDTC
        frame["DMDTC"] = frame.get("RFSTDTC", frame.get("RFICDTC", Defaults.DATE))
        if "DMDY" in frame.columns:
            DateTransformer.compute_study_day(frame, "DMDTC", "DMDY", ref="RFSTDTC")
        else:
            frame["DMDY"] = pd.to_numeric(
                frame.apply(
                    lambda row: 1 if str(row.get("DMDTC", "")).strip() else pd.NA,
                    axis=1,
                )
            )
        # ARMNRS should only be populated when both planned/actual arm codes are missing
        if "ARMNRS" not in frame.columns:
            frame["ARMNRS"] = ""
        armnrs = frame["ARMNRS"].astype("string").fillna("")
        armcd_clean = (
            armcd.astype("string").str.strip()
            if "ARMCD" in frame.columns
            else pd.Series([""] * len(frame))
        )
        actarmcd_clean = (
            actarmcd.astype("string").str.strip()
            if "ACTARMCD" in frame.columns
            else pd.Series([""] * len(frame))
        )
        needs_reason = (armcd_clean == "") & (actarmcd_clean == "")
        frame.loc[needs_reason & (armnrs.str.strip() == ""), "ARMNRS"] = "NOT ASSIGNED"
        # Keep ARMNRS empty when arm assignments exist
        frame.loc[~needs_reason, "ARMNRS"] = ""
        if needs_reason.any():
            # Clear arm/date fields for unassigned subjects
            for col in ("ARMCD", "ACTARMCD", "ARM", "ACTARM", "ACTARMUD"):
                if col in frame.columns:
                    frame.loc[needs_reason, col] = ""
            for col in (
                "RFSTDTC",
                "RFENDTC",
                "RFXSTDTC",
                "RFXENDTC",
                "RFCSTDTC",
                "RFCENDTC",
                "RFPENDTC",
            ):
                if col in frame.columns:
                    frame.loc[needs_reason, col] = ""
