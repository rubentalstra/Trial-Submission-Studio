"""Study command - Process an entire study folder and generate SDTM submission files.

This module serves as a thin adapter between the Click CLI framework and the
application layer's StudyProcessingUseCase. It is responsible for:
1. Parsing CLI arguments
2. Creating the ProcessStudyRequest
3. Calling the use case
4. Formatting the response for user output
"""

from dataclasses import dataclass
from pathlib import Path
from typing import cast

import click
from rich.console import Console

from ...application.models import ProcessStudyRequest
from ...config import ConfigLoader
from ...infrastructure.container import DependencyContainer
from ..presenters.summary import SummaryPresenter, SummaryRequest

console = Console()

MIN_STUDY_ID_PARTS = 2
SUPP_PREFIX_LEN = 4


@dataclass(frozen=True)
class StudyCommandOptions:
    output_dir: Path | None
    config_file: Path | None
    study_id: str | None
    output_format: str
    generate_define: bool
    generate_sas: bool
    sdtm_version: str
    define_context: str
    streaming: bool
    chunk_size: int
    min_confidence: float
    write_conformance_report_json: bool
    fail_on_conformance_errors: bool
    verbose: int

    @classmethod
    def from_kwargs(cls, options: dict[str, object]) -> StudyCommandOptions:
        return cls(
            output_dir=cast("Path | None", options.get("output_dir")),
            config_file=cast("Path | None", options.get("config_file")),
            study_id=cast("str | None", options.get("study_id")),
            output_format=cast("str", options["output_format"]),
            generate_define=cast("bool", options["generate_define"]),
            generate_sas=cast("bool", options["generate_sas"]),
            sdtm_version=cast("str", options["sdtm_version"]),
            define_context=cast("str", options["define_context"]),
            streaming=cast("bool", options["streaming"]),
            chunk_size=cast("int", options["chunk_size"]),
            min_confidence=cast("float", options["min_confidence"]),
            write_conformance_report_json=cast(
                "bool", options["write_conformance_report_json"]
            ),
            fail_on_conformance_errors=cast(
                "bool", options["fail_on_conformance_errors"]
            ),
            verbose=cast("int", options["verbose"]),
        )


@click.command()
@click.argument("study_folder", type=click.Path(exists=True, path_type=Path))
@click.option(
    "--config",
    "config_file",
    type=click.Path(exists=True, path_type=Path),
    help="Path to a cdisc_transpiler.toml config file (default: ./cdisc_transpiler.toml)",
)
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
    **options: object,
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

    command_options = StudyCommandOptions.from_kwargs(dict(options))

    # Derive study ID from folder name if not provided
    if command_options.study_id is None:
        folder_name = study_folder.name
        parts = folder_name.split("_")
        study_id = (
            "_".join(parts[:MIN_STUDY_ID_PARTS])
            if len(parts) >= MIN_STUDY_ID_PARTS
            else folder_name
        )
    else:
        study_id = command_options.study_id

    # Set output directory
    output_dir = command_options.output_dir or (study_folder / "output")

    # Convert output format to set
    output_formats = (
        {"xpt", "xml"}
        if command_options.output_format == "both"
        else {command_options.output_format}
    )

    runtime_config = ConfigLoader.load(config_file=command_options.config_file)

    # Create request object
    request = ProcessStudyRequest(
        study_folder=study_folder,
        study_id=study_id,
        output_dir=output_dir,
        output_formats=output_formats,
        generate_define_xml=command_options.generate_define,
        generate_sas=command_options.generate_sas,
        sdtm_version=command_options.sdtm_version,
        define_context=command_options.define_context,
        streaming=command_options.streaming,
        chunk_size=command_options.chunk_size,
        min_confidence=command_options.min_confidence,
        verbose=command_options.verbose,
        write_conformance_report_json=command_options.write_conformance_report_json,
        fail_on_conformance_errors=command_options.fail_on_conformance_errors,
        default_country=runtime_config.default_country,
    )

    # Create dependency container and use case
    container = DependencyContainer(verbose=command_options.verbose, console=console)
    use_case = container.create_study_processing_use_case()

    # Domain descriptions (used in the Study Processing Summary table)
    domain_definition_repository = container.create_domain_definition_repository()

    def _describe_domain(domain_code: str) -> str:
        code = (domain_code or "").upper()
        if not code:
            return ""

        # SDTMIG metadata uses a placeholder label for SUPPQUAL
        # ("Supplemental Qualifiers for [domain name]"). For concrete SUPP--
        # datasets we always prefer a resolved label.
        if code == "SUPPQUAL":
            return "Supplemental Qualifiers"
        if code.startswith("SUPP") and len(code) > SUPP_PREFIX_LEN:
            return f"Supplemental Qualifiers for {code[SUPP_PREFIX_LEN:]}"
        try:
            domain = domain_definition_repository.get_domain(code)
            return str(getattr(domain, "description", "") or "")
        except Exception:
            return ""

    # Execute the use case
    response = use_case.execute(request)

    domain_descriptions: dict[str, str] = {}
    for result in response.domain_results:
        domain_descriptions[result.domain_code] = _describe_domain(result.domain_code)
        for supp in result.suppqual_domains:
            domain_descriptions[supp.domain_code] = _describe_domain(supp.domain_code)

    # Display summary using presenter
    presenter = SummaryPresenter(console)
    presenter.present(
        SummaryRequest(
            results=response.domain_results,
            errors=response.errors,
            output_dir=output_dir,
            output_format=command_options.output_format,
            generate_define=command_options.generate_define,
            generate_sas=command_options.generate_sas,
            domain_descriptions=domain_descriptions,
            conformance_report_path=response.conformance_report_path,
            conformance_report_error=response.conformance_report_error,
        )
    )

    if command_options.fail_on_conformance_errors and not response.success:
        raise click.ClickException(
            "Run failed due to errors (conformance gating may have been enabled)."
        )

    # Exit with error code if there were errors
    if not response.success or response.has_errors:
        raise click.ClickException("Study processing completed with errors")
