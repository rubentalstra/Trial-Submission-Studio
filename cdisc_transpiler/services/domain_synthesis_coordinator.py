"""Domain Synthesis Coordinator - Handles domain synthesis with file generation.

This service coordinates the synthesis of domains that don't exist in source data
and generates the appropriate output files.

Extracted from cli/commands/study.py for improved maintainability.
"""

from __future__ import annotations

from pathlib import Path

import pandas as pd

from ..domains_module import SDTMVariable, SDTMDomain, get_domain
from ..mapping_module import ColumnMapping, MappingConfig, build_config
from ..sas_module import generate_sas_program, write_sas_file
from ..xpt_module import write_xpt_file
from ..xpt_module.builder import build_domain_dataframe
from ..xml_module.dataset_module import write_dataset_xml


def _log_success(message: str) -> None:
    """Log a success message. Deferred import to avoid circular dependency."""
    from ..cli.utils import log_success
    log_success(message)


class DomainSynthesisCoordinator:
    """Coordinates synthesis of domains with file generation."""

    def synthesize_trial_design_domain(
        self,
        domain_code: str,
        study_id: str,
        output_format: str,
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
        generate_sas: bool,
        reference_starts: dict[str, str] | None = None,
    ) -> dict:
        """Synthesize a trial design domain with file generation.

        Args:
            domain_code: Domain code (TS, TA, TE, SE, DS)
            study_id: Study identifier
            output_format: Output format ("xpt", "xml", or "both")
            xpt_dir: Directory for XPT files
            xml_dir: Directory for Dataset-XML files
            sas_dir: Directory for SAS programs
            generate_sas: Whether to generate SAS programs
            reference_starts: Reference start dates by subject

        Returns:
            Dictionary with synthesis results including dataframe, config, and file paths
        """
        # Pick a reference subject and date
        subject_id, base_date = self._pick_subject(reference_starts)
        domain = get_domain(domain_code)

        # Generate domain data
        rows = self._generate_trial_design_rows(
            domain_code, subject_id, base_date, domain, study_id
        )

        if not rows:
            # Create empty dataframe
            frame = pd.DataFrame(
                {
                    var.name: pd.Series(dtype=var.pandas_dtype())
                    for var in domain.variables
                }
            )
        else:
            frame = pd.DataFrame(rows)

        # Build configuration and domain dataframe
        config = self._build_identity_config(domain_code, frame, study_id)
        domain_dataframe = build_domain_dataframe(frame, config, lenient=True)

        # Generate output files
        return self._generate_domain_files(
            domain_dataframe,
            domain_code,
            study_id,
            config,
            output_format,
            xpt_dir,
            xml_dir,
            sas_dir,
            generate_sas,
            domain,
        )

    def synthesize_empty_observation_domain(
        self,
        domain_code: str,
        study_id: str,
        output_format: str,
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
        generate_sas: bool,
        reference_starts: dict[str, str] | None = None,
    ) -> dict:
        """Generate an empty observation class domain with structure.

        Args:
            domain_code: Domain code (AE, LB, VS, EX)
            study_id: Study identifier
            output_format: Output format ("xpt", "xml", or "both")
            xpt_dir: Directory for XPT files
            xml_dir: Directory for Dataset-XML files
            sas_dir: Directory for SAS programs
            generate_sas: Whether to generate SAS programs
            reference_starts: Reference start dates by subject

        Returns:
            Dictionary with synthesis results
        """
        subject_id, base_date = self._pick_subject(reference_starts)
        domain = get_domain(domain_code)

        # Generate one row with minimal data
        rows = self._generate_empty_observation_rows(
            domain_code, subject_id, base_date, domain, study_id
        )

        frame = (
            pd.DataFrame(rows)
            if rows
            else pd.DataFrame(
                {
                    var.name: pd.Series(dtype=var.pandas_dtype())
                    for var in domain.variables
                }
            )
        )

        # Build configuration and domain dataframe
        config = self._build_identity_config(domain_code, frame, study_id)
        domain_dataframe = build_domain_dataframe(frame, config, lenient=True)

        # Generate output files
        return self._generate_domain_files(
            domain_dataframe,
            domain_code,
            study_id,
            config,
            output_format,
            xpt_dir,
            xml_dir,
            sas_dir,
            generate_sas,
            domain,
        )

    def _pick_subject(self, ref_starts: dict[str, str] | None) -> tuple[str, str]:
        """Pick a reference subject and date."""
        if ref_starts:
            first_id = sorted(ref_starts.keys())[0]
            return first_id, ref_starts.get(first_id) or "2023-01-01"
        return "SYNTH001", "2023-01-01"

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
                        "TEDUR": "P1D",
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

    def _generate_empty_observation_rows(
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

    def _generate_domain_files(
        self,
        domain_dataframe: pd.DataFrame,
        domain_code: str,
        study_id: str,
        config: MappingConfig,
        output_format: str,
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
        generate_sas: bool,
        domain: SDTMDomain,
    ) -> dict:
        """Generate output files for a synthesized domain."""
        base_filename = domain.resolved_dataset_name()
        disk_name = base_filename.lower()

        result = {
            "domain_code": domain_code,
            "records": len(domain_dataframe),
            "domain_dataframe": domain_dataframe,
            "config": config,
            "xpt_path": None,
            "xml_path": None,
            "sas_path": None,
        }

        # Generate XPT file
        if xpt_dir and output_format in ("xpt", "both"):
            xpt_path = xpt_dir / f"{disk_name}.xpt"
            write_xpt_file(domain_dataframe, domain_code, xpt_path)
            result["xpt_path"] = xpt_path
            result["xpt_filename"] = xpt_path.name
            _log_success(f"Generated {domain_code} XPT: {xpt_path}")

        # Generate Dataset-XML file
        if xml_dir and output_format in ("xml", "both"):
            xml_path = xml_dir / f"{disk_name}.xml"
            write_dataset_xml(domain_dataframe, domain_code, config, xml_path)
            result["xml_path"] = xml_path
            result["xml_filename"] = xml_path.name
            _log_success(f"Generated {domain_code} Dataset-XML: {xml_path}")

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
            result["sas_path"] = sas_path
            _log_success(f"Generated {domain_code} SAS: {sas_path}")

        return result
