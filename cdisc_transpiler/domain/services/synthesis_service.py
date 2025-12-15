"""Domain synthesis service.

This module provides services for synthesizing SDTM domains that are not
present in source data. This includes trial design domains (TS, TA, TE,
SE, DS) and empty observation domains (AE, LB, VS, EX).

SDTM Reference:
    Trial Design domains are defined in SDTMIG v3.4 Section 5.
    Observation class domains are defined in Section 6.

This service replaces `legacy/domain_synthesis_coordinator.py` and follows
the Ports & Adapters architecture pattern.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING, Any

import pandas as pd

from ...constants import Defaults

if TYPE_CHECKING:
    from ...application.ports import FileGeneratorPort, LoggerPort
    from ...domains_module import SDTMDomain, SDTMVariable
    from ...mapping_module import MappingConfig


@dataclass
class SynthesisResult:
    """Result of domain synthesis.
    
    Attributes:
        domain_code: SDTM domain code
        records: Number of records in the domain
        domain_dataframe: The synthesized domain DataFrame
        config: Mapping configuration used
        xpt_path: Path to generated XPT file (if any)
        xml_path: Path to generated XML file (if any)
        sas_path: Path to generated SAS file (if any)
        success: Whether synthesis succeeded
        error: Error message if synthesis failed
    """
    
    domain_code: str
    records: int = 0
    domain_dataframe: pd.DataFrame | None = None
    config: "MappingConfig | None" = None
    xpt_path: Path | None = None
    xml_path: Path | None = None
    sas_path: Path | None = None
    success: bool = True
    error: str | None = None
    
    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary format for compatibility."""
        return {
            "domain_code": self.domain_code,
            "records": self.records,
            "domain_dataframe": self.domain_dataframe,
            "config": self.config,
            "xpt_path": self.xpt_path,
            "xpt_filename": self.xpt_path.name if self.xpt_path else None,
            "xml_path": self.xml_path,
            "xml_filename": self.xml_path.name if self.xml_path else None,
            "sas_path": self.sas_path,
        }


