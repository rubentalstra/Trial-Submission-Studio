"""Domain Processing Coordinator - Coordinates domain file processing workflow.

This service coordinates the processing of domain files including loading data,
transformations, mapping, supplemental qualifiers, and file generation.

Extracted from cli/commands/study.py for improved maintainability.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

import pandas as pd
from rich.console import Console

if TYPE_CHECKING:
    from ..metadata import StudyMetadata

from ..domains import get_domain
from ..io import build_column_hints, load_input_dataset
from ..mapping import ColumnMapping, build_config, create_mapper
from ..sas import generate_sas_program, write_sas_file
from ..submission import build_suppqual
from ..xpt_module import write_xpt_file
from ..xml.dataset import write_dataset_xml
from ..xpt_module.builder import build_domain_dataframe
from ..cli.helpers import unquote_safe, write_variant_splits
from .study_orchestration_service import StudyOrchestrationService


console = Console()


class DomainProcessingCoordinator:
    """Coordinates the processing of domain files through the full workflow."""

    def __init__(self, orchestration_service: StudyOrchestrationService | None = None):
        """Initialize the domain processing coordinator.

        Args:
            orchestration_service: Optional orchestration service for domain transformations
        """
        self.orchestration_service = orchestration_service or StudyOrchestrationService()

    def process_and_merge_domain(
        self,
        files_for_domain: list[tuple[Path, str]],
        domain_code: str,
        study_id: str,
        output_format: str,
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
        min_confidence: float,
        streaming: bool,
        chunk_size: int,
        generate_sas: bool,
        verbose: bool,
        metadata: StudyMetadata | None = None,
        reference_starts: dict[str, str] | None = None,
        common_column_counts: dict[str, int] | None = None,
        total_input_files: int | None = None,
    ) -> dict:
        """Process multiple domain variant files and merge them into one output file.

        Args:
            files_for_domain: List of (file_path, variant_name) tuples
            domain_code: SDTM domain code
            study_id: Study identifier
            output_format: Output format ("xpt", "xml", or "both")
            xpt_dir: Directory for XPT files
            xml_dir: Directory for Dataset-XML files
            sas_dir: Directory for SAS programs
            min_confidence: Minimum confidence for fuzzy matching
            streaming: Whether to use streaming mode
            chunk_size: Chunk size for streaming
            generate_sas: Whether to generate SAS programs
            verbose: Whether to log verbose messages
            metadata: Study metadata (Items.csv, CodeLists.csv)
            reference_starts: Reference start dates by subject
            common_column_counts: Common column frequency counts
            total_input_files: Total number of input files

        Returns:
            Dictionary with processing results including dataframe, config, and file paths
        """
        domain = get_domain(domain_code)
        all_dataframes = []
        variant_frames: list[tuple[str, pd.DataFrame]] = []
        last_config = None
        supp_frames: list[pd.DataFrame] = []
        lb_long = False

        # Process each input file
        for input_file, variant_name in files_for_domain:
            processed_data = self._process_single_file(
                input_file,
                variant_name,
                domain_code,
                study_id,
                metadata,
                min_confidence,
                verbose,
            )

            if processed_data is None:
                continue

            frame, config, is_lb_long = processed_data
            all_dataframes.append(frame)
            variant_frames.append((variant_name or domain_code, frame))
            last_config = config

            if is_lb_long:
                lb_long = True

            # Build supplemental qualifiers
            if domain_code.upper() != "LB":
                supp_df = self._build_supplemental_qualifiers(
                    domain_code,
                    load_input_dataset(input_file),
                    frame,
                    config,
                    study_id,
                    common_column_counts,
                    total_input_files,
                )
                if supp_df is not None and not supp_df.empty:
                    supp_frames.append(supp_df)

                # Add AE treatment emergent flag
                if domain_code.upper() == "AE" and "AESEQ" in frame.columns:
                    trt_supp = self._build_ae_treatment_emergent(frame, study_id)
                    if trt_supp:
                        supp_frames.append(pd.DataFrame(trt_supp))

        if not all_dataframes:
            raise ValueError(f"No data could be processed for {domain_code}")

        assert last_config is not None, "Config should be set if we have dataframes"

        # Merge and post-process
        merged_dataframe = self._merge_dataframes(all_dataframes, domain_code, verbose)

        # De-duplicate LB data
        if domain_code.upper() == "LB":
            merged_dataframe = self._deduplicate_lb_data(merged_dataframe, domain_code)

        # Generate output files
        return self._generate_output_files(
            merged_dataframe,
            domain_code,
            study_id,
            last_config,
            output_format,
            xpt_dir,
            xml_dir,
            sas_dir,
            generate_sas,
            streaming,
            chunk_size,
            supp_frames,
            variant_frames,
            files_for_domain,
            domain,
            verbose,
        )

    def _process_single_file(
        self,
        input_file: Path,
        variant_name: str,
        domain_code: str,
        study_id: str,
        metadata: StudyMetadata | None,
        min_confidence: float,
        verbose: bool,
    ) -> tuple[pd.DataFrame, object, bool] | None:
        """Process a single input file.

        Returns:
            Tuple of (dataframe, config, is_lb_long) or None if file should be skipped
        """
        from ..cli.helpers import log_verbose

        display_name = (
            f"{domain_code}"
            if variant_name == domain_code
            else f"{domain_code} ({variant_name})"
        )

        # Load input data
        frame = load_input_dataset(input_file)
        log_verbose(verbose, f"  Loaded {len(frame)} rows from {input_file.name}")

        # Skip VSTAT helper files
        is_vstat = (
            domain_code.upper() == "VS"
            and variant_name
            and "VSTAT" in variant_name.upper()
        )
        if is_vstat:
            log_verbose(
                verbose,
                f"  Skipping {input_file.name} (VSTAT helper not mapped to SDTM)",
            )
            return None

        # Apply domain-specific transformations
        frame, vs_long = self._apply_vs_transformation(
            frame, domain_code, study_id, display_name, verbose
        )
        if frame is None:
            return None

        frame, lb_long = self._apply_lb_transformation(
            frame, domain_code, study_id, display_name, verbose
        )
        if frame is None:
            return None

        # Build configuration
        if vs_long or lb_long:
            config = self._build_identity_config(domain_code, frame)
        else:
            config = self._build_mapped_config(
                domain_code, frame, metadata, min_confidence, display_name
            )
            if config is None:
                return None

        config.study_id = study_id

        # Build domain dataframe
        from ..xpt_module.builder import build_domain_dataframe

        domain_dataframe = build_domain_dataframe(
            frame,
            config,
            lenient=True,
            metadata=metadata,
            reference_starts=None,
        )

        log_verbose(verbose, f"  Processed {len(domain_dataframe)} rows for {variant_name}")

        return domain_dataframe, config, lb_long

    def _apply_vs_transformation(
        self, frame: pd.DataFrame, domain_code: str, study_id: str, display_name: str, verbose: bool
    ) -> tuple[pd.DataFrame | None, bool]:
        """Apply VS domain transformation if needed."""
        from ..cli.helpers import log_verbose

        if domain_code.upper() != "VS":
            return frame, False

        frame = self.orchestration_service.reshape_vs_to_long(frame, study_id)
        log_verbose(verbose, f"  Normalized VS wide data to {len(frame)} long-form rows")

        if frame.empty:
            console.print(
                f"[yellow]⚠[/yellow] {display_name}: No vital signs records after reshaping"
            )
            return None, True

        return frame, True

    def _apply_lb_transformation(
        self, frame: pd.DataFrame, domain_code: str, study_id: str, display_name: str, verbose: bool
    ) -> tuple[pd.DataFrame | None, bool]:
        """Apply LB domain transformation if needed."""
        from ..cli.helpers import log_verbose

        if domain_code.upper() != "LB":
            return frame, False

        reshaped = self.orchestration_service.reshape_lb_to_long(frame, study_id)
        if "LBTESTCD" in reshaped.columns:
            log_verbose(verbose, f"  Normalized LB wide data to {len(reshaped)} long-form rows")

            if reshaped.empty:
                console.print(
                    f"[yellow]⚠[/yellow] {display_name}: No laboratory records after reshaping"
                )
                return None, True

            return reshaped, True
        else:
            log_verbose(verbose, "  Skipping LB reshape (no recognizable tests)")
            return None, False

    def _build_identity_config(self, domain_code: str, frame: pd.DataFrame) -> object:
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
        return build_config(domain_code, mappings)

    def _build_mapped_config(
        self,
        domain_code: str,
        frame: pd.DataFrame,
        metadata: StudyMetadata | None,
        min_confidence: float,
        display_name: str,
    ) -> object | None:
        """Build mapped configuration using fuzzy matching."""
        column_hints = build_column_hints(frame)
        engine = create_mapper(
            domain_code,
            metadata=metadata,
            min_confidence=min_confidence,
            column_hints=column_hints,
        )
        suggestions = engine.suggest(frame)

        if not suggestions.mappings:
            console.print(
                f"[yellow]⚠[/yellow] {display_name}: No mappings found, skipping"
            )
            return None

        return build_config(domain_code, suggestions.mappings)

    def _build_supplemental_qualifiers(
        self,
        domain_code: str,
        source_frame: pd.DataFrame,
        domain_frame: pd.DataFrame,
        config: object,
        study_id: str,
        common_column_counts: dict[str, int] | None,
        total_files: int | None,
    ) -> pd.DataFrame | None:
        """Build supplemental qualifiers for unmapped columns."""
        used_source_columns: set[str] = set()
        if config and config.mappings:
            for m in config.mappings:
                used_source_columns.add(unquote_safe(m.source_column))
                if getattr(m, "use_code_column", None):
                    used_source_columns.add(unquote_safe(m.use_code_column))

        supp_df, _ = build_suppqual(
            domain_code,
            source_frame,
            domain_frame,
            used_source_columns,
            study_id=study_id,
            common_column_counts=common_column_counts,
            total_files=total_files,
        )
        return supp_df

    def _build_ae_treatment_emergent(
        self, domain_frame: pd.DataFrame, study_id: str
    ) -> list[dict]:
        """Build treatment emergent flag supplemental records for AE domain."""
        trt_records = []
        for _, r in domain_frame.iterrows():
            seq_val = r.get("AESEQ", "")
            try:
                seq_str = (
                    str(int(seq_val))
                    if pd.notna(seq_val)
                    and str(seq_val).strip() != ""
                    and float(seq_val).is_integer()
                    else str(seq_val)
                )
            except Exception:
                seq_str = str(seq_val)
            trt_records.append(
                {
                    "STUDYID": study_id,
                    "RDOMAIN": "AE",
                    "USUBJID": r.get("USUBJID", ""),
                    "IDVAR": "AESEQ",
                    "IDVARVAL": seq_str,
                    "QNAM": "AETRTEM",
                    "QLABEL": "Treatment Emergent Flag",
                    "QVAL": "Y",
                    "QORIG": "DERIVED",
                    "QEVAL": "",
                }
            )
        return trt_records

    def _merge_dataframes(
        self, all_dataframes: list[pd.DataFrame], domain_code: str, verbose: bool
    ) -> pd.DataFrame:
        """Merge multiple dataframes and re-sequence."""
        from ..cli.helpers import log_verbose

        if len(all_dataframes) == 1:
            return all_dataframes[0]

        merged_dataframe = pd.concat(all_dataframes, ignore_index=True)

        # Re-assign sequence numbers per subject after merge
        seq_col = f"{domain_code}SEQ"
        if seq_col in merged_dataframe.columns and "USUBJID" in merged_dataframe.columns:
            merged_dataframe[seq_col] = (
                merged_dataframe.groupby("USUBJID").cumcount() + 1
            )

        log_verbose(
            verbose,
            f"Merged {len(all_dataframes)} files into {len(merged_dataframe)} rows",
        )

        return merged_dataframe

    def _deduplicate_lb_data(
        self, merged_dataframe: pd.DataFrame, domain_code: str
    ) -> pd.DataFrame:
        """Deduplicate LB data to avoid SD1152 issues."""
        dedup_keys = [
            key
            for key in ("USUBJID", "LBTESTCD", "LBDTC")
            if key in merged_dataframe.columns
        ]
        if dedup_keys:
            merged_dataframe = (
                merged_dataframe.copy()
                .sort_values(by=dedup_keys)
                .drop_duplicates(subset=dedup_keys, keep="first")
                .reset_index(drop=True)
            )
            seq_col = f"{domain_code}SEQ"
            if seq_col in merged_dataframe.columns and "USUBJID" in merged_dataframe.columns:
                merged_dataframe[seq_col] = (
                    merged_dataframe.groupby("USUBJID").cumcount() + 1
                )
        return merged_dataframe

    def _generate_output_files(
        self,
        merged_dataframe: pd.DataFrame,
        domain_code: str,
        study_id: str,
        config: object,
        output_format: str,
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
        generate_sas: bool,
        streaming: bool,
        chunk_size: int,
        supp_frames: list[pd.DataFrame],
        variant_frames: list[tuple[str, pd.DataFrame]],
        files_for_domain: list[tuple[Path, str]],
        domain: object,
        verbose: bool,
    ) -> dict:
        """Generate output files and return processing results."""
        from ..cli.utils import log_success

        base_filename = domain.resolved_dataset_name()
        disk_name = base_filename.lower()

        result = {
            "domain_code": domain_code,
            "records": len(merged_dataframe),
            "domain_dataframe": merged_dataframe,
            "config": config,
            "xpt_path": None,
            "xml_path": None,
            "sas_path": None,
            "split_xpt_paths": [],
            "supplementals": [],
        }

        # Handle supplemental qualifiers
        if supp_frames:
            supp_result = self._generate_supplemental_files(
                supp_frames, domain_code, study_id, output_format, xpt_dir, xml_dir
            )
            result["supplementals"].append(supp_result)

        # Generate main domain files
        if xpt_dir and output_format in ("xpt", "both"):
            xpt_path = xpt_dir / f"{disk_name}.xpt"
            write_xpt_file(merged_dataframe, domain_code, xpt_path)
            result["xpt_path"] = xpt_path
            result["xpt_filename"] = xpt_path.name
            log_success(f"Generated XPT: {xpt_path}")

            # Handle domain variant splits (SDTMIG v3.4 Section 4.1.7)
            # Any domain can be split when there are multiple variant files
            if len(variant_frames) > 1:
                split_paths = write_variant_splits(
                    merged_dataframe, variant_frames, domain, xpt_dir, console
                )
                result["split_xpt_paths"] = split_paths

        if xml_dir and output_format in ("xml", "both"):
            xml_path = xml_dir / f"{disk_name}.xml"
            if streaming:
                console.print(
                    f"[yellow]⚠[/yellow] Streaming mode not implemented, using regular write"
                )
            write_dataset_xml(merged_dataframe, domain_code, config, xml_path)
            result["xml_path"] = xml_path
            result["xml_filename"] = xml_path.name
            log_success(f"Generated Dataset-XML: {xml_path}")

        if sas_dir and generate_sas:
            sas_path = sas_dir / f"{disk_name}.sas"
            first_input_file = files_for_domain[0][0]
            sas_code = generate_sas_program(
                domain_code, config, first_input_file.stem, base_filename
            )
            write_sas_file(sas_code, sas_path)
            result["sas_path"] = sas_path
            log_success(f"Generated SAS: {sas_path}")

        return result

    def _generate_supplemental_files(
        self,
        supp_frames: list[pd.DataFrame],
        domain_code: str,
        study_id: str,
        output_format: str,
        xpt_dir: Path | None,
        xml_dir: Path | None,
    ) -> dict:
        """Generate supplemental qualifier files."""
        merged_supp = (
            supp_frames[0]
            if len(supp_frames) == 1
            else pd.concat(supp_frames, ignore_index=True)
        )

        supp_domain_code = f"SUPP{domain_code.upper()}"
        supp_config = self._build_identity_config(supp_domain_code, merged_supp)
        supp_config.study_id = study_id

        base_filename = get_domain(supp_domain_code).resolved_dataset_name()
        disk_name = base_filename.lower()

        supp_result = {
            "domain_code": supp_domain_code,
            "records": len(merged_supp),
            "domain_dataframe": merged_supp,
            "config": supp_config,
            "xpt_path": None,
            "xml_path": None,
            "sas_path": None,
        }

        if xpt_dir and output_format in ("xpt", "both"):
            xpt_path = xpt_dir / f"{disk_name}.xpt"
            file_label = f"Supplemental Qualifiers for {domain_code.upper()}"
            write_xpt_file(merged_supp, supp_domain_code, xpt_path, file_label=file_label)
            supp_result["xpt_path"] = xpt_path
            supp_result["xpt_filename"] = xpt_path.name

        if xml_dir and output_format in ("xml", "both"):
            xml_path = xml_dir / f"{disk_name}.xml"
            write_dataset_xml(merged_supp, supp_domain_code, supp_config, xml_path)
            supp_result["xml_path"] = xml_path
            supp_result["xml_filename"] = xml_path.name

        return supp_result
