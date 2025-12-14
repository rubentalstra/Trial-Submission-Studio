"""Domain Processing Coordinator - Coordinates domain file processing workflow.

This service coordinates the processing of SDTM domain files including loading data,
transformations, mapping, supplemental qualifiers (SUPPQUAL), and file generation.

SDTM Reference:
    SDTMIG v3.4 Section 4.1.7 describes domain splitting for large datasets.
    Supplemental qualifiers (SUPPQUAL) are defined in Section 8.4 for
    sponsor-defined variables that don't fit in standard domain structure.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Any

import pandas as pd

from ..cli.helpers import write_variant_splits
from ..cli.logging_config import get_logger

if TYPE_CHECKING:
    from ..metadata_module import StudyMetadata
    from ..domains_module import SDTMDomain

from ..domains_module import SDTMDomain, get_domain, get_domain_class
from ..io_module import build_column_hints, load_input_dataset
from ..mapping_module import (
    ColumnMapping,
    MappingConfig,
    build_config,
    create_mapper,
)
from ..sas_module import generate_sas_program, write_sas_file
from ..submission_module import build_suppqual, extract_used_columns
from ..transformations.base import TransformationContext
from ..transformations.findings import VSTransformer
from ..terminology_module import normalize_testcd, get_testcd_label
from ..xpt_module import write_xpt_file
from ..xpt_module.builder import build_domain_dataframe
from ..xml_module.dataset_module import write_dataset_xml
from .study_orchestration_service import StudyOrchestrationService


class DomainProcessingCoordinator:
    """Coordinates the processing of domain files through the full workflow."""

    def __init__(self, orchestration_service: StudyOrchestrationService | None = None):
        """Initialize the domain processing coordinator.

        Args:
            orchestration_service: Optional orchestration service for domain transformations
        """
        self.orchestration_service = (
            orchestration_service or StudyOrchestrationService()
        )

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

            frame, config, _is_lb_long = processed_data
            all_dataframes.append(frame)
            variant_frames.append((variant_name or domain_code, frame))
            last_config = config

            # Build supplemental qualifiers using submission_module
            if domain_code.upper() != "LB":
                used_columns = extract_used_columns(config)
                supp_df, _ = build_suppqual(
                    domain_code,
                    load_input_dataset(input_file),
                    frame,
                    used_columns,
                    study_id=study_id,
                    common_column_counts=common_column_counts,
                    total_files=total_input_files,
                )
                if supp_df is not None and not supp_df.empty:
                    supp_frames.append(supp_df)

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
    ) -> tuple[pd.DataFrame, MappingConfig, bool] | None:
        """Process a single input file.

        Returns:
            Tuple of (dataframe, config, is_lb_long) or None if file should be skipped
        """

        # Get global logger for stats tracking
        logger = get_logger()

        display_name = (
            f"{domain_code}"
            if variant_name == domain_code
            else f"{domain_code} ({variant_name})"
        )

        # Get domain class for context (side effect: validate domain code)
        _ = get_domain_class(domain_code)

        # Load input data
        frame = load_input_dataset(input_file)
        row_count = len(frame)
        col_count = len(frame.columns)

        # Log file loading and update stats
        logger.log_file_loaded(input_file.name, row_count, col_count)

        # Log column names at verbose level
        if verbose and row_count > 0:
            col_names = ", ".join(frame.columns[:10].tolist())
            if len(frame.columns) > 10:
                col_names += f" ... (+{len(frame.columns) - 10} more)"
            logger.verbose(f"    Columns: {col_names}")

        # Skip VSTAT helper files - these are operational vital signs files
        # used for data preparation but not part of SDTM submission
        is_vstat = (
            domain_code.upper() == "VS"
            and variant_name
            and "VSTAT" in variant_name.upper()
        )
        if is_vstat:
            if verbose:
                logger.verbose(
                    f"  Skipping {input_file.name} (VSTAT is an operational helper file, not an SDTM domain)"
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
            config: MappingConfig = self._build_identity_config(domain_code, frame)

            if verbose:
                logger.verbose("    Using identity mapping (post-transformation)")
        else:
            mapped_config = self._build_mapped_config(
                domain_code, frame, metadata, min_confidence, display_name
            )
            if mapped_config is None:
                return None
            config = mapped_config

            # Log mapping summary - safely get mapping count
            mapping_count = len(getattr(config, "mappings", []))
            if verbose:
                logger.verbose(f"    Column mappings: {mapping_count} variables mapped")

        config.study_id = study_id

        # Build domain dataframe

        domain_dataframe = build_domain_dataframe(
            frame,
            config,
            lenient=True,
            metadata=metadata,
            reference_starts=None,
        )

        output_rows = len(domain_dataframe)
        logger.log_rows_processed(domain_code, output_rows, variant_name)

        # Log transformation summary if rows changed
        if output_rows != row_count and verbose:
            change_pct = (
                ((output_rows - row_count) / row_count * 100) if row_count > 0 else 0
            )
            direction = "+" if change_pct > 0 else ""
            logger.verbose(
                f"    Row count changed: {row_count:,} → {output_rows:,} ({direction}{change_pct:.1f}%)"
            )

        return domain_dataframe, config, lb_long

    def _apply_vs_transformation(
        self,
        frame: pd.DataFrame,
        domain_code: str,
        study_id: str,
        display_name: str,
        verbose: bool,
    ) -> tuple[pd.DataFrame | None, bool]:
        """Apply VS domain transformation if needed.

        This handles wide-to-long transformation for Vital Signs data per SDTMIG v3.4.
        VS domain requires one record per vital sign measurement per time point.
        
        Uses the new VSTransformer for consistent, testable transformation logic.
        """

        if domain_code.upper() != "VS":
            return frame, False

        logger = get_logger()
        input_rows = len(frame)
        
        # Use new VSTransformer instead of orchestration_service
        transformer = VSTransformer(
            test_code_normalizer=normalize_testcd,
            test_label_getter=get_testcd_label
        )
        context = TransformationContext(domain="VS", study_id=study_id)
        result = transformer.transform(frame, context)
        
        if not result.success:
            logger.warning(f"{display_name}: VS transformation failed: {result.message}")
            if result.errors:
                for error in result.errors:
                    logger.error(f"  - {error}")
            return None, True
        
        frame = result.data
        output_rows = len(frame)

        # Enhanced logging for VS transformation
        logger.log_transformation(
            domain_code, "reshape", input_rows, output_rows, details="wide-to-long"
        )

        if frame.empty:
            logger.warning(f"{display_name}: No vital signs records after transformation")
            if verbose:
                logger.verbose("    Note: Check source data for VSTESTCD/VSORRES columns")
            return None, True

        return frame, True

    def _apply_lb_transformation(
        self,
        frame: pd.DataFrame,
        domain_code: str,
        study_id: str,
        display_name: str,
        verbose: bool,
    ) -> tuple[pd.DataFrame | None, bool]:
        """Apply LB domain transformation if needed.

        This handles wide-to-long transformation for Laboratory data per SDTMIG v3.4.
        LB domain requires one record per lab test per time point per visit per subject.
        """

        if domain_code.upper() != "LB":
            return frame, False

        logger = get_logger()
        input_rows = len(frame)
        reshaped = self.orchestration_service.reshape_lb_to_long(frame, study_id)

        if "LBTESTCD" in reshaped.columns:
            output_rows = len(reshaped)

            # Count unique tests for context
            unique_tests = (
                reshaped["LBTESTCD"].nunique() if "LBTESTCD" in reshaped.columns else 0
            )

            # Enhanced logging for LB transformation using logger
            logger.log_transformation(
                domain_code,
                "reshape",
                input_rows,
                output_rows,
                details=f"wide-to-long, {unique_tests} test codes",
            )

            if reshaped.empty:
                logger.warning(f"{display_name}: No laboratory records after transformation")
                if verbose:
                    logger.verbose("    Note: Check source data for lab test columns")
                return None, True

            return reshaped, True
        else:
            if verbose:
                logger.verbose("  Skipping LB reshape (no recognizable test columns found)")
                logger.verbose("    Expected columns like: WBC, RBC, HGB, or LBTESTCD")
            return None, False

    def _build_identity_config(
        self, domain_code: str, frame: pd.DataFrame
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
        return build_config(domain_code, mappings)

    def _build_mapped_config(
        self,
        domain_code: str,
        frame: pd.DataFrame,
        metadata: StudyMetadata | None,
        min_confidence: float,
        display_name: str,
    ) -> MappingConfig | None:
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
            logger = get_logger()
            logger.warning(f"{display_name}: No mappings found, skipping")
            return None

        return build_config(domain_code, suggestions.mappings)

    def _merge_dataframes(
        self, all_dataframes: list[pd.DataFrame], domain_code: str, verbose: bool
    ) -> pd.DataFrame:
        """Merge multiple dataframes and re-sequence.

        When multiple variant files are merged (e.g., LBCC + LBHM), sequence
        numbers are reassigned to maintain uniqueness per SDTMIG v3.4 requirements.
        """

        if len(all_dataframes) == 1:
            return all_dataframes[0]

        # Calculate totals for logging
        input_rows_list = [len(df) for df in all_dataframes]
        total_input = sum(input_rows_list)

        merged_dataframe = pd.concat(all_dataframes, ignore_index=True)
        merged_rows = len(merged_dataframe)

        # Re-assign sequence numbers per subject after merge
        seq_col = f"{domain_code}SEQ"
        if (
            seq_col in merged_dataframe.columns
            and "USUBJID" in merged_dataframe.columns
        ):
            merged_dataframe[seq_col] = (
                merged_dataframe.groupby("USUBJID").cumcount() + 1
            )

            if verbose:
                logger = get_logger()
                logger.verbose(f"    Reassigned {seq_col} values after merge")

        # Enhanced merge logging
        if verbose:
            logger = get_logger()
            logger.verbose(
                f"Merged {len(all_dataframes)} files: {total_input:,} → {merged_rows:,} rows"
            )
            # Log individual file contributions
            for i, rows in enumerate(input_rows_list):
                pct = (rows / merged_rows * 100) if merged_rows > 0 else 0
                logger.verbose(f"    File {i + 1}: {rows:,} rows ({pct:.1f}%)")

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
            if (
                seq_col in merged_dataframe.columns
                and "USUBJID" in merged_dataframe.columns
            ):
                merged_dataframe[seq_col] = (
                    merged_dataframe.groupby("USUBJID").cumcount() + 1
                )
        return merged_dataframe

    def _generate_output_files(
        self,
        merged_dataframe: pd.DataFrame,
        domain_code: str,
        study_id: str,
        config: MappingConfig,
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
        domain: SDTMDomain,
        verbose: bool,
    ) -> dict[str, Any]:
        """Generate output files and return processing results."""

        base_filename = domain.resolved_dataset_name()
        disk_name = base_filename.lower()

        result: dict[str, Any] = {
            "domain_code": domain_code,
            "records": len(merged_dataframe),
            "domain_dataframe": merged_dataframe,
            "config": config,
            "xpt_path": None,
            "xml_path": None,
            "sas_path": None,
            "split_xpt_paths": [],
            "split_datasets": [],  # List of (name, dataframe, path) tuples for Define-XML
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

            logger = get_logger()
            logger.success(f"Generated XPT: {xpt_path}")

            # Handle domain variant splits (SDTMIG v3.4 Section 4.1.7)
            # Any domain can be split when there are multiple variant files
            if len(variant_frames) > 1:
                split_paths, split_datasets = write_variant_splits(
                    variant_frames, domain, xpt_dir
                )
                result["split_xpt_paths"] = split_paths
                result["split_datasets"] = split_datasets

        if xml_dir and output_format in ("xml", "both"):
            xml_path = xml_dir / f"{disk_name}.xml"
            if streaming:
                logger = get_logger()
                logger.warning("Streaming mode not implemented, using regular write")
            write_dataset_xml(merged_dataframe, domain_code, config, xml_path)
            result["xml_path"] = xml_path
            result["xml_filename"] = xml_path.name

            logger = get_logger()
            logger.success(f"Generated Dataset-XML: {xml_path}")

        if sas_dir and generate_sas:
            sas_path = sas_dir / f"{disk_name}.sas"
            first_input_file = files_for_domain[0][0]
            sas_code = generate_sas_program(
                domain_code, config, first_input_file.stem, base_filename
            )
            write_sas_file(sas_code, sas_path)
            result["sas_path"] = sas_path

            logger = get_logger()
            logger.success(f"Generated SAS: {sas_path}")

        return result

    def _generate_supplemental_files(
        self,
        supp_frames: list[pd.DataFrame],
        domain_code: str,
        study_id: str,
        output_format: str,
        xpt_dir: Path | None,
        xml_dir: Path | None,
    ) -> dict[str, Any]:
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

        supp_result: dict[str, Any] = {
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
            write_xpt_file(
                merged_supp, supp_domain_code, xpt_path, file_label=file_label
            )
            supp_result["xpt_path"] = xpt_path
            supp_result["xpt_filename"] = xpt_path.name

        if xml_dir and output_format in ("xml", "both"):
            xml_path = xml_dir / f"{disk_name}.xml"
            write_dataset_xml(merged_supp, supp_domain_code, supp_config, xml_path)
            supp_result["xml_path"] = xml_path
            supp_result["xml_filename"] = xml_path.name

        return supp_result
