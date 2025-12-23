"""Summary presenter for study processing results.

This module provides the SummaryPresenter class that formats and displays
study processing results in a Rich table format.
"""

from __future__ import annotations

from collections.abc import Iterable, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path

from rich.console import Console
from rich.table import Table

from ...application.models import DomainProcessingResult


@dataclass(frozen=True, slots=True)
class _DomainRow:
    description: str
    records: int
    has_xpt: bool
    has_xml: bool
    has_sas: bool
    notes: str
    is_supp: bool


@dataclass(frozen=True, slots=True)
class _ErrorRow:
    domain: str
    kind: str
    code: str
    variable: str
    count: str
    message: str
    examples: str
    codelist: str


class SummaryPresenter:
    """Presenter for formatting study processing summaries.

    This class is responsible for taking raw study processing results and
    formatting them into a user-friendly Rich table display. It separates
    presentation logic from business logic.

    Attributes:
        console: Rich console for output

    Example:
        >>> console = Console()
        >>> presenter = SummaryPresenter(console)
        >>> presenter.present(results, errors, output_dir, "xpt", True, True)
    """

    def __init__(self, console: Console):
        """Initialize the presenter with a console.

        Args:
            console: Rich console for output
        """
        super().__init__()
        self.console = console

    def present(
        self,
        results: Sequence[DomainProcessingResult],
        errors: list[tuple[str, str]],
        output_dir: Path,
        output_format: str,
        generate_define: bool,
        generate_sas: bool,
        *,
        domain_descriptions: Mapping[str, str] | None = None,
        conformance_report_path: Path | None = None,
        conformance_report_error: str | None = None,
    ) -> None:
        """Present study processing results in a formatted table.

        Args:
            results: List of processing results with domain information
            errors: List of (domain, error) tuples for failed domains
            output_dir: Output directory path
            output_format: Output format (xpt, xml, both)
            generate_define: Whether Define-XML was generated
            generate_sas: Whether SAS programs were generated
            domain_descriptions: Optional mapping of domain code to description

        Example:
            >>> presenter.present(
            ...     results=[{"domain_code": "DM", "records": 100}],
            ...     errors=[],
            ...     output_dir=Path("output"),
            ...     output_format="xpt",
            ...     generate_define=True,
            ...     generate_sas=True,
            ... )
        """
        self.console.print()

        # Build and display the summary table
        table = self._build_summary_table(
            results, domain_descriptions=domain_descriptions
        )
        self.console.print(table)
        self.console.print()

        # Display status and output information
        total_records = sum(
            self._result_records(r) for r in self._iter_all_results(results)
        )
        self._print_status_summary(len(results), len(errors))
        self._print_output_information(
            output_dir,
            output_format,
            generate_define,
            generate_sas,
            total_records,
            conformance_report_path=conformance_report_path,
            conformance_report_error=conformance_report_error,
            results=results,
        )

        # Print a readable list of all errors (processing + conformance)
        self._print_error_details(errors=errors, results=results)

    def _build_summary_table(
        self,
        results: Sequence[DomainProcessingResult],
        *,
        domain_descriptions: Mapping[str, str] | None,
    ) -> Table:
        """Build the Rich table with domain processing results.

        Args:
            results: List of processing results
            domain_descriptions: Optional mapping of domain code to description

        Returns:
            Formatted Rich Table
        """
        table = Table(
            title="📊 Study Processing Summary",
            show_header=True,
            header_style="bold cyan",
            border_style="bright_blue",
            title_style="bold magenta",
        )

        # Define columns
        # NOTE: Avoid fixed widths so Rich can adapt to narrow consoles (e.g.,
        # Click's CliRunner capture during tests) without eliding core headers
        # like "Records".
        table.add_column("Domain", style="cyan", no_wrap=True)
        table.add_column(
            "Description",
            style="white",
            no_wrap=False,
            overflow="fold",
            ratio=3,
        )
        table.add_column("Records", justify="right", style="yellow", no_wrap=True)
        table.add_column("XPT", justify="center", style="green", no_wrap=True)
        table.add_column("Dataset-XML", justify="center", style="green", no_wrap=True)
        table.add_column("SAS", justify="center", style="green", no_wrap=True)
        table.add_column(
            "Notes",
            style="dim",
            overflow="fold",
            ratio=2,
        )

        # Process and organize results
        main_domains, suppqual_domains, total_records = self._organize_results(
            results, domain_descriptions=domain_descriptions
        )

        # Add rows to table
        self._add_table_rows(table, main_domains, suppqual_domains)

        # Add total row
        table.add_section()
        table.add_row(
            "[bold]Total[/bold]",
            "",
            f"[bold yellow]{total_records:,}[/bold yellow]",
            "",
            "",
            "",
            "",
        )

        return table

    def _organize_results(
        self,
        results: Sequence[DomainProcessingResult],
        *,
        domain_descriptions: Mapping[str, str] | None,
    ) -> tuple[dict[str, _DomainRow], dict[str, list[tuple[str, _DomainRow]]], int]:
        """Organize results into main domains and SUPPQUAL domains.

        Args:
            results: List of processing results
            domain_descriptions: Optional mapping of domain code to description

        Returns:
            Tuple of (main_domains, suppqual_domains, total_records)
        """
        main_domains: dict[str, _DomainRow] = {}
        suppqual_domains: dict[str, list[tuple[str, _DomainRow]]] = {}
        total_records = 0

        for result in results:
            domain_code = result.domain_code.upper()
            records = self._result_records(result)
            description = self._describe_domain(
                domain_code, domain_descriptions=domain_descriptions
            )

            # Check if this is a SUPPQUAL domain
            is_supp = domain_code.startswith("SUPP")

            # Determine output indicators
            has_xpt = result.xpt_path is not None
            has_xml = result.xml_path is not None
            has_sas = result.sas_path is not None

            # Build notes
            notes = self._build_notes(result)

            domain_data = _DomainRow(
                records=records,
                description=description,
                has_xpt=has_xpt,
                has_xml=has_xml,
                has_sas=has_sas,
                notes=notes,
                is_supp=is_supp,
            )

            if is_supp:
                # Extract parent domain (e.g., SUPPDM -> DM)
                parent_domain = domain_code[4:]  # Remove "SUPP" prefix
                if parent_domain not in suppqual_domains:
                    suppqual_domains[parent_domain] = []
                suppqual_domains[parent_domain].append((domain_code, domain_data))
            else:
                main_domains[domain_code] = domain_data

                # Handle nested SUPPQUAL domains (newer result shape)
                for supp in result.suppqual_domains:
                    supp_code = supp.domain_code.upper()
                    if not supp_code.startswith("SUPP"):
                        continue

                    supp_records = self._result_records(supp)
                    supp_description = self._describe_domain(
                        supp_code, domain_descriptions=domain_descriptions
                    )

                    supp_domain_data = _DomainRow(
                        records=supp_records,
                        description=supp_description,
                        has_xpt=supp.xpt_path is not None,
                        has_xml=supp.xml_path is not None,
                        has_sas=supp.sas_path is not None,
                        notes=self._build_notes(supp),
                        is_supp=True,
                    )

                    # Prefer nesting under current main domain; fall back to suffix parent.
                    suffix_parent = supp_code[4:]
                    parent_domain = (
                        domain_code if suffix_parent == domain_code else suffix_parent
                    )

                    if parent_domain not in suppqual_domains:
                        suppqual_domains[parent_domain] = []
                    suppqual_domains[parent_domain].append(
                        (supp_code, supp_domain_data)
                    )

                    total_records += supp_records

            total_records += records

        return main_domains, suppqual_domains, total_records

    def _build_notes(self, result: DomainProcessingResult) -> str:
        """Build notes string for a domain result.

        Args:
            result: Domain processing result

        Returns:
            Formatted notes string
        """
        notes: list[str] = []

        if result.synthesized:
            reason = f": {result.synthesis_reason}" if result.synthesis_reason else ""
            notes.append(f"Synthesized{reason}")

        report = result.conformance_report
        if report is not None:
            warnings = report.warning_count()
            errors = report.error_count()
            if errors:
                notes.append(f"{errors} conformance error(s)")
            if warnings:
                notes.append(f"{warnings} warning(s)")

        return "; ".join(notes)

    @staticmethod
    def _describe_domain(
        domain_code: str, *, domain_descriptions: Mapping[str, str] | None
    ) -> str:
        if domain_descriptions is None:
            return ""
        return domain_descriptions.get(domain_code.upper(), "")

    @staticmethod
    def _result_records(result: DomainProcessingResult) -> int:
        if result.domain_dataframe is not None:
            return len(result.domain_dataframe)
        return result.records

    @staticmethod
    def _format_flag(value: bool) -> str:
        return "✓" if value else "–"

    def _add_table_rows(
        self,
        table: Table,
        main_domains: dict[str, _DomainRow],
        suppqual_domains: dict[str, list[tuple[str, _DomainRow]]],
    ) -> None:
        """Add domain rows to the table.

        Args:
            table: Rich Table to add rows to
            main_domains: Main domain data
            suppqual_domains: SUPPQUAL domain data organized by parent
        """
        for domain_code in sorted(main_domains.keys()):
            data = main_domains[domain_code]

            # Add main domain row
            table.add_row(
                f"[bold cyan]{domain_code}[/bold cyan]",
                data.description,
                f"[yellow]{data.records:,}[/yellow]",
                self._format_flag(data.has_xpt),
                self._format_flag(data.has_xml),
                self._format_flag(data.has_sas),
                data.notes,
            )

            # Add child rows (SUPPQUAL domains) with a proper tree connector.
            children = sorted(suppqual_domains.get(domain_code, []))
            for idx, (child_code, child_data) in enumerate(children):
                connector = "└─" if idx == len(children) - 1 else "├─"

                table.add_row(
                    f"[dim cyan] {connector} {child_code}[/dim cyan]",
                    f"[dim]{child_data.description}[/dim]",
                    f"[dim yellow]{child_data.records:,}[/dim yellow]",
                    f"[dim]{self._format_flag(child_data.has_xpt)}[/dim]",
                    f"[dim]{self._format_flag(child_data.has_xml)}[/dim]",
                    f"[dim]{self._format_flag(child_data.has_sas)}[/dim]",
                    f"[dim]{child_data.notes}[/dim]",
                )

    def _print_status_summary(self, success_count: int, error_count: int) -> None:
        """Print status summary line.

        Args:
            success_count: Number of successful domains
            error_count: Number of failed domains
        """
        if error_count == 0:
            status_line = f"[bold green]✓ {success_count} domains processed successfully[/bold green]"
        else:
            status_line = (
                f"[green]✓ {success_count} succeeded[/green]  "
                f"[red]✗ {error_count} failed[/red]"
            )

        self.console.print(status_line)

    def _print_output_information(
        self,
        output_dir: Path,
        output_format: str,
        generate_define: bool,
        generate_sas: bool,
        total_records: int,
        *,
        conformance_report_path: Path | None = None,
        conformance_report_error: str | None = None,
        results: Sequence[DomainProcessingResult] | None = None,
    ) -> None:
        """Print output directory and file information.

        Args:
            output_dir: Output directory path
            output_format: Output format (xpt, xml, both)
            generate_define: Whether Define-XML was generated
            generate_sas: Whether SAS programs were generated
            total_records: Total number of records processed
        """
        # Avoid Rich's path highlighter splitting strings (keeps tests simple).
        self.console.print(
            f"[bold]📁 Output:[/bold] [cyan]{output_dir}[/cyan]", highlight=False
        )
        self.console.print(
            f"[bold]📈 Total records:[/bold] [yellow]{total_records:,}[/yellow]"
        )

        # If `results` are present, only show artifacts that were actually written.
        # If `results` is omitted (e.g., unit tests calling this method directly),
        # fall back to showing the *requested* outputs.
        if results is None:
            generated_xpt = output_format in ("xpt", "both")
            generated_xml = output_format in ("xml", "both")
            generated_sas = generate_sas
            generated_define = generate_define
        else:
            all_results = self._iter_all_results(results)
            generated_xpt = any(r.xpt_path is not None for r in all_results)
            generated_xml = any(r.xml_path is not None for r in all_results)
            generated_sas = any(r.sas_path is not None for r in all_results)
            define_path = output_dir / "define.xml"
            generated_define = generate_define and define_path.exists()

        outputs: list[str] = []
        if output_format in ("xpt", "both") and generated_xpt:
            outputs.append(
                f"  [dim]├─[/dim] XPT files: [cyan]{output_dir / 'xpt'}[/cyan]"
            )
        if output_format in ("xml", "both") and generated_xml:
            outputs.append(
                f"  [dim]├─[/dim] Dataset-XML: [cyan]{output_dir / 'dataset-xml'}[/cyan]"
            )
        if generate_sas and generated_sas:
            outputs.append(
                f"  [dim]├─[/dim] SAS programs: [cyan]{output_dir / 'sas'}[/cyan]"
            )
        if generated_define:
            outputs.append(
                f"  [dim]└─[/dim] Define-XML: [cyan]{output_dir / 'define.xml'}[/cyan]"
            )

        if outputs:
            outputs[-1] = outputs[-1].replace("├─", "└─")
            self.console.print("[bold]📦 Generated:[/bold]")
            for output in outputs:
                self.console.print(output, highlight=False)

        if conformance_report_path is not None:
            self.console.print(
                f"[bold]🧾 Conformance report JSON:[/bold] [cyan]{conformance_report_path}[/cyan]",
                highlight=False,
            )
        elif conformance_report_error is not None:
            self.console.print(
                f"[bold]🧾 Conformance report JSON:[/bold] [red]{conformance_report_error}[/red]"
            )

    def _print_error_details(
        self,
        *,
        errors: list[tuple[str, str]],
        results: Sequence[DomainProcessingResult],
    ) -> None:
        rows: list[_ErrorRow] = []

        for domain, message in errors:
            rows.append(
                _ErrorRow(
                    domain=(domain or "(UNKNOWN)").upper(),
                    kind="Processing",
                    code="",
                    variable="",
                    count="",
                    message=message,
                    examples="",
                    codelist="",
                )
            )

        for result in self._iter_all_results(results):
            report = result.conformance_report
            if report is None:
                continue

            for issue in report.issues:
                if issue.severity != "error":
                    continue

                domain = (issue.domain or report.domain or "(UNKNOWN)").upper()

                raw_message = issue.message or ""
                message = raw_message
                examples = ""
                codelist = issue.codelist_code or ""

                # Many messages follow: "...; examples: a, b, c".
                # Split examples into a separate column for readability.
                lower = raw_message.lower()
                marker = "; examples:"
                idx = lower.find(marker)
                if idx == -1:
                    marker = " examples:"
                    idx = lower.find(marker)
                if idx != -1:
                    message = raw_message[:idx].rstrip(" ;")
                    examples = raw_message[idx + len(marker) :].strip()

                rows.append(
                    _ErrorRow(
                        domain=domain,
                        kind="Conformance",
                        code=issue.code or "",
                        variable=issue.variable or "",
                        count="" if issue.count is None else str(issue.count),
                        message=message,
                        examples=examples,
                        codelist=codelist,
                    )
                )

        if not rows:
            return

        self.console.print()
        self.console.print("[bold]Error details[/bold]")

        table = Table(
            show_header=True,
            header_style="bold red",
            border_style="red",
            show_lines=False,
        )
        table.add_column("Domain", style="cyan", no_wrap=True, width=10)
        table.add_column("Type", style="red", no_wrap=True, width=11)
        table.add_column("Code", style="dim", no_wrap=True, width=16)
        table.add_column("Var", style="yellow", no_wrap=True, width=12)
        table.add_column("Codelist", style="magenta", no_wrap=True, width=10)
        table.add_column("Count", justify="right", style="yellow", width=7)
        table.add_column("Message", style="white")
        table.add_column("Examples", style="dim")

        rows_sorted = sorted(
            rows,
            key=lambda r: (
                r.domain,
                0 if r.kind == "Processing" else 1,
                r.code,
                r.variable,
                r.codelist,
            ),
        )

        for r in rows_sorted:
            domain = r.domain or "(UNKNOWN)"
            table.add_row(
                domain,
                r.kind,
                r.code,
                r.variable,
                r.codelist,
                r.count,
                r.message,
                r.examples,
            )

        self.console.print(table)

    def _iter_all_results(
        self, results: Sequence[DomainProcessingResult]
    ) -> Iterable[DomainProcessingResult]:
        for result in results:
            yield result
            for supp in result.suppqual_domains:
                yield supp
