from dataclasses import dataclass, field
import traceback
from typing import TYPE_CHECKING

import pandas as pd

from ..domain.entities.column_hints import ColumnHint
from ..domain.entities.mapping import ColumnMapping, build_config
from ..domain.services.domain_frame_builder import DomainFrameBuildRequest
from ..domain.services.sdtm_conformance_checker import check_domain_dataframe
from ..domain.services.suppqual_service import SuppqualBuildRequest
from .models import DatasetOutputDirs, DatasetOutputRequest, ProcessDomainResponse

if TYPE_CHECKING:
    from pathlib import Path

    from ..domain.entities.controlled_terminology import ControlledTerminology
    from ..domain.entities.mapping import MappingConfig
    from ..domain.entities.sdtm_domain import SDTMDomain, SDTMVariable
    from ..domain.services.sdtm_conformance_checker import ConformanceReport
    from .models import ProcessDomainRequest
    from .ports.repositories import (
        CTRepositoryPort,
        DomainDefinitionRepositoryPort,
        StudyDataRepositoryPort,
    )
    from .ports.services import (
        DatasetOutputPort,
        DomainFrameBuilderPort,
        LoggerPort,
        MappingPort,
        SuppqualPort,
    )


def _build_column_hints(frame: pd.DataFrame) -> dict[str, ColumnHint]:
    hints: dict[str, ColumnHint] = {}
    row_count = len(frame)
    for column in frame.columns:
        series = frame[column]
        is_numeric = pd.api.types.is_numeric_dtype(series)
        non_null = int(series.notna().sum())
        unique_non_null = series.nunique(dropna=True)
        unique_ratio = float(unique_non_null / non_null) if non_null else 0.0
        null_ratio = float(1 - non_null / row_count) if row_count else 0.0
        hints[str(column)] = ColumnHint(
            is_numeric=bool(is_numeric),
            unique_ratio=unique_ratio,
            null_ratio=null_ratio,
        )
    return hints


VERBOSE_TRACEBACK_LEVEL = 2
COLUMN_SAMPLE_LIMIT = 10


@dataclass(slots=True)
class DomainProcessingDependencies:
    logger: LoggerPort
    study_data_repository: StudyDataRepositoryPort
    mapping_service: MappingPort
    domain_frame_builder: DomainFrameBuilderPort
    suppqual_service: SuppqualPort
    domain_definition_repository: DomainDefinitionRepositoryPort
    dataset_output: DatasetOutputPort | None = None
    ct_repository: CTRepositoryPort | None = None


def _empty_suppqual_results() -> list[SuppqualOutputResult]:
    return []


@dataclass(slots=True)
class SuppqualOutputResult:
    domain_code: str
    records: int
    domain_dataframe: pd.DataFrame
    config: MappingConfig
    xpt_path: Path | None = None
    xml_path: Path | None = None
    sas_path: Path | None = None


@dataclass(slots=True)
class OutputStageResult:
    domain_code: str
    records: int
    domain_dataframe: pd.DataFrame
    config: MappingConfig
    xpt_path: Path | None = None
    xml_path: Path | None = None
    sas_path: Path | None = None
    suppqual_domains: list[SuppqualOutputResult] = field(
        default_factory=_empty_suppqual_results
    )


