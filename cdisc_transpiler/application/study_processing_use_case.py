"""Study processing use case.

This module contains the main use case for processing a complete study,
orchestrating file discovery, domain processing, synthesis, and Define-XML
generation.

CLEAN2-D2: This use case is now fully implemented with injected dependencies,
without compatibility shims or cross-layer imports.

The use case orchestrates:
- Discovery → per-domain processing → synthesis → Define-XML generation
using injected ports/use cases.
"""

from collections import defaultdict
from dataclasses import dataclass, replace
from pathlib import Path
from typing import TYPE_CHECKING, Literal, cast

import pandas as pd

from ..constants import Defaults, SDTMVersions
from ..domain.services.domain_frame_builder import DomainFrameBuildRequest
from ..domain.services.sdtm_conformance_checker import (
    ConformanceReport,
    check_domain_dataframe,
)
from ..pandas_utils import normalize_missing_strings
from .models import (
    DatasetOutputDirs,
    DatasetOutputRequest,
    DefineDatasetDTO,
    DomainProcessingResult,
    ProcessDomainRequest,
    ProcessingSummary,
    ProcessStudyResponse,
)

if TYPE_CHECKING:
    from ..domain.entities.controlled_terminology import ControlledTerminology
    from ..domain.entities.sdtm_domain import SDTMDomain, SDTMVariable
    from ..domain.entities.study_metadata import StudyMetadata
    from ..domain.services.relrec_service import RelrecService
    from ..domain.services.relspec_service import RelspecService
    from ..domain.services.relsub_service import RelsubService
    from .domain_processing_use_case import DomainProcessingUseCase
    from .models import ProcessStudyRequest
    from .ports.repositories import (
        CTRepositoryPort,
        DomainDefinitionRepositoryPort,
        StudyDataRepositoryPort,
    )
    from .ports.services import (
        ConformanceReportWriterPort,
        DatasetOutputPort,
        DefineXMLGeneratorPort,
        DomainDiscoveryPort,
        DomainFrameBuilderPort,
        LoggerPort,
        OutputPreparerPort,
    )


@dataclass(frozen=True, slots=True)
class _SynthesisJob:
    domain_code: str
    kind: Literal["relrec", "relsub", "relspec"]
    reason: str


@dataclass(frozen=True, slots=True)
class _DomainRunContext:
    request: ProcessStudyRequest
    study_metadata: StudyMetadata
    reference_starts: dict[str, str]
    common_column_counts: dict[str, int]
    total_input_files: int
    output_dirs: DatasetOutputDirs


@dataclass(slots=True)
class _SynthesisContext:
    request: ProcessStudyRequest
    study_datasets: list[DefineDatasetDTO]
    output_dirs: DatasetOutputDirs


@dataclass(slots=True)
class StudyProcessingDependencies:
    logger: LoggerPort
    study_data_repository: StudyDataRepositoryPort
    domain_processing_use_case: DomainProcessingUseCase
    domain_discovery_service: DomainDiscoveryPort
    domain_frame_builder: DomainFrameBuilderPort
    relrec_service: RelrecService
    relsub_service: RelsubService
    relspec_service: RelspecService
    domain_definition_repository: DomainDefinitionRepositoryPort
    dataset_output: DatasetOutputPort | None = None
    define_xml_generator: DefineXMLGeneratorPort | None = None
    output_preparer: OutputPreparerPort | None = None
    ct_repository: CTRepositoryPort | None = None
    conformance_report_writer: ConformanceReportWriterPort | None = None


