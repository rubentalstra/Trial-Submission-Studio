"""Trial Design Domain Synthesis Service.

This service generates trial design domains (TS, TA, TE, SE, DS, RELREC)
when they are not present in the source data.

SDTM Reference:
    Trial Design domains are defined in SDTMIG v3.4 Section 5:
    - TS (Trial Summary): Key study parameters and characteristics
    - TA (Trial Arms): Planned arms and element sequences
    - TE (Trial Elements): Study elements with start/end rules
    - TV (Trial Visits): Planned visits for each arm
    - TI (Trial Inclusion/Exclusion): Entry criteria
    - TD (Trial Disease Assessments): Disease assessment schedule
    - TM (Trial Disease Milestones): Disease milestone definitions

    Special-Purpose domains include:
    - SE (Subject Elements): Elements actually experienced by subjects
    - DS (Disposition): Subject disposition events

    Relationship domains:
    - RELREC (Related Records): Links between related observations
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd

if TYPE_CHECKING:
    from ..domains_module import SDTMDomain

from ..domains_module import get_domain
from ..mapping_module import ColumnMapping, build_config


class TrialDesignService:
    """Service for synthesizing SDTM trial design domains.

    This service creates scaffold trial design domains when source data
    doesn't include them. These domains are required by validation tools
    like Pinnacle 21 for regulatory submission packages.
    """

    def __init__(
        self,
        study_id: str,
        reference_starts: dict[str, str] | None = None,
    ):
        """Initialize the trial design service.

        Args:
            study_id: Study identifier
            reference_starts: Optional USUBJID -> RFSTDTC mapping
        """
        self.study_id = study_id
        self.reference_starts = reference_starts or {}

    def synthesize_ts(self) -> tuple[pd.DataFrame, object]:
        """Synthesize Trial Summary (TS) domain.

        Returns:
            Tuple of (dataframe, config)
        """
        domain = get_domain("TS")
        row = self._base_row(domain)
        row.update(
            {
                "TSPARMCD": "TITLE",
                "TSPARM": "Study Title",
                "TSVAL": "Synthetic Trial",
                "TSVCDREF": "",
                "TSVCDVER": "",
            }
        )

        df = pd.DataFrame([row])
        config = self._build_config("TS", df)
        return df, config

    def synthesize_ta(self) -> tuple[pd.DataFrame, object]:
        """Synthesize Trial Arms (TA) domain.

        Returns:
            Tuple of (dataframe, config)
        """
        domain = get_domain("TA")
        elements = [
            ("SCRN", "SCREENING", 0),
            ("TRT", "TREATMENT", 1),
        ]

        rows = []
        for etcd, element, order in elements:
            row = self._base_row(domain)
            row.update(
                {
                    "ARMCD": "ARM1",
                    "ARM": "Treatment Arm",
                    "ETCD": etcd,
                    "ELEMENT": element,
                    "TAETORD": order,
                    "EPOCH": element,
                }
            )
            rows.append(row)

        df = pd.DataFrame(rows)
        config = self._build_config("TA", df)
        return df, config

    def synthesize_te(self) -> tuple[pd.DataFrame, object]:
        """Synthesize Trial Elements (TE) domain.

        Returns:
            Tuple of (dataframe, config)
        """
        domain = get_domain("TE")
        base_date = self._get_reference_date()

        elements = [
            ("SCRN", "SCREENING", base_date, base_date),
            ("TRT", "TREATMENT", base_date, base_date),
        ]

        rows = []
        for etcd, element, start, end in elements:
            row = self._base_row(domain)
            row.update(
                {
                    "ETCD": etcd,
                    "ELEMENT": element,
                    "TESTRL": start,
                    "TEENRL": end,
                    "TEDUR": "P1D",
                }
            )
            rows.append(row)

        df = pd.DataFrame(rows)
        config = self._build_config("TE", df)
        return df, config

    def synthesize_se(self) -> tuple[pd.DataFrame, object]:
        """Synthesize Subject Elements (SE) domain.

        Returns:
            Tuple of (dataframe, config)
        """
        domain = get_domain("SE")
        subjects = (
            self.reference_starts.keys() if self.reference_starts else ["SYNTH001"]
        )
        base_date = self._get_reference_date()

        elements = [
            ("SCRN", "SCREENING", "SCREENING"),
            ("TRT", "TREATMENT", "TREATMENT"),
        ]

        rows = []
        for usubjid in subjects:
            start_date = self.reference_starts.get(usubjid, base_date)
            for etcd, element, epoch in elements:
                row = self._base_row(domain)
                row.update(
                    {
                        "USUBJID": usubjid,
                        "ETCD": etcd,
                        "ELEMENT": element,
                        "EPOCH": epoch,
                        "SESTDTC": start_date,
                        "SEENDTC": start_date,
                        "SESTDY": 1,
                        "SEENDY": 1,
                    }
                )
                rows.append(row)

        df = pd.DataFrame(rows)
        config = self._build_config("SE", df)
        return df, config

    def synthesize_ds(self) -> tuple[pd.DataFrame, object]:
        """Synthesize Disposition (DS) domain.

        Returns:
            Tuple of (dataframe, config)
        """
        domain = get_domain("DS")
        subjects = (
            self.reference_starts.keys() if self.reference_starts else ["SYNTH001"]
        )
        base_date = self._get_reference_date()

        rows = []
        for usubjid in subjects:
            start_date = self.reference_starts.get(usubjid, base_date)

            # Informed consent
            row = self._base_row(domain)
            row.update(
                {
                    "USUBJID": usubjid,
                    "DSDECOD": "INFORMED CONSENT OBTAINED",
                    "DSTERM": "INFORMED CONSENT OBTAINED",
                    "DSCAT": "PROTOCOL MILESTONE",
                    "DSSTDTC": start_date,
                    "DSSEQ": None,
                    "EPOCH": "SCREENING",
                    "DSSTDY": 1,
                    "DSDY": 1,
                }
            )
            rows.append(row)

            # Disposition event
            row = self._base_row(domain)
            row.update(
                {
                    "USUBJID": usubjid,
                    "DSDECOD": "COMPLETED",
                    "DSTERM": "COMPLETED",
                    "DSCAT": "DISPOSITION EVENT",
                    "DSSTDTC": start_date,
                    "DSSEQ": None,
                    "EPOCH": "TREATMENT",
                    "DSSTDY": 1,
                    "DSDY": 1,
                }
            )
            rows.append(row)

        df = pd.DataFrame(rows)
        config = self._build_config("DS", df)
        return df, config

    def synthesize_relrec(
        self, domain_results: dict[str, pd.DataFrame]
    ) -> tuple[pd.DataFrame, object]:
        """Synthesize Relationship Records (RELREC) domain.

        Args:
            domain_results: Dictionary of domain_code -> dataframe

        Returns:
            Tuple of (dataframe, config)
        """
        records = self._build_relrec_records(domain_results)

        if records.empty:
            # Return empty but structured dataframe
            domain = get_domain("RELREC")
            df = pd.DataFrame(
                {
                    var.name: pd.Series(dtype=var.pandas_dtype())
                    for var in domain.variables
                }
            )
        else:
            df = records

        config = self._build_config("RELREC", df)
        return df, config

    def _build_relrec_records(
        self, domain_results: dict[str, pd.DataFrame]
    ) -> pd.DataFrame:
        """Build RELREC records from domain data."""
        ae_df = domain_results.get("AE")
        ds_df = domain_results.get("DS")
        ex_df = domain_results.get("EX")

        records = []

        # Get DS sequence mapping
        ds_seq_map = {}
        if ds_df is not None and "DSSEQ" in ds_df.columns:
            numeric = pd.to_numeric(ds_df["DSSEQ"], errors="coerce")
            ds_seq_map = (
                pd.DataFrame({"USUBJID": ds_df["USUBJID"], "DSSEQ": numeric})
                .dropna(subset=["USUBJID", "DSSEQ"])
                .groupby("USUBJID")["DSSEQ"]
                .min()
                .to_dict()
            )

        # Link AE to DS
        if ae_df is not None and ds_seq_map:
            for idx, row in ae_df.iterrows():
                usubjid = str(row.get("USUBJID", "")).strip()
                if not usubjid:
                    continue
                aeseq = self._stringify(row.get("AESEQ"), idx + 1)
                relid = f"AE_DS_{usubjid}_{aeseq}"

                records.append(
                    {
                        "STUDYID": self.study_id,
                        "RDOMAIN": "AE",
                        "USUBJID": usubjid,
                        "IDVAR": "AESEQ",
                        "IDVARVAL": aeseq,
                        "RELTYPE": "",
                        "RELID": relid,
                    }
                )

                ds_seq = ds_seq_map.get(usubjid)
                if ds_seq is not None:
                    records.append(
                        {
                            "STUDYID": self.study_id,
                            "RDOMAIN": "DS",
                            "USUBJID": usubjid,
                            "IDVAR": "DSSEQ",
                            "IDVARVAL": self._stringify(ds_seq, 1),
                            "RELTYPE": "",
                            "RELID": relid,
                        }
                    )

        # Link EX to DS
        if ex_df is not None and ds_seq_map:
            for idx, row in ex_df.iterrows():
                usubjid = str(row.get("USUBJID", "")).strip()
                if not usubjid:
                    continue
                exseq = self._stringify(row.get("EXSEQ"), idx + 1)
                relid = f"EX_DS_{usubjid}_{exseq}"

                records.append(
                    {
                        "STUDYID": self.study_id,
                        "RDOMAIN": "EX",
                        "USUBJID": usubjid,
                        "IDVAR": "EXSEQ",
                        "IDVARVAL": exseq,
                        "RELTYPE": "",
                        "RELID": relid,
                    }
                )

                ds_seq = ds_seq_map.get(usubjid)
                if ds_seq is not None:
                    records.append(
                        {
                            "STUDYID": self.study_id,
                            "RDOMAIN": "DS",
                            "USUBJID": usubjid,
                            "IDVAR": "DSSEQ",
                            "IDVARVAL": self._stringify(ds_seq, 1),
                            "RELTYPE": "",
                            "RELID": relid,
                        }
                    )

        return pd.DataFrame(records)

    def _base_row(self, domain: SDTMDomain) -> dict:
        """Create base row for a domain with default values."""
        base_date = self._get_reference_date()
        subject_id = self._get_reference_subject()

        row = {}
        for var in domain.variables:
            name = var.name.upper()
            if name in {"STUDYID", "DOMAIN"}:
                row[var.name] = None
            elif name == "USUBJID":
                row[var.name] = subject_id
            elif name.endswith("SEQ"):
                row[var.name] = 1
            elif name == "TAETORD":
                row[var.name] = 1
            elif name.endswith("DY"):
                row[var.name] = 1
            elif (
                name.endswith("DTC") or name.endswith("STDTC") or name.endswith("ENDTC")
            ):
                row[var.name] = base_date
            elif var.type == "Num":
                row[var.name] = 0
            else:
                row[var.name] = ""

        row["STUDYID"] = self.study_id
        row["DOMAIN"] = domain.code

        return row

    def _build_config(self, domain_code: str, df: pd.DataFrame) -> object:
        """Build mapping configuration for a synthesized domain."""
        mappings = [
            ColumnMapping(
                source_column=col,
                target_variable=col,
                transformation=None,
                confidence_score=1.0,
            )
            for col in df.columns
        ]
        config = build_config(domain_code, mappings)
        config.study_id = self.study_id
        return config

    def _get_reference_date(self) -> str:
        """Get a reference date from reference starts or default."""
        if self.reference_starts:
            first_id = sorted(self.reference_starts.keys())[0]
            return self.reference_starts.get(first_id, "2023-01-01")
        return "2023-01-01"

    def _get_reference_subject(self) -> str:
        """Get a reference subject ID or default."""
        if self.reference_starts:
            return sorted(self.reference_starts.keys())[0]
        return "SYNTH001"

    @staticmethod
    def _stringify(val: object, fallback_index: int) -> str:
        """Convert value to string, handling NaN and numeric formatting."""
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
