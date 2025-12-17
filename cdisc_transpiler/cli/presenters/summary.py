"""Summary presenter for study processing results.

This module provides the SummaryPresenter class that formats and displays
study processing results in a Rich table format.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from rich.console import Console
from rich.table import Table


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
        self.console = console

    def present(
        self,
        results: list[dict[str, Any]],
        errors: list[tuple[str, str]],
        output_dir: Path,
        output_format: str,
        generate_define: bool,
        generate_sas: bool,
        *,
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
        table = self._build_summary_table(results)
        self.console.print(table)
        self.console.print()

        # Display status and output information
        total_records = sum(
            r.get("records", 0) for r in self._iter_all_results(results)
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

    def _build_summary_table(self, results: list[dict[str, Any]]) -> Table:
        """Build the Rich table with domain processing results.

        Args:
            results: List of processing results

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
        main_domains, supp_domains, split_domains, total_records = (
            self._organize_results(results)
        )

        # Add rows to table
        self._add_table_rows(table, main_domains, supp_domains, split_domains)

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
        self, results: list[dict[str, Any]]
    ) -> tuple[dict[str, dict], dict[str, list], dict[str, list], int]:
        """Organize results into main domains and supplemental domains.

        Args:
            results: List of processing results

        Returns:
            Tuple of (main_domains, supp_domains, split_domains, total_records)
        """
        main_domains = {}
        supp_domains = {}
        split_domains: dict[str, list[tuple[str, dict[str, Any]]]] = {}
        total_records = 0

        for result in results:
            domain_code = result.get("domain_code", "").upper()
            records = result.get("records", 0)
            description = str(result.get("description") or "")

            # Check if this is a supplemental domain
            is_supp = domain_code.startswith("SUPP")

            # Determine output indicators
            # When a domain is split (SDTMIG v3.4 4.1.7), the parent dataset may not
            # be emitted. In that case, treat split XPTs as valid XPT output.
            split_xpt_paths = result.get("split_xpt_paths") or []
            split_datasets = result.get("split_datasets") or []
            has_xpt = (
                "✓"
                if (result.get("xpt_path") or split_xpt_paths or split_datasets)
                else "–"
            )
            has_xml = "✓" if result.get("xml_path") else "–"
            has_sas = "✓" if result.get("sas_path") else "–"

            # Build notes
            notes = self._build_notes(result)

            domain_data = {
                "records": records,
                "description": description,
                "has_xpt": has_xpt,
                "has_xml": has_xml,
                "has_sas": has_sas,
                "notes": notes,
                "is_supp": is_supp,
            }

            if is_supp:
                # Extract parent domain (e.g., SUPPDM -> DM)
                parent_domain = domain_code[4:]  # Remove "SUPP" prefix
                if parent_domain not in supp_domains:
                    supp_domains[parent_domain] = []
                supp_domains[parent_domain].append((domain_code, domain_data))
            else:
                main_domains[domain_code] = domain_data

                # Handle split datasets (shown as nested rows, like SUPP, but purple)
                splits: list[dict[str, Any]] = []
                raw_splits = result.get("splits")
                if isinstance(raw_splits, list):
                    splits = [s for s in raw_splits if isinstance(s, dict)]
                else:
                    # Backwards-compatible fallback: derive split dataset names from paths.
                    split_paths = result.get("split_xpt_paths") or []
                    splits = [
                        {"domain_code": getattr(p, "stem", ""), "records": 0}
                        for p in split_paths
                    ]

                for split in splits:
                    split_code = str(split.get("domain_code") or "").upper()
                    if not split_code:
                        continue

                    split_records = int(split.get("records") or 0)
                    split_domain_data = {
                        "records": split_records,
                        "description": "",
                        "has_xpt": "✓",
                        "has_xml": "–",
                        "has_sas": "–",
                        "notes": "",
                        "is_supp": False,
                        "is_split": True,
                    }
                    split_domains.setdefault(domain_code, []).append(
                        (split_code, split_domain_data)
                    )

                # Handle nested supplemental domains (newer result shape)
                for supp in result.get("supplementals", []) or []:
                    if not isinstance(supp, dict):
                        continue

                    supp_code = str(supp.get("domain_code") or "").upper()
                    if not supp_code.startswith("SUPP"):
                        continue

                    supp_records = int(supp.get("records") or 0)
                    supp_description = str(supp.get("description") or "")

                    supp_domain_data = {
                        "records": supp_records,
                        "description": supp_description,
                        "has_xpt": "✓" if supp.get("xpt_path") else "–",
                        "has_xml": "✓" if supp.get("xml_path") else "–",
                        "has_sas": "✓" if supp.get("sas_path") else "–",
                        "notes": self._build_notes(supp),
                        "is_supp": True,
                    }

                    # Prefer nesting under current main domain; fall back to suffix parent.
                    suffix_parent = supp_code[4:]
                    parent_domain = (
                        domain_code if suffix_parent == domain_code else suffix_parent
                    )

                    if parent_domain not in supp_domains:
                        supp_domains[parent_domain] = []
                    supp_domains[parent_domain].append((supp_code, supp_domain_data))

                    total_records += supp_records

            total_records += records

        return main_domains, supp_domains, split_domains, total_records

    def _build_notes(self, result: dict[str, Any]) -> str:
        """Build notes string for a domain result.

        Args:
            result: Domain processing result

        Returns:
            Formatted notes string
        """
        return ""

    def _add_table_rows(
        self,
        table: Table,
        main_domains: dict[str, dict],
        supp_domains: dict[str, list],
        split_domains: dict[str, list],
    ) -> None:
        """Add domain rows to the table.

        Args:
            table: Rich Table to add rows to
            main_domains: Main domain data
            supp_domains: Supplemental domain data organized by parent
        """
        for domain_code in sorted(main_domains.keys()):
            data = main_domains[domain_code]

            # Add main domain row
            table.add_row(
                f"[bold cyan]{domain_code}[/bold cyan]",
                str(data.get("description") or ""),
                f"[yellow]{data['records']:,}[/yellow]",
                data["has_xpt"],
                data["has_xml"],
                data["has_sas"],
                data["notes"],
            )

            # Add child rows (splits + supplementals) with a proper tree connector.
            children: list[tuple[str, str, dict[str, Any]]] = []
            for split_code, split_data in sorted(split_domains.get(domain_code, [])):
                children.append(("split", split_code, split_data))
            for supp_code, supp_data in sorted(supp_domains.get(domain_code, [])):
                children.append(("supp", supp_code, supp_data))

            for idx, (kind, child_code, child_data) in enumerate(children):
                connector = "└─" if idx == len(children) - 1 else "├─"

                if kind == "split":
                    table.add_row(
                        f"[magenta] {connector} {child_code}[/magenta]",
                        "[magenta]Split dataset[/magenta]",
                        f"[magenta]{child_data['records']:,}[/magenta]",
                        f"[magenta]{child_data['has_xpt']}[/magenta]",
                        f"[magenta]{child_data['has_xml']}[/magenta]",
                        f"[magenta]{child_data['has_sas']}[/magenta]",
                        "",
                    )
                else:
                    table.add_row(
                        f"[dim cyan] {connector} {child_code}[/dim cyan]",
                        f"[dim]{child_data.get('description') or ''}[/dim]",
                        f"[dim yellow]{child_data['records']:,}[/dim yellow]",
                        f"[dim]{child_data['has_xpt']}[/dim]",
                        f"[dim]{child_data['has_xml']}[/dim]",
                        f"[dim]{child_data['has_sas']}[/dim]",
                        f"[dim]{child_data['notes']}[/dim]",
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
        results: list[dict[str, Any]] | None = None,
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
            generated_xpt = any(
                (r.get("xpt_path") is not None)
                or bool(r.get("split_xpt_paths"))
                or bool(r.get("split_datasets"))
                for r in (results or [])
            )
            generated_xml = any(
                (r.get("xml_path") is not None) for r in (results or [])
            )
            generated_sas = any(
                (r.get("sas_path") is not None) for r in (results or [])
            )
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
        self, *, errors: list[tuple[str, str]], results: list[dict[str, Any]]
    ) -> None:
        rows: list[dict[str, Any]] = []

        for domain, message in errors:
            rows.append(
                {
                    "domain": (domain or "(UNKNOWN)").upper(),
                    "kind": "Processing",
                    "code": "",
                    "variable": "",
                    "count": "",
                    "message": message,
                    "examples": "",
                    "codelist": "",
                }
            )

        for result in self._iter_all_results(results):
            report = result.get("conformance_report")
            if not isinstance(report, dict):
                continue

            issues = report.get("issues")
            if not isinstance(issues, list):
                continue

            for issue in issues:
                if not isinstance(issue, dict):
                    continue
                if issue.get("severity") != "error":
                    continue

                domain = str(
                    issue.get("domain") or report.get("domain") or "(UNKNOWN)"
                ).upper()

                raw_message = str(issue.get("message") or "")
                message = raw_message
                examples = ""
                codelist = str(issue.get("codelist_code") or "")

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
                    {
                        "domain": domain,
                        "kind": "Conformance",
                        "code": str(issue.get("code") or ""),
                        "variable": str(issue.get("variable") or ""),
                        "count": ""
                        if issue.get("count") is None
                        else str(issue.get("count")),
                        "message": message,
                        "examples": examples,
                        "codelist": codelist,
                    }
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
                r.get("domain") or "",
                0 if r.get("kind") == "Processing" else 1,
                r.get("code") or "",
                r.get("variable") or "",
                r.get("codelist") or "",
            ),
        )

        for r in rows_sorted:
            domain = str(r.get("domain") or "(UNKNOWN)")
            table.add_row(
                domain,
                str(r.get("kind") or ""),
                str(r.get("code") or ""),
                str(r.get("variable") or ""),
                str(r.get("codelist") or ""),
                str(r.get("count") or ""),
                str(r.get("message") or ""),
                str(r.get("examples") or ""),
            )

        self.console.print(table)

    def _iter_all_results(self, results: list[dict[str, Any]]):
        for result in results:
            yield result
            for supp in result.get("supplementals", []) or []:
                if isinstance(supp, dict):
                    yield supp
