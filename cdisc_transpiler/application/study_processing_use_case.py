"""Study processing use case.

This module contains the main use case for processing a complete study,
orchestrating file discovery, domain processing, synthesis, and Define-XML
generation.

CLEAN2-D2: This use case is now fully implemented with injected dependencies,
removing the delegation to legacy coordinators and old module imports.

CLEAN2-D3: Synthesis now uses the new SynthesisService from domain/services
instead of the legacy DomainSynthesisCoordinator.

The use case orchestrates:
- Discovery → per-domain processing → synthesis → Define-XML generation
using injected ports/use cases.
"""

from __future__ import annotations

from collections import defaultdict
from dataclasses import replace
from pathlib import Path
from typing import TYPE_CHECKING, Any

import pandas as pd

from ..constants import SDTMVersions
from ..constants import Defaults
from ..pandas_utils import normalize_missing_strings

from .models import (
    DefineDatasetDTO,
    DomainProcessingResult,
    ProcessDomainRequest,
    ProcessStudyRequest,
    ProcessStudyResponse,
)
from .ports import (
    DefineXMLGeneratorPort,
    DomainDiscoveryPort,
    DomainFrameBuilderPort,
    FileGeneratorPort,
    LoggerPort,
    DomainDefinitionRepositoryPort,
    StudyDataRepositoryPort,
    OutputPreparerPort,
    CTRepositoryPort,
    ConformanceReportWriterPort,
)

if TYPE_CHECKING:
    from .domain_processing_use_case import DomainProcessingUseCase
    from ..domain.services import (
        RelrecService,
        RelspecService,
        RelsubService,
        SynthesisService,
    )


