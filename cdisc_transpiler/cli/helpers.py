"""Helper functions for CLI operations.

This module contains utility functions extracted from the main CLI module
to improve code organization and reusability.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

import pandas as pd
from rich.console import Console

if TYPE_CHECKING:
    from ..domains import SDTMDomain

console = Console()


def unquote_safe(name: str | None) -> str:
    """Remove quotes from a column name safely.

    Args:
        name: Column name that may be quoted

    Returns:
        Unquoted column name
    """
    if not name:
        return ""
    name = str(name)
    if len(name) >= 3 and name.startswith('"') and name.endswith("n"):
        inner = name[1:-1]
        if inner.endswith('"'):
            inner = inner[:-1]
        return inner.replace('""', '"')
    return name


def log_verbose(enabled: bool, message: str) -> None:
    """Log a verbose message if verbose mode is enabled.

    Args:
        enabled: Whether verbose logging is enabled
        message: Message to log
    """
    if enabled:
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
) -> list[Path]:
    """Write split XPT files for domain variants (e.g., LB splits).

    Args:
        merged_dataframe: Merged domain dataframe
        variant_frames: List of (variant_name, dataframe) tuples
        domain: SDTM domain metadata
        xpt_dir: Directory for XPT files
        console: Rich console for output

    Returns:
        List of paths to generated split files
    """
    from ..xpt_module import write_xpt_file

    split_paths: list[Path] = []
    for variant_name, variant_df in variant_frames:
        # Clean variant name for filename
        table = variant_name.replace(" ", "_").replace("(", "").replace(")", "")
        if table == domain.code:
            continue
        split_name = table.lower()
        split_path = xpt_dir / f"{split_name}.xpt"
        file_label = f"{domain.description} - {variant_name}"
        write_xpt_file(variant_df, domain.code, split_path, file_label=file_label)
        split_paths.append(split_path)
        console.print(f"[green]✓[/green] Split XPT: {split_path} (table={table})")
    return split_paths


def print_study_summary(
    results: list[dict],
    errors: list[tuple[str, str]],
    output_dir: Path,
    output_format: str,
    generate_define: bool,
    generate_sas: bool,
) -> None:
    """Print summary of study processing results.

    Args:
        results: List of processing results
        errors: List of (domain, error) tuples
        output_dir: Output directory path
        output_format: Output format (xpt, xml, both)
        generate_define: Whether Define-XML was generated
        generate_sas: Whether SAS programs were generated
    """
    # Calculate total records
    total_records = sum(r.get("records", 0) for r in results)

    # Final summary panel
    console.print()

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
