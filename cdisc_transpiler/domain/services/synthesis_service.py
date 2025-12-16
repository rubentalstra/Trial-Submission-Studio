"""Domain synthesis service.

This module provides services for synthesizing SDTM domains that are not
present in source data. This includes trial design domains (TS, TA, TE,
SE, DS) and empty observation domains (AE, LB, VS, EX).

SDTM Reference:
    Trial Design domains are defined in SDTMIG v3.4 Section 5.
    Observation class domains are defined in Section 6.

This service is a pure domain service - it returns only domain data
(DataFrames + metadata), with no file I/O or infrastructure concerns.
File generation is handled by the application layer.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

import pandas as pd

from ...constants import Defaults

if TYPE_CHECKING:
    from ..entities.sdtm_domain import SDTMDomain, SDTMVariable
    from ..entities.mapping import MappingConfig


@dataclass
class SynthesisResult:
    """Result of domain synthesis (pure domain data).

    This is a pure domain object containing only synthesized data,
    with no file paths or I/O concerns. File generation is handled
    by the application layer.

    Attributes:
        domain_code: SDTM domain code
        records: Number of records in the domain
        domain_dataframe: The synthesized domain DataFrame
        config: Mapping configuration used
        success: Whether synthesis succeeded
        error: Error message if synthesis failed
    """

    domain_code: str
    records: int = 0
    domain_dataframe: pd.DataFrame | None = None
    config: "MappingConfig | None" = None
    success: bool = True
    error: str | None = None

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary format for compatibility."""
        return {
            "domain_code": self.domain_code,
            "records": self.records,
            "domain_dataframe": self.domain_dataframe,
            "config": self.config,
        }


