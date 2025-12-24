from typing import TYPE_CHECKING, Protocol, runtime_checkable

if TYPE_CHECKING:
    from collections.abc import Iterable
    from pathlib import Path

    import pandas as pd

    from ...domain.entities.column_hints import Hints
    from ...domain.entities.mapping import MappingConfig, MappingSuggestions
    from ...domain.entities.sdtm_domain import SDTMDomain
    from ...domain.entities.study_metadata import StudyMetadata
    from ...domain.services.domain_frame_builder import DomainFrameBuildRequest
    from ...domain.services.sdtm_conformance_checker import ConformanceReport
    from ...domain.services.suppqual_service import SuppqualBuildRequest
    from ..models import (
        DatasetOutputRequest,
        DatasetOutputResult,
        DefineDatasetDTO,
        ProcessingSummary,
    )


@runtime_checkable
class OutputPreparerPort(Protocol):
    pass

    def prepare(
        self,
        *,
        output_dir: Path,
        output_formats: set[str],
        generate_sas: bool,
        generate_define_xml: bool,
    ) -> None: ...


@runtime_checkable
class LoggerPort(Protocol):
    pass

    def info(self, message: str) -> None: ...

    def success(self, message: str) -> None: ...

    def warning(self, message: str) -> None: ...

    def error(self, message: str) -> None: ...

    def debug(self, message: str) -> None: ...

    def verbose(self, message: str) -> None: ...

    def log_study_start(
        self,
        study_id: str,
        study_folder: Path,
        output_format: str,
        supported_domains: list[str],
    ) -> None: ...

    def log_metadata_loaded(
        self, *, items_count: int | None, codelists_count: int | None
    ) -> None: ...

    def log_processing_summary(self, summary: ProcessingSummary) -> None: ...

    def log_final_stats(self) -> None: ...

    def log_domain_start(
        self, domain_code: str, files_for_domain: list[tuple[Path, str]]
    ) -> None: ...

    def log_domain_complete(
        self,
        domain_code: str,
        final_row_count: int,
        final_column_count: int,
        *,
        skipped: bool = False,
        reason: str | None = None,
    ) -> None: ...

    def log_file_loaded(
        self, filename: str, row_count: int, column_count: int | None = None
    ) -> None: ...

    def log_synthesis_start(self, domain_code: str, reason: str) -> None: ...

    def log_synthesis_complete(self, domain_code: str, records: int) -> None: ...


@runtime_checkable
class DatasetOutputPort(Protocol):
    pass

    def generate(self, request: DatasetOutputRequest) -> DatasetOutputResult: ...


@runtime_checkable
class DomainDiscoveryPort(Protocol):
    pass

    def discover_domain_files(
        self, csv_files: list[Path], supported_domains: list[str]
    ) -> dict[str, list[tuple[Path, str]]]: ...


@runtime_checkable
class DomainFrameBuilderPort(Protocol):
    pass

    def build_domain_dataframe(
        self, request: DomainFrameBuildRequest
    ) -> pd.DataFrame: ...


@runtime_checkable
class SuppqualPort(Protocol):
    pass

    def extract_used_columns(self, config: MappingConfig | None) -> set[str]: ...

    def build_suppqual(
        self, request: SuppqualBuildRequest
    ) -> tuple[pd.DataFrame | None, set[str]]: ...

    def finalize_suppqual(
        self, supp_df: pd.DataFrame, *, supp_domain_def: SDTMDomain | None = None
    ) -> pd.DataFrame: ...


@runtime_checkable
class MappingPort(Protocol):
    pass

    def suggest(
        self,
        *,
        domain_code: str,
        frame: pd.DataFrame,
        metadata: StudyMetadata | None = None,
        min_confidence: float = 0.5,
        column_hints: Hints | None = None,
    ) -> MappingSuggestions: ...


@runtime_checkable
class XPTWriterPort(Protocol):
    pass

    def write(
        self,
        dataframe: pd.DataFrame,
        domain_code: str,
        output_path: Path,
        *,
        file_label: str | None = None,
        table_name: str | None = None,
    ) -> None: ...


@runtime_checkable
class DatasetXMLWriterPort(Protocol):
    pass

    def write(
        self,
        dataframe: pd.DataFrame,
        domain_code: str,
        config: MappingConfig,
        output_path: Path,
    ) -> None: ...


@runtime_checkable
class SASWriterPort(Protocol):
    pass

    def write(
        self,
        domain_code: str,
        config: MappingConfig,
        output_path: Path,
        input_dataset: str | None = None,
        output_dataset: str | None = None,
    ) -> None: ...


@runtime_checkable
class ConformanceReportWriterPort(Protocol):
    pass

    def write_json(
        self,
        *,
        output_dir: Path,
        study_id: str,
        reports: Iterable[ConformanceReport],
        filename: str = "conformance_report.json",
    ) -> Path: ...


@runtime_checkable
class DefineXMLGeneratorPort(Protocol):
    pass

    def generate(
        self,
        datasets: Iterable[DefineDatasetDTO],
        output_path: Path,
        *,
        sdtm_version: str,
        context: str,
    ) -> None: ...
