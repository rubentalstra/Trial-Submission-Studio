from typing import TYPE_CHECKING, override

from ...application.ports.services import LoggerPort

if TYPE_CHECKING:
    from pathlib import Path

    from ...application.models import ProcessingSummary


class NullLogger(LoggerPort):
    pass

    @override
    def info(self, message: str) -> None:
        return

    @override
    def success(self, message: str) -> None:
        return

    @override
    def warning(self, message: str) -> None:
        return

    @override
    def error(self, message: str) -> None:
        return

    @override
    def debug(self, message: str) -> None:
        return

    @override
    def verbose(self, message: str) -> None:
        return

    @override
    def log_study_start(
        self,
        study_id: str,
        study_folder: Path,
        output_format: str,
        supported_domains: list[str],
    ) -> None:
        return None

    @override
    def log_metadata_loaded(
        self, *, items_count: int | None, codelists_count: int | None
    ) -> None:
        return None

    @override
    def log_processing_summary(self, summary: ProcessingSummary) -> None:
        return None

    @override
    def log_final_stats(self) -> None:
        return None

    @override
    def log_domain_start(
        self, domain_code: str, files_for_domain: list[tuple[Path, str]]
    ) -> None:
        return None

    @override
    def log_domain_complete(
        self,
        domain_code: str,
        final_row_count: int,
        final_column_count: int,
        *,
        skipped: bool = False,
        reason: str | None = None,
    ) -> None:
        return None

    @override
    def log_file_loaded(
        self, filename: str, row_count: int, column_count: int | None = None
    ) -> None:
        return None

    @override
    def log_synthesis_start(self, domain_code: str, reason: str) -> None:
        return None

    @override
    def log_synthesis_complete(self, domain_code: str, records: int) -> None:
        return None