class SynthesisService:
    """Service for synthesizing SDTM domains.
    
    This service creates scaffold domains when source data doesn't include them.
    These domains are required by validation tools like Pinnacle 21 for
    regulatory submission packages.
    
    The service uses injected dependencies (FileGeneratorPort, LoggerPort)
    for file generation and logging, following the Ports & Adapters pattern.
    
    Example:
        >>> service = SynthesisService(file_generator=file_gen, logger=logger)
        >>> result = service.synthesize_trial_design(
        ...     domain_code="TS",
        ...     study_id="STUDY001",
        ...     xpt_dir=Path("output/xpt"),
        ... )
        >>> if result.success:
        ...     print(f"Generated {result.records} records")
    """
    
    def __init__(
        self,
        file_generator: FileGeneratorPort | None = None,
        logger: LoggerPort | None = None,
    ):
        """Initialize the synthesis service.
        
        Args:
            file_generator: Optional file generator for output generation
            logger: Optional logger for progress reporting
        """
        self._file_generator = file_generator
        self._logger = logger
    
    def synthesize_trial_design(
        self,
        domain_code: str,
        study_id: str,
        output_formats: set[str] | None = None,
        xpt_dir: Path | None = None,
        xml_dir: Path | None = None,
        sas_dir: Path | None = None,
        generate_sas: bool = True,
        reference_starts: dict[str, str] | None = None,
    ) -> SynthesisResult:
        """Synthesize a trial design domain.
        
        Creates scaffold trial design domains (TS, TA, TE, SE, DS) with
        minimal required data to pass validation.
        
        Args:
            domain_code: Domain code (TS, TA, TE, SE, DS)
            study_id: Study identifier
            output_formats: Set of formats to generate ({"xpt", "xml"})
            xpt_dir: Directory for XPT files
            xml_dir: Directory for Dataset-XML files
            sas_dir: Directory for SAS programs
            generate_sas: Whether to generate SAS programs
            reference_starts: Reference start dates by subject
            
        Returns:
            SynthesisResult with generated data and file paths
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
            
            # Generate output files
            return self._generate_output_files(
                domain_dataframe=domain_dataframe,
                domain_code=domain_code,
                study_id=study_id,
                config=config,
                output_formats=output_formats or {"xpt", "xml"},
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
                generate_sas=generate_sas,
                domain=domain,
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
        output_formats: set[str] | None = None,
        xpt_dir: Path | None = None,
        xml_dir: Path | None = None,
        sas_dir: Path | None = None,
        generate_sas: bool = True,
        reference_starts: dict[str, str] | None = None,
    ) -> SynthesisResult:
        """Synthesize an empty observation domain.
        
        Creates minimal observation domains (AE, LB, VS, EX) with required
        structure but minimal data.
        
        Args:
            domain_code: Domain code (AE, LB, VS, EX)
            study_id: Study identifier
            output_formats: Set of formats to generate ({"xpt", "xml"})
            xpt_dir: Directory for XPT files
            xml_dir: Directory for Dataset-XML files
            sas_dir: Directory for SAS programs
            generate_sas: Whether to generate SAS programs
            reference_starts: Reference start dates by subject
            
        Returns:
            SynthesisResult with generated data and file paths
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
            
            # Generate output files
            return self._generate_output_files(
                domain_dataframe=domain_dataframe,
                domain_code=domain_code,
                study_id=study_id,
                config=config,
                output_formats=output_formats or {"xpt", "xml"},
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
                generate_sas=generate_sas,
                domain=domain,
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
        from ...mapping_module import ColumnMapping, build_config
        
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
    
    def _generate_output_files(
        self,
        domain_dataframe: pd.DataFrame,
        domain_code: str,
        study_id: str,
        config: MappingConfig,
        output_formats: set[str],
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
        generate_sas: bool,
        domain: SDTMDomain,
    ) -> SynthesisResult:
        """Generate output files for a synthesized domain."""
        base_filename = domain.resolved_dataset_name()
        disk_name = base_filename.lower()
        
        result = SynthesisResult(
            domain_code=domain_code,
            records=len(domain_dataframe),
            domain_dataframe=domain_dataframe,
            config=config,
            success=True,
        )
        
        # Use FileGeneratorPort if available
        if self._file_generator is not None:
            from ...infrastructure.io.models import OutputDirs, OutputRequest
            
            request = OutputRequest(
                dataframe=domain_dataframe,
                domain_code=domain_code,
                config=config,
                output_dirs=OutputDirs(
                    xpt_dir=xpt_dir if "xpt" in output_formats else None,
                    xml_dir=xml_dir if "xml" in output_formats else None,
                    sas_dir=sas_dir if generate_sas else None,
                ),
                formats=output_formats | ({"sas"} if generate_sas else set()),
                base_filename=base_filename,
            )
            
            output_result = self._file_generator.generate(request)
            result.xpt_path = output_result.xpt_path
            result.xml_path = output_result.xml_path
            result.sas_path = output_result.sas_path
            
            if not output_result.success:
                result.success = False
                result.error = "; ".join(output_result.errors)
        else:
            # Direct file generation (fallback)
            result = self._generate_files_directly(
                domain_dataframe=domain_dataframe,
                domain_code=domain_code,
                config=config,
                output_formats=output_formats,
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
                generate_sas=generate_sas,
                base_filename=base_filename,
                disk_name=disk_name,
                result=result,
            )
        
        # Log success
        if self._logger is not None and result.success:
            if result.xpt_path:
                self._logger.success(f"Generated {domain_code} XPT: {result.xpt_path}")
            if result.xml_path:
                self._logger.success(f"Generated {domain_code} Dataset-XML: {result.xml_path}")
            if result.sas_path:
                self._logger.success(f"Generated {domain_code} SAS: {result.sas_path}")
        
        return result
    
    def _generate_files_directly(
        self,
        domain_dataframe: pd.DataFrame,
        domain_code: str,
        config: MappingConfig,
        output_formats: set[str],
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
        generate_sas: bool,
        base_filename: str,
        disk_name: str,
        result: SynthesisResult,
    ) -> SynthesisResult:
        """Generate files directly without FileGeneratorPort."""
        from ...xpt_module import write_xpt_file
        from ...xml_module.dataset_module import write_dataset_xml
        from ...sas_module import generate_sas_program, write_sas_file
        
        # Generate XPT file
        if xpt_dir and "xpt" in output_formats:
            xpt_path = xpt_dir / f"{disk_name}.xpt"
            write_xpt_file(domain_dataframe, domain_code, xpt_path)
            result.xpt_path = xpt_path
        
        # Generate Dataset-XML file
        if xml_dir and "xml" in output_formats:
            xml_path = xml_dir / f"{disk_name}.xml"
            write_dataset_xml(domain_dataframe, domain_code, config, xml_path)
            result.xml_path = xml_path
        
        # Generate SAS program
        if sas_dir and generate_sas:
            sas_path = sas_dir / f"{disk_name}.sas"
            sas_code = generate_sas_program(
                domain_code,
                config,
                input_dataset=f"work.{domain_code.lower()}",
                output_dataset=f"sdtm.{base_filename}",
            )
            write_sas_file(sas_code, sas_path)
            result.sas_path = sas_path
        
        return result
