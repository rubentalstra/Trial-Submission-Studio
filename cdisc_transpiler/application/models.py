from dataclasses import dataclass, field
from typing import TYPE_CHECKING

from ..constants import Defaults, SDTMVersions

if TYPE_CHECKING:
    from pathlib import Path

    import pandas as pd

    from ..domain.entities.mapping import MappingConfig
    from ..domain.entities.study_metadata import StudyMetadata
    from ..domain.services.sdtm_conformance_checker import ConformanceReport


def _default_output_formats() -> set[str]:
    return {"xpt", "xml"}


def _empty_str_list() -> list[str]:
    return []


def _empty_str_set() -> set[str]:
    return set()


def _empty_output_dirs() -> dict[str, Path | None]:
    return {}


def _empty_domain_results() -> list[DomainProcessingResult]:
    return []


def _empty_domain_responses() -> list[ProcessDomainResponse]:
    return []


def _empty_error_list() -> list[tuple[str, str]]:
    return []


@dataclass(slots=True)
class DatasetOutputDirs:
    xpt_dir: Path | None = None
    xml_dir: Path | None = None
    sas_dir: Path | None = None


@dataclass(slots=True)
class DatasetOutputRequest:
    dataframe: pd.DataFrame
    domain_code: str
    config: MappingConfig
    output_dirs: DatasetOutputDirs
    formats: set[str]
    base_filename: str | None = None
    input_dataset: str | None = None
    output_dataset: str | None = None


@dataclass(slots=True)
class DatasetOutputResult:
    xpt_path: Path | None = None
    xml_path: Path | None = None
    sas_path: Path | None = None
    errors: list[str] = field(default_factory=_empty_str_list)

    @property
    def success(self) -> bool:
        return len(self.errors) == 0


@dataclass(slots=True)
class ProcessingSummary:
    study_id: str
    domain_count: int
    file_count: int
    output_format: str
    generate_define: bool
    generate_sas: bool


@dataclass(slots=True)
class DefineDatasetDTO:
    domain_code: str
    dataframe: pd.DataFrame
    config: MappingConfig
    label: str | None = None
    structure: str = "One record per subject per domain-specific entity"
    archive_location: Path | None = None


@dataclass(slots=True)
class ProcessStudyRequest:
    study_folder: Path
    study_id: str
    output_dir: Path
    output_formats: set[str] = field(default_factory=_default_output_formats)
    generate_define_xml: bool = True
    generate_sas: bool = True
    sdtm_version: str = SDTMVersions.DEFAULT_VERSION
    define_context: str = SDTMVersions.DEFINE_CONTEXT_SUBMISSION
    streaming: bool = False
    chunk_size: int = Defaults.CHUNK_SIZE
    min_confidence: float = Defaults.MIN_CONFIDENCE
    verbose: int = 0
    write_conformance_report_json: bool = True
    fail_on_conformance_errors: bool = False
    default_country: str | None = None


@dataclass(slots=True)
class DomainProcessingResult:
    domain_code: str
    success: bool = True
    records: int = 0
    domain_dataframe: pd.DataFrame | None = None
    config: MappingConfig | None = None
    xpt_path: Path | None = None
    xml_path: Path | None = None
    sas_path: Path | None = None
    suppqual_domains: list[DomainProcessingResult] = field(
        default_factory=_empty_domain_results
    )
    error: str | None = None
    synthesized: bool = False
    synthesis_reason: str | None = None
    conformance_report: ConformanceReport | None = None


@dataclass(slots=True)
class ProcessStudyResponse:
    success: bool = True
    study_id: str = ""
    processed_domains: set[str] = field(default_factory=_empty_str_set)
    domain_results: list[DomainProcessingResult] = field(
        default_factory=_empty_domain_results
    )
    errors: list[tuple[str, str]] = field(default_factory=_empty_error_list)
    define_xml_path: Path | None = None
    define_xml_error: str | None = None
    output_dir: Path | None = None
    total_records: int = 0
    conformance_report_path: Path | None = None
    conformance_report_error: str | None = None

    @property
    def has_errors(self) -> bool:
        return len(self.errors) > 0 or self.define_xml_error is not None

    @property
    def successful_domains(self) -> list[str]:
        return [r.domain_code for r in self.domain_results if r.success]

    @property
    def failed_domains(self) -> list[str]:
        return [code for code, _ in self.errors]


@dataclass(slots=True)
class ProcessDomainRequest:
    files_for_domain: list[tuple[Path, str]]
    domain_code: str
    study_id: str
    output_formats: set[str] = field(default_factory=_default_output_formats)
    output_dirs: dict[str, Path | None] = field(default_factory=_empty_output_dirs)
    min_confidence: float = 0.5
    streaming: bool = False
    chunk_size: int = 1000
    generate_sas: bool = True
    verbose: int = 0
    metadata: StudyMetadata | None = None
    reference_starts: dict[str, str] | None = None
    common_column_counts: dict[str, int] | None = None
    total_input_files: int | None = None
    fail_on_conformance_errors: bool = False
    default_country: str | None = None


@dataclass(slots=True)
class ProcessDomainResponse:
    success: bool = True
    domain_code: str = ""
    records: int = 0
    domain_dataframe: pd.DataFrame | None = None
    config: MappingConfig | None = None
    xpt_path: Path | None = None
    xml_path: Path | None = None
    sas_path: Path | None = None
    suppqual_domains: list[ProcessDomainResponse] = field(
        default_factory=_empty_domain_responses
    )
    error: str | None = None
    warnings: list[str] = field(default_factory=_empty_str_list)
    conformance_report: ConformanceReport | None = None

    def to_dict(self) -> dict[str, object]:
        result: dict[str, object] = {
            "domain_code": self.domain_code,
            "records": self.records,
            "domain_dataframe": self.domain_dataframe,
            "config": self.config,
            "xpt_path": self.xpt_path,
            "xml_path": self.xml_path,
            "sas_path": self.sas_path,
            "suppqual_domains": [
                {
                    "domain_code": supp.domain_code,
                    "records": supp.records,
                    "domain_dataframe": supp.domain_dataframe,
                    "config": supp.config,
                    "xpt_path": supp.xpt_path,
                    "xml_path": supp.xml_path,
                    "sas_path": supp.sas_path,
                }
                for supp in self.suppqual_domains
            ],
            "conformance_report": self.conformance_report,
        }
        return result
