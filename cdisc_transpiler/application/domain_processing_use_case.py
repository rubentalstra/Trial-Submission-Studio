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
from typing import TYPE_CHECKING, Any

import pandas as pd

from .models import ProcessDomainRequest, ProcessDomainResponse
from .ports import (
    CTRepositoryPort,
    DomainDefinitionRepositoryPort,
    DomainFrameBuilderPort,
    FileGeneratorPort,
    LoggerPort,
    MappingPort,
    OutputPreparerPort,
    SuppqualPort,
    StudyDataRepositoryPort,
    TerminologyPort,
    XPTWriterPort,
)

if TYPE_CHECKING:
    from ..domain.entities.sdtm_domain import SDTMDomain
    from ..domain.entities.mapping import MappingConfig

from ..domain.entities.column_hints import ColumnHint, Hints


def _build_column_hints(frame: pd.DataFrame) -> Hints:
    """Derive lightweight per-column hints used by mapping heuristics."""
    hints: dict[str, ColumnHint] = {}
    row_count = len(frame)
    for column in frame.columns:
        series = frame[column]
        is_numeric = pd.api.types.is_numeric_dtype(series)
        non_null = int(series.notna().sum())
        unique_non_null = series.nunique(dropna=True)
        unique_ratio = float(unique_non_null / non_null) if non_null else 0.0
        null_ratio = float(1 - (non_null / row_count)) if row_count else 0.0
        hints[str(column)] = ColumnHint(
            is_numeric=bool(is_numeric),
            unique_ratio=unique_ratio,
            null_ratio=null_ratio,
        )
    return hints


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
        ...     study_data_repository=repo,
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
        study_data_repository: StudyDataRepositoryPort | None = None,
        file_generator: FileGeneratorPort | None = None,
        mapping_service: MappingPort | None = None,
        output_preparer: OutputPreparerPort | None = None,
        domain_frame_builder: DomainFrameBuilderPort | None = None,
        suppqual_service: SuppqualPort | None = None,
        terminology_service: TerminologyPort | None = None,
        domain_definition_repository: DomainDefinitionRepositoryPort | None = None,
        xpt_writer: XPTWriterPort | None = None,
        ct_repository: CTRepositoryPort | None = None,
    ):
        """Initialize the use case with injected dependencies.

        Args:
            logger: Logger for progress and error reporting
            study_data_repository: Repository for loading study data files
            file_generator: Generator for output files (XPT, XML, SAS)
        """
        self.logger = logger
        self._study_data_repository = study_data_repository
        self._file_generator = file_generator
        if mapping_service is None:
            raise RuntimeError(
                "MappingPort is not configured. "
                "Wire an infrastructure adapter in the composition root."
            )
        self._mapping_service = mapping_service
        self._output_preparer = output_preparer
        if domain_frame_builder is None:
            raise RuntimeError(
                "DomainFrameBuilderPort is not configured. "
                "Wire an infrastructure adapter in the composition root."
            )
        self._domain_frame_builder = domain_frame_builder
        if suppqual_service is None:
            raise RuntimeError(
                "SuppqualPort is not configured. "
                "Wire an infrastructure adapter in the composition root."
            )
        self._suppqual_service = suppqual_service
        if terminology_service is None:
            raise RuntimeError(
                "TerminologyPort is not configured. "
                "Wire an infrastructure adapter in the composition root."
            )
        self._terminology_service = terminology_service
        self._domain_definition_repository = domain_definition_repository
        self._xpt_writer = xpt_writer
        self._ct_repository = ct_repository

    def _log_conformance_report(
        self,
        *,
        frame: pd.DataFrame,
        domain: SDTMDomain,
        strict: bool,
    ) -> Any | None:
        if not strict:
            return None

        from ..domain.services.sdtm_conformance_checker import check_domain_dataframe

        ct_repo = self._ct_repository

        def _ct_resolver(variable: Any):
            if ct_repo is None:
                return None
            if getattr(variable, "codelist_code", None):
                return ct_repo.get_by_code(variable.codelist_code)
            return ct_repo.get_by_name(getattr(variable, "name", ""))

        report = check_domain_dataframe(frame, domain, ct_resolver=_ct_resolver)
        if not report.issues:
            return report

        header = (
            f"{domain.code}: conformance issues (errors={report.error_count()}, "
            f"warnings={report.warning_count()})"
        )
        # Prefer surfacing conformance problems loudly in strict mode.
        if report.has_errors():
            self.logger.error(header)
        else:
            self.logger.warning(header)

        max_lines = 20
        for issue in report.issues[:max_lines]:
            line = f"{issue.code}: {issue.message}"
            if issue.severity == "error":
                self.logger.error(line)
            else:
                self.logger.warning(line)

        if len(report.issues) > max_lines:
            remaining = len(report.issues) - max_lines
            self.logger.warning(f"{domain.code}: {remaining} more issue(s) not shown")

        return report

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

                frame, config, _is_findings_long = result
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
                raise ValueError(
                    f"No data could be processed for {request.domain_code}"
                )

            if last_config is None:
                raise RuntimeError("Config should be set if we have dataframes")

            # Merge dataframes if multiple files
            merged_df = self._merge_dataframes(
                all_dataframes, request.domain_code, request.verbose > 0
            )

            # Deduplicate LB data
            if request.domain_code.upper() == "LB":
                merged_df = self._deduplicate_lb_data(merged_df, request.domain_code)

            # Conformance checks (deterministic, strict-output only)
            # Note: strictness is based on requested strict outputs (XPT/SAS),
            # not on whether we currently have output directories wired.
            strict = ("xpt" in request.output_formats) or request.generate_sas

            report = self._log_conformance_report(
                frame=merged_df,
                domain=domain,
                strict=strict,
            )
            response.conformance_report = report

            if (
                strict
                and request.fail_on_conformance_errors
                and report is not None
                and getattr(report, "has_errors", lambda: False)()
            ):
                response.success = False
                response.records = len(merged_df)
                response.domain_dataframe = merged_df
                response.config = last_config
                response.error = (
                    f"{request.domain_code}: conformance errors present; "
                    "strict output generation aborted"
                )
                return response

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

        except Exception as exc:  # noqa: BLE001
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

        # Log file loading (and update processing stats)
        self.logger.log_file_loaded(
            input_file.name,
            row_count=row_count,
            column_count=col_count,
        )

        # Keep the existing verbose context line for continuity in output.
        self.logger.info(
            f"Loaded {input_file.name}: {row_count:,} rows, {col_count} columns"
        )

        if request.verbose > 0 and row_count > 0:
            col_names = ", ".join(frame.columns[:10].tolist())
            if len(frame.columns) > 10:
                col_names += f" ... (+{len(frame.columns) - 10} more)"
            self.logger.verbose(f"    Columns: {col_names}")

        # Skip VSTAT helper files (VS domain operational files)
        if self._should_skip_vstat(
            request.domain_code, variant_name, request.verbose > 0
        ):
            return None

        # Stage 2: Apply domain-specific wide-to-long transformations when applicable.
        # This prevents fuzzy-mapping wide EDC extracts into topic variables like *TESTCD.
        is_findings_long = False
        if request.domain_code.upper() in {"VS", "LB"}:
            from ..domain.services.wide_to_long import (
                transform_lb_wide_to_long,
                transform_vs_wide_to_long,
            )

            if request.domain_code.upper() == "VS":
                transformed = transform_vs_wide_to_long(
                    frame, study_id=request.study_id
                )
            else:
                transformed = transform_lb_wide_to_long(
                    frame, study_id=request.study_id
                )

            if transformed.is_long:
                frame = transformed.frame
                is_findings_long = True

                if request.verbose > 0:
                    self.logger.verbose(
                        f"    Applied wide-to-long transformation ({request.domain_code}: {len(frame):,} rows)"
                    )

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
        # Dataset-XML can be generated in a more permissive mode (lenient)
        # to support streaming/partial metadata scenarios; XPT/SAS should
        # be strict because they are typically validated by downstream tools.
        lenient = ("xpt" not in request.output_formats) and (not request.generate_sas)
        domain_df = self._build_domain_dataframe(
            frame=frame,
            config=config,
            domain=domain,
            metadata=request.metadata,
            reference_starts=request.reference_starts,
            lenient=lenient,
        )

        output_rows = len(domain_df)
        self.logger.info(f"{request.domain_code}: {output_rows:,} rows processed")

        if output_rows != row_count and request.verbose > 0:
            change_pct = (
                ((output_rows - row_count) / row_count * 100) if row_count > 0 else 0
            )
            direction = "+" if change_pct > 0 else ""
            self.logger.verbose(
                f"    Row count changed: {row_count:,} → {output_rows:,} ({direction}{change_pct:.1f}%)"
            )

        return domain_df, config, is_findings_long

    def _load_file(self, file_path: Path) -> pd.DataFrame:
        """Stage 1: Load and validate input file."""
        if self._study_data_repository is not None:
            return self._study_data_repository.read_dataset(file_path)

        raise RuntimeError(
            "StudyDataRepositoryPort is not configured. "
            "Wire an infrastructure adapter in the composition root."
        )

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
        from ..domain.entities.mapping import ColumnMapping, build_config

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

        # Build mapped configuration using fuzzy matching.
        column_hints = _build_column_hints(frame)
        suggestions = self._mapping_service.suggest(
            domain_code=domain_code,
            frame=frame,
            metadata=metadata,
            min_confidence=min_confidence,
            column_hints=column_hints,
        )

        if not suggestions.mappings:
            self.logger.warning(f"{display_name}: No mappings found, skipping")
            return None

        config = build_config(domain_code, suggestions.mappings)

        if verbose:
            mapping_count = len(config.mappings) if config.mappings else 0
            self.logger.verbose(
                f"    Column mappings: {mapping_count} variables mapped"
            )

        return config

    def _build_domain_dataframe(
        self,
        frame: pd.DataFrame,
        config: MappingConfig,
        domain: SDTMDomain,
        metadata: Any,
        reference_starts: dict[str, str] | None,
        *,
        lenient: bool,
    ) -> pd.DataFrame:
        """Stage 4: Build SDTM domain dataframe."""
        return self._domain_frame_builder.build_domain_dataframe(
            frame,
            config,
            domain,
            lenient=lenient,
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
        used_columns = self._suppqual_service.extract_used_columns(config)
        supp_df, _ = self._suppqual_service.build_suppqual(
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
        """Stage 6: Generate output files (XPT, XML, SAS) using FileGeneratorPort."""
        from .models import OutputDirs, OutputRequest

        base_filename = domain.resolved_dataset_name()

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
                supp_frames,
                request.domain_code,
                request.study_id,
                request.output_formats,
                request.output_dirs,
            )
            result["supplementals"].append(supp_result)

        # Use FileGeneratorPort if available
        if self._file_generator is not None:
            # Determine output formats
            formats = set()
            xpt_dir = request.output_dirs.get("xpt")
            xml_dir = request.output_dirs.get("xml")
            sas_dir = request.output_dirs.get("sas")

            if xpt_dir and "xpt" in request.output_formats:
                formats.add("xpt")
            if xml_dir and "xml" in request.output_formats:
                formats.add("xml")
            if sas_dir and request.generate_sas:
                formats.add("sas")

            if formats:
                # Determine SAS dataset names
                first_input_file = (
                    request.files_for_domain[0][0] if request.files_for_domain else None
                )
                input_dataset = first_input_file.stem if first_input_file else None

                # Create output request
                output_request = OutputRequest(
                    dataframe=merged_df,
                    domain_code=request.domain_code,
                    config=config,
                    output_dirs=OutputDirs(
                        xpt_dir=xpt_dir,
                        xml_dir=xml_dir,
                        sas_dir=sas_dir,
                    ),
                    formats=formats,
                    base_filename=base_filename,
                    input_dataset=input_dataset,
                    output_dataset=base_filename,
                )

                # Generate outputs
                output_result = self._file_generator.generate(output_request)

                # Update result
                if output_result.xpt_path:
                    result["xpt_path"] = output_result.xpt_path
                    result["xpt_filename"] = output_result.xpt_path.name
                    self.logger.success(f"Generated XPT: {output_result.xpt_path}")

                    # Handle domain variant splits (SDTMIG v3.4 Section 4.1.7)
                    if len(variant_frames) > 1:
                        assert xpt_dir is not None
                        split_paths, split_datasets = self._write_variant_splits(
                            variant_frames, domain, xpt_dir
                        )
                        result["split_xpt_paths"] = split_paths
                        result["split_datasets"] = split_datasets

                if output_result.xml_path:
                    result["xml_path"] = output_result.xml_path
                    result["xml_filename"] = output_result.xml_path.name
                    self.logger.success(
                        f"Generated Dataset-XML: {output_result.xml_path}"
                    )

                if output_result.sas_path:
                    result["sas_path"] = output_result.sas_path
                    self.logger.success(f"Generated SAS: {output_result.sas_path}")

                # Log any errors
                for error in output_result.errors:
                    self.logger.error(f"Output generation error: {error}")

        return result

    def _generate_supplemental_files(
        self,
        supp_frames: list[pd.DataFrame],
        domain_code: str,
        study_id: str,
        output_formats: set[str],
        output_dirs: dict[str, Path | None],
    ) -> dict[str, Any]:
        """Generate supplemental qualifier files using FileGeneratorPort."""
        from .models import OutputDirs, OutputRequest
        from ..domain.entities.mapping import ColumnMapping, build_config

        merged_supp = (
            supp_frames[0]
            if len(supp_frames) == 1
            else pd.concat(supp_frames, ignore_index=True)
        )

        supp_domain_code = f"SUPP{domain_code.upper()}"

        # Finalize (ordering + dedup) after merge to avoid duplicates across files.
        try:
            supp_domain_def = self._get_domain(supp_domain_code)
        except Exception:  # noqa: BLE001
            supp_domain_def = None
        if not merged_supp.empty:
            merged_supp = self._suppqual_service.finalize_suppqual(
                merged_supp,
                supp_domain_def=supp_domain_def,
                parent_domain_code=domain_code,
            )

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

        supp_domain = supp_domain_def or self._get_domain(supp_domain_code)
        base_filename = supp_domain.resolved_dataset_name()

        supp_result: dict[str, Any] = {
            "domain_code": supp_domain_code,
            "records": len(merged_supp),
            "domain_dataframe": merged_supp,
            "config": supp_config,
            "xpt_path": None,
            "xml_path": None,
            "sas_path": None,
        }

        # Use FileGeneratorPort if available
        if self._file_generator is not None:
            formats = set()
            xpt_dir = output_dirs.get("xpt")
            xml_dir = output_dirs.get("xml")

            if xpt_dir and "xpt" in output_formats:
                formats.add("xpt")
            if xml_dir and "xml" in output_formats:
                formats.add("xml")

            if formats:
                output_request = OutputRequest(
                    dataframe=merged_supp,
                    domain_code=supp_domain_code,
                    config=supp_config,
                    output_dirs=OutputDirs(xpt_dir=xpt_dir, xml_dir=xml_dir),
                    formats=formats,
                    base_filename=base_filename,
                )

                output_result = self._file_generator.generate(output_request)

                if output_result.xpt_path:
                    supp_result["xpt_path"] = output_result.xpt_path
                    supp_result["xpt_filename"] = output_result.xpt_path.name

                if output_result.xml_path:
                    supp_result["xml_path"] = output_result.xml_path
                    supp_result["xml_path"] = output_result.xml_path.name

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
        split_paths: list[Path] = []
        split_datasets: list[tuple[str, pd.DataFrame, Path]] = []
        domain_code = domain.code.upper()

        for variant_name, variant_df in variant_frames:
            table = (
                variant_name.replace(" ", "_").replace("(", "").replace(")", "").upper()
            )

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
            if self._output_preparer is None:
                raise RuntimeError(
                    "OutputPreparerPort is not configured. "
                    "Wire an infrastructure adapter in the composition root."
                )
            self._output_preparer.ensure_dir(split_dir)

            split_name = table.lower()
            split_path = split_dir / f"{split_name}.xpt"

            split_suffix = table[len(domain_code) :]
            file_label = (
                f"{domain.description} - {split_suffix}"
                if split_suffix
                else domain.description
            )

            if self._xpt_writer is None:
                raise RuntimeError(
                    "XPTWriterPort is not configured. "
                    "Wire an infrastructure adapter in the composition root."
                )

            self._xpt_writer.write(
                variant_df,
                domain.code,
                split_path,
                file_label=file_label,
                table_name=table,
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
        if self._domain_definition_repository is None:
            raise RuntimeError(
                "DomainDefinitionRepositoryPort is not configured. "
                "Wire an infrastructure adapter in the composition root."
            )
        return self._domain_definition_repository.get_domain(domain_code)

    def _merge_dataframes(
        self, all_dataframes: list[pd.DataFrame], domain_code: str, verbose: bool
    ) -> pd.DataFrame:
        """Merge multiple dataframes and re-sequence."""
        if len(all_dataframes) == 1:
            return all_dataframes[0]

        # Avoid pandas FutureWarning around dtype inference when concatenating
        # empty/all-NA frames, while still preserving the union of columns.
        union_columns: list[str] = sorted(
            {col for df in all_dataframes for col in df.columns.astype(str)}
        )
        non_empty = [df for df in all_dataframes if not df.empty]
        if not non_empty:
            return pd.DataFrame(columns=union_columns)

        input_rows_list = [len(df) for df in all_dataframes]
        total_input = sum(input_rows_list)

        merged_df = pd.concat(non_empty, ignore_index=True)
        if union_columns:
            merged_df = merged_df.reindex(columns=union_columns)
        merged_rows = len(merged_df)

        # Re-assign sequence numbers per subject after merge
        seq_col = f"{domain_code}SEQ"
        if seq_col in merged_df.columns and "USUBJID" in merged_df.columns:
            merged_df.loc[:, seq_col] = merged_df.groupby("USUBJID").cumcount() + 1

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
            key for key in ("USUBJID", "LBTESTCD", "LBDTC") if key in merged_df.columns
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
                merged_df.loc[:, seq_col] = merged_df.groupby("USUBJID").cumcount() + 1
        return merged_df
