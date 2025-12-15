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
        total_records = sum(r.get("records", 0) for r in results)
        self._print_status_summary(len(results), len(errors))
        self._print_output_information(
            output_dir, output_format, generate_define, generate_sas, total_records
        )
    
    def _build_summary_table(
        self, results: list[dict[str, Any]]
    ) -> Table:
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
        table.add_column("Domain", style="cyan", no_wrap=True, width=15)
        table.add_column("Records", justify="right", style="yellow", width=9)
        table.add_column("XPT", justify="center", style="green", width=5)
        table.add_column("Dataset-XML", justify="center", style="green", width=13)
        table.add_column("SAS", justify="center", style="green", width=5)
        table.add_column("Notes", style="dim", width=25)
        
        # Process and organize results
        main_domains, supp_domains, total_records = self._organize_results(results)
        
        # Add rows to table
        self._add_table_rows(table, main_domains, supp_domains)
        
        # Add total row
        table.add_section()
        table.add_row(
            "[bold]Total[/bold]",
            f"[bold yellow]{total_records:,}[/bold yellow]",
            "",
            "",
            "",
            "",
        )
        
        return table
    
    def _organize_results(
        self, results: list[dict[str, Any]]
    ) -> tuple[dict[str, dict], dict[str, list], int]:
        """Organize results into main domains and supplemental domains.
        
        Args:
            results: List of processing results
            
        Returns:
            Tuple of (main_domains, supp_domains, total_records)
        """
        main_domains = {}
        supp_domains = {}
        total_records = 0
        
        for result in results:
            domain_code = result.get("domain_code", "").upper()
            records = result.get("records", 0)
            
            # Check if this is a supplemental domain
            is_supp = domain_code.startswith("SUPP")
            
            # Determine output indicators
            has_xpt = "✓" if result.get("xpt_path") else "–"
            has_xml = "✓" if result.get("xml_path") else "–"
            has_sas = "✓" if result.get("sas_path") else "–"
            
            # Build notes
            notes = self._build_notes(result)
            
            domain_data = {
                "records": records,
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
            
            total_records += records
        
        return main_domains, supp_domains, total_records
    
    def _build_notes(self, result: dict[str, Any]) -> str:
        """Build notes string for a domain result.
        
        Args:
            result: Domain processing result
            
        Returns:
            Formatted notes string
        """
        notes = []
        split_paths = result.get("split_xpt_paths", [])
        if split_paths:
            split_names = ", ".join(p.name for p in split_paths[:2])
            if len(split_paths) > 2:
                split_names += f", +{len(split_paths) - 2}"
            notes.append(f"splits: {split_names}")
        
        return " • ".join(notes) if notes else ""
    
    def _add_table_rows(
        self,
        table: Table,
        main_domains: dict[str, dict],
        supp_domains: dict[str, list],
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
                f"[yellow]{data['records']:,}[/yellow]",
                data["has_xpt"],
                data["has_xml"],
                data["has_sas"],
                data["notes"],
            )
            
            # Add supplemental domains for this parent
            if domain_code in supp_domains:
                for supp_code, supp_data in sorted(supp_domains[domain_code]):
                    table.add_row(
                        f"[dim cyan] └─ {supp_code}[/dim cyan]",
                        f"[dim yellow]{supp_data['records']:,}[/dim yellow]",
                        f"[dim]{supp_data['has_xpt']}[/dim]",
                        f"[dim]{supp_data['has_xml']}[/dim]",
                        f"[dim]{supp_data['has_sas']}[/dim]",
                        f"[dim]{supp_data['notes']}[/dim]",
                    )
    
    def _print_status_summary(
        self, success_count: int, error_count: int
    ) -> None:
        """Print status summary line.
        
        Args:
            success_count: Number of successful domains
            error_count: Number of failed domains
        """
        if error_count == 0:
            status_line = (
                f"[bold green]✓ {success_count} domains processed successfully[/bold green]"
            )
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
    ) -> None:
        """Print output directory and file information.
        
        Args:
            output_dir: Output directory path
            output_format: Output format (xpt, xml, both)
            generate_define: Whether Define-XML was generated
            generate_sas: Whether SAS programs were generated
            total_records: Total number of records processed
        """
        self.console.print(f"[bold]📁 Output:[/bold] [cyan]{output_dir}[/cyan]")
        self.console.print(
            f"[bold]📈 Total records:[/bold] [yellow]{total_records:,}[/yellow]"
        )
        
        # Build output list
        outputs = []
        if output_format in ("xpt", "both"):
            outputs.append(
                f"  [dim]├─[/dim] XPT files: [cyan]{output_dir / 'xpt'}[/cyan]"
            )
        if output_format in ("xml", "both"):
            outputs.append(
                f"  [dim]├─[/dim] Dataset-XML: [cyan]{output_dir / 'dataset-xml'}[/cyan]"
            )
        if generate_sas:
            outputs.append(
                f"  [dim]├─[/dim] SAS programs: [cyan]{output_dir / 'sas'}[/cyan]"
            )
        if generate_define:
            outputs.append(
                f"  [dim]└─[/dim] Define-XML: [cyan]{output_dir / 'define.xml'}[/cyan]"
            )
        
        # Fix last item to use └─
        if outputs:
            outputs[-1] = outputs[-1].replace("├─", "└─")
            self.console.print("[bold]📦 Generated:[/bold]")
            for output in outputs:
                self.console.print(output)
