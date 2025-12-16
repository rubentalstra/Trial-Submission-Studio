"""Study command - Process an entire study folder and generate SDTM submission files.

This module serves as a thin adapter between the Click CLI framework and the
application layer's StudyProcessingUseCase. It is responsible for:
1. Parsing CLI arguments
2. Creating the ProcessStudyRequest
3. Calling the use case
4. Formatting the response for user output
"""

from __future__ import annotations

from pathlib import Path

import click
from rich.console import Console

from ...application.models import ProcessStudyRequest
from ...infrastructure.container import DependencyContainer
from ..presenters import SummaryPresenter


console = Console()


@click.command()
@click.argument("study_folder", type=click.Path(exists=True, path_type=Path))
@click.option(
    "--output-dir",
    "output_dir",
    type=click.Path(path_type=Path),
    help="Output directory for generated files (default: <study_folder>/output)",
)
@click.option(
    "--study-id",
    help="Study identifier (default: derived from folder name)",
)
@click.option(
    "--format",
    "output_format",
    type=click.Choice(["xpt", "xml", "both"]),
    default="both",
    show_default=True,
    help="Output format: xpt (SAS transport), xml (Dataset-XML), or both",
)
@click.option(
    "--define-xml/--no-define-xml",
    "generate_define",
    default=True,
    show_default=True,
    help="Generate Define-XML 2.1 metadata file",
)
@click.option(
    "--sas/--no-sas",
    "generate_sas",
    default=True,
    show_default=True,
    help="Generate SAS programs for each domain",
)
@click.option(
    "--sdtm-version",
    default="3.4",
    show_default=True,
    help="SDTM-IG version for Define-XML",
)
@click.option(
    "--define-context",
    type=click.Choice(["Submission", "Other"]),
    default="Submission",
    show_default=True,
    help="Define-XML context (Submission for FDA, Other for internal)",
)
@click.option(
    "--streaming",
    is_flag=True,
    help="Use streaming mode for large datasets (Dataset-XML only)",
)
@click.option(
    "--chunk-size",
    type=int,
    default=1000,
    show_default=True,
    help="Chunk size for streaming mode",
)
@click.option(
    "--min-confidence",
    type=click.FloatRange(0.0, 1.0),
    default=0.5,
    show_default=True,
    help="Minimum confidence required for fuzzy matches",
)
@click.option(
    "--conformance-json/--no-conformance-json",
    "write_conformance_report_json",
    default=True,
    show_default=True,
    help="Write a machine-readable conformance report JSON to the output directory",
)
@click.option(
    "--fail-on-conformance-errors/--no-fail-on-conformance-errors",
    "fail_on_conformance_errors",
    default=False,
    show_default=True,
    help="Fail the run (strict outputs only) when conformance errors are detected",
)
@click.option(
    "-v", "--verbose", count=True, help="Increase verbosity level (e.g., -v, -vv)"
)
def study_command(
    study_folder: Path,
    output_dir: Path | None,
    study_id: str | None,
    output_format: str,
    generate_define: bool,
    generate_sas: bool,
    sdtm_version: str,
    define_context: str,
    streaming: bool,
    chunk_size: int,
    min_confidence: float,
    write_conformance_report_json: bool,
    fail_on_conformance_errors: bool,
    verbose: int,
) -> None:
    """Process an entire study folder and generate SDTM submission files.

    This command scans the study folder for CSV files matching known SDTM domain
    patterns and generates the requested output files:

    \b
    - XPT files: SAS transport files for regulatory submission
    - Dataset-XML: CDISC Dataset-XML 1.0 files
    - Define-XML: Single Define-XML 2.1 metadata file for the entire study
    - SAS programs: SAS code for data transformation

    Domain variants like LBCC, LBHM, LB_PREG are recognized and merged
    into their base domain files for SDTM compliance.

    Examples:

    \b
        # Generate both XPT and Dataset-XML with Define-XML
        cdisc-transpiler study Mockdata/DEMO_GDISC_20240903_072908/

    \b
        # Generate only XPT files
        cdisc-transpiler study Mockdata/DEMO_GDISC_20240903_072908/ --format xpt

    \b
        # Generate only Dataset-XML files
        cdisc-transpiler study Mockdata/DEMO_GDISC_20240903_072908/ --format xml

    \b
        # Skip Define-XML generation
        cdisc-transpiler study Mockdata/DEMO_GDISC_20240903_072908/ --no-define-xml

    \b
        # Custom output directory and study ID
        cdisc-transpiler study data/ --output-dir submission/ --study-id STUDY123
    """

    # Derive study ID from folder name if not provided
    if study_id is None:
        folder_name = study_folder.name
        parts = folder_name.split("_")
        if len(parts) >= 2:
            study_id = "_".join(parts[:2])
        else:
            study_id = folder_name

    # Set output directory
    if output_dir is None:
        output_dir = study_folder / "output"

    # Convert output format to set
    output_formats = {"xpt", "xml"} if output_format == "both" else {output_format}

    # Create request object
    request = ProcessStudyRequest(
        study_folder=study_folder,
        study_id=study_id,
        output_dir=output_dir,
        output_formats=output_formats,
        generate_define_xml=generate_define,
        generate_sas=generate_sas,
        sdtm_version=sdtm_version,
        define_context=define_context,
        streaming=streaming,
        chunk_size=chunk_size,
        min_confidence=min_confidence,
        verbose=verbose,
        write_conformance_report_json=write_conformance_report_json,
        fail_on_conformance_errors=fail_on_conformance_errors,
    )

    # Create dependency container and use case
    container = DependencyContainer(verbose=verbose, console=console)
    use_case = container.create_study_processing_use_case()

    # Execute the use case
    response = use_case.execute(request)

    # Convert response to the format expected by SummaryPresenter
    def _report_to_dict(report: object | None) -> dict[str, object] | None:
        if report is None:
            return None
        to_dict = getattr(report, "to_dict", None)
        if callable(to_dict):
            payload = to_dict()
            if isinstance(payload, dict):
                return payload
        return None

    results = []
    for result in response.domain_results:
        conformance_report = _report_to_dict(
            getattr(result, "conformance_report", None)
        )
        result_dict = {
            "domain_code": result.domain_code,
            "records": result.records,
            "domain_dataframe": result.domain_dataframe,
            "config": result.config,
            "xpt_path": result.xpt_path,
            "xml_path": result.xml_path,
            "sas_path": result.sas_path,
            "conformance_report": conformance_report,
            "supplementals": [
                {
                    "domain_code": supp.domain_code,
                    "records": supp.records,
                    "domain_dataframe": supp.domain_dataframe,
                    "config": supp.config,
                    "xpt_path": supp.xpt_path,
                    "xml_path": supp.xml_path,
                    "sas_path": supp.sas_path,
                    "conformance_report": _report_to_dict(
                        getattr(supp, "conformance_report", None)
                    ),
                }
                for supp in result.supplementals
            ],
        }
        results.append(result_dict)

    # Display summary using presenter
    presenter = SummaryPresenter(console)
    presenter.present(
        results,
        response.errors,
        output_dir,
        output_format,
        generate_define,
        generate_sas,
        conformance_report_path=response.conformance_report_path,
        conformance_report_error=response.conformance_report_error,
    )

    if fail_on_conformance_errors and not response.success:
        raise click.ClickException(
            "Run failed due to errors (conformance gating may have been enabled)."
        )

    # Exit with error code if there were errors
    if not response.success or response.has_errors:
        raise click.ClickException("Study processing completed with errors")