class StudyProcessingUseCase:
    """Use case for processing a complete study.

    This class orchestrates the entire study processing workflow using injected
    dependencies. It follows the Ports & Adapters architecture pattern.

    The use case orchestrates:
    1. Discovers domain files in the study folder
    2. Processes each domain (via DomainProcessingUseCase)
    3. Optionally synthesizes missing domains (if enabled)
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
        ...     dataset_output=file_gen,
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

    def __init__(self, dependencies: StudyProcessingDependencies) -> None:
        """Initialize the use case with injected dependencies.

        Args:
            dependencies: Bundled ports/adapters for study processing
        """
        super().__init__()
        self.logger = dependencies.logger
        self._study_data_repository = dependencies.study_data_repository
        self._domain_processing_use_case = dependencies.domain_processing_use_case
        self._domain_discovery_service = dependencies.domain_discovery_service
        self._domain_frame_builder = dependencies.domain_frame_builder
        self._relrec_service = dependencies.relrec_service
        self._relsub_service = dependencies.relsub_service
        self._relspec_service = dependencies.relspec_service
        self._dataset_output = dependencies.dataset_output
        self._define_xml_generator = dependencies.define_xml_generator
        self._output_preparer = dependencies.output_preparer
        self._domain_definition_repository = dependencies.domain_definition_repository
        self._ct_repository = dependencies.ct_repository
        self._conformance_report_writer = dependencies.conformance_report_writer

    def _log_conformance_report(
        self,
        *,
        frame: pd.DataFrame,
        domain: SDTMDomain,
        strict: bool,
    ) -> ConformanceReport | None:
        if not strict:
            return None

        ct_repo = self._ct_repository

        def _ct_resolver(variable: SDTMVariable) -> ControlledTerminology | None:
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
            for supp in getattr(domain_result, "suppqual_domains", []) or []:
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

            summary = ProcessingSummary(
                study_id=request.study_id,
                domain_count=len(domain_files),
                file_count=total_input_files,
                output_format="/".join(request.output_formats),
                generate_define=request.generate_define_xml,
                generate_sas=request.generate_sas,
            )
            self.logger.log_processing_summary(summary)

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

            output_dirs = DatasetOutputDirs(
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
            )
            context = _DomainRunContext(
                request=request,
                study_metadata=study_metadata,
                reference_starts=reference_starts,
                common_column_counts=common_column_counts,
                total_input_files=total_input_files,
                output_dirs=output_dirs,
            )

            ordered_domains = sorted(
                domain_files.keys(), key=lambda code: (code != "DM", code)
            )

            for domain_code in ordered_domains:
                result = self._process_domain(
                    domain_code=domain_code,
                    files_for_domain=domain_files[domain_code],
                    context=context,
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

                # SUPPQUAL datasets are not treated as top-level "domains" in the
                # CLI, but their rows should contribute to record totals.
                for supp in result.suppqual_domains:
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

            # Always run synthesis pass (RELREC/RELSUB/RELSPEC are always attempted).
            synthesis_context = _SynthesisContext(
                request=request,
                study_datasets=study_datasets,
                output_dirs=output_dirs,
            )
            self._run_synthesis_pass(
                response=response,
                present_domains=processed_domains,
                context=synthesis_context,
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

        except Exception as exc:
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
            response.conformance_report_error = "ConformanceReportWriterPort is not configured. Wire an infrastructure adapter in the composition root."
            return

        reports: list[ConformanceReport] = []
        for domain_result in response.domain_results:
            report = domain_result.conformance_report
            if isinstance(report, ConformanceReport):
                reports.append(report)
            for supp in domain_result.suppqual_domains:
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
        except Exception as exc:
            response.conformance_report_error = str(exc)

    def _setup_output_directories(self, request: ProcessStudyRequest) -> None:
        """Set up output directory structure."""
        if self._output_preparer is None:
            raise RuntimeError(
                "OutputPreparerPort is not configured. Wire an infrastructure adapter in the composition root."
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
        context: _DomainRunContext,
    ) -> DomainProcessingResult:
        """Process a single domain using DomainProcessingUseCase."""
        self.logger.log_domain_start(domain_code, files_for_domain)

        try:
            # Get the domain processing use case
            domain_use_case = self._get_domain_processing_use_case()
            request = context.request

            # Build request for domain processing
            domain_request = ProcessDomainRequest(
                files_for_domain=files_for_domain,
                domain_code=domain_code,
                study_id=request.study_id,
                output_formats=request.output_formats,
                output_dirs={
                    "xpt": context.output_dirs.xpt_dir,
                    "xml": context.output_dirs.xml_dir,
                    "sas": context.output_dirs.sas_dir,
                },
                min_confidence=request.min_confidence,
                streaming=request.streaming,
                chunk_size=request.chunk_size,
                generate_sas=request.generate_sas,
                verbose=request.verbose,
                metadata=context.study_metadata,
                reference_starts=context.reference_starts or None,
                common_column_counts=context.common_column_counts or None,
                total_input_files=context.total_input_files,
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
                error=domain_response.error,
                conformance_report=domain_response.conformance_report,
            )

            # Handle SUPPQUAL domains
            for suppqual_response in domain_response.suppqual_domains:
                suppqual_result = DomainProcessingResult(
                    domain_code=suppqual_response.domain_code,
                    success=suppqual_response.success,
                    records=suppqual_response.records,
                    domain_dataframe=suppqual_response.domain_dataframe,
                    config=suppqual_response.config,
                    xpt_path=suppqual_response.xpt_path,
                    xml_path=suppqual_response.xml_path,
                    sas_path=suppqual_response.sas_path,
                    conformance_report=suppqual_response.conformance_report,
                )
                result.suppqual_domains.append(suppqual_result)

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
            study_datasets.append(
                DefineDatasetDTO(
                    domain_code=result.domain_code,
                    dataframe=result.domain_dataframe,
                    config=result.config,
                    archive_location=dataset_href,
                )
            )

            # Add SUPPQUAL domains
            for supp in result.suppqual_domains:
                if supp.domain_dataframe is not None and supp.config is not None:
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

    def _run_synthesis_pass(
        self,
        *,
        response: ProcessStudyResponse,
        present_domains: set[str],
        context: _SynthesisContext,
    ) -> None:
        jobs = self._build_synthesis_jobs(
            response=response,
            present_domains=present_domains,
        )

        for job in jobs:
            # Avoid duplicates if something already produced the domain.
            if job.domain_code in response.processed_domains:
                continue

            if job.kind == "relrec":
                self._synthesize_relrec(
                    response=response,
                    context=context,
                )
            elif job.kind == "relsub":
                self._synthesize_relsub(
                    response=response,
                    context=context,
                )
            elif job.kind == "relspec":
                self._synthesize_relspec(
                    response=response,
                    context=context,
                )

    def _build_synthesis_jobs(
        self,
        *,
        response: ProcessStudyResponse,
        present_domains: set[str],
    ) -> list[_SynthesisJob]:
        """Build an ordered list of synthesis jobs.

        `present_domains` includes domains that were discovered from files (even
        if processing later failed). We should not synthesize over a dataset
        that exists in source.
        """

        scheduled: set[str] = set()
        jobs: list[_SynthesisJob] = []

        def _missing(code: str) -> bool:
            upper = code.upper()
            return (upper not in present_domains) and (
                upper not in response.processed_domains
            )

        # 1) Relationship domains (always attempt synthesis if missing)
        for code, kind, reason in [
            ("RELREC", "relrec", "Relationship scaffold"),
            ("RELSUB", "relsub", "Related subjects scaffold"),
            ("RELSPEC", "relspec", "Related specimens scaffold"),
        ]:
            if _missing(code) and code not in scheduled:
                kind_literal = cast("Literal['relrec', 'relsub', 'relspec']", kind)
                jobs.append(
                    _SynthesisJob(
                        domain_code=code,
                        kind=kind_literal,
                        reason=reason,
                    )
                )
                scheduled.add(code)

        return jobs

    @staticmethod
    def _build_output_formats(request: ProcessStudyRequest) -> set[str]:
        formats: set[str] = set()
        if "xpt" in request.output_formats:
            formats.add("xpt")
        if "xml" in request.output_formats:
            formats.add("xml")
        if request.generate_sas:
            formats.add("sas")
        return formats

    def _fill_tsparm_labels(self, frame: pd.DataFrame) -> None:
        if frame.empty:
            return
        if "TSPARMCD" not in frame.columns or "TSPARM" not in frame.columns:
            return

        labels = {
            "ADDON": "Added on to Existing Treatments",
            "AGEMAX": "Planned Maximum Age of Subjects",
            "AGEMIN": "Planned Minimum Age of Subjects",
            "LENGTH": "Trial Length",
            "PLANSUB": "Planned Number of Subjects",
            "RANDOM": "Trial is Randomized",
            "SEXPOP": "Sex of Participants",
            "STOPRULE": "Study Stop Rules",
            "TBLIND": "Trial Blinding Schema",
            "TCNTRL": "Control Type",
            "TDIGRP": "Diagnosis Group",
            "TINDTP": "Trial Intent Type",
            "TITLE": "Trial Title",
            "TPHASE": "Trial Phase Classification",
            "TTYPE": "Trial Type",
            "CURTRT": "Current Therapy or Treatment",
            "OBJPRIM": "Trial Primary Objective",
            "SPONSOR": "Clinical Study Sponsor",
            "TRT": "Investigational Therapy or Treatment",
            "REGID": "Registry Identifier",
            "OUTMSPRI": "Primary Outcome Measure",
            "PCLAS": "Pharmacologic Class",
            "FCNTRY": "Planned Country of Investigational Sites",
            "ADAPT": "Adaptive Design",
            "DCUTDTC": "Data Cutoff Date",
            "DCUTDESC": "Data Cutoff Description",
            "INTMODEL": "Intervention Model",
            "NARMS": "Planned Number of Arms",
            "STYPE": "Study Type",
            "INTTYPE": "Intervention Type",
            "SSTDTC": "Study Start Date",
            "SENDTC": "Study End Date",
            "ACTSUB": "Actual Number of Subjects",
            "HLTSUBJI": "Healthy Subject Indicator",
            "EXTTIND": "Extension Trial Indicator",
            "NCOHORT": "Number of Groups/Cohorts",
            "OBJSEC": "Trial Secondary Objective",
            "PDPSTIND": "Pediatric Postmarket Study Indicator",
            "PDSTIND": "Pediatric Study Indicator",
            "PIPIND": "Pediatric Investigation Plan Indicator",
            "RDIND": "Rare Disease Indicator",
            "SDTIGVER": "SDTM IG Version",
            "SDTMVER": "SDTM Version",
            "THERAREA": "Therapeutic Area",
            "ONGOSIND": "Ongoing Study Indicator",
        }

        parmcd = frame["TSPARMCD"].astype("string").fillna("").str.strip().str.upper()
        fill = parmcd.map(labels)

        tsparm_str = frame["TSPARM"].astype("string").fillna("").str.strip()
        mask_blank = frame["TSPARM"].isna() | tsparm_str.eq("")
        mask_fillable = mask_blank & fill.notna()
        frame.loc[mask_fillable, "TSPARM"] = fill[mask_fillable]

    def _synthesize_relrec(
        self,
        response: ProcessStudyResponse,
        context: _SynthesisContext,
    ) -> None:
        """Synthesize RELREC domain.

        CLEAN2-D4: Now uses the new RelrecService from domain/services
        instead of an external orchestration shim.
        """
        self.logger.log_synthesis_start("RELREC", "Relationship scaffold")

        try:
            request = context.request
            output_dirs = context.output_dirs
            study_datasets = context.study_datasets

            # Build dictionary of domain dataframes for RELREC service
            domain_dataframes: dict[str, pd.DataFrame] = {}
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
                DomainFrameBuildRequest(
                    frame=relrec_df,
                    config=relrec_config,
                    domain=relrec_domain,
                    lenient=lenient,
                )
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

            output_formats = self._build_output_formats(request)
            output_request = DatasetOutputRequest(
                dataframe=domain_dataframe,
                domain_code="RELREC",
                config=relrec_config,
                output_dirs=output_dirs,
                formats=output_formats,
            )

            dataset_output = self._dataset_output
            output_result = (
                dataset_output.generate(output_request)
                if dataset_output is not None
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
        context: _SynthesisContext,
    ) -> None:
        """Synthesize RELSUB domain (Related Subjects)."""
        try:
            request = context.request
            output_dirs = context.output_dirs
            study_datasets = context.study_datasets

            relsub_service = self._get_relsub_service()
            relsub_df, relsub_config = relsub_service.build_relsub(
                domain_dataframes=None,
                study_id=request.study_id,
            )

            # RELSUB relationships are often not inferable; if we have no rows,
            # do not generate empty dataset outputs and do not include the domain
            # in Define-XML inputs.
            if relsub_df.empty:
                self.logger.verbose("RELSUB: no relationships detected; skipping")
                return

            self.logger.log_synthesis_start("RELSUB", "Relationship scaffold")

            relsub_domain = self._get_domain("RELSUB")
            lenient = ("xpt" not in request.output_formats) and (
                not request.generate_sas
            )
            domain_dataframe = self._domain_frame_builder.build_domain_dataframe(
                DomainFrameBuildRequest(
                    frame=relsub_df,
                    config=relsub_config,
                    domain=relsub_domain,
                    lenient=lenient,
                )
            )

            if domain_dataframe.empty:
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

            output_formats = self._build_output_formats(request)
            output_request = DatasetOutputRequest(
                dataframe=domain_dataframe,
                domain_code="RELSUB",
                config=relsub_config,
                output_dirs=output_dirs,
                formats=output_formats,
            )

            dataset_output = self._dataset_output
            output_result = (
                dataset_output.generate(output_request)
                if dataset_output is not None
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
        context: _SynthesisContext,
    ) -> None:
        """Synthesize RELSPEC domain (Related Specimens)."""
        self.logger.log_synthesis_start("RELSPEC", "Relationship scaffold")

        try:
            request = context.request
            output_dirs = context.output_dirs
            study_datasets = context.study_datasets

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
                DomainFrameBuildRequest(
                    frame=relspec_df,
                    config=relspec_config,
                    domain=relspec_domain,
                    lenient=lenient,
                )
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

            output_formats = self._build_output_formats(request)
            output_request = DatasetOutputRequest(
                dataframe=domain_dataframe,
                domain_code="RELSPEC",
                config=relspec_config,
                output_dirs=output_dirs,
                formats=output_formats,
            )

            dataset_output = self._dataset_output
            output_result = (
                dataset_output.generate(output_request)
                if dataset_output is not None
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
                "DomainDefinitionRepositoryPort is not configured. Wire an infrastructure adapter in the composition root."
            )
        return list(self._domain_definition_repository.list_domains())

    def _get_domain(self, domain_code: str) -> SDTMDomain:
        """Get domain definition via DomainDefinitionRepositoryPort."""
        if self._domain_definition_repository is None:
            raise RuntimeError(
                "DomainDefinitionRepositoryPort is not configured. Wire an infrastructure adapter in the composition root."
            )
        return self._domain_definition_repository.get_domain(domain_code)

    def _load_study_metadata(self, study_folder: Path) -> StudyMetadata:
        """Load study metadata via repository or fallback."""
        if self._study_data_repository is not None:
            return self._study_data_repository.load_study_metadata(study_folder)

        raise RuntimeError(
            "StudyDataRepositoryPort is not configured. Wire an infrastructure adapter in the composition root."
        )

    def _load_dataset(self, file_path: Path) -> pd.DataFrame:
        """Load a dataset via repository or fallback."""
        if self._study_data_repository is not None:
            return self._study_data_repository.read_dataset(file_path)

        raise RuntimeError(
            "StudyDataRepositoryPort is not configured. Wire an infrastructure adapter in the composition root."
        )

    def _get_domain_discovery_service(self) -> DomainDiscoveryPort:
        """Get injected domain discovery service."""
        return self._domain_discovery_service

    def _get_domain_processing_use_case(self) -> DomainProcessingUseCase:
        """Get or create domain processing use case."""
        if self._domain_processing_use_case is not None:
            return self._domain_processing_use_case

        raise RuntimeError(
            "DomainProcessingUseCase is not configured. Wire it in the composition root (DependencyContainer)."
        )

    def _get_relrec_service(self) -> RelrecService:
        """Get injected RELREC service."""
        return self._relrec_service

    def _get_relsub_service(self) -> RelsubService:
        """Get injected RELSUB service."""
        return self._relsub_service

    def _get_relspec_service(self) -> RelspecService:
        """Get injected RELSPEC service."""
        return self._relspec_service
