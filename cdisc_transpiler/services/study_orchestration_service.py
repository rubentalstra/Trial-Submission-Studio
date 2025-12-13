"""Study Orchestration Service - High-level study processing workflow.

This service orchestrates the entire study processing workflow, including
domain-specific transformations, trial design synthesis, and relationship
record generation.

SDTM Reference:
    SDTMIG v3.4 Section 6 describes the Findings class domains including:
    - VS (Vital Signs): Blood pressure, heart rate, temperature, etc.
    - LB (Laboratory Test Results): Hematology, chemistry, urinalysis
    
    Findings domains use the normalized structure with --TESTCD, --TEST,
    --ORRES, --ORRESU for each measurement. Source data often comes in
    wide format and must be reshaped to the vertical SDTM structure.
    
    Section 6.4 describes RELREC (Related Records) for linking observations
    across domains (e.g., linking AE records to DS disposition events).
"""

from __future__ import annotations

import re
from pathlib import Path

import pandas as pd

from ..domains_module import SDTMVariable, get_domain
from ..mapping_module import ColumnMapping, build_config
from ..sas_module import generate_sas_program, write_sas_file
from ..xpt_module import write_xpt_file


# SDTM Controlled Terminology for common VS tests
VS_TEST_LABELS = {
    "HR": "Heart Rate",
    "SYSBP": "Systolic Blood Pressure",
    "DIABP": "Diastolic Blood Pressure",
    "TEMP": "Temperature",
    "WEIGHT": "Weight",
    "HEIGHT": "Height",
    "BMI": "Body Mass Index",
}

# Default units per CDISC Controlled Terminology
VS_UNIT_DEFAULTS = {
    "HR": "beats/min",
    "SYSBP": "mmHg",
    "DIABP": "mmHg",
    "TEMP": "C",
    "WEIGHT": "kg",
    "HEIGHT": "cm",
    "BMI": "kg/m2",
}

# Aliases mapping source test codes to standard CDISC CT
VS_TEST_ALIASES = {
    "PLS": "HR",  # Pulse -> Heart Rate
    "HR": "HR",
    "SYSBP": "SYSBP",
    "DIABP": "DIABP",
    "TEMP": "TEMP",
    "WEIGHT": "WEIGHT",
    "HEIGHT": "HEIGHT",
    "BMI": "BMI",
}

# SDTM Controlled Terminology for common LB tests
LB_TEST_LABELS = {
    "CHOL": "Cholesterol",
    "AST": "Aspartate Aminotransferase",
    "ALT": "Alanine Aminotransferase",
    "GLUC": "Glucose",
    "HGB": "Hemoglobin",
    "HCT": "Hematocrit",
    "RBC": "Erythrocytes",
    "WBC": "Leukocytes",
    "PLAT": "Platelets",
}


