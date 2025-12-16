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
        pass

    def success(self, message: str) -> None:
        """Log a success message (discarded).

        Args:
            message: The message to log (ignored)
        """
        pass

    def warning(self, message: str) -> None:
        """Log a warning message (discarded).

        Args:
            message: The message to log (ignored)
        """
        pass

    def error(self, message: str) -> None:
        """Log an error message (discarded).

        Args:
            message: The message to log (ignored)
        """
        pass

    def debug(self, message: str) -> None:
        """Log a debug message (discarded).

        Args:
            message: The message to log (ignored)
        """
        pass

    def verbose(self, message: str) -> None:
        """Log a verbose message (discarded).

        Args:
            message: The message to log (ignored)
        """
        pass

    def log_study_start(
        self,
        study_id: str,
        study_folder: Path,
        output_format: str,
        supported_domains: list[str],
    ) -> None:
        pass

    def log_metadata_loaded(
        self,
        *,
        items_count: int | None,
        codelists_count: int | None,
    ) -> None:
        pass

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
        pass

    def log_final_stats(self) -> None:
        pass

    def log_domain_start(
        self, domain_code: str, files_for_domain: list[tuple[Path, str]]
    ) -> None:
        pass

    def log_synthesis_start(self, domain_code: str, reason: str) -> None:
        pass

    def log_synthesis_complete(self, domain_code: str, records: int) -> None:
        pass
