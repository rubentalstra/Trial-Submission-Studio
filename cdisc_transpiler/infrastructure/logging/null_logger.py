"""Null logger for testing.

This module provides a silent logger implementation that discards
all log messages. Useful for testing without console output.
"""

from __future__ import annotations

from pathlib import Path

from ...application.ports.services import LoggerPort


class NullLogger(LoggerPort):
    """Silent logger that discards all messages.

    This logger implements the LoggerPort protocol but produces no output.
    Useful for testing services without cluttering test output.

    Example:
        >>> logger = NullLogger()
        >>> logger.info("This message is discarded")
        >>> logger.error("This too")
    """

    def info(self, message: str) -> None:
        """Log an informational message (discarded).

        Args:
            message: The message to log (ignored)
        """
        return None

    def success(self, message: str) -> None:
        """Log a success message (discarded).

        Args:
            message: The message to log (ignored)
        """
        return None

    def warning(self, message: str) -> None:
        """Log a warning message (discarded).

        Args:
            message: The message to log (ignored)
        """
        return None

    def error(self, message: str) -> None:
        """Log an error message (discarded).

        Args:
            message: The message to log (ignored)
        """
        return None

    def debug(self, message: str) -> None:
        """Log a debug message (discarded).

        Args:
            message: The message to log (ignored)
        """
        return None

    def verbose(self, message: str) -> None:
        """Log a verbose message (discarded).

        Args:
            message: The message to log (ignored)
        """
        return None

    def log_study_start(
        self,
        study_id: str,
        study_folder: Path,
        output_format: str,
        supported_domains: list[str],
    ) -> None:
        return None

    def log_metadata_loaded(
        self,
        *,
        items_count: int | None,
        codelists_count: int | None,
    ) -> None:
        return None

    def log_processing_summary(
        self,
        *,
        study_id: str,
        domain_count: int,
        file_count: int,
        output_format: str,
        generate_define: bool,
        generate_sas: bool,
    ) -> None:
        return None

    def log_final_stats(self) -> None:
        return None

    def log_domain_start(
        self, domain_code: str, files_for_domain: list[tuple[Path, str]]
    ) -> None:
        return None

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

    def log_file_loaded(
        self,
        filename: str,
        row_count: int,
        column_count: int | None = None,
    ) -> None:
        return None

    def log_synthesis_start(self, domain_code: str, reason: str) -> None:
        return None

    def log_synthesis_complete(self, domain_code: str, records: int) -> None:
        return None
