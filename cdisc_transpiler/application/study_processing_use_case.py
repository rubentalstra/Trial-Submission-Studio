"""Study processing use case.

This module contains the main use case for processing a complete study,
orchestrating file discovery, domain processing, synthesis, and Define-XML
generation.
"""

from __future__ import annotations

from collections import defaultdict
from pathlib import Path
from typing import Any

import pandas as pd

from .models import (
    DomainProcessingResult,
    ProcessStudyRequest,
    ProcessStudyResponse,
)
from .ports import LoggerPort
from ..domains_module import get_domain, list_domains
from ..io_module import load_input_dataset
from ..metadata_module import load_study_metadata
from ..services import (
    DomainDiscoveryService,
    DomainProcessingCoordinator,
    DomainSynthesisCoordinator,
    StudyOrchestrationService,
)
from ..services import ensure_acrf_pdf
from ..xml_module.define_module import (
    StudyDataset,
    write_study_define_file,
)
from ..xml_module.define_module.constants import (
    CONTEXT_SUBMISSION,
    CONTEXT_OTHER,
    ACRF_HREF,
)


class StudyProcessingUseCase:
    """Use case for processing a complete study.
    
    This class orchestrates the entire study processing workflow, delegating
    specific tasks to specialized services. It follows the Ports & Adapters
    architecture, with dependencies injected via the constructor.
    
    The use case:
    1. Discovers domain files in the study folder
    2. Processes each domain (via DomainProcessingCoordinator)
    3. Synthesizes missing required domains
    4. Generates Define-XML metadata
    5. Collects and aggregates results
    
    Example:
        >>> use_case = StudyProcessingUseCase(logger=my_logger)
        >>> request = ProcessStudyRequest(
        ...     study_folder=Path("study001"),
        ...     study_id="STUDY001",
        ...     output_dir=Path("output"),
        ... )
        >>> response = use_case.execute(request)
        >>> if response.success:
        ...     print(f"Processed {len(response.domain_results)} domains")
    """
    
    def __init__(self, logger: LoggerPort):
        """Initialize the use case with injected dependencies.
        
        Args:
            logger: Logger for progress and error reporting
        """
        self.logger = logger
        self.discovery_service = DomainDiscoveryService(logger=logger)
        self.domain_processor = DomainProcessingCoordinator()
        self.synthesis_coordinator = DomainSynthesisCoordinator()
        self.orchestration_service = StudyOrchestrationService()
    
    def execute(self, request: ProcessStudyRequest) -> ProcessStudyResponse:
        """Execute the study processing workflow.
        
        Args:
            request: Study processing request with all parameters
            
        Returns:
            Study processing response with results and any errors
            
        Example:
            >>> response = use_case.execute(request)
            >>> print(f"Success: {response.success}")
            >>> print(f"Domains: {response.processed_domains}")
            >>> print(f"Errors: {response.errors}")
        """
        response = ProcessStudyResponse(
            study_id=request.study_id,
            output_dir=request.output_dir,
        )
        
        try:
            # Log study initialization
            supported_domains = list(list_domains())
            self.logger.log_study_start(
                request.study_id,
                request.study_folder,
                "/".join(request.output_formats),
                supported_domains,
            )
            
            # Load study metadata
            study_metadata = load_study_metadata(request.study_folder)
            self.logger.log_metadata_loaded(
                items_count=len(study_metadata.items) if study_metadata.items else None,
                codelists_count=len(study_metadata.codelists) if study_metadata.codelists else None,
            )
            
            # Set up output directories
            self._setup_output_directories(request)
            
            # Discover domain files
            csv_files = list(request.study_folder.glob("*.csv"))
            self.logger.verbose(f"Found {len(csv_files)} CSV files in study folder")
            
            domain_files = self.discovery_service.discover_domain_files(
                csv_files, supported_domains
            )
            
            if not domain_files:
                response.success = False
                response.errors.append(
                    ("DISCOVERY", f"No domain files found in {request.study_folder}")
                )
                return response
            
            # Build common column counts for heuristic analysis
            common_column_counts = self._build_column_counts(domain_files)
            total_input_files = sum(len(files) for files in domain_files.values())
            
            self.logger.log_processing_summary(
                study_id=request.study_id,
                domain_count=len(domain_files),
                file_count=total_input_files,
                output_format="/".join(request.output_formats),
                generate_define=request.generate_define_xml,
                generate_sas=request.generate_sas,
            )
            
            # Process domains
            study_datasets: list[StudyDataset] = []
            reference_starts: dict[str, str] = {}
            processed_domains = set(domain_files.keys())
            
            # Determine output directories
            xpt_dir = request.output_dir / "xpt" if "xpt" in request.output_formats else None
            xml_dir = request.output_dir / "dataset-xml" if "xml" in request.output_formats else None
            sas_dir = request.output_dir / "sas" if request.generate_sas else None
            
            # Process each domain in order (DM first)
            ordered_domains = sorted(
                domain_files.keys(), key=lambda code: (code != "DM", code)
            )
            
            for domain_code in ordered_domains:
                result = self._process_domain(
                    domain_code=domain_code,
                    files_for_domain=domain_files[domain_code],
                    request=request,
                    study_metadata=study_metadata,
                    reference_starts=reference_starts,
                    common_column_counts=common_column_counts,
                    total_input_files=total_input_files,
                    xpt_dir=xpt_dir,
                    xml_dir=xml_dir,
                    sas_dir=sas_dir,
                )
                
                response.domain_results.append(result)
                
                if result.success:
                    response.processed_domains.add(domain_code)
                    response.total_records += result.records
                    
                    # Update reference starts from DM
                    if domain_code == "DM" and result.domain_dataframe is not None:
                        reference_starts.update(
                            self._extract_reference_starts(result.domain_dataframe)
                        )
                    
                    # Collect for Define-XML
                    if request.generate_define_xml and result.domain_dataframe is not None:
                        self._add_to_study_datasets(
                            result, study_datasets, request.output_dir, request.output_formats
                        )
                else:
                    response.errors.append((domain_code, result.error or "Unknown error"))
            
            # Synthesize missing required domains
            self._synthesize_missing_domains(
                response=response,
                processed_domains=processed_domains,
                request=request,
                reference_starts=reference_starts,
                study_datasets=study_datasets,
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
            )
            
            # Generate Define-XML
            if request.generate_define_xml and study_datasets:
                self._generate_define_xml(
                    study_datasets=study_datasets,
                    response=response,
                    request=request,
                )
            
            # Log final statistics
            self.logger.log_final_stats()
            
            # Overall success if no critical errors
            response.success = len(response.errors) == 0
            
        except Exception as exc:
            response.success = False
            response.errors.append(("GENERAL", str(exc)))
            self.logger.error(f"Study processing failed: {exc}")
        
        return response
    
    def _setup_output_directories(self, request: ProcessStudyRequest) -> None:
        """Set up output directory structure."""
        request.output_dir.mkdir(parents=True, exist_ok=True)
        
        if "xpt" in request.output_formats:
            (request.output_dir / "xpt").mkdir(parents=True, exist_ok=True)
        
        if "xml" in request.output_formats:
            (request.output_dir / "dataset-xml").mkdir(parents=True, exist_ok=True)
        
        if request.generate_sas:
            (request.output_dir / "sas").mkdir(parents=True, exist_ok=True)
        
        if request.generate_define_xml:
            ensure_acrf_pdf(request.output_dir / ACRF_HREF)
    
    def _build_column_counts(
        self, domain_files: dict[str, list[tuple[Path, str]]]
    ) -> dict[str, int]:
        """Build common column counts for heuristic analysis."""
        common_column_counts: dict[str, int] = defaultdict(int)
        
        for files in domain_files.values():
            for file_path, _ in files:
                try:
                    headers = load_input_dataset(file_path)
                except Exception:
                    continue
                for col in headers.columns:
                    common_column_counts[str(col).strip().lower()] += 1
        
        return common_column_counts
    
    def _process_domain(
        self,
        domain_code: str,
        files_for_domain: list[tuple[Path, str]],
        request: ProcessStudyRequest,
        study_metadata: Any,
        reference_starts: dict[str, str],
        common_column_counts: dict[str, int],
        total_input_files: int,
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
    ) -> DomainProcessingResult:
        """Process a single domain."""
        self.logger.log_domain_start(domain_code, files_for_domain)
        
        try:
            result_dict = self.domain_processor.process_and_merge_domain(
                files_for_domain=files_for_domain,
                domain_code=domain_code,
                study_id=request.study_id,
                output_format="/".join(request.output_formats),
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
                min_confidence=request.min_confidence,
                streaming=request.streaming,
                chunk_size=request.chunk_size,
                generate_sas=request.generate_sas,
                verbose=request.verbose > 0,
                metadata=study_metadata,
                reference_starts=reference_starts or None,
                common_column_counts=common_column_counts or None,
                total_input_files=total_input_files,
            )
            
            # Convert dict result to DomainProcessingResult
            result = DomainProcessingResult(
                domain_code=domain_code,
                success=True,
                records=result_dict.get("records", 0),
                domain_dataframe=result_dict.get("domain_dataframe"),
                config=result_dict.get("config"),
                xpt_path=result_dict.get("xpt_path"),
                xml_path=result_dict.get("xml_path"),
                sas_path=result_dict.get("sas_path"),
                split_datasets=result_dict.get("split_datasets", []),
            )
            
            # Handle supplemental domains
            for supp_dict in result_dict.get("supplementals", []):
                supp_result = DomainProcessingResult(
                    domain_code=supp_dict.get("domain_code", ""),
                    success=True,
                    records=supp_dict.get("records", 0),
                    domain_dataframe=supp_dict.get("domain_dataframe"),
                    config=supp_dict.get("config"),
                    xpt_path=supp_dict.get("xpt_path"),
                    xml_path=supp_dict.get("xml_path"),
                    sas_path=supp_dict.get("sas_path"),
                )
                result.supplementals.append(supp_result)
            
            return result
            
        except Exception as exc:
            self.logger.error(f"{domain_code}: {exc}")
            return DomainProcessingResult(
                domain_code=domain_code,
                success=False,
                error=str(exc),
            )
    
    def _extract_reference_starts(
        self, dm_frame: pd.DataFrame
    ) -> dict[str, str]:
        """Extract RFSTDTC reference starts from DM domain."""
        reference_starts: dict[str, str] = {}
        baseline_default = "2023-01-01"
        
        # Ensure RFSTDTC exists and is populated
        if "RFSTDTC" not in dm_frame.columns:
            dm_frame["RFSTDTC"] = baseline_default
        else:
            rfstdtc_series = (
                dm_frame["RFSTDTC"]
                .astype("string")
                .replace({"nan": "", "<NA>": "", "None": ""})
                .fillna("")
                .str.strip()
            )
            dm_frame.loc[rfstdtc_series == "", "RFSTDTC"] = baseline_default
        
        if {"USUBJID", "RFSTDTC"}.issubset(dm_frame.columns):
            cleaned = dm_frame[["USUBJID", "RFSTDTC"]].copy()
            cleaned["RFSTDTC"] = pd.to_datetime(
                cleaned["RFSTDTC"], errors="coerce"
            ).fillna(pd.to_datetime(baseline_default))
            baseline_map = (
                cleaned.set_index("USUBJID")["RFSTDTC"]
                .dt.date.astype(str)
                .to_dict()
            )
            reference_starts.update(baseline_map)
        
        return reference_starts
    
    def _add_to_study_datasets(
        self,
        result: DomainProcessingResult,
        study_datasets: list[StudyDataset],
        output_dir: Path,
        output_formats: set[str],
    ) -> None:
        """Add domain result to study datasets for Define-XML."""
        domain = get_domain(result.domain_code)
        disk_name = domain.resolved_dataset_name().lower()
        
        # Determine dataset href
        if "xpt" in output_formats and result.xpt_path:
            dataset_href = result.xpt_path.relative_to(output_dir)
        elif "xml" in output_formats and result.xml_path:
            dataset_href = result.xml_path.relative_to(output_dir)
        else:
            dataset_href = Path(f"{disk_name}.xpt")
        
        if result.config and result.domain_dataframe is not None:
            study_datasets.append(
                StudyDataset(
                    domain_code=result.domain_code,
                    dataframe=result.domain_dataframe,
                    config=result.config,
                    archive_location=dataset_href,
                )
            )
            
            # Add split datasets
            for split_name, split_df, split_path in result.split_datasets:
                split_href = split_path.relative_to(output_dir)
                study_datasets.append(
                    StudyDataset(
                        domain_code=split_name,
                        dataframe=split_df,
                        config=result.config,
                        archive_location=split_href,
                        is_split=True,
                        split_suffix=split_name[len(result.domain_code) :]
                        if split_name.startswith(result.domain_code)
                        else split_name,
                    )
                )
            
            # Add supplemental domains
            for supp in result.supplementals:
                if supp.domain_dataframe is not None:
                    supp_href = (
                        supp.xpt_path.relative_to(output_dir) if supp.xpt_path
                        else supp.xml_path.relative_to(output_dir) if supp.xml_path
                        else Path(f"{supp.domain_code.lower()}.xpt")
                    )
                    study_datasets.append(
                        StudyDataset(
                            domain_code=supp.domain_code,
                            dataframe=supp.domain_dataframe,
                            config=supp.config,
                            archive_location=supp_href,
                        )
                    )
    
    def _synthesize_missing_domains(
        self,
        response: ProcessStudyResponse,
        processed_domains: set[str],
        request: ProcessStudyRequest,
        reference_starts: dict[str, str],
        study_datasets: list[StudyDataset],
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
    ) -> None:
        """Synthesize missing required domains."""
        # Synthesize core observation domains
        for missing_domain in ["AE", "LB", "VS", "EX"]:
            if missing_domain not in processed_domains:
                self._synthesize_domain(
                    domain_code=missing_domain,
                    reason="No source files found",
                    response=response,
                    request=request,
                    reference_starts=reference_starts,
                    study_datasets=study_datasets,
                    xpt_dir=xpt_dir,
                    xml_dir=xml_dir,
                    sas_dir=sas_dir,
                )
        
        # Synthesize trial design domains
        for td_domain in ["TS", "TA", "TE", "SE", "DS"]:
            if td_domain not in processed_domains:
                self._synthesize_trial_design_domain(
                    domain_code=td_domain,
                    reason="Trial design scaffold",
                    response=response,
                    request=request,
                    reference_starts=reference_starts,
                    study_datasets=study_datasets,
                    xpt_dir=xpt_dir,
                    xml_dir=xml_dir,
                    sas_dir=sas_dir,
                )
        
        # Synthesize RELREC
        if "RELREC" not in processed_domains:
            self._synthesize_relrec(
                response=response,
                request=request,
                study_datasets=study_datasets,
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
            )
    
    def _synthesize_domain(
        self,
        domain_code: str,
        reason: str,
        response: ProcessStudyResponse,
        request: ProcessStudyRequest,
        reference_starts: dict[str, str],
        study_datasets: list[StudyDataset],
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
    ) -> None:
        """Synthesize a missing observation domain."""
        self.logger.log_synthesis_start(domain_code, reason)
        
        try:
            result_dict = self.synthesis_coordinator.synthesize_empty_observation_domain(
                domain_code=domain_code,
                study_id=request.study_id,
                output_format="/".join(request.output_formats),
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
                generate_sas=request.generate_sas,
                reference_starts=reference_starts,
            )
            
            result = DomainProcessingResult(
                domain_code=domain_code,
                success=True,
                records=result_dict.get("records", 0),
                domain_dataframe=result_dict.get("domain_dataframe"),
                config=result_dict.get("config"),
                xpt_path=result_dict.get("xpt_path"),
                xml_path=result_dict.get("xml_path"),
                sas_path=result_dict.get("sas_path"),
                synthesized=True,
                synthesis_reason=reason,
            )
            
            response.domain_results.append(result)
            response.processed_domains.add(domain_code)
            response.total_records += result.records
            
            if request.generate_define_xml and result.domain_dataframe is not None:
                self._add_to_study_datasets(
                    result, study_datasets, request.output_dir, request.output_formats
                )
            
            self.logger.log_synthesis_complete(domain_code, result.records)
            
        except Exception as exc:
            self.logger.error(f"{domain_code}: {exc}")
            response.errors.append((domain_code, str(exc)))
    
    def _synthesize_trial_design_domain(
        self,
        domain_code: str,
        reason: str,
        response: ProcessStudyResponse,
        request: ProcessStudyRequest,
        reference_starts: dict[str, str],
        study_datasets: list[StudyDataset],
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
    ) -> None:
        """Synthesize a missing trial design domain."""
        self.logger.log_synthesis_start(domain_code, reason)
        
        try:
            result_dict = self.synthesis_coordinator.synthesize_trial_design_domain(
                domain_code=domain_code,
                study_id=request.study_id,
                output_format="/".join(request.output_formats),
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
                generate_sas=request.generate_sas,
                reference_starts=reference_starts,
            )
            
            result = DomainProcessingResult(
                domain_code=domain_code,
                success=True,
                records=result_dict.get("records", 0),
                domain_dataframe=result_dict.get("domain_dataframe"),
                config=result_dict.get("config"),
                xpt_path=result_dict.get("xpt_path"),
                xml_path=result_dict.get("xml_path"),
                sas_path=result_dict.get("sas_path"),
                synthesized=True,
                synthesis_reason=reason,
            )
            
            response.domain_results.append(result)
            response.processed_domains.add(domain_code)
            response.total_records += result.records
            
            if request.generate_define_xml and result.domain_dataframe is not None:
                self._add_to_study_datasets(
                    result, study_datasets, request.output_dir, request.output_formats
                )
            
            self.logger.log_synthesis_complete(domain_code, result.records)
            
        except Exception as exc:
            self.logger.error(f"{domain_code}: {exc}")
            response.errors.append((domain_code, str(exc)))
    
    def _synthesize_relrec(
        self,
        response: ProcessStudyResponse,
        request: ProcessStudyRequest,
        study_datasets: list[StudyDataset],
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
    ) -> None:
        """Synthesize RELREC domain."""
        self.logger.log_synthesis_start("RELREC", "Relationship scaffold")
        
        try:
            # Convert domain results to dict format for orchestration service
            domain_results_dicts = []
            for result in response.domain_results:
                if result.domain_dataframe is not None:
                    domain_results_dicts.append({
                        "domain_code": result.domain_code,
                        "domain_dataframe": result.domain_dataframe,
                        "config": result.config,
                        "xpt_path": result.xpt_path,
                        "xml_path": result.xml_path,
                        "sas_path": result.sas_path,
                    })
            
            result_dict = self.orchestration_service.synthesize_relrec(
                study_id=request.study_id,
                output_format="/".join(request.output_formats),
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
                generate_sas=request.generate_sas,
                domain_results=domain_results_dicts,
            )
            
            result = DomainProcessingResult(
                domain_code="RELREC",
                success=True,
                records=result_dict.get("records", 0),
                domain_dataframe=result_dict.get("domain_dataframe"),
                config=result_dict.get("config"),
                xpt_path=result_dict.get("xpt_path"),
                xml_path=result_dict.get("xml_path"),
                sas_path=result_dict.get("sas_path"),
                synthesized=True,
                synthesis_reason="Relationship scaffold",
            )
            
            response.domain_results.append(result)
            response.processed_domains.add("RELREC")
            response.total_records += result.records
            
            if request.generate_define_xml and result.domain_dataframe is not None:
                self._add_to_study_datasets(
                    result, study_datasets, request.output_dir, request.output_formats
                )
            
            self.logger.success("Generated RELREC")
            
        except Exception as exc:
            self.logger.error(f"RELREC: {exc}")
            response.errors.append(("RELREC", str(exc)))
    
    def _generate_define_xml(
        self,
        study_datasets: list[StudyDataset],
        response: ProcessStudyResponse,
        request: ProcessStudyRequest,
    ) -> None:
        """Generate Define-XML file."""
        define_path = request.output_dir / "define.xml"
        
        try:
            context = (
                CONTEXT_SUBMISSION
                if request.define_context == "Submission"
                else CONTEXT_OTHER
            )
            write_study_define_file(
                study_datasets,
                define_path,
                sdtm_version=request.sdtm_version,
                context=context,
            )
            response.define_xml_path = define_path
            self.logger.success(f"Generated Define-XML 2.1 at {define_path}")
            
        except Exception as exc:
            response.define_xml_error = str(exc)
            self.logger.error(f"Define-XML generation failed: {exc}")
            response.errors.append(("Define-XML", str(exc)))
