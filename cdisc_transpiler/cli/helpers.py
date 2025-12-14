"""Helper functions for CLI operations.

This module contains utility functions extracted from the main CLI module
to improve code organization and reusability.

SDTM Reference:
    These utilities support SDTM-compliant output generation as defined
    in SDTMIG v3.4. The module handles verbose logging, PDF generation
    for Define-XML, and split dataset management per Section 4.1.7.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

import pandas as pd
from rich.console import Console
from rich.table import Table

from .logging_config import get_logger, SDTMLogger

if TYPE_CHECKING:
    from ..domains_module import SDTMDomain

console = Console()


def log_verbose(enabled: bool, message: str) -> None:
    """Log a verbose message if verbose mode is enabled.

    Args:
        enabled: Whether verbose logging is enabled
        message: Message to log
        
    Note:
        This function maintains backward compatibility.
        New code should use SDTMLogger directly via get_logger().
    """
    if enabled:
        # Use new logger if available, otherwise fall back to console
        logger = get_logger()
        if logger.verbosity >= 1:
            logger.verbose(message)
        else:
            console.print(f"[dim]{message}[/dim]")


def ensure_acrf_pdf(path: Path) -> None:
    """Create a minimal, valid PDF at path if one is not already present.

    This creates a placeholder Annotated CRF PDF file required by Define-XML.

    Args:
        path: Path where PDF should be created
    """
    if path.exists():
        return

    path.parent.mkdir(parents=True, exist_ok=True)

    obj_bodies: dict[int, str] = {
        1: "<< /Type /Catalog /Pages 2 0 R >>",
        2: "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        3: (
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
            "/Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>"
        ),
    }
    stream_text = "Annotated CRF placeholder"
    stream_content = f"BT /F1 12 Tf 72 720 Td ({stream_text}) Tj ET".encode("latin-1")
    obj_bodies[4] = (
        f"<< /Length {len(stream_content)} >>\nstream\n"
        + stream_content.decode("latin-1")
        + "\nendstream"
    )
    obj_bodies[5] = "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"

    parts: list[str] = ["%PDF-1.4\n"]
    offsets: dict[int, int] = {}
    for obj_num in sorted(obj_bodies):
        offsets[obj_num] = sum(len(p.encode("latin-1")) for p in parts)
        parts.append(f"{obj_num} 0 obj\n{obj_bodies[obj_num]}\nendobj\n")

    xref_start = sum(len(p.encode("latin-1")) for p in parts)
    size = max(obj_bodies) + 1
    xref_lines = ["xref", f"0 {size}", "0000000000 65535 f "]
    for i in range(1, size):
        offset = offsets.get(i, 0)
        xref_lines.append(f"{offset:010d} 00000 n ")
    xref_section = "\n".join(xref_lines) + "\n"
    trailer = (
        f"trailer\n<< /Size {size} /Root 1 0 R >>\nstartxref\n{xref_start}\n%%EOF\n"
    )
    parts.append(xref_section)
    parts.append(trailer)

    pdf_bytes = "".join(parts).encode("latin-1")
    path.write_bytes(pdf_bytes)


def write_variant_splits(
    merged_dataframe: pd.DataFrame,
    variant_frames: list[tuple[str, pd.DataFrame]],
    domain: SDTMDomain,
    xpt_dir: Path,
    console: Console,
) -> tuple[list[Path], list[tuple[str, pd.DataFrame, Path]]]:
    """Write split XPT files for domain variants following SDTMIG v3.4 Section 4.1.7.

    According to SDTMIG v3.4 Section 4.1.7 "Splitting Domains":
    - Split datasets follow naming pattern: [DOMAIN][SPLIT] (e.g., LB → LBHM, LBCC)
    - All splits maintain the same DOMAIN variable value
    - Each split is documented as a separate dataset in Define-XML
    - Dataset names must be ≤ 8 characters
    - Split suffix should be meaningful (typically 2-4 characters)

    Args:
        merged_dataframe: Merged domain dataframe
        variant_frames: List of (variant_name, dataframe) tuples
        domain: SDTM domain metadata
        xpt_dir: Directory for XPT files
        console: Rich console for output

    Returns:
        Tuple of (list of paths, list of (split_name, dataframe, path) tuples)
    """
    from ..xpt_module import write_xpt_file

    split_paths: list[Path] = []
    split_datasets: list[tuple[str, pd.DataFrame, Path]] = []
    domain_code = domain.code.upper()
    
    for variant_name, variant_df in variant_frames:
        # Clean variant name for filename
        table = variant_name.replace(" ", "_").replace("(", "").replace(")", "").upper()
        
        # Skip if this is the base domain (not a split)
        if table == domain_code:
            continue
        
        # Validate split dataset name follows SDTMIG v3.4 naming convention
        # Split name must start with domain code and be ≤ 8 characters
        if not table.startswith(domain_code):
            console.print(
                f"[yellow]⚠[/yellow] Warning: Split dataset '{table}' does not start "
                f"with domain code '{domain_code}'. Skipping."
            )
            continue
        
        if len(table) > 8:
            console.print(
                f"[yellow]⚠[/yellow] Warning: Split dataset name '{table}' exceeds "
                "8 characters. Truncating to comply with SDTMIG v3.4."
            )
            table = table[:8]
        
        # Ensure DOMAIN variable is set correctly (must match parent domain)
        if "DOMAIN" in variant_df.columns:
            variant_df = variant_df.copy()
            variant_df["DOMAIN"] = domain_code
        
        # Create split subdirectory for better organization
        split_dir = xpt_dir / "split"
        split_dir.mkdir(parents=True, exist_ok=True)
        
        split_name = table.lower()
        split_path = split_dir / f"{split_name}.xpt"
        
        # Extract split suffix for better labeling
        split_suffix = table[len(domain_code):]
        file_label = (
            f"{domain.description} - {split_suffix}" if split_suffix 
            else domain.description
        )
        
        write_xpt_file(variant_df, domain.code, split_path, file_label=file_label, table_name=table)
        split_paths.append(split_path)
        split_datasets.append((table, variant_df, split_path))
        console.print(
            f"[green]✓[/green] Split dataset: {split_path} "
            f"(DOMAIN={domain_code}, table={table})"
        )
    
    return split_paths, split_datasets


def print_study_summary(
    results: list[dict],
    errors: list[tuple[str, str]],
    output_dir: Path,
    output_format: str,
    generate_define: bool,
    generate_sas: bool,
) -> None:
    """Print summary of study processing results with detailed table.

    Args:
        results: List of processing results
        errors: List of (domain, error) tuples
        output_dir: Output directory path
        output_format: Output format (xpt, xml, both)
        generate_define: Whether Define-XML was generated
        generate_sas: Whether SAS programs were generated
    """
    console.print()

    # Create summary table
    table = Table(
        title="📊 Study Processing Summary",
        show_header=True,
        header_style="bold cyan",
        border_style="bright_blue",
        title_style="bold magenta",
    )

    table.add_column("Domain", style="cyan", no_wrap=True, width=15)
    table.add_column("Records", justify="right", style="yellow", width=9)
    table.add_column("XPT", justify="center", style="green", width=5)
    table.add_column("Dataset-XML", justify="center", style="green", width=13)
    table.add_column("SAS", justify="center", style="green", width=5)
    table.add_column("Notes", style="dim", width=25)

    # Track domains and their data
    main_domains = {}
    supp_domains = {}
    total_records = 0

    # Process all results
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
        notes = []
        split_paths = result.get("split_xpt_paths", [])
        if split_paths:
            split_names = ", ".join(p.name for p in split_paths[:2])
            if len(split_paths) > 2:
                split_names += f", +{len(split_paths) - 2}"
            notes.append(f"splits: {split_names}")

        domain_data = {
            "records": records,
            "has_xpt": has_xpt,
            "has_xml": has_xml,
            "has_sas": has_sas,
            "notes": " • ".join(notes) if notes else "",
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

    # Add rows to table in sorted order
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

    # Add separator and total row
    table.add_section()
    table.add_row(
        "[bold]Total[/bold]",
        f"[bold yellow]{total_records:,}[/bold yellow]",
        "",
        "",
        "",
        "",
    )

    # Print the table
    console.print(table)
    console.print()

    # Status summary
    success_count = len(results)
    error_count = len(errors)

    if error_count == 0:
        status_line = (
            f"[bold green]✓ {success_count} domains processed successfully[/bold green]"
        )
    else:
        status_line = f"[green]✓ {success_count} succeeded[/green]  [red]✗ {error_count} failed[/red]"

    # Build output list
    outputs = []
    if output_format in ("xpt", "both"):
        outputs.append(f"  [dim]├─[/dim] XPT files: [cyan]{output_dir / 'xpt'}[/cyan]")
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

    console.print(status_line)
    console.print(f"[bold]📁 Output:[/bold] [cyan]{output_dir}[/cyan]")
    console.print(f"[bold]📈 Total records:[/bold] [yellow]{total_records:,}[/yellow]")
    if outputs:
        console.print("[bold]📦 Generated:[/bold]")
        for output in outputs:
            console.print(output)
