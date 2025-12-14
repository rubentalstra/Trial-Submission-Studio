"""Progress Reporting Service - User feedback and status reporting.

This service handles progress tracking, status messages, and result
summaries for study processing operations.

Extracted from cli/commands/study.py as part of Phase 2 refactoring.
"""

from __future__ import annotations

from typing import Any


def _log_verbose(enabled: bool, message: str) -> None:
    """Deferred import wrapper to avoid circular imports."""
    from ..cli.helpers import log_verbose
    log_verbose(enabled, message)


class ProgressReportingService:
    """Service for reporting progress and status to the user.

    This service provides a consistent interface for progress tracking,
    status messages, and summary reporting during study processing.
    """

    def __init__(self, console: Any, verbose: int = 0):
        """Initialize the progress reporting service.

        Args:
            console: Rich console instance for output
            verbose: Verbosity level (0 = normal, 1+ = verbose)
        """
        self.console = console
        self.verbose = verbose

    def report_study_info(
        self,
        study_id: str,
        study_folder: Any,
        output_format: str,
        supported_domains: list[str],
    ) -> None:
        """Report initial study processing information.

        Args:
            study_id: Study identifier
            study_folder: Path to study folder
            output_format: Output format being used
            supported_domains: List of supported domain codes
        """

        _log_verbose(self.verbose > 0, f"Processing study folder: {study_folder}")
        _log_verbose(self.verbose > 0, f"Study ID: {study_id}")
        _log_verbose(self.verbose > 0, f"Output format: {output_format}")
        log_verbose(
            self.verbose > 0,
            f"Supported domains: {', '.join(supported_domains)}",
        )

    def report_metadata_loaded(
        self, items_count: int | None, codelists_count: int | None
    ) -> None:
        """Report loaded metadata information.

        Args:
            items_count: Number of items loaded from Items.csv
            codelists_count: Number of codelists loaded from CodeLists.csv
        """

        if items_count:
            log_verbose(
                self.verbose > 0,
                f"Loaded {items_count} column definitions from Items.csv",
            )
        if codelists_count:
            log_verbose(
                self.verbose > 0,
                f"Loaded {codelists_count} codelists from CodeLists.csv",
            )

    def report_files_found(self, csv_count: int) -> None:
        """Report number of CSV files found.

        Args:
            csv_count: Number of CSV files found
        """

        _log_verbose(self.verbose > 0, f"Found {csv_count} CSV files")

    def report_study_summary(
        self,
        study_id: str,
        domain_files: dict[str, list[Any]],
        total_files: int,
        output_format: str,
        generate_define: bool,
        generate_sas: bool,
    ) -> None:
        """Report summary of study processing configuration.

        Args:
            study_id: Study identifier
            domain_files: Dictionary of domains to process
            total_files: Total number of files to process
            output_format: Output format
            generate_define: Whether Define-XML will be generated
            generate_sas: Whether SAS programs will be generated
        """
        self.console.print(f"\n[bold]Study: {study_id}[/bold]")
        self.console.print(
            f"[bold]Found {len(domain_files)} domains ({total_files} files) to process[/bold]"
        )
        self.console.print(f"[bold]Output format:[/bold] {output_format.upper()}")
        if generate_define:
            self.console.print("[bold]Define-XML:[/bold] Will be generated")
        if generate_sas:
            self.console.print("[bold]SAS programs:[/bold] Will be generated")

    def report_domain_processing(
        self, domain_code: str, files_for_domain: list[tuple[Any, str]]
    ) -> None:
        """Report domain processing start.

        Args:
            domain_code: Domain code being processed
            files_for_domain: List of (file_path, variant_name) tuples
        """
        variant_names = [v for _, v in files_for_domain]
        if len(files_for_domain) == 1:
            display_name = domain_code
        else:
            display_name = f"{domain_code} (merging {', '.join(variant_names)})"

        self.console.print(f"\n[bold]Processing {display_name}[/bold]")
        for input_file, variant_name in files_for_domain:
            self.console.print(f"  - {input_file.name}")

    def report_synthesis(self, domain_code: str, reason: str) -> None:
        """Report domain synthesis.

        Args:
            domain_code: Domain code being synthesized
            reason: Reason for synthesis
        """
        self.console.print(f"\n[bold]Synthesizing {domain_code}[/bold]: {reason}")
