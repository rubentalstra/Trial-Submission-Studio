"""Domain processing use case.

This module contains the use case for processing a single SDTM domain,
orchestrating file loading, transformations, mapping, and output generation.

The use case implements a clean pipeline architecture with explicit stages:
1. Load input files via StudyDataRepositoryPort
2. Apply transformations via TransformationPipeline (VS/LB)
3. Map columns via mapping service/engine
4. Build SDTM domain dataframe
5. Generate SUPPQUAL (supplemental qualifiers)
6. Generate outputs via FileGeneratorPort

CLEAN2-D1: This use case is now fully implemented with injected dependencies,
removing the delegation to legacy DomainProcessingCoordinator.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Any, Callable

import pandas as pd

from .models import ProcessDomainRequest, ProcessDomainResponse
from .ports import FileGeneratorPort, LoggerPort, StudyDataRepositoryPort

if TYPE_CHECKING:
    from ..domain.entities.sdtm_domain import SDTMDomain
    from ..mapping_module import MappingConfig
    from ..transformations.base import TransformationContext


def _get_transformation_helpers() -> tuple[type, Callable[[str], str], Callable[[str], str]]:
    """Lazy import of transformation dependencies to avoid circular imports.
    
    Returns:
        Tuple of (TransformationContext class, normalize_testcd function, get_testcd_label function)
    """
    from ..transformations.base import TransformationContext
    from ..terminology_module import normalize_testcd, get_testcd_label
    return TransformationContext, normalize_testcd, get_testcd_label


class DomainProcessingUseCase:
    """Use case for processing a single SDTM domain.
    
    This class orchestrates domain-level processing through clear pipeline stages:
    1. Load files stage - Load and validate input files
    2. Transform stage - Apply domain-specific transformations (VS, LB)
    3. Map columns stage - Map source columns to SDTM variables
    4. Build domain stage - Create final domain dataframe
    5. Generate SUPPQUAL stage - Create supplemental qualifiers
    6. Generate outputs stage - Create XPT/XML/SAS files
    
    The use case follows the Ports & Adapters architecture with dependencies
    injected via the constructor.
    
    Example:
        >>> use_case = DomainProcessingUseCase(
        ...     logger=my_logger,
        ...     study_data_repo=repo,
        ...     file_generator=generator,
        ... )
        >>> request = ProcessDomainRequest(
        ...     files_for_domain=[(Path("DM.csv"), "DM")],
        ...     domain_code="DM",
        ...     study_id="STUDY001",
        ...     output_formats={"xpt", "xml"},
        ...     output_dirs={"xpt": Path("output/xpt")},
        ... )
        >>> response = use_case.execute(request)
        >>> if response.success:
        ...     print(f"Processed {response.records} records")
    """
    
    def __init__(
        self,
        logger: LoggerPort,
        study_data_repo: StudyDataRepositoryPort | None = None,
        file_generator: FileGeneratorPort | None = None,
    ):
        """Initialize the use case with injected dependencies.
        
        Args:
            logger: Logger for progress and error reporting
            study_data_repo: Repository for loading study data files
            file_generator: Generator for output files (XPT, XML, SAS)
        """
        self.logger = logger
        self._study_data_repo = study_data_repo
        self._file_generator = file_generator
    
    def execute(self, request: ProcessDomainRequest) -> ProcessDomainResponse:
        """Execute the domain processing workflow.
        
        This method orchestrates the complete domain processing pipeline:
        - Loads input files
        - Applies transformations
        - Maps columns
        - Builds domain dataframe
        - Generates SUPPQUAL
        - Generates output files
        
        Args:
            request: Domain processing request with all parameters
            
        Returns:
            Domain processing response with results and any errors
            
        Example:
            >>> response = use_case.execute(request)
            >>> print(f"Success: {response.success}")
            >>> print(f"Records: {response.records}")
            >>> print(f"Errors: {response.error}")
        """
        response = ProcessDomainResponse(domain_code=request.domain_code)
        
        try:
            # Get domain definition
            domain = self._get_domain(request.domain_code)
            
            # Track all processed dataframes and configs for merging
            all_dataframes: list[pd.DataFrame] = []
            variant_frames: list[tuple[str, pd.DataFrame]] = []
            last_config: MappingConfig | None = None
            supp_frames: list[pd.DataFrame] = []
            
            # Process each input file
            for input_file, variant_name in request.files_for_domain:
                result = self._process_single_file(
                    input_file=input_file,
                    variant_name=variant_name,
                    request=request,
                    domain=domain,
                )
                
                if result is None:
                    continue
                
                frame, config, is_findings_long = result
                all_dataframes.append(frame)
                variant_frames.append((variant_name or request.domain_code, frame))
                last_config = config
                
                # Build SUPPQUAL for non-LB domains
                if request.domain_code.upper() != "LB":
                    supp_df = self._generate_suppqual_stage(
                        source_df=self._load_file(input_file),
                        domain_df=frame,
                        config=config,
                        domain=domain,
                        request=request,
                    )
                    if supp_df is not None and not supp_df.empty:
                        supp_frames.append(supp_df)
            
            if not all_dataframes:
                raise ValueError(f"No data could be processed for {request.domain_code}")
            
            if last_config is None:
                raise RuntimeError("Config should be set if we have dataframes")
            
            # Merge dataframes if multiple files
            merged_df = self._merge_dataframes(
                all_dataframes, request.domain_code, request.verbose > 0
            )
            
            # Deduplicate LB data
            if request.domain_code.upper() == "LB":
                merged_df = self._deduplicate_lb_data(merged_df, request.domain_code)
            
            # Generate output files
            output_result = self._generate_outputs_stage(
                merged_df=merged_df,
                config=last_config,
                domain=domain,
                request=request,
                supp_frames=supp_frames,
                variant_frames=variant_frames,
            )
            
            # Populate response
            response.success = True
            response.records = len(merged_df)
            response.domain_dataframe = merged_df
            response.config = last_config
            response.xpt_path = output_result.get("xpt_path")
            response.xml_path = output_result.get("xml_path")
            response.sas_path = output_result.get("sas_path")
            response.split_datasets = output_result.get("split_datasets", [])
            
            # Handle supplemental domains
            for supp_dict in output_result.get("supplementals", []):
                supp_response = ProcessDomainResponse(
                    success=True,
                    domain_code=supp_dict.get("domain_code", ""),
                    records=supp_dict.get("records", 0),
                    domain_dataframe=supp_dict.get("domain_dataframe"),
                    config=supp_dict.get("config"),
                    xpt_path=supp_dict.get("xpt_path"),
                    xml_path=supp_dict.get("xml_path"),
                    sas_path=supp_dict.get("sas_path"),
                )
                response.supplementals.append(supp_response)
            
        except Exception as exc:
            response.success = False
            response.error = str(exc)
            self.logger.error(f"{request.domain_code}: {exc}")
        
        return response
    
    # ========== Pipeline Stages ==========
    
    def _process_single_file(
        self,
        input_file: Path,
        variant_name: str,
        request: ProcessDomainRequest,
        domain: SDTMDomain,
    ) -> tuple[pd.DataFrame, MappingConfig, bool] | None:
        """Process a single input file through the pipeline.
        
        Returns:
            Tuple of (dataframe, config, is_findings_long) or None if file should be skipped
        """
        display_name = (
            f"{request.domain_code}"
            if variant_name == request.domain_code
            else f"{request.domain_code} ({variant_name})"
        )
        
        # Stage 1: Load input file
        frame = self._load_file(input_file)
        row_count = len(frame)
        col_count = len(frame.columns)
        
        # Log file loading
        self.logger.info(f"Loaded {input_file.name}: {row_count:,} rows, {col_count} columns")
        
        if request.verbose > 0 and row_count > 0:
            col_names = ", ".join(frame.columns[:10].tolist())
            if len(frame.columns) > 10:
                col_names += f" ... (+{len(frame.columns) - 10} more)"
            self.logger.verbose(f"    Columns: {col_names}")
        
        # Skip VSTAT helper files (VS domain operational files)
        if self._should_skip_vstat(request.domain_code, variant_name, request.verbose > 0):
            return None
        
        # Stage 2: Apply domain-specific transformations
        frame, vs_long = self._apply_vs_transformation(
            frame, request.domain_code, request.study_id, display_name, request.verbose > 0
        )
        if frame is None:
            return None
        
        frame, lb_long = self._apply_lb_transformation(
            frame, request.domain_code, request.study_id, display_name, request.verbose > 0
        )
        if frame is None:
            return None
        
        is_findings_long = vs_long or lb_long
        
        # Stage 3: Map columns
        config = self._build_config(
            frame=frame,
            domain_code=request.domain_code,
            metadata=request.metadata,
            min_confidence=request.min_confidence,
            is_findings_long=is_findings_long,
            display_name=display_name,
            verbose=request.verbose > 0,
        )
        if config is None:
            return None
        
        config.study_id = request.study_id
        
        # Stage 4: Build domain dataframe
        domain_df = self._build_domain_dataframe(
            frame=frame,
            config=config,
            domain=domain,
            metadata=request.metadata,
            reference_starts=request.reference_starts,
        )
        
        output_rows = len(domain_df)
        self.logger.info(f"{request.domain_code}: {output_rows:,} rows processed")
        
        if output_rows != row_count and request.verbose > 0:
            change_pct = ((output_rows - row_count) / row_count * 100) if row_count > 0 else 0
            direction = "+" if change_pct > 0 else ""
            self.logger.verbose(
                f"    Row count changed: {row_count:,} → {output_rows:,} ({direction}{change_pct:.1f}%)"
            )
        
        return domain_df, config, is_findings_long
    
    def _load_file(self, file_path: Path) -> pd.DataFrame:
        """Stage 1: Load and validate input file."""
        if self._study_data_repo is not None:
            return self._study_data_repo.read_dataset(file_path)
        
        # Fallback to io_module if repository not injected
        from ..io_module import load_input_dataset
        return load_input_dataset(file_path)
    
    def _should_skip_vstat(
        self, domain_code: str, variant_name: str | None, verbose: bool
    ) -> bool:
        """Check if file should be skipped (VSTAT operational helper)."""
        if (
            domain_code.upper() == "VS"
            and variant_name
            and "VSTAT" in variant_name.upper()
        ):
            if verbose:
                self.logger.verbose(
                    f"  Skipping {variant_name} (VSTAT is an operational helper file, not an SDTM domain)"
                )
            return True
        return False
    
    def _apply_vs_transformation(
        self,
        frame: pd.DataFrame,
        domain_code: str,
        study_id: str,
        display_name: str,
        verbose: bool,
    ) -> tuple[pd.DataFrame | None, bool]:
        """Stage 2a: Apply VS domain transformation if needed."""
        if domain_code.upper() != "VS":
            return frame, False
        
        from ..transformations.findings import VSTransformer
        
        TransformationContext, normalize_testcd, get_testcd_label = _get_transformation_helpers()
        
        input_rows = len(frame)
        
        transformer = VSTransformer(
            test_code_normalizer=normalize_testcd,
            test_label_getter=get_testcd_label,
        )
        context = TransformationContext(domain="VS", study_id=study_id)
        result = transformer.transform(frame, context)
        
        if not result.success:
            self.logger.warning(f"{display_name}: VS transformation failed: {result.message}")
            if result.errors:
                for error in result.errors:
                    self.logger.error(f"  - {error}")
            return None, True
        
        frame = result.data
        output_rows = len(frame)
        
        self.logger.info(
            f"{domain_code}: reshape transformation {input_rows:,} → {output_rows:,} rows (wide-to-long)"
        )
        
        if frame.empty:
            self.logger.warning(f"{display_name}: No vital signs records after transformation")
            if verbose:
                self.logger.verbose("    Note: Check source data for VSTESTCD/VSORRES columns")
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
        """Stage 2b: Apply LB domain transformation if needed."""
        if domain_code.upper() != "LB":
            return frame, False
        
        from ..transformations.findings import LBTransformer
        
        TransformationContext, normalize_testcd, get_testcd_label = _get_transformation_helpers()
        
        input_rows = len(frame)
        
        transformer = LBTransformer(
            test_code_normalizer=normalize_testcd,
            test_label_getter=get_testcd_label,
        )
        context = TransformationContext(domain="LB", study_id=study_id)
        result = transformer.transform(frame, context)
        
        if not result.success:
            self.logger.warning(f"{display_name}: LB transformation failed: {result.message}")
            if result.errors:
                for error in result.errors:
                    self.logger.error(f"  - {error}")
            return None, True
        
        reshaped = result.data
        
        if "LBTESTCD" in reshaped.columns:
            output_rows = len(reshaped)
            unique_tests = reshaped["LBTESTCD"].nunique()
            
            self.logger.info(
                f"{domain_code}: reshape transformation {input_rows:,} → {output_rows:,} rows "
                f"(wide-to-long, {unique_tests} test codes)"
            )
            
            if reshaped.empty:
                self.logger.warning(f"{display_name}: No laboratory records after transformation")
                if verbose:
                    self.logger.verbose("    Note: Check source data for lab test columns")
                return None, True
            
            return reshaped, True
        else:
            if verbose:
                self.logger.verbose("  Skipping LB reshape (no recognizable test columns found)")
                self.logger.verbose("    Expected columns like: WBC, RBC, HGB, or LBTESTCD")
            return None, False
    
    def _build_config(
        self,
        frame: pd.DataFrame,
        domain_code: str,
        metadata: Any,
        min_confidence: float,
        is_findings_long: bool,
        display_name: str,
        verbose: bool,
    ) -> MappingConfig | None:
        """Stage 3: Build mapping configuration."""
        from ..mapping_module import (
            ColumnMapping,
            MappingConfig,
            build_config,
            create_mapper,
        )
        from ..io_module import build_column_hints
        
        if is_findings_long:
            # Use identity mapping for post-transformation data
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
            
            if verbose:
                self.logger.verbose("    Using identity mapping (post-transformation)")
            
            return config
        
        # Build mapped configuration using fuzzy matching
        column_hints = build_column_hints(frame)
        engine = create_mapper(
            domain_code,
            metadata=metadata,
            min_confidence=min_confidence,
            column_hints=column_hints,
        )
        suggestions = engine.suggest(frame)
        
        if not suggestions.mappings:
            self.logger.warning(f"{display_name}: No mappings found, skipping")
            return None
        
        config = build_config(domain_code, suggestions.mappings)
        
        if verbose:
            mapping_count = len(config.mappings) if config.mappings else 0
            self.logger.verbose(f"    Column mappings: {mapping_count} variables mapped")
        
        return config
    
    def _build_domain_dataframe(
        self,
        frame: pd.DataFrame,
        config: MappingConfig,
        domain: SDTMDomain,
        metadata: Any,
        reference_starts: dict[str, str] | None,
    ) -> pd.DataFrame:
        """Stage 4: Build SDTM domain dataframe."""
        from ..domain.services import build_domain_dataframe
        
        return build_domain_dataframe(
            frame,
            config,
            domain,
            lenient=True,
            metadata=metadata,
            reference_starts=reference_starts,
        )
    
    def _generate_suppqual_stage(
        self,
        source_df: pd.DataFrame,
        domain_df: pd.DataFrame,
        config: MappingConfig,
        domain: SDTMDomain,
        request: ProcessDomainRequest,
    ) -> pd.DataFrame | None:
        """Stage 5: Generate supplemental qualifiers."""
        from ..domain.services import build_suppqual, extract_used_columns
        
        used_columns = extract_used_columns(config)
        supp_df, _ = build_suppqual(
            request.domain_code,
            source_df,
            domain_df,
            domain,
            used_columns,
            study_id=request.study_id,
            common_column_counts=request.common_column_counts,
            total_files=request.total_input_files,
        )
        return supp_df
    
    def _generate_outputs_stage(
        self,
        merged_df: pd.DataFrame,
        config: MappingConfig,
        domain: SDTMDomain,
        request: ProcessDomainRequest,
        supp_frames: list[pd.DataFrame],
        variant_frames: list[tuple[str, pd.DataFrame]],
    ) -> dict[str, Any]:
        """Stage 6: Generate output files (XPT, XML, SAS)."""
        from ..xpt_module import write_xpt_file
        from ..xml_module.dataset_module import write_dataset_xml
        from ..sas_module import generate_sas_program, write_sas_file
        
        base_filename = domain.resolved_dataset_name()
        disk_name = base_filename.lower()
        
        result: dict[str, Any] = {
            "domain_code": request.domain_code,
            "records": len(merged_df),
            "domain_dataframe": merged_df,
            "config": config,
            "xpt_path": None,
            "xml_path": None,
            "sas_path": None,
            "split_xpt_paths": [],
            "split_datasets": [],
            "supplementals": [],
        }
        
        # Handle supplemental qualifiers
        if supp_frames:
            supp_result = self._generate_supplemental_files(
                supp_frames, request.domain_code, request.study_id,
                request.output_formats, request.output_dirs
            )
            result["supplementals"].append(supp_result)
        
        xpt_dir = request.output_dirs.get("xpt")
        xml_dir = request.output_dirs.get("xml")
        sas_dir = request.output_dirs.get("sas")
        output_format = "/".join(request.output_formats)
        
        # Generate XPT file
        if xpt_dir and output_format in ("xpt", "xpt/xml", "xml/xpt", "both"):
            xpt_path = xpt_dir / f"{disk_name}.xpt"
            write_xpt_file(merged_df, request.domain_code, xpt_path)
            result["xpt_path"] = xpt_path
            result["xpt_filename"] = xpt_path.name
            self.logger.success(f"Generated XPT: {xpt_path}")
            
            # Handle domain variant splits (SDTMIG v3.4 Section 4.1.7)
            if len(variant_frames) > 1:
                split_paths, split_datasets = self._write_variant_splits(
                    variant_frames, domain, xpt_dir
                )
                result["split_xpt_paths"] = split_paths
                result["split_datasets"] = split_datasets
        
        # Generate Dataset-XML file
        if xml_dir and output_format in ("xml", "xpt/xml", "xml/xpt", "both"):
            xml_path = xml_dir / f"{disk_name}.xml"
            if request.streaming:
                self.logger.warning("Streaming mode not implemented, using regular write")
            write_dataset_xml(merged_df, request.domain_code, config, xml_path)
            result["xml_path"] = xml_path
            result["xml_filename"] = xml_path.name
            self.logger.success(f"Generated Dataset-XML: {xml_path}")
        
        # Generate SAS program
        if sas_dir and request.generate_sas:
            sas_path = sas_dir / f"{disk_name}.sas"
            first_input_file = request.files_for_domain[0][0]
            sas_code = generate_sas_program(
                request.domain_code, config, first_input_file.stem, base_filename
            )
            write_sas_file(sas_code, sas_path)
            result["sas_path"] = sas_path
            self.logger.success(f"Generated SAS: {sas_path}")
        
        return result
    
    def _generate_supplemental_files(
        self,
        supp_frames: list[pd.DataFrame],
        domain_code: str,
        study_id: str,
        output_formats: set[str],
        output_dirs: dict[str, Path | None],
    ) -> dict[str, Any]:
        """Generate supplemental qualifier files."""
        from ..xpt_module import write_xpt_file
        from ..xml_module.dataset_module import write_dataset_xml
        from ..mapping_module import ColumnMapping, build_config
        
        merged_supp = (
            supp_frames[0]
            if len(supp_frames) == 1
            else pd.concat(supp_frames, ignore_index=True)
        )
        
        supp_domain_code = f"SUPP{domain_code.upper()}"
        
        # Build identity config for SUPPQUAL
        mappings = [
            ColumnMapping(
                source_column=col,
                target_variable=col,
                transformation=None,
                confidence_score=1.0,
            )
            for col in merged_supp.columns
        ]
        supp_config = build_config(supp_domain_code, mappings)
        supp_config.study_id = study_id
        
        supp_domain = self._get_domain(supp_domain_code)
        base_filename = supp_domain.resolved_dataset_name()
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
        
        xpt_dir = output_dirs.get("xpt")
        xml_dir = output_dirs.get("xml")
        output_format = "/".join(output_formats)
        
        if xpt_dir and output_format in ("xpt", "xpt/xml", "xml/xpt", "both"):
            xpt_path = xpt_dir / f"{disk_name}.xpt"
            file_label = f"Supplemental Qualifiers for {domain_code.upper()}"
            write_xpt_file(merged_supp, supp_domain_code, xpt_path, file_label=file_label)
            supp_result["xpt_path"] = xpt_path
            supp_result["xpt_filename"] = xpt_path.name
        
        if xml_dir and output_format in ("xml", "xpt/xml", "xml/xpt", "both"):
            xml_path = xml_dir / f"{disk_name}.xml"
            write_dataset_xml(merged_supp, supp_domain_code, supp_config, xml_path)
            supp_result["xml_path"] = xml_path
            supp_result["xml_filename"] = xml_path.name
        
        return supp_result
    
    def _write_variant_splits(
        self,
        variant_frames: list[tuple[str, pd.DataFrame]],
        domain: SDTMDomain,
        xpt_dir: Path,
    ) -> tuple[list[Path], list[tuple[str, pd.DataFrame, Path]]]:
        """Write split XPT files for domain variants.
        
        Per SDTMIG v3.4 Section 4.1.7 "Splitting Domains".
        """
        from ..xpt_module import write_xpt_file
        
        split_paths: list[Path] = []
        split_datasets: list[tuple[str, pd.DataFrame, Path]] = []
        domain_code = domain.code.upper()
        
        for variant_name, variant_df in variant_frames:
            table = variant_name.replace(" ", "_").replace("(", "").replace(")", "").upper()
            
            if table == domain_code:
                continue
            
            if not table.startswith(domain_code):
                self.logger.warning(
                    f"Warning: Split dataset '{table}' does not start "
                    f"with domain code '{domain_code}'. Skipping."
                )
                continue
            
            if len(table) > 8:
                self.logger.warning(
                    f"Warning: Split dataset name '{table}' exceeds "
                    "8 characters. Truncating to comply with SDTMIG v3.4."
                )
                table = table[:8]
            
            if "DOMAIN" in variant_df.columns:
                variant_df = variant_df.copy()
                variant_df["DOMAIN"] = domain_code
            
            split_dir = xpt_dir / "split"
            split_dir.mkdir(parents=True, exist_ok=True)
            
            split_name = table.lower()
            split_path = split_dir / f"{split_name}.xpt"
            
            split_suffix = table[len(domain_code):]
            file_label = (
                f"{domain.description} - {split_suffix}"
                if split_suffix
                else domain.description
            )
            
            write_xpt_file(
                variant_df, domain.code, split_path, file_label=file_label, table_name=table
            )
            split_paths.append(split_path)
            split_datasets.append((table, variant_df, split_path))
            self.logger.success(
                f"Split dataset: {split_path} (DOMAIN={domain_code}, table={table})"
            )
        
        return split_paths, split_datasets
    
    # ========== Helper Methods ==========
    
    def _get_domain(self, domain_code: str) -> SDTMDomain:
        """Get SDTM domain definition."""
        from ..domains_module import get_domain
        return get_domain(domain_code)
    
    def _merge_dataframes(
        self, all_dataframes: list[pd.DataFrame], domain_code: str, verbose: bool
    ) -> pd.DataFrame:
        """Merge multiple dataframes and re-sequence."""
        if len(all_dataframes) == 1:
            return all_dataframes[0]
        
        input_rows_list = [len(df) for df in all_dataframes]
        total_input = sum(input_rows_list)
        
        merged_df = pd.concat(all_dataframes, ignore_index=True)
        merged_rows = len(merged_df)
        
        # Re-assign sequence numbers per subject after merge
        seq_col = f"{domain_code}SEQ"
        if seq_col in merged_df.columns and "USUBJID" in merged_df.columns:
            merged_df[seq_col] = merged_df.groupby("USUBJID").cumcount() + 1
            
            if verbose:
                self.logger.verbose(f"    Reassigned {seq_col} values after merge")
        
        if verbose:
            self.logger.verbose(
                f"Merged {len(all_dataframes)} files: {total_input:,} → {merged_rows:,} rows"
            )
            for i, rows in enumerate(input_rows_list):
                pct = (rows / merged_rows * 100) if merged_rows > 0 else 0
                self.logger.verbose(f"    File {i + 1}: {rows:,} rows ({pct:.1f}%)")
        
        return merged_df
    
    def _deduplicate_lb_data(
        self, merged_df: pd.DataFrame, domain_code: str
    ) -> pd.DataFrame:
        """Deduplicate LB data to avoid SD1152 issues."""
        dedup_keys = [
            key for key in ("USUBJID", "LBTESTCD", "LBDTC")
            if key in merged_df.columns
        ]
        if dedup_keys:
            merged_df = (
                merged_df.copy()
                .sort_values(by=dedup_keys)
                .drop_duplicates(subset=dedup_keys, keep="first")
                .reset_index(drop=True)
            )
            seq_col = f"{domain_code}SEQ"
            if seq_col in merged_df.columns and "USUBJID" in merged_df.columns:
                merged_df[seq_col] = merged_df.groupby("USUBJID").cumcount() + 1
        return merged_df
