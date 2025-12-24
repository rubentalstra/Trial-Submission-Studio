from dataclasses import dataclass
from typing import TYPE_CHECKING

from rich.table import Table

if TYPE_CHECKING:
    from collections.abc import Iterable, Mapping, Sequence
    from pathlib import Path

    from rich.console import Console

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


@dataclass(frozen=True, slots=True)
class SummaryRequest:
    results: Sequence[DomainProcessingResult]
    errors: list[tuple[str, str]]
    output_dir: Path
    output_format: str
    generate_define: bool
    generate_sas: bool
    domain_descriptions: Mapping[str, str] | None = None
    conformance_report_path: Path | None = None
    conformance_report_error: str | None = None


@dataclass(frozen=True, slots=True)
class _OutputInfo:
    output_dir: Path
    output_format: str
    generate_define: bool
    generate_sas: bool
    total_records: int
    conformance_report_path: Path | None
    conformance_report_error: str | None
    results: Sequence[DomainProcessingResult] | None


class SummaryPresenter:
    pass

    def __init__(self, console: Console) -> None:
        super().__init__()
        self.console = console

    def present(self, request: SummaryRequest) -> None:
        self.console.print()
        table = self._build_summary_table(
            request.results, domain_descriptions=request.domain_descriptions
        )
        self.console.print(table)
        self.console.print()
        total_records = sum(
            self._result_records(r) for r in self._iter_all_results(request.results)
        )
        self._print_status_summary(len(request.results), len(request.errors))
        self._print_output_information(
            _OutputInfo(
                output_dir=request.output_dir,
                output_format=request.output_format,
                generate_define=request.generate_define,
                generate_sas=request.generate_sas,
                total_records=total_records,
                conformance_report_path=request.conformance_report_path,
                conformance_report_error=request.conformance_report_error,
                results=request.results,
            )
        )
        self._print_error_details(errors=request.errors, results=request.results)

    def _build_summary_table(
        self,
        results: Sequence[DomainProcessingResult],
        *,
        domain_descriptions: Mapping[str, str] | None,
    ) -> Table:
        table = Table(
            title="📊 Study Processing Summary",
            show_header=True,
            header_style="bold cyan",
            border_style="bright_blue",
            title_style="bold magenta",
        )
        table.add_column("Domain", style="cyan", no_wrap=True)
        table.add_column(
            "Description", style="white", no_wrap=False, overflow="fold", ratio=3
        )
        table.add_column("Records", justify="right", style="yellow", no_wrap=True)
        table.add_column("XPT", justify="center", style="green", no_wrap=True)
        table.add_column("Dataset-XML", justify="center", style="green", no_wrap=True)
        table.add_column("SAS", justify="center", style="green", no_wrap=True)
        table.add_column("Notes", style="dim", overflow="fold", ratio=2)
        main_domains, suppqual_domains, total_records = self._organize_results(
            results, domain_descriptions=domain_descriptions
        )
        self._add_table_rows(table, main_domains, suppqual_domains)
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
        main_domains: dict[str, _DomainRow] = {}
        suppqual_domains: dict[str, list[tuple[str, _DomainRow]]] = {}
        total_records = 0
        for result in results:
            domain_code = result.domain_code.upper()
            records = self._result_records(result)
            description = self._describe_domain(
                domain_code, domain_descriptions=domain_descriptions
            )
            is_supp = domain_code.startswith("SUPP")
            has_xpt = result.xpt_path is not None
            has_xml = result.xml_path is not None
            has_sas = result.sas_path is not None
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
                parent_domain = domain_code[4:]
                if parent_domain not in suppqual_domains:
                    suppqual_domains[parent_domain] = []
                suppqual_domains[parent_domain].append((domain_code, domain_data))
            else:
                main_domains[domain_code] = domain_data
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
        return (main_domains, suppqual_domains, total_records)

    def _build_notes(self, result: DomainProcessingResult) -> str:
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
        return "✓" if value else "-"

    def _add_table_rows(
        self,
        table: Table,
        main_domains: dict[str, _DomainRow],
        suppqual_domains: dict[str, list[tuple[str, _DomainRow]]],
    ) -> None:
        for domain_code in sorted(main_domains.keys()):
            data = main_domains[domain_code]
            table.add_row(
                f"[bold cyan]{domain_code}[/bold cyan]",
                data.description,
                f"[yellow]{data.records:,}[/yellow]",
                self._format_flag(data.has_xpt),
                self._format_flag(data.has_xml),
                self._format_flag(data.has_sas),
                data.notes,
            )
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
        if error_count == 0:
            status_line = f"[bold green]✓ {success_count} domains processed successfully[/bold green]"
        else:
            status_line = f"[green]✓ {success_count} succeeded[/green]  [red]✗ {error_count} failed[/red]"
        self.console.print(status_line)

    def _print_output_information(self, info: _OutputInfo) -> None:
        self.console.print(
            f"[bold]📁 Output:[/bold] [cyan]{info.output_dir}[/cyan]", highlight=False
        )
        self.console.print(
            f"[bold]📈 Total records:[/bold] [yellow]{info.total_records:,}[/yellow]"
        )
        if info.results is None:
            generated_xpt = info.output_format in ("xpt", "both")
            generated_xml = info.output_format in ("xml", "both")
            generated_sas = info.generate_sas
            generated_define = info.generate_define
        else:
            all_results = self._iter_all_results(info.results)
            generated_xpt = any(r.xpt_path is not None for r in all_results)
            generated_xml = any(r.xml_path is not None for r in all_results)
            generated_sas = any(r.sas_path is not None for r in all_results)
            define_path = info.output_dir / "define.xml"
            generated_define = info.generate_define and define_path.exists()
        outputs: list[str] = []
        if info.output_format in ("xpt", "both") and generated_xpt:
            outputs.append(
                f"  [dim]├─[/dim] XPT files: [cyan]{info.output_dir / 'xpt'}[/cyan]"
            )
        if info.output_format in ("xml", "both") and generated_xml:
            outputs.append(
                f"  [dim]├─[/dim] Dataset-XML: [cyan]{info.output_dir / 'dataset-xml'}[/cyan]"
            )
        if info.generate_sas and generated_sas:
            outputs.append(
                f"  [dim]├─[/dim] SAS programs: [cyan]{info.output_dir / 'sas'}[/cyan]"
            )
        if generated_define:
            outputs.append(
                f"  [dim]└─[/dim] Define-XML: [cyan]{info.output_dir / 'define.xml'}[/cyan]"
            )
        if outputs:
            outputs[-1] = outputs[-1].replace("├─", "└─")
            self.console.print("[bold]📦 Generated:[/bold]")
            for output in outputs:
                self.console.print(output, highlight=False)
        if info.conformance_report_path is not None:
            self.console.print(
                f"[bold]🧾 Conformance report JSON:[/bold] [cyan]{info.conformance_report_path}[/cyan]",
                highlight=False,
            )
        elif info.conformance_report_error is not None:
            self.console.print(
                f"[bold]🧾 Conformance report JSON:[/bold] [red]{info.conformance_report_error}[/red]"
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
            yield from result.suppqual_domains