class StudyProcessingUseCase:
    """Use case for processing a complete study.

    This class orchestrates the entire study processing workflow using injected
    dependencies. It follows the Ports & Adapters architecture pattern.

    The use case orchestrates:
    1. Discovers domain files in the study folder
    2. Processes each domain (via DomainProcessingUseCase)
    3. Synthesizes missing required domains
    4. Generates Define-XML metadata
    5. Collects and aggregates results

    All dependencies are injected via the constructor, enabling testability
    and allowing different implementations to be swapped.

    Example:
        >>> use_case = StudyProcessingUseCase(
        ...     logger=logger,
        ...     study_data_repository=repo,
        ...     domain_processing_use_case=domain_use_case,
        ...     domain_discovery_service=domain_discovery_service,
        ...     file_generator=file_gen,
        ... )
        >>> request = ProcessStudyRequest(
        ...     study_folder=Path("study001"),
        ...     study_id="STUDY001",
        ...     output_dir=Path("output"),
        ... )
        >>> response = use_case.execute(request)
        >>> if response.success:
        ...     print(f"Processed {len(response.domain_results)} domains")
    """

    def __init__(
        self,
        logger: LoggerPort,
        study_data_repository: StudyDataRepositoryPort | None = None,
        domain_processing_use_case: DomainProcessingUseCase | None = None,
        domain_discovery_service: DomainDiscoveryPort | None = None,
        domain_frame_builder: DomainFrameBuilderPort | None = None,
        synthesis_service: "SynthesisService | None" = None,
        relrec_service: "RelrecService | None" = None,
        relsub_service: "RelsubService | None" = None,
        relspec_service: "RelspecService | None" = None,
        file_generator: FileGeneratorPort | None = None,
        define_xml_generator: DefineXMLGeneratorPort | None = None,
        output_preparer: OutputPreparerPort | None = None,
        domain_definition_repository: DomainDefinitionRepositoryPort | None = None,
        ct_repository: CTRepositoryPort | None = None,
        conformance_report_writer: ConformanceReportWriterPort | None = None,
    ):
        """Initialize the use case with injected dependencies.

        Args:
            logger: Logger for progress and error reporting
            study_data_repository: Repository for loading study data and metadata
            domain_processing_use_case: Use case for processing individual domains
            domain_discovery_service: Service for discovering domain files
            file_generator: Generator for output files
            define_xml_generator: Generator for Define-XML files
        """
        if domain_discovery_service is None:
            raise ValueError(
                "StudyProcessingUseCase requires domain_discovery_service to be injected "
                "(use the DI container)."
            )
        if synthesis_service is None:
            raise ValueError(
                "StudyProcessingUseCase requires synthesis_service to be injected "
                "(use the DI container)."
            )
        if relrec_service is None:
            raise ValueError(
                "StudyProcessingUseCase requires relrec_service to be injected "
                "(use the DI container)."
            )
        if relsub_service is None:
            raise ValueError(
                "StudyProcessingUseCase requires relsub_service to be injected "
                "(use the DI container)."
            )
        if relspec_service is None:
            raise ValueError(
                "StudyProcessingUseCase requires relspec_service to be injected "
                "(use the DI container)."
            )
        if domain_frame_builder is None:
            raise ValueError(
                "StudyProcessingUseCase requires domain_frame_builder to be injected "
                "(use the DI container)."
            )
        self.logger = logger
        self._study_data_repository = study_data_repository
        self._domain_processing_use_case = domain_processing_use_case
        self._domain_discovery_service = domain_discovery_service
        self._domain_frame_builder = domain_frame_builder
        self._synthesis_service = synthesis_service
        self._relrec_service = relrec_service
        self._relsub_service = relsub_service
        self._relspec_service = relspec_service
        self._file_generator = file_generator
        self._define_xml_generator = define_xml_generator
        self._output_preparer = output_preparer
        self._domain_definition_repository = domain_definition_repository
        self._ct_repository = ct_repository
        self._conformance_report_writer = conformance_report_writer

    def _log_conformance_report(
        self,
        *,
        frame: pd.DataFrame,
        domain: Any,
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

    def execute(self, request: ProcessStudyRequest) -> ProcessStudyResponse:
        """Execute the study processing workflow."""

        strict_outputs_requested = (
            "xpt" in request.output_formats
        ) or request.generate_sas

        if strict_outputs_requested and request.fail_on_conformance_errors:
            # Preflight pass: process all domains, run strict conformance checks,
            # but do not write submission artifacts.
            preflight_request = replace(
                request,
                generate_define_xml=False,
                fail_on_conformance_errors=False,
            )
            preflight = self._execute_impl(preflight_request, write_outputs=False)

            conformance_error_domains = self._collect_conformance_error_domains(
                preflight
            )
            if conformance_error_domains:
                for code in sorted(set(conformance_error_domains)):
                    preflight.errors.append(
                        (code, "Conformance errors present; outputs not generated")
                    )
                preflight.success = False
                return preflight

            if preflight.errors or preflight.define_xml_error:
                preflight.success = False
                return preflight

        # Clean run (or non-strict): write outputs normally.
        return self._execute_impl(request, write_outputs=True)

    def _collect_conformance_error_domains(
        self, response: ProcessStudyResponse
    ) -> list[str]:
        domains: list[str] = []
        for domain_result in response.domain_results:
            report = getattr(domain_result, "conformance_report", None)
            if report is not None and getattr(report, "has_errors", lambda: False)():
                domains.append(domain_result.domain_code)
            for supp in getattr(domain_result, "supplementals", []) or []:
                supp_report = getattr(supp, "conformance_report", None)
                if (
                    supp_report is not None
                    and getattr(supp_report, "has_errors", lambda: False)()
                ):
                    domains.append(supp.domain_code)
        return domains

    def _execute_impl(
        self, request: ProcessStudyRequest, *, write_outputs: bool
    ) -> ProcessStudyResponse:
        response = ProcessStudyResponse(
            study_id=request.study_id,
            output_dir=request.output_dir,
        )

        try:
            supported_domains = list(self._list_domains())

            # Log study initialization
            self.logger.log_study_start(
                request.study_id,
                request.study_folder,
                "/".join(request.output_formats),
                supported_domains,
            )

            # Load study metadata via repository
            study_metadata = self._load_study_metadata(request.study_folder)
            self.logger.log_metadata_loaded(
                items_count=len(study_metadata.items) if study_metadata.items else None,
                codelists_count=len(study_metadata.codelists)
                if study_metadata.codelists
                else None,
            )

            if write_outputs:
                self._setup_output_directories(request)
            else:
                request.output_dir.mkdir(parents=True, exist_ok=True)

            # Discover domain files
            csv_files = list(request.study_folder.glob("*.csv"))
            self.logger.verbose(f"Found {len(csv_files)} CSV files in study folder")

            domain_discovery_service = self._get_domain_discovery_service()
            domain_files = domain_discovery_service.discover_domain_files(
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

            study_datasets: list[DefineDatasetDTO] = []
            reference_starts: dict[str, str] = {}
            processed_domains = set(domain_files.keys())

            if write_outputs:
                xpt_dir = (
                    request.output_dir / "xpt"
                    if "xpt" in request.output_formats
                    else None
                )
                xml_dir = (
                    request.output_dir / "dataset-xml"
                    if "xml" in request.output_formats
                    else None
                )
                sas_dir = request.output_dir / "sas" if request.generate_sas else None
            else:
                xpt_dir = None
                xml_dir = None
                sas_dir = None

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

                if result.domain_dataframe is not None:
                    self.logger.log_domain_complete(
                        domain_code,
                        final_row_count=len(result.domain_dataframe),
                        final_column_count=len(result.domain_dataframe.columns),
                        skipped=not result.success,
                        reason=result.error,
                    )
                else:
                    self.logger.log_domain_complete(
                        domain_code,
                        final_row_count=0,
                        final_column_count=0,
                        skipped=not result.success,
                        reason=result.error,
                    )

                # Supplemental datasets are not treated as top-level "domains" in the
                # CLI, but their rows should contribute to record totals.
                for supp in result.supplementals:
                    supp_frame = supp.domain_dataframe
                    if supp_frame is None:
                        self.logger.log_domain_complete(
                            supp.domain_code,
                            final_row_count=0,
                            final_column_count=0,
                            skipped=not supp.success,
                            reason=supp.error,
                        )
                    else:
                        self.logger.log_domain_complete(
                            supp.domain_code,
                            final_row_count=len(supp_frame),
                            final_column_count=len(supp_frame.columns),
                            skipped=not supp.success,
                            reason=supp.error,
                        )

                if result.success:
                    response.processed_domains.add(domain_code)
                    response.total_records += result.records

                    if domain_code == "DM" and result.domain_dataframe is not None:
                        reference_starts.update(
                            self._extract_reference_starts(result.domain_dataframe)
                        )

                    if (
                        write_outputs
                        and request.generate_define_xml
                        and result.domain_dataframe is not None
                    ):
                        self._add_to_study_datasets(
                            result,
                            study_datasets,
                            request.output_dir,
                            request.output_formats,
                        )
                else:
                    response.errors.append(
                        (domain_code, result.error or "Unknown error")
                    )

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

            self._write_conformance_report_json(request=request, response=response)

            if write_outputs and request.generate_define_xml and study_datasets:
                self._generate_define_xml(
                    study_datasets=study_datasets,
                    response=response,
                    request=request,
                )

            self.logger.log_final_stats()
            response.success = len(response.errors) == 0

        except Exception as exc:  # noqa: BLE001
            response.success = False
            response.errors.append(("GENERAL", str(exc)))
            self.logger.error(f"Study processing failed: {exc}")

        return response

    def _write_conformance_report_json(
        self, *, request: ProcessStudyRequest, response: ProcessStudyResponse
    ) -> None:
        if not request.write_conformance_report_json:
            return

        writer = self._conformance_report_writer
        if writer is None:
            response.conformance_report_error = (
                "ConformanceReportWriterPort is not configured. "
                "Wire an infrastructure adapter in the composition root."
            )
            return

        from ..domain.services.sdtm_conformance_checker import ConformanceReport

        reports: list[ConformanceReport] = []
        for domain_result in response.domain_results:
            report = domain_result.conformance_report
            if isinstance(report, ConformanceReport):
                reports.append(report)
            for supp in domain_result.supplementals:
                supp_report = getattr(supp, "conformance_report", None)
                if isinstance(supp_report, ConformanceReport):
                    reports.append(supp_report)

        if not reports:
            return

        try:
            response.conformance_report_path = writer.write_json(
                output_dir=request.output_dir,
                study_id=request.study_id,
                reports=reports,
            )
        except Exception as exc:  # noqa: BLE001
            response.conformance_report_error = str(exc)

    def _setup_output_directories(self, request: ProcessStudyRequest) -> None:
        """Set up output directory structure."""
        if self._output_preparer is None:
            raise RuntimeError(
                "OutputPreparerPort is not configured. "
                "Wire an infrastructure adapter in the composition root."
            )

        self._output_preparer.prepare(
            output_dir=request.output_dir,
            output_formats=request.output_formats,
            generate_sas=request.generate_sas,
            generate_define_xml=request.generate_define_xml,
        )

    def _build_column_counts(
        self, domain_files: dict[str, list[tuple[Path, str]]]
    ) -> dict[str, int]:
        """Build common column counts for heuristic analysis."""
        common_column_counts: dict[str, int] = defaultdict(int)

        for files in domain_files.values():
            for file_path, _ in files:
                try:
                    headers = self._load_dataset(file_path)
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
        """Process a single domain using DomainProcessingUseCase."""
        self.logger.log_domain_start(domain_code, files_for_domain)

        try:
            # Get the domain processing use case
            domain_use_case = self._get_domain_processing_use_case()

            # Build request for domain processing
            domain_request = ProcessDomainRequest(
                files_for_domain=files_for_domain,
                domain_code=domain_code,
                study_id=request.study_id,
                output_formats=request.output_formats,
                output_dirs={
                    "xpt": xpt_dir,
                    "xml": xml_dir,
                    "sas": sas_dir,
                },
                min_confidence=request.min_confidence,
                streaming=request.streaming,
                chunk_size=request.chunk_size,
                generate_sas=request.generate_sas,
                verbose=request.verbose,
                metadata=study_metadata,
                reference_starts=reference_starts or None,
                common_column_counts=common_column_counts or None,
                total_input_files=total_input_files,
                fail_on_conformance_errors=request.fail_on_conformance_errors,
                default_country=request.default_country,
            )

            # Execute domain processing
            domain_response = domain_use_case.execute(domain_request)

            # Convert ProcessDomainResponse to DomainProcessingResult
            result = DomainProcessingResult(
                domain_code=domain_code,
                success=domain_response.success,
                records=domain_response.records,
                domain_dataframe=domain_response.domain_dataframe,
                config=domain_response.config,
                xpt_path=domain_response.xpt_path,
                xml_path=domain_response.xml_path,
                sas_path=domain_response.sas_path,
                split_datasets=domain_response.split_datasets,
                error=domain_response.error,
                conformance_report=domain_response.conformance_report,
            )

            # Handle supplemental domains
            for supp_response in domain_response.supplementals:
                supp_result = DomainProcessingResult(
                    domain_code=supp_response.domain_code,
                    success=supp_response.success,
                    records=supp_response.records,
                    domain_dataframe=supp_response.domain_dataframe,
                    config=supp_response.config,
                    xpt_path=supp_response.xpt_path,
                    xml_path=supp_response.xml_path,
                    sas_path=supp_response.sas_path,
                    conformance_report=supp_response.conformance_report,
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

    def _extract_reference_starts(self, dm_frame: pd.DataFrame) -> dict[str, str]:
        """Extract RFSTDTC reference starts from DM domain."""
        reference_starts: dict[str, str] = {}
        baseline_default = Defaults.DATE

        # Ensure RFSTDTC exists and is populated
        if "RFSTDTC" not in dm_frame.columns:
            dm_frame["RFSTDTC"] = baseline_default
        else:
            rfstdtc_series = normalize_missing_strings(dm_frame["RFSTDTC"]).fillna("")
            dm_frame.loc[rfstdtc_series == "", "RFSTDTC"] = baseline_default

        if {"USUBJID", "RFSTDTC"}.issubset(dm_frame.columns):
            cleaned = dm_frame[["USUBJID", "RFSTDTC"]].copy()
            rfstdtc = pd.to_datetime(cleaned["RFSTDTC"], errors="coerce").fillna(
                pd.to_datetime(baseline_default)
            )
            rfstdtc_by_subj = pd.Series(
                rfstdtc.values,
                index=cleaned["USUBJID"].astype("string"),
                name="RFSTDTC",
            )
            baseline_map: dict[str, str] = {
                str(usubjid): str(timestamp.date().isoformat())
                for usubjid, timestamp in rfstdtc_by_subj.items()
            }
            reference_starts.update(baseline_map)

        return reference_starts

    def _add_to_study_datasets(
        self,
        result: DomainProcessingResult,
        study_datasets: list[DefineDatasetDTO],
        output_dir: Path,
        output_formats: set[str],
    ) -> None:
        """Add domain result to study datasets for Define-XML.

        Creates application-layer DefineDatasetDTO instances from domain
        processing results. These DTOs are later passed to the
        DefineXMLGeneratorPort which converts them to infrastructure models.
        """
        domain = self._get_domain(result.domain_code)
        disk_name = domain.resolved_dataset_name().lower()

        # Determine dataset href
        if "xpt" in output_formats and result.xpt_path:
            dataset_href = result.xpt_path.relative_to(output_dir)
        elif "xml" in output_formats and result.xml_path:
            dataset_href = result.xml_path.relative_to(output_dir)
        else:
            dataset_href = Path(f"{disk_name}.xpt")

        if result.config and result.domain_dataframe is not None:
            # If split datasets exist for this domain, do not include the unsplit
            # parent dataset in Define-XML (SDTMIG v3.4 4.1.7 splitting domains).
            if not result.split_datasets:
                study_datasets.append(
                    DefineDatasetDTO(
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
                    DefineDatasetDTO(
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
                        supp.xpt_path.relative_to(output_dir)
                        if supp.xpt_path
                        else supp.xml_path.relative_to(output_dir)
                        if supp.xml_path
                        else Path(f"{supp.domain_code.lower()}.xpt")
                    )
                    study_datasets.append(
                        DefineDatasetDTO(
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
        study_datasets: list[DefineDatasetDTO],
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

        # Synthesize RELSUB
        if "RELSUB" not in processed_domains:
            self._synthesize_relsub(
                response=response,
                request=request,
                study_datasets=study_datasets,
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
            )

        # Synthesize RELSPEC
        if "RELSPEC" not in processed_domains:
            self._synthesize_relspec(
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
        study_datasets: list[DefineDatasetDTO],
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
    ) -> None:
        """Synthesize a missing observation domain.

        The domain synthesis service returns pure domain data, and
        file generation is handled here in the application layer.
        """
        self.logger.log_synthesis_start(domain_code, reason)

        try:
            synthesis_service = self._get_synthesis_service()
            synthesis_result = synthesis_service.synthesize_observation(
                domain_code=domain_code,
                study_id=request.study_id,
                reference_starts=reference_starts,
            )

            if not synthesis_result.success:
                raise RuntimeError(synthesis_result.error or "Synthesis failed")

            # Generate output files using FileGeneratorPort (application layer)
            xpt_path, xml_path, sas_path = self._generate_synthesis_files(
                domain_dataframe=synthesis_result.domain_dataframe,
                domain_code=domain_code,
                config=synthesis_result.config,
                request=request,
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
            )

            result = DomainProcessingResult(
                domain_code=domain_code,
                success=True,
                records=synthesis_result.records,
                domain_dataframe=synthesis_result.domain_dataframe,
                config=synthesis_result.config,
                xpt_path=xpt_path,
                xml_path=xml_path,
                sas_path=sas_path,
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
        study_datasets: list[DefineDatasetDTO],
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
    ) -> None:
        """Synthesize a missing trial design domain.

        The domain synthesis service returns pure domain data, and
        file generation is handled here in the application layer.
        """
        self.logger.log_synthesis_start(domain_code, reason)

        try:
            synthesis_service = self._get_synthesis_service()
            synthesis_result = synthesis_service.synthesize_trial_design(
                domain_code=domain_code,
                study_id=request.study_id,
                reference_starts=reference_starts,
            )

            if not synthesis_result.success:
                raise RuntimeError(synthesis_result.error or "Synthesis failed")

            # Run synthesized Trial Design domains through the normal domain
            # frame builder so domain processors (TS/TA/TE/SE) can populate
            # required values and CT-backed names.
            domain_def = self._get_domain(domain_code)
            lenient = ("xpt" not in request.output_formats) and (
                not request.generate_sas
            )

            synthesis_config = synthesis_result.config
            if synthesis_config is None:
                from cdisc_transpiler.domain.entities.mapping import MappingConfig

                synthesis_config = MappingConfig(
                    domain=domain_code,
                    study_id=request.study_id,
                    mappings=[],
                )
            domain_dataframe = self._domain_frame_builder.build_domain_dataframe(
                synthesis_result.domain_dataframe
                if synthesis_result.domain_dataframe is not None
                else pd.DataFrame(),
                synthesis_config,
                domain_def,
                reference_starts=reference_starts,
                lenient=lenient,
            )

            strict = ("xpt" in request.output_formats) or request.generate_sas
            report = self._log_conformance_report(
                frame=domain_dataframe,
                domain=domain_def,
                strict=strict,
            )

            if (
                strict
                and request.fail_on_conformance_errors
                and report is not None
                and getattr(report, "has_errors", lambda: False)()
            ):
                raise ValueError(
                    f"{domain_code}: conformance errors present; strict output generation aborted"
                )

            # Generate output files using FileGeneratorPort (application layer)
            xpt_path, xml_path, sas_path = self._generate_synthesis_files(
                domain_dataframe=domain_dataframe,
                domain_code=domain_code,
                config=synthesis_result.config,
                request=request,
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
            )

            result = DomainProcessingResult(
                domain_code=domain_code,
                success=True,
                records=len(domain_dataframe),
                domain_dataframe=domain_dataframe,
                config=synthesis_result.config,
                xpt_path=xpt_path,
                xml_path=xml_path,
                sas_path=sas_path,
                synthesized=True,
                synthesis_reason=reason,
                conformance_report=report,
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
        study_datasets: list[DefineDatasetDTO],
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
    ) -> None:
        """Synthesize RELREC domain.

        CLEAN2-D4: Now uses the new RelrecService from domain/services
        instead of the legacy StudyOrchestrationService.
        """
        self.logger.log_synthesis_start("RELREC", "Relationship scaffold")

        try:
            # Build dictionary of domain dataframes for RELREC service
            domain_dataframes = {}
            for result in response.domain_results:
                if (
                    result.domain_dataframe is not None
                    and not result.domain_dataframe.empty
                ):
                    domain_dataframes[result.domain_code] = result.domain_dataframe

            relrec_service = self._get_relrec_service()
            relrec_df, relrec_config = relrec_service.build_relrec(
                domain_dataframes=domain_dataframes,
                study_id=request.study_id,
            )

            # Build domain dataframe with SDTM structure
            relrec_domain = self._get_domain("RELREC")
            lenient = ("xpt" not in request.output_formats) and (
                not request.generate_sas
            )
            domain_dataframe = self._domain_frame_builder.build_domain_dataframe(
                relrec_df,
                relrec_config,
                relrec_domain,
                lenient=lenient,
            )

            strict = ("xpt" in request.output_formats) or request.generate_sas
            report = self._log_conformance_report(
                frame=domain_dataframe,
                domain=relrec_domain,
                strict=strict,
            )

            if (
                strict
                and request.fail_on_conformance_errors
                and report is not None
                and getattr(report, "has_errors", lambda: False)()
            ):
                raise ValueError(
                    "RELREC: conformance errors present; strict output generation aborted"
                )

            # Generate output files
            from .models import OutputDirs, OutputRequest

            output_dirs = OutputDirs(
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
            )

            output_formats = set()
            if "xpt" in request.output_formats:
                output_formats.add("xpt")
            if "xml" in request.output_formats:
                output_formats.add("xml")
            if request.generate_sas:
                output_formats.add("sas")

            output_request = OutputRequest(
                dataframe=domain_dataframe,
                domain_code="RELREC",
                config=relrec_config,
                output_dirs=output_dirs,
                formats=output_formats,
            )

            file_generator = self._file_generator
            output_result = (
                file_generator.generate(output_request)
                if file_generator is not None
                else None
            )

            result = DomainProcessingResult(
                domain_code="RELREC",
                success=True,
                records=len(domain_dataframe),
                domain_dataframe=domain_dataframe,
                config=relrec_config,
                xpt_path=output_result.xpt_path if output_result else None,
                xml_path=output_result.xml_path if output_result else None,
                sas_path=output_result.sas_path if output_result else None,
                synthesized=True,
                synthesis_reason="Relationship scaffold",
                conformance_report=report,
            )

            response.domain_results.append(result)
            response.processed_domains.add("RELREC")
            response.total_records += result.records

            if request.generate_define_xml and result.domain_dataframe is not None:
                self._add_to_study_datasets(
                    result, study_datasets, request.output_dir, request.output_formats
                )

            self.logger.log_synthesis_complete("RELREC", result.records)

        except Exception as exc:
            self.logger.error(f"RELREC: {exc}")
            response.errors.append(("RELREC", str(exc)))

    def _synthesize_relsub(
        self,
        response: ProcessStudyResponse,
        request: ProcessStudyRequest,
        study_datasets: list[DefineDatasetDTO],
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
    ) -> None:
        """Synthesize RELSUB domain (Related Subjects)."""
        try:
            relsub_service = self._get_relsub_service()
            relsub_df, relsub_config = relsub_service.build_relsub(
                domain_dataframes=None,
                study_id=request.study_id,
            )

            # RELSUB relationships are often not inferable; if we have no rows,
            # do not generate empty output files and do not include the domain
            # in Define-XML inputs.
            if relsub_df is None or relsub_df.empty:
                self.logger.verbose("RELSUB: no relationships detected; skipping")
                return

            self.logger.log_synthesis_start("RELSUB", "Relationship scaffold")

            relsub_domain = self._get_domain("RELSUB")
            lenient = ("xpt" not in request.output_formats) and (
                not request.generate_sas
            )
            domain_dataframe = self._domain_frame_builder.build_domain_dataframe(
                relsub_df,
                relsub_config,
                relsub_domain,
                lenient=lenient,
            )

            if domain_dataframe is None or domain_dataframe.empty:
                self.logger.verbose("RELSUB: synthesized dataset is empty; skipping")
                return

            strict = ("xpt" in request.output_formats) or request.generate_sas
            report = self._log_conformance_report(
                frame=domain_dataframe,
                domain=relsub_domain,
                strict=strict,
            )

            if (
                strict
                and request.fail_on_conformance_errors
                and report is not None
                and getattr(report, "has_errors", lambda: False)()
            ):
                raise ValueError(
                    "RELSUB: conformance errors present; strict output generation aborted"
                )

            from .models import OutputDirs, OutputRequest

            output_dirs = OutputDirs(
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
            )

            output_formats = set()
            if "xpt" in request.output_formats:
                output_formats.add("xpt")
            if "xml" in request.output_formats:
                output_formats.add("xml")
            if request.generate_sas:
                output_formats.add("sas")

            output_request = OutputRequest(
                dataframe=domain_dataframe,
                domain_code="RELSUB",
                config=relsub_config,
                output_dirs=output_dirs,
                formats=output_formats,
            )

            file_generator = self._file_generator
            output_result = (
                file_generator.generate(output_request)
                if file_generator is not None
                else None
            )

            result = DomainProcessingResult(
                domain_code="RELSUB",
                success=True,
                records=len(domain_dataframe),
                domain_dataframe=domain_dataframe,
                config=relsub_config,
                xpt_path=output_result.xpt_path if output_result else None,
                xml_path=output_result.xml_path if output_result else None,
                sas_path=output_result.sas_path if output_result else None,
                synthesized=True,
                synthesis_reason="Relationship scaffold",
                conformance_report=report,
            )

            response.domain_results.append(result)
            response.processed_domains.add("RELSUB")
            response.total_records += result.records

            if request.generate_define_xml and result.domain_dataframe is not None:
                self._add_to_study_datasets(
                    result, study_datasets, request.output_dir, request.output_formats
                )

            self.logger.log_synthesis_complete("RELSUB", result.records)

        except Exception as exc:
            self.logger.error(f"RELSUB: {exc}")
            response.errors.append(("RELSUB", str(exc)))

    def _synthesize_relspec(
        self,
        response: ProcessStudyResponse,
        request: ProcessStudyRequest,
        study_datasets: list[DefineDatasetDTO],
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
    ) -> None:
        """Synthesize RELSPEC domain (Related Specimens)."""
        self.logger.log_synthesis_start("RELSPEC", "Relationship scaffold")

        try:
            domain_dataframes: dict[str, pd.DataFrame] = {}
            for result in response.domain_results:
                if (
                    result.domain_dataframe is not None
                    and not result.domain_dataframe.empty
                ):
                    domain_dataframes[result.domain_code] = result.domain_dataframe

            relspec_service = self._get_relspec_service()
            relspec_df, relspec_config = relspec_service.build_relspec(
                domain_dataframes=domain_dataframes,
                study_id=request.study_id,
            )

            relspec_domain = self._get_domain("RELSPEC")
            lenient = ("xpt" not in request.output_formats) and (
                not request.generate_sas
            )
            domain_dataframe = self._domain_frame_builder.build_domain_dataframe(
                relspec_df,
                relspec_config,
                relspec_domain,
                lenient=lenient,
            )

            strict = ("xpt" in request.output_formats) or request.generate_sas
            report = self._log_conformance_report(
                frame=domain_dataframe,
                domain=relspec_domain,
                strict=strict,
            )

            if (
                strict
                and request.fail_on_conformance_errors
                and report is not None
                and getattr(report, "has_errors", lambda: False)()
            ):
                raise ValueError(
                    "RELSPEC: conformance errors present; strict output generation aborted"
                )

            from .models import OutputDirs, OutputRequest

            output_dirs = OutputDirs(
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
            )

            output_formats = set()
            if "xpt" in request.output_formats:
                output_formats.add("xpt")
            if "xml" in request.output_formats:
                output_formats.add("xml")
            if request.generate_sas:
                output_formats.add("sas")

            output_request = OutputRequest(
                dataframe=domain_dataframe,
                domain_code="RELSPEC",
                config=relspec_config,
                output_dirs=output_dirs,
                formats=output_formats,
            )

            file_generator = self._file_generator
            output_result = (
                file_generator.generate(output_request)
                if file_generator is not None
                else None
            )

            result = DomainProcessingResult(
                domain_code="RELSPEC",
                success=True,
                records=len(domain_dataframe),
                domain_dataframe=domain_dataframe,
                config=relspec_config,
                xpt_path=output_result.xpt_path if output_result else None,
                xml_path=output_result.xml_path if output_result else None,
                sas_path=output_result.sas_path if output_result else None,
                synthesized=True,
                synthesis_reason="Relationship scaffold",
                conformance_report=report,
            )

            response.domain_results.append(result)
            response.processed_domains.add("RELSPEC")
            response.total_records += result.records

            if request.generate_define_xml and result.domain_dataframe is not None:
                self._add_to_study_datasets(
                    result, study_datasets, request.output_dir, request.output_formats
                )

            self.logger.log_synthesis_complete("RELSPEC", result.records)

        except Exception as exc:
            self.logger.error(f"RELSPEC: {exc}")
            response.errors.append(("RELSPEC", str(exc)))

    def _generate_define_xml(
        self,
        study_datasets: list[DefineDatasetDTO],
        response: ProcessStudyResponse,
        request: ProcessStudyRequest,
    ) -> None:
        """Generate Define-XML file using the injected generator.

        The generator accepts application-layer DefineDatasetDTO instances
        and converts them to infrastructure-specific models internally.
        """
        if self._define_xml_generator is None:
            response.define_xml_error = "Define-XML generator not available"
            self.logger.error("Define-XML generator not injected")
            response.errors.append(("Define-XML", "Generator not available"))
            return

        define_path = request.output_dir / "define.xml"

        try:
            # Context values per Define-XML 2.1 spec
            context = (
                SDTMVersions.DEFINE_CONTEXT_SUBMISSION
                if request.define_context == SDTMVersions.DEFINE_CONTEXT_SUBMISSION
                else SDTMVersions.DEFINE_CONTEXT_OTHER
            )
            self._define_xml_generator.generate(
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

    # ========== Helper Methods for Lazy Dependencies ==========

    def _list_domains(self) -> list[str]:
        """Get list of supported domains via DomainDefinitionRepositoryPort."""
        if self._domain_definition_repository is None:
            raise RuntimeError(
                "DomainDefinitionRepositoryPort is not configured. "
                "Wire an infrastructure adapter in the composition root."
            )
        return list(self._domain_definition_repository.list_domains())

    def _get_domain(self, domain_code: str):
        """Get domain definition via DomainDefinitionRepositoryPort."""
        if self._domain_definition_repository is None:
            raise RuntimeError(
                "DomainDefinitionRepositoryPort is not configured. "
                "Wire an infrastructure adapter in the composition root."
            )
        return self._domain_definition_repository.get_domain(domain_code)

    def _load_study_metadata(self, study_folder: Path):
        """Load study metadata via repository or fallback."""
        if self._study_data_repository is not None:
            return self._study_data_repository.load_study_metadata(study_folder)

        raise RuntimeError(
            "StudyDataRepositoryPort is not configured. "
            "Wire an infrastructure adapter in the composition root."
        )

    def _load_dataset(self, file_path: Path) -> pd.DataFrame:
        """Load a dataset via repository or fallback."""
        if self._study_data_repository is not None:
            return self._study_data_repository.read_dataset(file_path)

        raise RuntimeError(
            "StudyDataRepositoryPort is not configured. "
            "Wire an infrastructure adapter in the composition root."
        )

    def _get_domain_discovery_service(self):
        """Get injected domain discovery service."""
        if self._domain_discovery_service is None:
            raise RuntimeError(
                "DomainDiscoveryPort is not configured. "
                "Wire it in the composition root (DependencyContainer)."
            )
        return self._domain_discovery_service

    def _get_domain_processing_use_case(self):
        """Get or create domain processing use case."""
        if self._domain_processing_use_case is not None:
            return self._domain_processing_use_case

        raise RuntimeError(
            "DomainProcessingUseCase is not configured. "
            "Wire it in the composition root (DependencyContainer)."
        )

    def _get_synthesis_service(self):
        """Get injected domain synthesis service."""
        if self._synthesis_service is None:
            raise RuntimeError(
                "SynthesisService is not configured. "
                "Wire it in the composition root (DependencyContainer)."
            )
        return self._synthesis_service

    def _get_relrec_service(self):
        """Get injected RELREC service."""
        if self._relrec_service is None:
            raise RuntimeError(
                "RelrecService is not configured. "
                "Wire it in the composition root (DependencyContainer)."
            )
        return self._relrec_service

    def _get_relsub_service(self):
        """Get injected RELSUB service."""
        if self._relsub_service is None:
            raise RuntimeError(
                "RelsubService is not configured. "
                "Wire it in the composition root (DependencyContainer)."
            )
        return self._relsub_service

    def _get_relspec_service(self):
        """Get injected RELSPEC service."""
        if self._relspec_service is None:
            raise RuntimeError(
                "RelspecService is not configured. "
                "Wire it in the composition root (DependencyContainer)."
            )
        return self._relspec_service

    def _generate_synthesis_files(
        self,
        domain_dataframe: pd.DataFrame | None,
        domain_code: str,
        config: Any,
        request: ProcessStudyRequest,
        xpt_dir: Path | None,
        xml_dir: Path | None,
        sas_dir: Path | None,
    ) -> tuple[Path | None, Path | None, Path | None]:
        """Generate output files for synthesized domain data.

        This method handles file generation in the application layer,
        using the FileGeneratorPort to write output files.

        Args:
            domain_dataframe: Synthesized domain DataFrame
            domain_code: SDTM domain code
            config: Mapping configuration
            request: Study processing request
            xpt_dir: XPT output directory
            xml_dir: XML output directory
            sas_dir: SAS output directory

        Returns:
            Tuple of (xpt_path, xml_path, sas_path) - paths to generated files
        """
        file_generator = self._file_generator
        if domain_dataframe is None or file_generator is None:
            return None, None, None

        from .models import OutputDirs, OutputRequest

        # Determine output formats
        formats: set[str] = set()
        if xpt_dir and "xpt" in request.output_formats:
            formats.add("xpt")
        if xml_dir and "xml" in request.output_formats:
            formats.add("xml")
        if sas_dir and request.generate_sas:
            formats.add("sas")

        if not formats:
            return None, None, None

        output_request = OutputRequest(
            dataframe=domain_dataframe,
            domain_code=domain_code,
            config=config,
            output_dirs=OutputDirs(
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
            ),
            formats=formats,
        )

        output_result = file_generator.generate(output_request)

        # Log success
        if output_result.xpt_path:
            self.logger.success(
                f"Generated {domain_code} XPT: {output_result.xpt_path}"
            )
        if output_result.xml_path:
            self.logger.success(
                f"Generated {domain_code} Dataset-XML: {output_result.xml_path}"
            )
        if output_result.sas_path:
            self.logger.success(
                f"Generated {domain_code} SAS: {output_result.sas_path}"
            )

        return output_result.xpt_path, output_result.xml_path, output_result.sas_path