class DomainProcessingUseCase:
    pass

    def __init__(self, dependencies: DomainProcessingDependencies) -> None:
        super().__init__()
        self.logger = dependencies.logger
        self._study_data_repository = dependencies.study_data_repository
        self._dataset_output = dependencies.dataset_output
        self._mapping_service = dependencies.mapping_service
        self._domain_frame_builder = dependencies.domain_frame_builder
        self._suppqual_service = dependencies.suppqual_service
        self._domain_definition_repository = dependencies.domain_definition_repository
        self._ct_repository = dependencies.ct_repository

    def _log_conformance_report(
        self, *, frame: pd.DataFrame, domain: SDTMDomain, strict: bool
    ) -> ConformanceReport | None:
        if not strict:
            return None
        ct_repo = self._ct_repository

        def _ct_resolver(variable: SDTMVariable) -> ControlledTerminology | None:
            if ct_repo is None:
                return None
            codelist_code = getattr(variable, "codelist_code", None)
            if codelist_code:
                return ct_repo.get_by_code(codelist_code)
            return ct_repo.get_by_name(getattr(variable, "name", ""))

        report = check_domain_dataframe(frame, domain, ct_resolver=_ct_resolver)
        if not report.issues:
            return report
        header = f"{domain.code}: conformance issues (errors={report.error_count()}, warnings={report.warning_count()})"
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
        response = ProcessDomainResponse(domain_code=request.domain_code)
        try:
            domain = self._get_domain(request.domain_code)
            all_dataframes: list[pd.DataFrame] = []
            last_config: MappingConfig | None = None
            suppqual_frames: list[pd.DataFrame] = []
            for input_file, variant_name in request.files_for_domain:
                result = self._process_single_file(
                    input_file=input_file,
                    variant_name=variant_name,
                    request=request,
                    domain=domain,
                )
                if result is None:
                    continue
                frame, config, _is_findings_long, consumed_cols = result
                all_dataframes.append(frame)
                last_config = config
                if request.domain_code.upper() != "LB":
                    supp_df = self._generate_suppqual_stage(
                        source_df=self._load_file(input_file),
                        domain_df=frame,
                        config=config,
                        request=request,
                        extra_consumed_columns=consumed_cols,
                    )
                    if supp_df is not None and (not supp_df.empty):
                        suppqual_frames.append(supp_df)
            if not all_dataframes:
                raise ValueError(
                    f"No data could be processed for {request.domain_code}"
                )
            if last_config is None:
                raise RuntimeError("Config should be set if we have dataframes")
            merged_df = self._merge_dataframes(
                all_dataframes, request.domain_code, request.verbose > 0
            )
            if request.domain_code.upper() == "LB":
                merged_df = self._deduplicate_lb_data(merged_df, request.domain_code)
            strict = "xpt" in request.output_formats or request.generate_sas
            report = self._log_conformance_report(
                frame=merged_df, domain=domain, strict=strict
            )
            response.conformance_report = report
            if (
                strict
                and request.fail_on_conformance_errors
                and (report is not None)
                and getattr(report, "has_errors", lambda: False)()
            ):
                response.success = False
                response.records = len(merged_df)
                response.domain_dataframe = merged_df
                response.config = last_config
                response.error = f"{request.domain_code}: conformance errors present; strict output generation aborted"
                return response
            output_result = self._generate_outputs_stage(
                merged_df=merged_df,
                config=last_config,
                domain=domain,
                request=request,
                suppqual_frames=suppqual_frames,
            )
            response.success = True
            response.records = len(merged_df)
            response.domain_dataframe = merged_df
            response.config = last_config
            response.xpt_path = output_result.xpt_path
            response.xml_path = output_result.xml_path
            response.sas_path = output_result.sas_path
            for supp_result in output_result.suppqual_domains:
                supp_response = ProcessDomainResponse(
                    success=True,
                    domain_code=supp_result.domain_code,
                    records=supp_result.records,
                    domain_dataframe=supp_result.domain_dataframe,
                    config=supp_result.config,
                    xpt_path=supp_result.xpt_path,
                    xml_path=supp_result.xml_path,
                    sas_path=supp_result.sas_path,
                )
                response.suppqual_domains.append(supp_response)
        except Exception as exc:
            response.success = False
            response.error = str(exc)
            self.logger.error(f"{request.domain_code}: {exc}")
            if request.verbose >= VERBOSE_TRACEBACK_LEVEL:
                self.logger.error(traceback.format_exc())
        return response

    def _process_single_file(
        self,
        input_file: Path,
        variant_name: str,
        request: ProcessDomainRequest,
        domain: SDTMDomain,
    ) -> tuple[pd.DataFrame, MappingConfig, bool, set[str]] | None:
        display_name = (
            f"{request.domain_code}"
            if variant_name == request.domain_code
            else f"{request.domain_code} ({variant_name})"
        )
        frame = self._load_file(input_file)
        row_count = len(frame)
        col_count = len(frame.columns)
        self.logger.log_file_loaded(
            input_file.name, row_count=row_count, column_count=col_count
        )
        self.logger.info(
            f"Loaded {input_file.name}: {row_count:,} rows, {col_count} columns"
        )
        if request.verbose > 0 and row_count > 0:
            col_names = ", ".join(frame.columns[:COLUMN_SAMPLE_LIMIT].tolist())
            if len(frame.columns) > COLUMN_SAMPLE_LIMIT:
                col_names += f" ... (+{len(frame.columns) - COLUMN_SAMPLE_LIMIT} more)"
            self.logger.verbose(f"    Columns: {col_names}")
        if self._should_skip_vstat(
            request.domain_code, variant_name, request.verbose > 0
        ):
            return None
        is_findings_long = False
        consumed_columns: set[str] = set()
        config = self._build_config(
            frame=frame,
            request=request,
            is_findings_long=is_findings_long,
            display_name=display_name,
        )
        if config is None:
            return None
        config.study_id = request.study_id
        config.default_country = request.default_country
        lenient = "xpt" not in request.output_formats and (not request.generate_sas)
        build_request = DomainFrameBuildRequest(
            frame=frame,
            config=config,
            domain=domain,
            reference_starts=request.reference_starts,
            lenient=lenient,
            metadata=request.metadata,
        )
        domain_df = self._domain_frame_builder.build_domain_dataframe(build_request)
        output_rows = len(domain_df)
        self.logger.info(f"{request.domain_code}: {output_rows:,} rows processed")
        if output_rows != row_count and request.verbose > 0:
            change_pct = (
                (output_rows - row_count) / row_count * 100 if row_count > 0 else 0
            )
            direction = "+" if change_pct > 0 else ""
            self.logger.verbose(
                f"    Row count changed: {row_count:,} → {output_rows:,} ({direction}{change_pct:.1f}%)"
            )
        return (domain_df, config, is_findings_long, consumed_columns)

    def _load_file(self, file_path: Path) -> pd.DataFrame:
        return self._study_data_repository.read_dataset(file_path)

    def _should_skip_vstat(
        self, domain_code: str, variant_name: str | None, verbose: bool
    ) -> bool:
        if (
            domain_code.upper() == "VS"
            and variant_name
            and ("VSTAT" in variant_name.upper())
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
        request: ProcessDomainRequest,
        *,
        is_findings_long: bool,
        display_name: str,
    ) -> MappingConfig | None:
        if is_findings_long:
            mappings = [
                ColumnMapping(
                    source_column=col,
                    target_variable=col,
                    transformation=None,
                    confidence_score=1.0,
                )
                for col in frame.columns
            ]
            config = build_config(request.domain_code, mappings)
            if request.verbose > 0:
                self.logger.verbose("    Using identity mapping (post-transformation)")
            return config
        column_hints = _build_column_hints(frame)
        suggestions = self._mapping_service.suggest(
            domain_code=request.domain_code,
            frame=frame,
            metadata=request.metadata,
            min_confidence=request.min_confidence,
            column_hints=column_hints,
        )
        if not suggestions.mappings:
            self.logger.warning(f"{display_name}: No mappings found, skipping")
            return None
        config = build_config(request.domain_code, suggestions.mappings)
        if request.verbose > 0:
            mapping_count = len(config.mappings) if config.mappings else 0
            self.logger.verbose(
                f"    Column mappings: {mapping_count} variables mapped"
            )
        return config

    def _generate_suppqual_stage(
        self,
        source_df: pd.DataFrame,
        domain_df: pd.DataFrame,
        config: MappingConfig,
        request: ProcessDomainRequest,
        extra_consumed_columns: set[str] | None = None,
    ) -> pd.DataFrame | None:
        used_columns = self._suppqual_service.extract_used_columns(config)
        if extra_consumed_columns:
            used_columns.update(extra_consumed_columns)
        domain_def = self._get_domain(request.domain_code)
        supp_request = SuppqualBuildRequest(
            domain_code=request.domain_code,
            source_df=source_df,
            mapped_df=domain_df,
            domain_def=domain_def,
            used_source_columns=used_columns,
            study_id=request.study_id,
            common_column_counts=request.common_column_counts,
            total_files=request.total_input_files,
        )
        supp_df, _ = self._suppqual_service.build_suppqual(supp_request)
        if supp_df is not None and {"QNAM", "QLABEL"} <= set(supp_df.columns):
            qnam: pd.Series = supp_df["QNAM"].astype("string").fillna("")
            qlabel: pd.Series = supp_df["QLABEL"].astype("string").fillna("")
            canonical: dict[str, str] = {}
            for name in qnam.unique():
                if not str(name).strip():
                    continue
                labels: pd.Series = qlabel.loc[qnam == name]
                first_non_empty = next((str(v) for v in labels if str(v).strip()), "")
                canonical[str(name)] = first_non_empty.strip() or str(name)
            if canonical:
                supp_df.loc[:, "QLABEL"] = qnam.map(
                    lambda v: canonical.get(str(v), str(v))
                ).astype("string")
        if request.domain_code.upper() == "AE":
            required = {"USUBJID", "AESEQ"}
            if required <= set(domain_df.columns):
                base = domain_df
                usubjid = base["USUBJID"].astype("string").fillna("").str.strip()
                aeseq = base["AESEQ"].astype("Int64")
                qval = pd.Series(["Y"] * len(base), index=base.index, dtype="string")
                if "AESTDTC" in base.columns:
                    ae_start = pd.to_datetime(base["AESTDTC"], errors="coerce")
                    baseline = None
                    if request.reference_starts:
                        baseline = pd.to_datetime(
                            usubjid.map(request.reference_starts), errors="coerce"
                        )
                    if baseline is not None:
                        derived = (ae_start >= baseline).map(
                            lambda v: "Y" if bool(v) else "N"
                        )
                        derived = derived.where(
                            ae_start.notna() & baseline.notna(), "Y"
                        ).astype("string")
                        qval = derived
                trtem_records = pd.DataFrame(
                    {
                        "STUDYID": request.study_id,
                        "RDOMAIN": "AE",
                        "USUBJID": usubjid,
                        "IDVAR": "AESEQ",
                        "IDVARVAL": aeseq.astype("string"),
                        "QNAM": "TRTEMFL",
                        "QLABEL": "Treatment Emergent Flag",
                        "QVAL": qval,
                        "QORIG": "DERIVED",
                        "QEVAL": "",
                    }
                )
                if supp_df is None:
                    supp_df = trtem_records
                else:
                    supp_df = pd.concat([supp_df, trtem_records], ignore_index=True)
        return supp_df

    def _generate_outputs_stage(
        self,
        merged_df: pd.DataFrame,
        config: MappingConfig,
        domain: SDTMDomain,
        request: ProcessDomainRequest,
        suppqual_frames: list[pd.DataFrame],
    ) -> OutputStageResult:
        base_filename = domain.resolved_dataset_name()
        result = OutputStageResult(
            domain_code=request.domain_code,
            records=len(merged_df),
            domain_dataframe=merged_df,
            config=config,
        )
        if suppqual_frames:
            suppqual_result = self._generate_suppqual_files(
                suppqual_frames,
                request.domain_code,
                request.study_id,
                request.output_formats,
                request.output_dirs,
            )
            result.suppqual_domains.append(suppqual_result)
        if self._dataset_output is not None:
            formats: set[str] = set()
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
                first_input_file = (
                    request.files_for_domain[0][0] if request.files_for_domain else None
                )
                input_dataset = first_input_file.stem if first_input_file else None
                output_request = DatasetOutputRequest(
                    dataframe=merged_df,
                    domain_code=request.domain_code,
                    config=config,
                    output_dirs=DatasetOutputDirs(
                        xpt_dir=xpt_dir, xml_dir=xml_dir, sas_dir=sas_dir
                    ),
                    formats=formats,
                    base_filename=base_filename,
                    input_dataset=input_dataset,
                    output_dataset=base_filename,
                )
                output_result = self._dataset_output.generate(output_request)
                if output_result.xpt_path:
                    result.xpt_path = output_result.xpt_path
                    self.logger.success(f"Generated XPT: {output_result.xpt_path}")
                if output_result.xml_path:
                    result.xml_path = output_result.xml_path
                    self.logger.success(
                        f"Generated Dataset-XML: {output_result.xml_path}"
                    )
                if output_result.sas_path:
                    result.sas_path = output_result.sas_path
                    self.logger.success(f"Generated SAS: {output_result.sas_path}")
                for error in output_result.errors:
                    self.logger.error(f"Output generation error: {error}")
        return result

    def _generate_suppqual_files(
        self,
        suppqual_frames: list[pd.DataFrame],
        domain_code: str,
        study_id: str,
        output_formats: set[str],
        output_dirs: dict[str, Path | None],
    ) -> SuppqualOutputResult:
        merged_supp = (
            suppqual_frames[0]
            if len(suppqual_frames) == 1
            else pd.concat(suppqual_frames, ignore_index=True)
        )
        supp_domain_code = f"SUPP{domain_code.upper()}"
        try:
            supp_domain_def = self._get_domain(supp_domain_code)
        except Exception:
            supp_domain_def = None
        if not merged_supp.empty:
            merged_supp = self._suppqual_service.finalize_suppqual(
                merged_supp, supp_domain_def=supp_domain_def
            )
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
        suppqual_result = SuppqualOutputResult(
            domain_code=supp_domain_code,
            records=len(merged_supp),
            domain_dataframe=merged_supp,
            config=supp_config,
        )
        if self._dataset_output is not None:
            formats: set[str] = set()
            xpt_dir = output_dirs.get("xpt")
            xml_dir = output_dirs.get("xml")
            if xpt_dir and "xpt" in output_formats:
                formats.add("xpt")
            if xml_dir and "xml" in output_formats:
                formats.add("xml")
            if formats:
                output_request = DatasetOutputRequest(
                    dataframe=merged_supp,
                    domain_code=supp_domain_code,
                    config=supp_config,
                    output_dirs=DatasetOutputDirs(xpt_dir=xpt_dir, xml_dir=xml_dir),
                    formats=formats,
                    base_filename=base_filename,
                )
                output_result = self._dataset_output.generate(output_request)
                if output_result.xpt_path:
                    suppqual_result.xpt_path = output_result.xpt_path
                if output_result.xml_path:
                    suppqual_result.xml_path = output_result.xml_path
        return suppqual_result

    def _get_domain(self, domain_code: str) -> SDTMDomain:
        return self._domain_definition_repository.get_domain(domain_code)

    def _merge_dataframes(
        self, all_dataframes: list[pd.DataFrame], domain_code: str, verbose: bool
    ) -> pd.DataFrame:
        if len(all_dataframes) == 1:
            return all_dataframes[0]
        union_set = {col for df in all_dataframes for col in df.columns.astype(str)}
        union_columns: list[str]
        try:
            domain = self._get_domain(domain_code)
            ordered = [v.name for v in domain.variables if v.name in union_set]
            extras = sorted([c for c in union_set if c not in ordered])
            union_columns = ordered + extras
        except Exception:
            union_columns = sorted(union_set)
        non_empty = [df for df in all_dataframes if not df.empty]
        if not non_empty:
            return pd.DataFrame(columns=union_columns)
        input_rows_list = [len(df) for df in all_dataframes]
        total_input = sum(input_rows_list)
        merged_df = pd.concat(non_empty, ignore_index=True)
        if union_columns:
            merged_df = merged_df.reindex(columns=union_columns)
        merged_rows = len(merged_df)
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
                pct = rows / merged_rows * 100 if merged_rows > 0 else 0
                self.logger.verbose(f"    File {i + 1}: {rows:,} rows ({pct:.1f}%)")
        return merged_df

    def _deduplicate_lb_data(
        self, merged_df: pd.DataFrame, domain_code: str
    ) -> pd.DataFrame:
        dedup_keys = [
            key for key in ("USUBJID", "LBTESTCD", "LBDTC") if key in merged_df.columns
        ]
        if dedup_keys:
            merged_df = (
                merged_df.copy()
                .sort_values(by=dedup_keys, kind="mergesort")
                .drop_duplicates(subset=dedup_keys, keep="first")
                .reset_index(drop=True)
            )
            seq_col = f"{domain_code}SEQ"
            if seq_col in merged_df.columns and "USUBJID" in merged_df.columns:
                merged_df.loc[:, seq_col] = merged_df.groupby("USUBJID").cumcount() + 1
        return merged_df
