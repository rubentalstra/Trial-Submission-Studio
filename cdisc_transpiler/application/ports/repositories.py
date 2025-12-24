from typing import TYPE_CHECKING, Protocol, runtime_checkable

if TYPE_CHECKING:
    from pathlib import Path

    import pandas as pd

    from ...domain.entities.controlled_terminology import ControlledTerminology
    from ...domain.entities.sdtm_domain import SDTMDomain
    from ...domain.entities.study_metadata import StudyMetadata


@runtime_checkable
class CTRepositoryPort(Protocol):
    pass

    def get_by_code(self, codelist_code: str) -> ControlledTerminology | None: ...

    def get_by_name(self, codelist_name: str) -> ControlledTerminology | None: ...

    def list_all_codes(self) -> list[str]: ...


@runtime_checkable
class SDTMSpecRepositoryPort(Protocol):
    pass

    def get_domain_variables(self, domain_code: str) -> list[dict[str, str]]: ...

    def get_dataset_attributes(self, domain_code: str) -> dict[str, str] | None: ...

    def list_available_domains(self) -> list[str]: ...


@runtime_checkable
class DomainDefinitionRepositoryPort(Protocol):
    pass

    def list_domains(self) -> list[str]: ...

    def get_domain(self, domain_code: str) -> SDTMDomain: ...


@runtime_checkable
class StudyDataRepositoryPort(Protocol):
    pass

    def read_dataset(self, file_path: str | Path) -> pd.DataFrame: ...

    def load_study_metadata(self, study_folder: Path) -> StudyMetadata: ...

    def list_data_files(self, folder: Path, pattern: str = "*.csv") -> list[Path]: ...
