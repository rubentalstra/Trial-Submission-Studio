"""Study command - Process an entire study folder and generate SDTM submission files."""

from __future__ import annotations

from collections import defaultdict
from pathlib import Path

import click
import pandas as pd
from rich.console import Console

from ...io_module import ParseError, load_input_dataset
from ...metadata_module import load_study_metadata
from ...domains_module import get_domain, list_domains
from ...xml_module.define_module.constants import ACRF_HREF
from ...services import (
    DomainDiscoveryService,
    DomainProcessingCoordinator,
    DomainSynthesisCoordinator,
    StudyOrchestrationService,
)
from ..utils import ProgressTracker, log_success, log_error
from ..helpers import (
    log_verbose,
    print_study_summary,
)
from ..logging_config import create_logger, SDTMLogger
from ...services.file_organization_service import ensure_acrf_pdf


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
    from ...xml_module.define_module import (
        StudyDataset,
        write_study_define_file,
    )
    from ...xml_module.define_module.constants import (
        CONTEXT_SUBMISSION,
        CONTEXT_OTHER,
    )

    # Initialize the structured logger with appropriate verbosity
    logger = create_logger(console, verbosity=verbose)

    # Get list of supported domains
    supported_domains = list(list_domains())

    # Derive study ID from folder name if not provided
    if study_id is None:
        folder_name = study_folder.name
        parts = folder_name.split("_")
        if len(parts) >= 2:
            study_id = "_".join(parts[:2])
        else:
            study_id = folder_name

    # Log study initialization with enhanced context
    logger.log_study_start(study_id, study_folder, output_format, supported_domains)

    # Load study metadata (Items.csv, CodeLists.csv)
    study_metadata = load_study_metadata(study_folder)
    logger.log_metadata_loaded(
        items_count=len(study_metadata.items) if study_metadata.items else None,
        codelists_count=len(study_metadata.codelists)
        if study_metadata.codelists
        else None,
    )

    # Set output directory
    if output_dir is None:
        output_dir = study_folder / "output"

    # Create subdirectories based on output format
    xpt_dir = output_dir / "xpt" if output_format in ("xpt", "both") else None
    xml_dir = output_dir / "dataset-xml" if output_format in ("xml", "both") else None
    sas_dir = output_dir / "sas" if generate_sas else None

    output_dir.mkdir(parents=True, exist_ok=True)
    if xpt_dir:
        xpt_dir.mkdir(parents=True, exist_ok=True)
    if xml_dir:
        xml_dir.mkdir(parents=True, exist_ok=True)
    if sas_dir:
        sas_dir.mkdir(parents=True, exist_ok=True)
    if generate_define:
        ensure_acrf_pdf(output_dir / ACRF_HREF)

    # Find all CSV files in the study folder
    csv_files = list(study_folder.glob("*.csv"))
    logger.verbose(f"Found {len(csv_files)} CSV files in study folder")

    # Map files to domains using the discovery service
    class VerboseLogger:
        """Simple logger adapter for verbose output."""

        def log_verbose(self, message: str) -> None:
            logger.verbose(message)

    discovery_service = DomainDiscoveryService(VerboseLogger() if verbose > 0 else None)
    domain_files = discovery_service.discover_domain_files(csv_files, supported_domains)

    if not domain_files:
        raise click.ClickException(
            f"No domain files found in {study_folder}. "
            f"Supported domains: {', '.join(supported_domains)}"
        )

    # Study-level heuristic: count how often each source column appears (using
    # normalized headers from load_input_dataset) to detect operational
    # scaffolding columns (e.g., site/event/form identifiers).
    common_column_counts: dict[str, int] = defaultdict(int)
    for files in domain_files.values():
        for file_path, _ in files:
            try:
                headers = load_input_dataset(file_path)
            except Exception:
                continue
            for col in headers.columns:
                common_column_counts[str(col).strip().lower()] += 1
    total_input_files = sum(len(files) for files in domain_files.values())

    # Count total files to process and log summary
    total_files = sum(len(files) for files in domain_files.values())
    logger.log_processing_summary(
        study_id=study_id,
        domain_count=len(domain_files),
        file_count=total_files,
        output_format=output_format,
        generate_define=generate_define,
        generate_sas=generate_sas,
    )

    # Phase 3: Initialize progress tracker for better UX
    progress_tracker = ProgressTracker(total_domains=len(domain_files))

    processed_domains = set(domain_files.keys())
    # Process each domain
    results: list[dict] = []
    errors: list[tuple[str, str]] = []
    study_datasets: list[StudyDataset] = []  # For Define-XML generation

    def _register_synthesized_domain(
        *,
        domain_code: str,
        reason: str,
        builder,
        is_reference_data: bool = False,
    ) -> None:
        logger.log_synthesis_start(domain_code, reason)
        try:
            result = builder()
            results.append(result)
            processed_domains.add(domain_code)
            dataset = result.get("domain_dataframe")
            has_no_data = bool(result.get("has_no_data"))
            if isinstance(dataset, pd.DataFrame) and dataset.empty:
                has_no_data = True
            if dataset is None:
                has_no_data = True

            if generate_define:
                dataset_href = None
                if output_format in ("xpt", "both") and result.get("xpt_path"):
                    dataset_href = result["xpt_path"].relative_to(output_dir)
                elif output_format in ("xml", "both") and result.get("xml_path"):
                    dataset_href = result["xml_path"].relative_to(output_dir)

                study_datasets.append(
                    StudyDataset(
                        domain_code=domain_code,
                        dataframe=dataset,
                        config=result.get("config"),
                        archive_location=dataset_href,
                    )
                )

            # Handle supplemental domains generated by the builder (e.g., SUPPAE)
            for supp in result.get("supplementals", []):
                results.append(supp)
                supp_code = supp.get("domain_code")
                if supp_code:
                    processed_domains.add(supp_code)
                supp_dataset = supp.get("domain_dataframe")
                supp_has_no_data = (
                    supp.get("has_no_data", False)
                    or supp_dataset is None
                    or (isinstance(supp_dataset, pd.DataFrame) and supp_dataset.empty)
                )
                supp_href = None
                if output_format in ("xpt", "both") and supp.get("xpt_path"):
                    supp_href = supp["xpt_path"].relative_to(output_dir)
                elif output_format in ("xml", "both") and supp.get("xml_path"):
                    supp_href = supp["xml_path"].relative_to(output_dir)
                if generate_define:
                    study_datasets.append(
                        StudyDataset(
                            domain_code=supp_code or "",
                            dataframe=supp_dataset,
                            config=supp.get("config"),
                            archive_location=supp_href,
                        )
                    )

            # Handle split datasets (SDTMIG v3.4 Section 4.1.7)
            # Split datasets must be documented separately in Define-XML
            for split_name, split_df, split_path in result.get("split_datasets", []):
                if generate_define and output_format in ("xpt", "both"):
                    split_href = split_path.relative_to(output_dir)
                    study_datasets.append(
                        StudyDataset(
                            domain_code=split_name,
                            dataframe=split_df,
                            config=result.get("config"),
                            archive_location=split_href,
                            is_split=True,
                            split_suffix=split_name[len(domain_code) :]
                            if split_name.startswith(domain_code)
                            else split_name,
                        )
                    )

            record_count = result.get("records")
            if record_count is None and isinstance(dataset, pd.DataFrame):
                record_count = len(dataset)
            logger.log_synthesis_complete(domain_code, record_count or 0)
        except Exception as exc:
            logger.error(f"{domain_code}: {exc}")
            errors.append((domain_code, str(exc)))

    ordered_domains = sorted(domain_files.keys(), key=lambda code: (code != "DM", code))
    reference_starts: dict[str, str] = {}

    for domain_code in ordered_domains:
        files_for_domain = domain_files[domain_code]

        # Log domain processing start with enhanced context
        logger.log_domain_start(domain_code, files_for_domain)

        try:
            # Use DomainProcessingCoordinator to process the domain
            coordinator = DomainProcessingCoordinator()
            result = coordinator.process_and_merge_domain(
                files_for_domain=files_for_domain,
                domain_code=domain_code,
                study_id=study_id,
                output_format=output_format,
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
                min_confidence=min_confidence,
                streaming=streaming,
                chunk_size=chunk_size,
                generate_sas=generate_sas,
                verbose=verbose > 0,
                metadata=study_metadata,
                reference_starts=reference_starts or None,
                common_column_counts=common_column_counts or None,
                total_input_files=total_input_files,
            )

            results.append(result)
            # Append any supplemental domain results
            for supp in result.get("supplementals", []):
                results.append(supp)
                if generate_define and supp.get("domain_dataframe") is not None:
                    supp_domain_code = supp.get("domain_code", "").upper()
                    supp_df = supp.get("domain_dataframe")
                    supp_config = supp.get("config")
                    if (
                        xpt_dir
                        and output_format in ("xpt", "both")
                        and supp.get("xpt_path")
                    ):
                        dataset_href = supp["xpt_path"].relative_to(output_dir)
                    elif (
                        xml_dir
                        and output_format in ("xml", "both")
                        and supp.get("xml_path")
                    ):
                        dataset_href = supp["xml_path"].relative_to(output_dir)
                    else:
                        base_name = get_domain(supp_domain_code).resolved_dataset_name()
                        dataset_href = Path(f"{base_name.lower()}.xpt")

                    study_datasets.append(
                        StudyDataset(
                            domain_code=supp_domain_code,
                            dataframe=supp_df,
                            config=supp_config,
                            archive_location=dataset_href,
                        )
                    )

            # Capture RFSTDTC for study day derivations after DM is processed
            if (
                domain_code.upper() == "DM"
                and result.get("domain_dataframe") is not None
            ):
                dm_frame = result["domain_dataframe"]
                # Ensure RFSTDTC exists and is populated for reference starts
                baseline_default = "2023-01-01"
                if "RFSTDTC" not in dm_frame.columns:
                    dm_frame["RFSTDTC"] = baseline_default
                else:
                    rfstdtc_series = (
                        dm_frame["RFSTDTC"]
                        .astype("string")
                        .replace({"nan": "", "<NA>": "", "None": ""})
                        .fillna("")
                        .str.strip()
                    )
                    dm_frame.loc[rfstdtc_series == "", "RFSTDTC"] = baseline_default
                if {"USUBJID", "RFSTDTC"}.issubset(dm_frame.columns):
                    cleaned = dm_frame[["USUBJID", "RFSTDTC"]].copy()
                    cleaned["RFSTDTC"] = pd.to_datetime(
                        cleaned["RFSTDTC"], errors="coerce"
                    ).fillna(pd.to_datetime(baseline_default))
                    baseline_map = (
                        cleaned.set_index("USUBJID")["RFSTDTC"]
                        .dt.date.astype(str)
                        .to_dict()
                    )
                    reference_starts.update(baseline_map)

            # Collect for Define-XML
            if generate_define and result.get("domain_dataframe") is not None:
                domain = get_domain(domain_code)
                disk_name = domain.resolved_dataset_name().lower()
                if output_format in ("xpt", "both"):
                    dataset_path = xpt_dir / f"{disk_name}.xpt"
                    dataset_href = dataset_path.relative_to(output_dir)
                else:
                    dataset_path = xml_dir / f"{disk_name}.xml"
                    dataset_href = dataset_path.relative_to(output_dir)

                study_datasets.append(
                    StudyDataset(
                        domain_code=domain_code,
                        dataframe=result["domain_dataframe"],
                        config=result["config"],
                        archive_location=dataset_href,
                    )
                )

                # Handle split datasets (SDTMIG v3.4 Section 4.1.7)
                # Split datasets must be documented separately in Define-XML
                for split_name, split_df, split_path in result.get(
                    "split_datasets", []
                ):
                    if generate_define and output_format in ("xpt", "both"):
                        split_href = split_path.relative_to(output_dir)
                        study_datasets.append(
                            StudyDataset(
                                domain_code=split_name,
                                dataframe=split_df,
                                config=result.get("config"),
                                archive_location=split_href,
                                is_split=True,
                                split_suffix=split_name[len(domain_code) :]
                                if split_name.startswith(domain_code)
                                else split_name,
                            )
                        )

            # Phase 3: Increment progress tracker after successful domain processing
            progress_tracker.increment()

        except ParseError as exc:
            logger.error(f"{domain_code}: Parse error - {exc}")
            errors.append((domain_code, str(exc)))
            progress_tracker.increment()  # Count failed domains too
        except Exception as exc:
            import traceback

            logger.error(f"{domain_code}: {exc}")
            if verbose > 1:  # Only print traceback in very verbose mode
                traceback.print_exc()
            errors.append((domain_code, str(exc)))
            progress_tracker.increment()  # Count failed domains too

    # Synthesize core observation domains when the study provides no source data
    for missing_domain in [
        d for d in ["AE", "LB", "VS", "EX"] if d not in processed_domains
    ]:
        synthesis_coordinator = DomainSynthesisCoordinator()
        _register_synthesized_domain(
            domain_code=missing_domain,
            reason="No source files found",
            builder=lambda md=missing_domain: synthesis_coordinator.synthesize_empty_observation_domain(
                domain_code=md,
                study_id=study_id,
                output_format=output_format,
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
                generate_sas=generate_sas,
                reference_starts=reference_starts,
            ),
        )

    # Generate required trial design domains if not present in input data
    # These are required by Pinnacle 21: TS, DS, SE, TA, TE
    trial_design_domains = ["TS", "TA", "TE", "SE", "DS"]

    synthesis_coordinator = DomainSynthesisCoordinator()
    for td_domain in trial_design_domains:
        if td_domain in processed_domains:
            continue
        _register_synthesized_domain(
            domain_code=td_domain,
            reason="Trial design scaffold",
            builder=lambda td_domain=td_domain: synthesis_coordinator.synthesize_trial_design_domain(
                domain_code=td_domain,
                study_id=study_id,
                output_format=output_format,
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
                generate_sas=generate_sas,
                reference_starts=reference_starts,
            ),
            is_reference_data=td_domain in ["TS", "TA", "TE", "TI", "TV"],
        )

    # Generate RELREC if missing (populate from existing domain data)
    if "RELREC" not in processed_domains:
        logger.log_synthesis_start("RELREC", "Relationship scaffold")
        try:
            orchestration_service = StudyOrchestrationService()
            result = orchestration_service.synthesize_relrec(
                study_id=study_id,
                output_format=output_format,
                xpt_dir=xpt_dir,
                xml_dir=xml_dir,
                sas_dir=sas_dir,
                generate_sas=generate_sas,
                domain_results=results,
            )
            results.append(result)
            if generate_define and result.get("domain_dataframe") is not None:
                domain = get_domain("RELREC")
                disk_name = domain.resolved_dataset_name().lower()
                if output_format in ("xpt", "both") and result.get("xpt_path"):
                    dataset_href = result["xpt_path"].relative_to(output_dir)
                elif output_format in ("xml", "both") and result.get("xml_path"):
                    dataset_href = result["xml_path"].relative_to(output_dir)
                else:
                    dataset_href = Path(f"{disk_name}.xpt")
                study_datasets.append(
                    StudyDataset(
                        domain_code="RELREC",
                        dataframe=result["domain_dataframe"],
                        config=result["config"],
                        archive_location=dataset_href,
                    )
                )
            logger.success("Generated RELREC")
        except Exception as exc:
            logger.error(f"RELREC: {exc}")
            errors.append(("RELREC", str(exc)))

    if generate_define and study_datasets:
        define_path = output_dir / "define.xml"
        try:
            context = (
                CONTEXT_SUBMISSION if define_context == "Submission" else CONTEXT_OTHER
            )
            write_study_define_file(
                study_datasets,
                define_path,
                sdtm_version=sdtm_version,
                context=context,
            )
            logger.success(f"Generated Define-XML 2.1 at {define_path}")
        except Exception as exc:
            import traceback

            logger.error(f"Define-XML generation failed: {exc}")
            if verbose > 1:
                traceback.print_exc()
            errors.append(("Define-XML", str(exc)))

    # Log final processing statistics in verbose mode
    logger.log_final_stats()

    # Print summary
    print_study_summary(
        results, errors, output_dir, output_format, generate_define, generate_sas
    )


# Helper functions
