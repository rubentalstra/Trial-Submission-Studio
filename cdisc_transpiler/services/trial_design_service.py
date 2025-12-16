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

import math
from typing import TYPE_CHECKING

import pandas as pd

if TYPE_CHECKING:
    from ..domain.entities.sdtm_domain import SDTMDomain

from ..constants import Defaults
from ..infrastructure.sdtm_spec.registry import get_domain
from ..domain.entities.mapping import ColumnMapping, build_config


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
        from ..domain.services import RelrecService

        service = RelrecService()
        df, config = service.build_relrec(
            domain_dataframes=domain_results,
            study_id=self.study_id,
        )
        return df, config

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
            return self.reference_starts.get(first_id, Defaults.DATE)
        return Defaults.DATE

    def _get_reference_subject(self) -> str:
        """Get a reference subject ID or default."""
        if self.reference_starts:
            return sorted(self.reference_starts.keys())[0]
        return Defaults.SUBJECT_ID

    @staticmethod
    def _stringify(val: object, fallback_index: int) -> str:
        """Convert value to string, handling NaN and numeric formatting."""
        # Normalize collection-like inputs to a scalar
        if isinstance(val, pd.Series):
            val = val.iloc[0] if not val.empty else None
        elif isinstance(val, pd.Index):
            val = val[0] if len(val) else None
        elif isinstance(val, (list, tuple)):
            val = val[0] if len(val) else None

        if val is None:
            return str(fallback_index)

        try:
            num_f = float(str(val))
            if math.isnan(num_f):
                return str(fallback_index)
            if num_f.is_integer():
                return str(int(num_f))
            return str(num_f)
        except Exception:
            return str(val)