class SynthesisService:
    """Pure domain service for synthesizing SDTM domains.

    This service creates scaffold domains when source data doesn't include them.
    These domains are required by validation tools like Pinnacle 21 for
    regulatory submission packages.

    This is a pure domain service with no I/O concerns. It returns only
    domain data (DataFrames + configs). File generation is handled by the
    application layer using the FileGeneratorPort.

    Example:
        >>> service = SynthesisService()
        >>> result = service.synthesize_trial_design(
        ...     domain_code="TS",
        ...     study_id="STUDY001",
        ... )
        >>> if result.success:
        ...     print(f"Generated {result.records} records")
        ...     # Application layer handles file generation
    """

    def synthesize_trial_design(
        self,
        domain_code: str,
        study_id: str,
        reference_starts: dict[str, str] | None = None,
    ) -> SynthesisResult:
        """Synthesize a trial design domain.

        Creates scaffold trial design domains (TS, TA, TE, SE, DS) with
        minimal required data to pass validation.

        Args:
            domain_code: Domain code (TS, TA, TE, SE, DS)
            study_id: Study identifier
            reference_starts: Reference start dates by subject

        Returns:
            SynthesisResult with generated DataFrame and config
        """
        try:
            # Get domain definition
            domain = self._get_domain(domain_code)

            # Pick reference subject and date
            subject_id, base_date = self._pick_subject(reference_starts)

            # Generate rows based on domain type
            rows = self._generate_trial_design_rows(
                domain_code, subject_id, base_date, domain, study_id
            )

            # Create dataframe
            if not rows:
                frame = pd.DataFrame(
                    {
                        var.name: pd.Series(dtype=var.pandas_dtype())
                        for var in domain.variables
                    }
                )
            else:
                frame = pd.DataFrame(rows)

            # Build configuration
            config = self._build_identity_config(domain_code, frame, study_id)

            # Build domain dataframe through builder
            domain_dataframe = self._build_domain_dataframe(frame, config)

            return SynthesisResult(
                domain_code=domain_code,
                records=len(domain_dataframe),
                domain_dataframe=domain_dataframe,
                config=config,
                success=True,
            )

        except Exception as exc:
            return SynthesisResult(
                domain_code=domain_code,
                success=False,
                error=str(exc),
            )

    def synthesize_observation(
        self,
        domain_code: str,
        study_id: str,
        reference_starts: dict[str, str] | None = None,
    ) -> SynthesisResult:
        """Synthesize an empty observation domain.

        Creates minimal observation domains (AE, LB, VS, EX) with required
        structure but minimal data.

        Args:
            domain_code: Domain code (AE, LB, VS, EX)
            study_id: Study identifier
            reference_starts: Reference start dates by subject

        Returns:
            SynthesisResult with generated DataFrame and config
        """
        try:
            # Get domain definition
            domain = self._get_domain(domain_code)

            # Pick reference subject and date
            subject_id, base_date = self._pick_subject(reference_starts)

            # Generate rows
            rows = self._generate_observation_rows(
                domain_code, subject_id, base_date, domain, study_id
            )

            # Create dataframe
            if not rows:
                frame = pd.DataFrame(
                    {
                        var.name: pd.Series(dtype=var.pandas_dtype())
                        for var in domain.variables
                    }
                )
            else:
                frame = pd.DataFrame(rows)

            # Build configuration
            config = self._build_identity_config(domain_code, frame, study_id)

            # Build domain dataframe through builder
            domain_dataframe = self._build_domain_dataframe(frame, config)

            return SynthesisResult(
                domain_code=domain_code,
                records=len(domain_dataframe),
                domain_dataframe=domain_dataframe,
                config=config,
                success=True,
            )

        except Exception as exc:
            return SynthesisResult(
                domain_code=domain_code,
                success=False,
                error=str(exc),
            )

    def _pick_subject(self, ref_starts: dict[str, str] | None) -> tuple[str, str]:
        """Pick a reference subject and date."""
        if ref_starts:
            first_id = sorted(ref_starts.keys())[0]
            return first_id, ref_starts.get(first_id) or Defaults.DATE
        return Defaults.SUBJECT_ID, Defaults.DATE

    def _generate_trial_design_rows(
        self,
        domain_code: str,
        subject_id: str,
        base_date: str,
        domain: SDTMDomain,
        study_id: str,
    ) -> list[dict]:
        """Generate rows for trial design domains."""
        upper = domain_code.upper()

        def _default_value(var: SDTMVariable) -> object:
            name = var.name.upper()
            if name in {"STUDYID", "DOMAIN"}:
                return None
            if name == "USUBJID":
                return subject_id
            if name.endswith("SEQ"):
                return 1
            if name == "TAETORD":
                return 1
            if name.endswith("DY"):
                return 1
            if name.endswith("DTC") or name.endswith("STDTC") or name.endswith("ENDTC"):
                return base_date
            if var.type == "Num":
                return 0
            return ""

        def _base_row() -> dict[str, object]:
            row = {var.name: _default_value(var) for var in domain.variables}
            row["STUDYID"] = study_id if "STUDYID" in row else None
            row["DOMAIN"] = domain_code
            return row

        rows: list[dict[str, object]] = []

        if upper == "TS":
            row = _base_row()
            row.update(
                {
                    "TSPARMCD": "TITLE",
                    "TSPARM": "Study Title",
                    "TSVAL": "Synthetic Trial",
                    "TSVCDREF": "",
                    "TSVCDVER": "",
                }
            )
            rows.append(row)
        elif upper == "TA":
            for etcd, element, order in [
                ("SCRN", "SCREENING", 0),
                ("TRT", "TREATMENT", 1),
            ]:
                row = _base_row()
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
        elif upper == "TE":
            for etcd, element in [("SCRN", "SCREENING"), ("TRT", "TREATMENT")]:
                row = _base_row()
                row.update(
                    {
                        "ETCD": etcd,
                        "ELEMENT": element,
                        "TESTRL": base_date,
                        "TEENRL": base_date,
                        "TEDUR": Defaults.ELEMENT_DURATION,
                    }
                )
                rows.append(row)
        elif upper == "SE":
            for etcd, element, epoch in [("SCRN", "SCREENING", "SCREENING")]:
                row = _base_row()
                row.update(
                    {
                        "ETCD": etcd,
                        "ELEMENT": element,
                        "SESTDTC": base_date,
                        "SEENDTC": base_date,
                        "EPOCH": epoch,
                    }
                )
                rows.append(row)
        elif upper == "DS":
            row = _base_row()
            row.update(
                {
                    "DSTERM": "COMPLETED",
                    "DSDECOD": "COMPLETED",
                    "DSSTDTC": base_date,
                }
            )
            rows.append(row)

        return rows

    def _generate_observation_rows(
        self,
        domain_code: str,
        subject_id: str,
        base_date: str,
        domain: SDTMDomain,
        study_id: str,
    ) -> list[dict]:
        """Generate minimal rows for observation domains."""
        upper = domain_code.upper()

        def _default_value(var: SDTMVariable) -> object:
            name = var.name.upper()
            if name in {"STUDYID", "DOMAIN"}:
                return None
            if name == "USUBJID":
                return subject_id
            if name.endswith("SEQ"):
                return 1
            if name.endswith("DY"):
                return 1
            if name.endswith("DTC"):
                return base_date
            if var.type == "Num":
                return None
            return ""

        row = {var.name: _default_value(var) for var in domain.variables}
        row["STUDYID"] = study_id
        row["DOMAIN"] = domain_code

        # Domain-specific defaults
        if upper == "AE":
            row.update({"AETERM": "NO ADVERSE EVENTS", "AEDECOD": "NO ADVERSE EVENTS"})
        elif upper == "LB":
            row.update({"LBTESTCD": "GLUC", "LBTEST": "Glucose", "LBORRES": ""})
        elif upper == "VS":
            row.update({"VSTESTCD": "HR", "VSTEST": "Heart Rate", "VSORRES": ""})
        elif upper == "EX":
            row.update({"EXTRT": "PLACEBO", "EXDOSE": 0})

        return [row]

    def _build_identity_config(
        self, domain_code: str, frame: pd.DataFrame, study_id: str
    ) -> MappingConfig:
        """Build identity mapping configuration."""
        from ..entities.mapping import ColumnMapping, build_config

        mappings = [
            ColumnMapping(
                source_column=col,
                target_variable=col,
                transformation=None,
                confidence_score=1.0,
            )
            for col in frame.columns
        ]
        config = build_config(domain_code, mappings)
        config.study_id = study_id
        return config

    def _get_domain(self, domain_code: str) -> SDTMDomain:
        """Get domain definition via lazy import."""
        from ...domains_module import get_domain

        return get_domain(domain_code)

    def _build_domain_dataframe(
        self, frame: pd.DataFrame, config: MappingConfig
    ) -> pd.DataFrame:
        """Build domain dataframe via lazy import."""
        from ...xpt_module.builder import build_domain_dataframe

        return build_domain_dataframe(frame, config, lenient=True)
