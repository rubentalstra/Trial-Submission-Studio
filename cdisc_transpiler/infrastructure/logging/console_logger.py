from dataclasses import dataclass, field
from datetime import datetime
from enum import IntEnum
from typing import TYPE_CHECKING, override

from rich.console import Console

from ...application.ports.services import LoggerPort
from ..sdtm_spec.registry import get_domain_class

if TYPE_CHECKING:
    from pathlib import Path

    from ...application.models import ProcessingSummary


class LogLevel(IntEnum):
    NORMAL = 0
    VERBOSE = 1
    DEBUG = 2


@dataclass(slots=True)
class LogContext:
    study_id: str = ""
    domain_code: str = ""
    file_name: str = ""
    operation: str = ""
    start_time: datetime = field(default_factory=datetime.now)

    def elapsed_ms(self) -> float:
        return (datetime.now() - self.start_time).total_seconds() * 1000


class ConsoleLogger(LoggerPort):
    pass

    def __init__(self, console: Console | None = None, verbosity: int = 0) -> None:
        super().__init__()
        self.console = console or Console()
        self.verbosity = verbosity
        self._context: LogContext | None = None
        self._stats: dict[str, int] = {
            "files_processed": 0,
            "domains_processed": 0,
            "records_processed": 0,
            "warnings": 0,
            "errors": 0,
        }

    def set_context(self, **kwargs: str) -> None:
        if self._context is None:
            self._context = LogContext()
        for key, value in kwargs.items():
            if hasattr(self._context, key):
                setattr(self._context, key, value)

    def clear_context(self) -> None:
        self._context = None

    @override
    def info(self, message: str, *, level: int = LogLevel.NORMAL) -> None:
        if self.verbosity >= level:
            prefix = self._get_prefix()
            self.console.print(f"{prefix}{message}")

    @override
    def verbose(self, message: str) -> None:
        if self.verbosity >= LogLevel.VERBOSE:
            prefix = self._get_prefix()
            self.console.print(f"[dim]{prefix}{message}[/dim]")

    @override
    def debug(self, message: str) -> None:
        if self.verbosity >= LogLevel.DEBUG:
            prefix = self._get_prefix()
            self.console.print(f"[dim cyan]{prefix}{message}[/dim cyan]")

    @override
    def success(self, message: str) -> None:
        self.console.print(f"[green]✓[/green] {message}")

    @override
    def warning(self, message: str) -> None:
        self._stats["warnings"] += 1
        self.console.print(f"[yellow]⚠[/yellow] {message}")

    @override
    def error(self, message: str) -> None:
        self._stats["errors"] += 1
        self.console.print(f"[red]✗[/red] {message}")

    @override
    def log_study_start(
        self,
        study_id: str,
        study_folder: Path,
        output_format: str,
        supported_domains: list[str],
    ) -> None:
        self.set_context(study_id=study_id)
        self.verbose(f"Processing study folder: {study_folder}")
        self.verbose(f"Study ID: {study_id}")
        self.verbose(f"Output format: {output_format}")
        if self.verbosity >= LogLevel.VERBOSE:
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

    @override
    def log_metadata_loaded(
        self, *, items_count: int | None, codelists_count: int | None
    ) -> None:
        if items_count:
            self.verbose(f"Loaded {items_count} column definitions from Items.csv")
        if codelists_count:
            self.verbose(f"Loaded {codelists_count} codelists from CodeLists.csv")

    @override
    def log_domain_start(
        self,
        domain_code: str,
        files_for_domain: list[tuple[Path, str]] | None = None,
        *,
        files: list[tuple[Path, str]] | None = None,
    ) -> None:
        if files_for_domain is None:
            files_for_domain = files or []
        self.set_context(domain_code=domain_code)
        self._stats["domains_processed"] += 1
        domain_class = get_domain_class(domain_code)
        variant_names = [v for _, v in files_for_domain]
        if len(files_for_domain) == 1:
            display_name = domain_code
        else:
            display_name = f"{domain_code} (merging {', '.join(variant_names)})"
        self.console.print()
        header = f"[bold]Processing {display_name}[/bold]"
        if self.verbosity >= LogLevel.VERBOSE:
            header += f" [dim]({domain_class})[/dim]"
        self.console.print(header)
        for input_file, _variant_name in files_for_domain:
            self.console.print(f"  - {input_file.name}")

    @override
    def log_file_loaded(
        self, filename: str, row_count: int, column_count: int | None = None
    ) -> None:
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
        msg = f"  {transform_type.capitalize()} {domain_code}: {input_rows:,} → {output_rows:,} rows"
        if details:
            msg += f" ({details})"
        self.verbose(msg)
        if self.verbosity >= LogLevel.DEBUG:
            ratio = output_rows / input_rows if input_rows > 0 else 0
            self.debug(f"    Expansion ratio: {ratio:.2f}x")

    def log_rows_processed(
        self, domain_code: str, row_count: int, variant_name: str | None = None
    ) -> None:
        self._stats["records_processed"] += row_count
        label = variant_name or domain_code
        self.verbose(f"  Processed {row_count:,} rows for {label}")

    def log_merge_result(self, file_count: int, row_count: int) -> None:
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
        self.debug(f"  Column mappings for {domain_code}: {mappings_count} found")
        self.debug(f"  Confidence threshold: {confidence_threshold:.1%}")
        if low_confidence_mappings and self.verbosity >= LogLevel.DEBUG:
            for source, target, conf in low_confidence_mappings:
                self.debug(f"    Low confidence: {source} → {target} ({conf:.1%})")

    def log_suppqual_generated(
        self, domain_code: str, record_count: int, variable_count: int
    ) -> None:
        if record_count > 0:
            self._stats["records_processed"] += int(record_count or 0)
            supp_code = f"SUPP{domain_code.upper()}"
            msg = f"  Generated {supp_code}: {record_count:,} records ({variable_count} variables)"
            self.verbose(msg)

    @override
    def log_synthesis_start(self, domain_code: str, reason: str) -> None:
        self.console.print()
        domain_class = get_domain_class(domain_code)
        header = f"[bold]Synthesizing {domain_code}[/bold]: {reason}"
        if self.verbosity >= LogLevel.VERBOSE:
            header += f" [dim]({domain_class})[/dim]"
        self.console.print(header)

    @override
    def log_synthesis_complete(self, domain_code: str, records: int) -> None:
        self._stats["domains_processed"] += 1
        self._stats["records_processed"] += int(records or 0)
        self.success(f"Generated {domain_code} scaffold (records={records})")

    @override
    def log_processing_summary(self, summary: ProcessingSummary) -> None:
        self.console.print()
        self.console.print(f"[bold]Study: {summary.study_id}[/bold]")
        self.console.print(
            f"[bold]Found {summary.domain_count} domains ({summary.file_count} files) to process[/bold]"
        )
        self.console.print(
            f"[bold]Output format:[/bold] {summary.output_format.upper()}"
        )
        if summary.generate_define:
            self.console.print("[bold]Define-XML:[/bold] Will be generated")
        if summary.generate_sas:
            self.console.print("[bold]SAS programs:[/bold] Will be generated")

    @override
    def log_final_stats(self) -> None:
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
        if skipped:
            self.warning(f"Skipped {domain_code}: {reason}")
        else:
            self._stats["records_processed"] += final_row_count
            self.verbose(
                f"Final {domain_code} dataset: {final_row_count:,} rows x {final_column_count} columns"
            )

    def get_stats(self) -> dict[str, int]:
        return self._stats.copy()

    def reset_stats(self) -> None:
        self._stats = {
            "files_processed": 0,
            "domains_processed": 0,
            "records_processed": 0,
            "warnings": 0,
            "errors": 0,
        }

    def _get_prefix(self) -> str:
        if self._context is None or self.verbosity < LogLevel.DEBUG:
            return ""
        parts: list[str] = []
        if self._context.domain_code:
            parts.append(self._context.domain_code)
        return f"[{':'.join(parts)}] " if parts else ""
