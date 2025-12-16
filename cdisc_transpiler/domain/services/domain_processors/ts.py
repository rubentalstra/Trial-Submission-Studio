"""Domain processor for Trial Summary (TS) domain."""

from __future__ import annotations

from typing import Any, cast

import pandas as pd

from .base import BaseDomainProcessor
from ....terminology_module import get_controlled_terminology
from ....pandas_utils import ensure_series


class TSProcessor(BaseDomainProcessor):
    """Trial Summary domain processor.

    Handles domain-specific processing for the TS domain.
    """

    def process(self, frame: pd.DataFrame) -> None:
        """Process TS domain DataFrame.

        Args:
            frame: Domain DataFrame to process in-place
        """
        # Drop placeholder rows
        self._drop_placeholder_rows(frame)

        study_series = ensure_series(
            frame.get("STUDYID", pd.Series(["STUDY"])), index=frame.index
        )
        base_study = str(study_series.iloc[0]) if len(study_series) > 0 else "STUDY"
        ct_parmcd = get_controlled_terminology(variable="TSPARMCD")
        ct_parm = get_controlled_terminology(variable="TSPARM")

        def _parm_name(code: str) -> str:
            if not ct_parmcd or not ct_parm:
                return code
            nci = ct_parmcd.lookup_code(code)
            if not nci:
                return code
            for name, name_nci in ct_parm.nci_codes.items():
                if name_nci == nci:
                    return name
            return code

        def _row(
            code: str,
            val: str,
            *,
            valcd: str = "",
            tsvcdref_val: str = "",
            tsvcdver_val: str | None = None,
        ) -> dict[str, Any]:
            # Only provide a version when a reference dictionary is specified
            ref = tsvcdref_val
            if ref:
                ver = "2025-09-26" if tsvcdver_val is None else tsvcdver_val
            else:
                ver = ""
            return {
                "TSPARMCD": code,
                "TSPARM": _parm_name(code),
                "TSVAL": val,
                "TSVALCD": valcd,
                "TSVCDREF": ref,
                "TSVCDVER": ver,
                "TSGRPID": "",
                "TSVALNF": "",
                "STUDYID": base_study,
                "DOMAIN": "TS",
            }

        params = pd.DataFrame(
            [
                _row("SSTDTC", "2023-08-01"),
                _row("SENDTC", "2024-12-31"),
                _row("STYPE", "INTERVENTIONAL"),
                _row("TPHASE", "PHASE II TRIAL", valcd="C15601"),
                _row("TBLIND", "DOUBLE BLIND", valcd="C15228"),
                _row("RANDOM", "Y", valcd="C49488"),
                _row("INTMODEL", "PARALLEL", valcd="C82639"),
                _row("INTTYPE", "DRUG", valcd="C1909"),
                _row("TCNTRL", "NONE", valcd="C41132"),
                _row("TINDTP", "DIAGNOSIS", valcd="C49653"),
                _row("TTYPE", "BIO-AVAILABILITY", valcd="C49664"),
                _row("SEXPOP", "BOTH", valcd="C49636"),
                _row("AGEMIN", "P18Y"),
                _row("AGEMAX", "P65Y"),
                _row("PLANSUB", "3"),
                _row("NARMS", "1"),
                _row("ACTSUB", "3"),
                _row("NCOHORT", "1"),
                _row("ADDON", "N", valcd="C49487"),
                _row("ADAPT", "N", valcd="C49487"),
                _row("DCUTDTC", "2024-12-31"),
                _row("DCUTDESC", "FINAL ANALYSIS"),
                _row("PDPSTIND", "N", valcd="C49487"),
                _row("PDSTIND", "N", valcd="C49487"),
                _row("PIPIND", "N", valcd="C49487"),
                _row("RDIND", "N", valcd="C49487"),
                _row("ONGOSIND", "N", valcd="C49487"),
                _row("SDTIGVER", "3.4"),
                _row("SDTMVER", "3.4"),
                _row("THERAREA", "GENERAL"),
                _row("REGID", "NCT00000000"),
                _row("SPONSOR", "GDISC"),
                _row("TITLE", "DEMO GDISC STUDY"),
                _row("STOPRULE", "NONE"),
                _row("OBJPRIM", "ASSESS SAFETY"),
                _row("OBJSEC", "NONE"),
                _row("OUTMSPRI", "EFFICACY"),
                _row("HLTSUBJI", "0"),
                _row("EXTTIND", "N", valcd="C49487"),
                _row("LENGTH", "P24M"),
                _row(
                    "TRT",
                    "IBUPROFEN",
                    valcd="WK2XYI10QM",
                    tsvcdref_val="UNII",
                    tsvcdver_val="2025-09-26",
                ),
                _row(
                    "PCLAS",
                    "Nonsteroidal Anti-inflammatory Drug",
                    valcd="N0000175722",
                    tsvcdref_val="MED-RT",
                    tsvcdver_val="2025-09-26",
                ),
                _row(
                    "FCNTRY",
                    "USA",
                    valcd="",
                    tsvcdref_val="",
                    tsvcdver_val="",
                ),
            ]
        )
        # Keep TSVALCD consistent for identical TSVAL values to satisfy SD1278
        value_code_map: dict[str, tuple[str, str]] = {}
        for _, row in params.iterrows():
            val = str(row.get("TSVAL", "")).strip()
            code = str(row.get("TSVALCD", "")).strip()
            ref = str(row.get("TSVCDREF", "")).strip()
            if val and code:
                value_code_map.setdefault(val, (code, ref))
        missing_code = params["TSVALCD"].astype("string").str.strip() == ""
        if missing_code.any():
            tsvalcd_loc = cast(int, params.columns.get_loc("TSVALCD"))
            tsvcdref_loc = cast(int, params.columns.get_loc("TSVCDREF"))
            for pos in range(len(params)):
                if not bool(missing_code.iloc[pos]):
                    continue
                row = params.iloc[pos]
                val = str(row.get("TSVAL", "")).strip()
                if not val or val not in value_code_map:
                    continue
                code, ref = value_code_map[val]
                params.iat[pos, tsvalcd_loc] = code
                if not str(row.get("TSVCDREF", "")).strip() and ref:
                    params.iat[pos, tsvcdref_loc] = ref
        params["TSSEQ"] = range(1, len(params) + 1)
        frame.drop(index=frame.index.tolist(), inplace=True)
        frame.drop(columns=list(frame.columns), inplace=True)
        for col in params.columns:
            frame[col] = params[col].values
