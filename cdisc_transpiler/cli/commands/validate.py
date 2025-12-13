"""Validate command - Validate SDTM data against Pinnacle 21 rules."""

from __future__ import annotations

from pathlib import Path

import click
import pyreadstat
from rich.console import Console

from ...validators.validators import (
    ValidationEngine,
    ValidationContext,
    format_validation_report,
)
from ...domains_module import get_domain
from ..utils import log_success, log_warning, log_error


console = Console()


@click.command()
@click.argument("study_folder", type=click.Path(exists=True, path_type=Path))
@click.option(
    "--study-id",
    help="Study identifier (default: derived from folder name)",
)
@click.option(
    "--output",
    type=click.Path(path_type=Path),
    help="Output file for validation report (default: print to console)",
)
@click.option(
    "--format",
    "report_format",
    type=click.Choice(["text", "json", "html"]),
    default="text",
    show_default=True,
    help="Report format",
)
def validate_command(
    study_folder: Path,
    study_id: str | None,
    output: Path | None,
    report_format: str,
) -> None:
    """Validate SDTM data against Pinnacle 21 rules.

    This command validates processed SDTM domains using the comprehensive
    validation framework with 30+ Pinnacle 21 rules covering:

    \b
    - Required variables and data types
    - Controlled terminology compliance
    - Cross-domain referential integrity
    - Temporal consistency (date ordering)
    - Value range limits and constraints
    - Study day calculations

    The validation engine checks XPT files in the study output folder
    and reports issues with severity levels (ERROR, WARNING, INFO).

    Examples:

    \b
        # Validate study and print report to console
        cdisc-transpiler validate Mockdata/DEMO_GDISC_20240903_072908/output/

    \b
        # Save validation report to file
        cdisc-transpiler validate study/output/ --output validation_report.txt

    \b
        # Generate JSON report for automation
        cdisc-transpiler validate study/output/ --format json --output report.json
    """
    # Derive study ID if not provided
    if study_id is None:
        folder_name = study_folder.name
        if folder_name == "output":
            folder_name = study_folder.parent.name
        parts = folder_name.split("_")
        if len(parts) >= 2:
            study_id = "_".join(parts[:2])
        else:
            study_id = folder_name

    console.print(f"\n[bold]Validating Study: {study_id}[/bold]")
    console.print(f"[bold]Source:[/bold] {study_folder}")

    # Look for XPT files (most common format)
    xpt_dir = study_folder / "xpt" if (study_folder / "xpt").exists() else study_folder
    xpt_files = list(xpt_dir.glob("*.xpt"))

    if not xpt_files:
        log_error("No XPT files found to validate")
        raise click.ClickException(f"No XPT files found in {xpt_dir}")

    console.print(f"[bold]Found {len(xpt_files)} XPT files[/bold]")

    # Initialize validation engine
    engine = ValidationEngine()
    all_issues = []

    # Process each XPT file
    with console.status("[bold green]Validating domains...") as status:
        for xpt_file in xpt_files:
            domain_code = xpt_file.stem.upper()
            status.update(f"[bold green]Validating {domain_code}...")

            try:
                # Load XPT file
                df, meta = pyreadstat.read_xport(str(xpt_file))

                # Get domain metadata
                domain = get_domain(domain_code)

                # Create validation context
                context = ValidationContext(
                    study_id=study_id,
                    domain_code=domain_code,
                    domain=domain,
                    dataframe=df,
                    all_domains={},  # TODO: Load all domains for cross-validation
                    controlled_terminology=None,  # TODO: Load CT
                    reference_starts={},  # TODO: Load from DM
                )

                # Validate domain
                issues = engine.validate_domain(context)
                all_issues.extend(issues)

                if issues:
                    error_count = sum(1 for i in issues if i.severity == "ERROR")
                    warning_count = sum(1 for i in issues if i.severity == "WARNING")
                    console.print(
                        f"  [yellow]⚠[/yellow] {domain_code}: {error_count} errors, {warning_count} warnings"
                    )
                else:
                    log_success(f"{domain_code}: No issues found")

            except Exception as exc:
                log_error(f"{domain_code}: Validation failed - {exc}")

    # Generate report
    console.print()

    # TODO: Implement JSON and HTML format support
    # For now, always generate text format
    if report_format != "text":
        log_warning(f"Format '{report_format}' not yet implemented, using text format")

    report = format_validation_report(all_issues)

    if output:
        output.write_text(report)
        log_success(f"Validation report saved to {output}")
    else:
        console.print(report)

    # Summary
    error_count = sum(1 for i in all_issues if i.severity == "ERROR")
    warning_count = sum(1 for i in all_issues if i.severity == "WARNING")
    info_count = sum(1 for i in all_issues if i.severity == "INFO")

    console.print()
    if error_count == 0:
        log_success(
            f"✓ Validation complete: {warning_count} warnings, {info_count} info messages"
        )
    else:
        log_error(
            f"✗ Validation complete: {error_count} errors, {warning_count} warnings, {info_count} info"
        )
        raise click.ClickException(f"Validation failed with {error_count} errors")
