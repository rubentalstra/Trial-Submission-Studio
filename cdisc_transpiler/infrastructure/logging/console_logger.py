"""Console logger implementation using Rich.

This module provides structured logging with multiple verbosity levels,
rich formatting, and SDTM-specific context information. It implements
the LoggerPort protocol for dependency injection.

Verbosity Levels:
    0 = Normal: Essential progress messages only
    1 = Verbose (-v): Detailed processing information
    2 = Debug (-vv): Full debug output with domain context prefixes

SDTM Reference:
    Logging output is designed to help users understand SDTM domain
    processing as defined in SDTMIG v3.4, including:
    - Domain classification (Interventions, Events, Findings, etc.)
    - Data transformations (wide-to-long reshaping)
    - Supplemental qualifier generation
    - Split dataset handling per Section 4.1.7
"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime
from enum import IntEnum
from pathlib import Path
from typing import Any

from rich.console import Console

from ...application.ports.services import LoggerPort
from ..sdtm_spec.utils import get_domain_class


class LogLevel(IntEnum):
    """Logging verbosity levels."""

    NORMAL = 0
    VERBOSE = 1
    DEBUG = 2


@dataclass
class LogContext:
    """Context information for logging operations."""

    study_id: str = ""
    domain_code: str = ""
    file_name: str = ""
    operation: str = ""
    start_time: datetime = field(default_factory=datetime.now)

    def elapsed_ms(self) -> float:
        """Calculate elapsed time in milliseconds."""
        return (datetime.now() - self.start_time).total_seconds() * 1000


class ConsoleLogger(LoggerPort):
    """Structured logger for SDTM processing operations.

    Provides context-aware logging with SDTM-specific information
    and multiple verbosity levels. Implements LoggerPort for
    dependency injection.
    """

    def __init__(self, console: Console | None = None, verbosity: int = 0):
        """Initialize the logger.

        Args:
            console: Rich console for output (creates new if None)
            verbosity: Verbosity level (0=normal, 1=verbose, 2=debug)
        """
        self.console = console or Console()
        self.verbosity = verbosity
        self._context: LogContext | None = None
        self._stats: dict[str, Any] = {
            "files_processed": 0,
            "domains_processed": 0,
            "records_processed": 0,
            "warnings": 0,
            "errors": 0,
        }

    def set_context(self, **kwargs: Any) -> None:
        """Set the current logging context.

        Args:
            **kwargs: Context fields (study_id, domain_code, file_name, operation)
        """
        if self._context is None:
            self._context = LogContext(**kwargs)
        else:
            for key, value in kwargs.items():
                if hasattr(self._context, key):
                    setattr(self._context, key, value)

    def clear_context(self) -> None:
        """Clear the current logging context."""
        self._context = None

    # =========================================================================
    # Basic logging methods (LoggerPort interface)
    # =========================================================================

    def info(self, message: str, *, level: int = LogLevel.NORMAL) -> None:
        """Log an informational message.

        Args:
            message: Message to log
            level: Minimum verbosity level required to show this message
        """
        if self.verbosity >= level:
            prefix = self._get_prefix()
            self.console.print(f"{prefix}{message}")

    def verbose(self, message: str) -> None:
        """Log a verbose message (shown with -v).

        Args:
            message: Message to log
        """
        if self.verbosity >= LogLevel.VERBOSE:
            prefix = self._get_prefix()
            self.console.print(f"[dim]{prefix}{message}[/dim]")

    def debug(self, message: str) -> None:
        """Log a debug message (shown with -vv).

        Args:
            message: Message to log
        """
        if self.verbosity >= LogLevel.DEBUG:
            prefix = self._get_prefix()
            self.console.print(f"[dim cyan]{prefix}{message}[/dim cyan]")

    def success(self, message: str) -> None:
        """Log a success message (always shown).

        Args:
            message: Message to log
        """
        self.console.print(f"[green]✓[/green] {message}")

    def warning(self, message: str) -> None:
        """Log a warning message (always shown).

        Args:
            message: Message to log
        """
        self._stats["warnings"] += 1
        self.console.print(f"[yellow]⚠[/yellow] {message}")

    def error(self, message: str) -> None:
        """Log an error message (always shown).

        Args:
            message: Message to log
        """
        self._stats["errors"] += 1
        self.console.print(f"[red]✗[/red] {message}")

    # =========================================================================
    # Structured logging methods for SDTM operations
    # =========================================================================

    def log_study_start(
        self,
        study_id: str,
        study_folder: Path,
        output_format: str,
        supported_domains: list[str],
    ) -> None:
        """Log the start of study processing.

        Args:
            study_id: Study identifier
            study_folder: Path to study folder
            output_format: Output format (xpt, xml, both)
            supported_domains: List of supported domain codes
        """
        self.set_context(study_id=study_id)

        self.verbose(f"Processing study folder: {study_folder}")
        self.verbose(f"Study ID: {study_id}")
        self.verbose(f"Output format: {output_format}")

        if self.verbosity >= LogLevel.VERBOSE:
            # Group domains by class for clearer output
            domain_groups: dict[str, list[str]] = {}
            for domain in supported_domains:
                cls = get_domain_class(domain)
                if cls not in domain_groups:
                    domain_groups[cls] = []
                domain_groups[cls].append(domain)

            self.verbose(f"Supported domains ({len(supported_domains)} total):")
            for cls in sorted(domain_groups.keys()):
                domains = ", ".join(sorted(domain_groups[cls]))
                self.verbose(f"  {cls}: {domains}")

    def log_metadata_loaded(
        self,
        *,
        items_count: int | None,
        codelists_count: int | None,
    ) -> None:
        """Log metadata loading results.

        Args:
            items_count: Number of items loaded from Items.csv
            codelists_count: Number of codelists from CodeLists.csv
        """
        if items_count:
            self.verbose(f"Loaded {items_count} column definitions from Items.csv")
        if codelists_count:
            self.verbose(f"Loaded {codelists_count} codelists from CodeLists.csv")

    def log_files_discovered(
        self,
        total_files: int,
        domain_files: dict[str, list[tuple[Path, str]]],
    ) -> None:
        """Log file discovery results.

        Args:
            total_files: Total CSV files found
            domain_files: Dictionary of domains to their files
        """
        self.verbose(f"Found {total_files} CSV files in study folder")

        if self.verbosity >= LogLevel.DEBUG:
            self.debug("Domain file mapping:")
            for domain, files in sorted(domain_files.items()):
                variants = [v for _, v in files]
                self.debug(f"  {domain}: {', '.join(variants)}")

    def log_file_match(
        self,
        filename: str,
        domain: str | None,
        variant: str | None,
        *,
        is_metadata: bool = False,
        match_type: str | None = None,
        category: str | None = None,
    ) -> None:
        """Log file matching result.

        Args:
            filename: Name of the file
            domain: Matched domain code or None
            variant: Variant name or None
            is_metadata: Whether this is a metadata file being skipped
            match_type: Type of match (exact, variant, etc.)
            category: Domain category (EVENTS, FINDINGS, etc.)
        """
        if is_metadata:
            self.verbose(f"Skipping metadata file: {filename}")
        elif domain:
            msg = f"Matched {filename} → {domain} (variant: {variant}"
            if match_type:
                msg += f", type: {match_type}"
            if category:
                msg += f", category: {category}"
            msg += ")"
            self.verbose(msg)
        else:
            self.verbose(f"No domain match for: {filename}")

    def log_domain_start(
        self,
        domain_code: str,
        files_for_domain: list[tuple[Path, str]] | None = None,
        *,
        files: list[tuple[Path, str]] | None = None,
    ) -> None:
        """Log the start of domain processing.

        Args:
            domain_code: SDTM domain code
            files: List of (file_path, variant_name) tuples
        """
        if files_for_domain is None:
            files_for_domain = files or []

        self.set_context(domain_code=domain_code)
        self._stats["domains_processed"] += 1

        domain_class = get_domain_class(domain_code)
        variant_names = [v for _, v in files_for_domain]

        # Build display name
        if len(files_for_domain) == 1:
            display_name = domain_code
        else:
            display_name = f"{domain_code} (merging {', '.join(variant_names)})"

        # Print processing header
        self.console.print()
        header = f"[bold]Processing {display_name}[/bold]"
        if self.verbosity >= LogLevel.VERBOSE:
            header += f" [dim]({domain_class})[/dim]"
        self.console.print(header)

        # List input files
        for input_file, _variant_name in files_for_domain:
            self.console.print(f"  - {input_file.name}")

    def log_file_loaded(
        self,
        filename: str,
        row_count: int,
        column_count: int | None = None,
    ) -> None:
        """Log file loading results.

        Args:
            filename: Name of the loaded file
            row_count: Number of rows loaded
            column_count: Number of columns (optional)
        """
        self._stats["files_processed"] += 1

        msg = f"  Loaded {row_count:,} rows from {filename}"
        if column_count is not None and self.verbosity >= LogLevel.DEBUG:
            msg += f" ({column_count} columns)"

        self.verbose(msg)

    def log_transformation(
        self,
        domain_code: str,
        transform_type: str,
        input_rows: int,
        output_rows: int,
        *,
        details: str | None = None,
    ) -> None:
        """Log a data transformation operation.

        Args:
            domain_code: Domain being transformed
            transform_type: Type of transformation (e.g., 'reshape', 'normalize')
            input_rows: Number of input rows
            output_rows: Number of output rows
            details: Additional transformation details
        """
        msg = f"  {transform_type.capitalize()} {domain_code}: {input_rows:,} → {output_rows:,} rows"
        if details:
            msg += f" ({details})"

        self.verbose(msg)

        if self.verbosity >= LogLevel.DEBUG:
            ratio = output_rows / input_rows if input_rows > 0 else 0
            self.debug(f"    Expansion ratio: {ratio:.2f}x")

    def log_rows_processed(
        self,
        domain_code: str,
        row_count: int,
        variant_name: str | None = None,
    ) -> None:
        """Log rows processed for a domain.

        Args:
            domain_code: Domain code
            row_count: Number of rows processed
            variant_name: Optional variant name
        """
        self._stats["records_processed"] += row_count

        label = variant_name or domain_code
        self.verbose(f"  Processed {row_count:,} rows for {label}")

    def log_merge_result(
        self,
        file_count: int,
        row_count: int,
    ) -> None:
        """Log merge operation result.

        Args:
            file_count: Number of files merged
            row_count: Total rows after merge
        """
        if file_count > 1:
            self.verbose(f"Merged {file_count} files into {row_count:,} rows")

    def log_mapping_info(
        self,
        domain_code: str,
        mappings_count: int,
        confidence_threshold: float,
        *,
        low_confidence_mappings: list[tuple[str, str, float]] | None = None,
    ) -> None:
        """Log column mapping information.

        Args:
            domain_code: Domain code
            mappings_count: Number of successful mappings
            confidence_threshold: Minimum confidence threshold
            low_confidence_mappings: List of (source, target, confidence) tuples
        """
        self.debug(f"  Column mappings for {domain_code}: {mappings_count} found")
        self.debug(f"  Confidence threshold: {confidence_threshold:.1%}")

        if low_confidence_mappings and self.verbosity >= LogLevel.DEBUG:
            for source, target, conf in low_confidence_mappings:
                self.debug(f"    Low confidence: {source} → {target} ({conf:.1%})")

    def log_suppqual_generated(
        self,
        domain_code: str,
        record_count: int,
        variable_count: int,
    ) -> None:
        """Log supplemental qualifier generation.

        Args:
            domain_code: Parent domain code
            record_count: Number of SUPPQUAL records
            variable_count: Number of supplemental variables
        """
        if record_count > 0:
            # Supplemental records should count towards the overall record total.
            # We intentionally do not increment `domains_processed` here because
            # the summary reports "domains" as top-level SDTM datasets.
            self._stats["records_processed"] += int(record_count or 0)
            supp_code = f"SUPP{domain_code.upper()}"
            msg = f"  Generated {supp_code}: {record_count:,} records ({variable_count} variables)"
            self.verbose(msg)

    def log_file_generated(
        self,
        file_type: str,
        path: Path,
        *,
        record_count: int | None = None,
    ) -> None:
        """Log file generation success.

        Args:
            file_type: Type of file (XPT, Dataset-XML, SAS, Define-XML)
            path: Path to generated file
            record_count: Number of records in file (optional)
        """
        msg = f"Generated {file_type}: {path}"
        if record_count is not None and self.verbosity >= LogLevel.VERBOSE:
            msg += f" ({record_count:,} records)"

        self.success(msg)

    def log_synthesis_start(
        self,
        domain_code: str,
        reason: str,
    ) -> None:
        """Log the start of domain synthesis.

        Args:
            domain_code: Domain being synthesized
            reason: Reason for synthesis
        """
        self.console.print()
        domain_class = get_domain_class(domain_code)
        header = f"[bold]Synthesizing {domain_code}[/bold]: {reason}"
        if self.verbosity >= LogLevel.VERBOSE:
            header += f" [dim]({domain_class})[/dim]"
        self.console.print(header)

    def log_synthesis_complete(
        self,
        domain_code: str,
        records: int,
    ) -> None:
        """Log synthesis completion.

        Args:
            domain_code: Domain that was synthesized
            record_count: Number of records generated
        """
        # Synthesis should count towards overall stats.
        self._stats["domains_processed"] += 1
        self._stats["records_processed"] += int(records or 0)
        self.success(f"Generated {domain_code} scaffold (records={records})")

    # =========================================================================
    # Summary and statistics methods
    # =========================================================================

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
        """Log study processing summary before starting.

        Args:
            study_id: Study identifier
            domain_count: Number of domains to process
            file_count: Number of files to process
            output_format: Output format
            generate_define: Whether Define-XML will be generated
            generate_sas: Whether SAS will be generated
        """
        self.console.print()
        self.console.print(f"[bold]Study: {study_id}[/bold]")
        self.console.print(
            f"[bold]Found {domain_count} domains ({file_count} files) to process[/bold]"
        )
        self.console.print(f"[bold]Output format:[/bold] {output_format.upper()}")

        if generate_define:
            self.console.print("[bold]Define-XML:[/bold] Will be generated")
        if generate_sas:
            self.console.print("[bold]SAS programs:[/bold] Will be generated")

    def log_final_stats(self) -> None:
        """Log final processing statistics (verbose mode)."""
        if self.verbosity >= LogLevel.VERBOSE:
            self.console.print()
            self.console.print("[dim]Processing Statistics:[/dim]")
            self.console.print(
                f"[dim]  Files processed: {self._stats['files_processed']}[/dim]"
            )
            self.console.print(
                f"[dim]  Domains processed: {self._stats['domains_processed']}[/dim]"
            )
            self.console.print(
                f"[dim]  Total records: {self._stats['records_processed']:,}[/dim]"
            )
            if self._stats["warnings"] > 0:
                self.console.print(
                    f"[dim yellow]  Warnings: {self._stats['warnings']}[/dim yellow]"
                )
            if self._stats["errors"] > 0:
                self.console.print(
                    f"[dim red]  Errors: {self._stats['errors']}[/dim red]"
                )

    def log_domain_complete(
        self,
        domain_code: str,
        final_row_count: int,
        final_column_count: int,
        *,
        skipped: bool = False,
        reason: str | None = None,
    ) -> None:
        """Log domain processing completion.

        Args:
            domain_code: SDTM domain code
            final_row_count: Final row count
            final_column_count: Final column count
            skipped: Whether domain was skipped
            reason: Reason for skipping (if skipped=True)
        """
        if skipped:
            self.warning(f"Skipped {domain_code}: {reason}")
        else:
            self._stats["records_processed"] += final_row_count
            self.verbose(
                f"Final {domain_code} dataset: "
                f"{final_row_count:,} rows × {final_column_count} columns"
            )

    def log_file_written(
        self,
        file_path: Path,
        file_type: str,
        *,
        size_bytes: int | None = None,
    ) -> None:
        """Log file write results.

        Args:
            file_path: Path to written file
            file_type: Type of file (XPT, XML, SAS)
            size_bytes: File size in bytes (optional)
        """
        msg = f"Wrote {file_type}: {file_path.name}"
        if size_bytes:
            kb = size_bytes / 1024
            if kb < 1024:
                msg += f" ({kb:.1f} KB)"
            else:
                mb = kb / 1024
                msg += f" ({mb:.1f} MB)"
        self.verbose(msg)

    def log_study_summary(self) -> None:
        """Log study processing summary."""
        self.console.print()
        self.console.print("[bold green]Study Processing Complete[/bold green]")
        self.console.print(f"  Domains processed: {self._stats['domains_processed']}")
        self.console.print(f"  Files processed: {self._stats['files_processed']}")
        self.console.print(f"  Records processed: {self._stats['records_processed']:,}")
        if self._stats["warnings"] > 0:
            self.console.print(
                f"  [yellow]Warnings: {self._stats['warnings']}[/yellow]"
            )
        if self._stats["errors"] > 0:
            self.console.print(f"  [red]Errors: {self._stats['errors']}[/red]")

    def get_stats(self) -> dict[str, Any]:
        """Get processing statistics.

        Returns:
            Dictionary of processing statistics
        """
        return self._stats.copy()

    def reset_stats(self) -> None:
        """Reset processing statistics."""
        self._stats = {
            "files_processed": 0,
            "domains_processed": 0,
            "records_processed": 0,
            "warnings": 0,
            "errors": 0,
        }

    # =========================================================================
    # Internal methods
    # =========================================================================

    def _get_prefix(self) -> str:
        """Get the logging prefix based on current context.

        Returns:
            Formatted prefix string or empty string
        """
        if self._context is None or self.verbosity < LogLevel.DEBUG:
            return ""

        parts = []
        if self._context.domain_code:
            parts.append(self._context.domain_code)

        return f"[{':'.join(parts)}] " if parts else ""