class StudyOrchestrationService:
    """Service for orchestrating study processing workflows.

    This service contains domain-specific logic for data transformations,
    trial design synthesis, and relationship record generation.
    
    The service handles:
    - Reshaping wide-format Findings data to SDTM vertical structure
    - Building RELREC relationship records between domains
    - Synthesizing RELREC domain from processed domain data
    """

    def reshape_vs_to_long(self, frame: pd.DataFrame, study_id: str) -> pd.DataFrame:
        """Convert source VS wide data to SDTM-compliant long rows using dynamic tests.

        Transforms wide-format vital signs data (one column per test) to the
        vertical SDTM structure (one row per test per subject per visit).

        SDTM Reference:
            SDTMIG v3.4 Section 6.3.7 defines the VS domain structure with
            required variables VSTESTCD, VSTEST, VSORRES, VSORRESU.

        Args:
            frame: Input dataframe with wide-format vital signs data
            study_id: Study identifier

        Returns:
            Long-format dataframe with SDTM VS structure
        """
        df = frame.copy()
        
        # Common source column name variations
        rename_map = {
            "Subject Id": "USUBJID",
            "SubjectId": "USUBJID",
            "Event name": "VISIT",
            "Event Name": "VISIT",
            "EventName": "VISIT",
            "Event sequence number": "VISITNUM",
            "Event Sequence Number": "VISITNUM",
            "EventSeq": "VISITNUM",
            "Event date": "VSDTC",
            "Event Date": "VSDTC",
            "EventDate": "VSDTC",
        }
        df = df.rename(columns=rename_map)

        # Normalize visit identifiers
        if "VISITNUM" in df.columns:
            df["VISITNUM"] = pd.to_numeric(df["VISITNUM"], errors="coerce")
        if "VISIT" not in df.columns and "VISITNUM" in df.columns:
            df["VISIT"] = df["VISITNUM"].apply(
                lambda n: f"Visit {int(n)}" if pd.notna(n) else ""
            )

        # Discover tests dynamically from ORRES_* columns
        tests = []
        for col in df.columns:
            m = re.match(r"^ORRES_([A-Za-z0-9]+)$", col, re.I)
            if m:
                testcd = m.group(1).upper()
                if testcd.endswith("CD"):
                    continue
                tests.append(testcd)
        tests = sorted(set(tests))

        if not tests:
            return pd.DataFrame()

        records: list[dict] = []
        for _, row in df.iterrows():
            usubjid = str(row.get("USUBJID", "") or "").strip()
            if not usubjid:
                continue
            visitnum = row.get("VISITNUM", pd.NA)
            visit = str(row.get("VISIT", "") or "").strip()
            if not visit and pd.notna(visitnum):
                visit = (
                    f"Visit {int(visitnum)}" if float(visitnum).is_integer() else ""
                )

            vsdtc = row.get("VSDTC", "")
            status_cd = str(row.get("VSPERFCD", "") or "").strip().upper()
            reason = str(row.get("VSREASND", "") or "").strip()

            for testcd_raw in tests:
                std_testcd = VS_TEST_ALIASES.get(testcd_raw, None)
                if not std_testcd:
                    continue
                value = row.get(f"ORRES_{testcd_raw}", pd.NA)
                if status_cd != "N" and pd.isna(value):
                    continue
                unit_val = row.get(f"ORRESU_{testcd_raw}", "")
                pos_val = row.get(f"POS_{testcd_raw}", "")
                label_val = row.get(f"TEST_{testcd_raw}", "")
                stat_val = "NOT DONE" if status_cd == "N" else ""
                if stat_val != "NOT DONE" and (
                    unit_val is None or str(unit_val).strip() == ""
                ):
                    unit_val = VS_UNIT_DEFAULTS.get(std_testcd, "")
                records.append(
                    {
                        "STUDYID": study_id,
                        "DOMAIN": "VS",
                        "USUBJID": usubjid,
                        "VSTESTCD": std_testcd[:8],
                        "VSTEST": str(
                            VS_TEST_LABELS.get(std_testcd, label_val or std_testcd)
                        ),
                        "VSORRES": "" if stat_val else value,
                        "VSORRESU": "" if stat_val else unit_val,
                        "VSSTAT": stat_val,
                        "VSREASND": reason if stat_val else "",
                        "VISITNUM": visitnum,
                        "VISIT": visit,
                        "VSDTC": vsdtc,
                        "VSPOS": pos_val,
                    }
                )

        if not records:
            return pd.DataFrame()
        return pd.DataFrame(records)

    def reshape_lb_to_long(self, frame: pd.DataFrame, study_id: str) -> pd.DataFrame:
        """Convert wide LB source data to long-form SDTM rows.

        Transforms wide-format laboratory data (one column per test) to the
        vertical SDTM structure (one row per test per subject per timepoint).

        SDTM Reference:
            SDTMIG v3.4 Section 6.3.3 defines the LB domain structure with
            required variables LBTESTCD, LBTEST, LBORRES, LBORRESU,
            LBORNRLO, LBORNRHI for normal ranges.

        Args:
            frame: Input dataframe with wide-format laboratory data
            study_id: Study identifier

        Returns:
            Long-format dataframe with SDTM LB structure
        """
        df = frame.copy()
        
        # Common source column name variations
        rename_map = {
            "Subject Id": "USUBJID",
            "SubjectId": "USUBJID",
            "Event date": "LBDTC",
            "Event Date": "LBDTC",
            "EventDate": "LBDTC",
            "Date of blood sample": "LBDTC",
            "Date ofstool sample": "LBDTC",
            "Date ofurine sample": "LBDTC",
            "Date of pregnancy test": "LBDTC",
        }
        df = df.rename(columns=rename_map)

        usubjid_col = "USUBJID" if "USUBJID" in df.columns else None
        dtc_candidates = [
            c
            for c in df.columns
            if str(c).upper() == "LBDTC" or str(c).upper().endswith("DAT")
        ]

        # Collect test-specific columns by prefix
        test_defs: dict[str, dict[str, str]] = {}
        for col in df.columns:
            m = re.match(
                r"^([A-Za-z0-9]+)\s+result or finding in original units$", col, re.I
            )
            if m:
                test = m.group(1).upper()
                test_defs.setdefault(test, {})["orres"] = col
                continue
            m = re.match(r"^TEST_([A-Za-z0-9]+)$", col, re.I)
            if m:
                test = m.group(1).upper()
                test_defs.setdefault(test, {})["label"] = col
                continue
            m = re.match(r"^ORRES_([A-Za-z0-9]+)$", col, re.I)
            if m:
                test = m.group(1).upper()
                if test.endswith("CD"):
                    continue
                test_defs.setdefault(test, {})["orres"] = col
                continue
            m = re.match(r"^([A-Za-z0-9]+)ORRES$", col, re.I)
            if m:
                test = m.group(1).upper()
                if test.endswith("CD"):
                    continue
                test_defs.setdefault(test, {})["orres"] = col
                continue
            m = re.match(r"^([A-Za-z0-9]+)\s+unit(?:\s*-.*)?$", col, re.I)
            if m:
                test = m.group(1).upper()
                test_defs.setdefault(test, {})["unit"] = col
                continue
            m = re.match(r"^ORRESU_([A-Za-z0-9]+)$", col, re.I)
            if m:
                test = m.group(1).upper()
                if test.endswith("CD"):
                    continue
                test_defs.setdefault(test, {})["unit"] = col
                continue
            m = re.match(r"^([A-Za-z0-9]+)\s+range \(lower limit\)$", col, re.I)
            if m:
                test = m.group(1).upper()
                test_defs.setdefault(test, {})["nrlo"] = col
                continue
            m = re.match(r"^ORNR_([A-Za-z0-9]+)_Lower$", col, re.I)
            if m:
                test = m.group(1).upper()
                test_defs.setdefault(test, {})["nrlo"] = col
                continue
            m = re.match(r"^([A-Za-z0-9]+)\s+range \(upper limit\)$", col, re.I)
            if m:
                test = m.group(1).upper()
                test_defs.setdefault(test, {})["nrhi"] = col
                continue
            m = re.match(r"^ORNR_([A-Za-z0-9]+)_Upper$", col, re.I)
            if m:
                test = m.group(1).upper()
                test_defs.setdefault(test, {})["nrhi"] = col
                continue

        if not test_defs:
            return df

        def _pick_first(val: object) -> object:
            if isinstance(val, pd.Series):
                for v in val:
                    if pd.isna(v):
                        continue
                    if str(v).strip():
                        return v
                return None
            return val

        records: list[dict] = []
        for _, row in df.iterrows():
            usubjid = (
                str(row.get(usubjid_col, "") or "").strip() if usubjid_col else ""
            )
            if not usubjid or usubjid.lower() == "subjectid":
                continue
            lbdtc = ""
            for cand in dtc_candidates:
                val = _pick_first(row.get(cand, ""))
                if val is None:
                    continue
                sval = str(val).strip()
                if sval:
                    lbdtc = sval
                    break

            for testcd, cols in test_defs.items():
                # Map/normalize test code; skip unsupported tests
                norm_testcd = testcd.upper()
                if norm_testcd == "GLUCU":
                    norm_testcd = "GLUC"
                if norm_testcd not in LB_TEST_LABELS:
                    continue
                orres_col = cols.get("orres")
                if not orres_col:
                    continue
                value = _pick_first(row.get(orres_col, ""))
                if value is None:
                    continue
                value_str = str(value).strip()
                if not value_str or value_str.upper().startswith("ORRES"):
                    continue
                unit_val = (
                    _pick_first(row.get(cols.get("unit"), ""))
                    if cols.get("unit")
                    else ""
                )
                nrlo_val = (
                    _pick_first(row.get(cols.get("nrlo"), ""))
                    if cols.get("nrlo")
                    else ""
                )
                nrhi_val = (
                    _pick_first(row.get(cols.get("nrhi"), ""))
                    if cols.get("nrhi")
                    else ""
                )
                label_val = (
                    _pick_first(row.get(cols.get("label"), ""))
                    if cols.get("label")
                    else ""
                )
                records.append(
                    {
                        "STUDYID": study_id,
                        "DOMAIN": "LB",
                        "USUBJID": usubjid,
                        "LBTESTCD": norm_testcd[:8],
                        "LBTEST": LB_TEST_LABELS.get(
                            norm_testcd, label_val or norm_testcd
                        ),
                        "LBORRES": value_str,
                        "LBORRESU": unit_val,
                        "LBORNRLO": nrlo_val,
                        "LBORNRHI": nrhi_val,
                        "LBDTC": lbdtc,
                    }
                )

        if not records:
            return pd.DataFrame()
        return pd.DataFrame(records)

    def build_relrec_records(
        self, domain_results: list[dict], study_id: str
    ) -> pd.DataFrame:
        """Build RELREC rows linking AE/EX records back to DS by subject.

        Args:
            domain_results: List of domain processing results
            study_id: Study identifier

        Returns:
            DataFrame with RELREC records
        """

        def _get_domain_df(code: str) -> pd.DataFrame | None:
            for entry in domain_results:
                if entry.get("domain_code", "").upper() == code.upper():
                    df = entry.get("domain_dataframe")
                    if isinstance(df, pd.DataFrame) and not df.empty:
                        return df
            return None

        ae_df = _get_domain_df("AE")
        ds_df = _get_domain_df("DS")
        ex_df = _get_domain_df("EX")

        records: list[dict] = []

        def _seq_map(df: pd.DataFrame, seq_col: str) -> dict:
            if seq_col not in df.columns:
                return {}
            numeric = pd.to_numeric(df[seq_col], errors="coerce")
            return (
                pd.DataFrame({"USUBJID": df["USUBJID"], seq_col: numeric})
                .dropna(subset=["USUBJID", seq_col])
                .groupby("USUBJID")[seq_col]
                .min()
                .to_dict()
            )

        ds_seq_map = _seq_map(ds_df, "DSSEQ") if ds_df is not None else {}

        def _stringify(val: object, fallback_index: int) -> str:
            if pd.isna(val):
                return str(fallback_index)
            try:
                numeric = pd.to_numeric(val)
                if pd.isna(numeric):
                    return str(val)
                if float(numeric).is_integer():
                    return str(int(numeric))
                return str(numeric)
            except Exception:
                return str(val)

        def _add_pair(
            rdomain: str,
            usubjid: str,
            idvar: str,
            idvarval: str,
            relid: str,
            reltype: str | None = None,
        ) -> None:
            records.append(
                {
                    "STUDYID": study_id,
                    "RDOMAIN": rdomain,
                    "USUBJID": usubjid,
                    "IDVAR": idvar,
                    "IDVARVAL": idvarval,
                    "RELTYPE": reltype or "",
                    "RELID": relid,
                }
            )

        if ae_df is not None and ds_seq_map:
            for idx, row in ae_df.iterrows():
                usubjid = str(row.get("USUBJID", "") or "").strip()
                if not usubjid:
                    continue
                aeseq = _stringify(row.get("AESEQ"), idx + 1)
                relid = f"AE_DS_{usubjid}_{aeseq}"
                _add_pair("AE", usubjid, "AESEQ", aeseq, relid, None)
                ds_seq = ds_seq_map.get(usubjid)
                if ds_seq is not None:
                    _add_pair("DS", usubjid, "DSSEQ", _stringify(ds_seq, 1), relid, None)

        if ex_df is not None and ds_seq_map:
            for idx, row in ex_df.iterrows():
                usubjid = str(row.get("USUBJID", "") or "").strip()
                if not usubjid:
                    continue
                exseq = _stringify(row.get("EXSEQ"), idx + 1)
                relid = f"EX_DS_{usubjid}_{exseq}"
                _add_pair("EX", usubjid, "EXSEQ", exseq, relid, None)
                ds_seq = ds_seq_map.get(usubjid)
                if ds_seq is not None:
                    _add_pair("DS", usubjid, "DSSEQ", _stringify(ds_seq, 1), relid, None)

        if not records and ds_df is not None:
            # Fallback: relate first two DS records per subject if nothing else available
            ds_seq_map = _seq_map(ds_df, "DSSEQ")
            for usubjid, ds_seq in ds_seq_map.items():
                relid = f"DS_ONLY_{usubjid}"
                _add_pair("DS", str(usubjid), "DSSEQ", _stringify(ds_seq, 1), relid, None)

        return pd.DataFrame(records)

    def synthesize_relrec(
        self,
        study_id: str,
        output_format: str,
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
        generate_sas: bool,
        domain_results: list[dict],
    ) -> dict:
        """Create a populated RELREC dataset based on existing domain data.

        Args:
            study_id: Study identifier
            output_format: Output format ("xpt", "xml", or "both")
            xpt_dir: Directory for XPT files
            xml_dir: Directory for Dataset-XML files
            sas_dir: Directory for SAS programs
            generate_sas: Whether to generate SAS programs
            domain_results: List of domain processing results

        Returns:
            Dictionary with RELREC processing results
        """
        from ..xml.dataset import write_dataset_xml
        from ..xpt_module.builder import build_domain_dataframe

        domain = get_domain("RELREC")

        relrec_records = self.build_relrec_records(domain_results, study_id)
        if relrec_records.empty:
            frame = pd.DataFrame(
                {
                    var.name: pd.Series(dtype=var.pandas_dtype())
                    for var in domain.variables
                }
            )
        else:
            frame = relrec_records

        mappings = [
            ColumnMapping(
                source_column=col,
                target_variable=col,
                transformation=None,
                confidence_score=1.0,
            )
            for col in frame.columns
        ]
        config = build_config("RELREC", mappings)
        config.study_id = study_id

        domain_dataframe = build_domain_dataframe(frame, config, lenient=True)

        result = {
            "domain_code": "RELREC",
            "records": len(domain_dataframe),
            "domain_dataframe": domain_dataframe,
            "config": config,
            "xpt_path": None,
            "xml_path": None,
            "sas_path": None,
            "has_no_data": relrec_records.empty,
        }

        base_filename = domain.resolved_dataset_name()
        disk_name = base_filename.lower()

        # Generate XPT file
        if xpt_dir and output_format in ("xpt", "both"):
            xpt_filename = f"{disk_name}.xpt"
            xpt_path = xpt_dir / xpt_filename
            write_xpt_file(domain_dataframe, "RELREC", xpt_path)
            result["xpt_path"] = xpt_path
            result["xpt_filename"] = xpt_filename

        # Generate Dataset-XML file
        if xml_dir and output_format in ("xml", "both"):
            xml_filename = f"{disk_name}.xml"
            xml_path = xml_dir / xml_filename
            write_dataset_xml(domain_dataframe, "RELREC", config, xml_path)
            result["xml_path"] = xml_path
            result["xml_filename"] = xml_filename

        # Generate SAS program
        if sas_dir and generate_sas:
            sas_filename = f"{disk_name}.sas"
            sas_path = sas_dir / sas_filename
            sas_code = generate_sas_program(
                "RELREC",
                config,
                input_dataset="work.relrec",
                output_dataset=f"sdtm.{base_filename}",
            )
            write_sas_file(sas_code, sas_path)
            result["sas_path"] = sas_path

        return result
