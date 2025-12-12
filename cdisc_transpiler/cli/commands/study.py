"""Study command - Process an entire study folder and generate SDTM submission files."""

from __future__ import annotations

from collections import defaultdict
import re
from pathlib import Path

import click
import pandas as pd
from rich.console import Console

from ...sas import generate_sas_program, write_sas_file
from ...xpt_module import write_xpt_file
from ...io import build_column_hints, ParseError, load_input_dataset
from ...mapping import ColumnMapping, build_config, create_mapper
from ...submission import build_suppqual
from ...metadata import load_study_metadata, StudyMetadata
from ...domains import get_domain, list_domains
from ...domains import SDTMVariable
from ...xml.define.constants import ACRF_HREF
from ..utils import ProgressTracker, log_success, log_warning, log_error
from ..helpers import (
    unquote_safe,
    log_verbose,
    ensure_acrf_pdf,
    write_variant_splits,
    print_study_summary,
)


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
    from ...xml.define import (
        StudyDataset,
        write_study_define_file,
    )
    from ...xml.define.constants import (
        CONTEXT_SUBMISSION,
        CONTEXT_OTHER,
    )

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

    log_verbose(verbose > 0, f"Processing study folder: {study_folder}")
    log_verbose(verbose > 0, f"Study ID: {study_id}")
    log_verbose(verbose > 0, f"Output format: {output_format}")
    log_verbose(verbose > 0, f"Supported domains: {', '.join(supported_domains)}")

    # Load study metadata (Items.csv, CodeLists.csv)
    study_metadata = load_study_metadata(study_folder)
    if study_metadata.items:
        log_verbose(
            verbose > 0,
            f"Loaded {len(study_metadata.items)} column definitions from Items.csv",
        )
    if study_metadata.codelists:
        log_verbose(
            verbose > 0,
            f"Loaded {len(study_metadata.codelists)} codelists from CodeLists.csv",
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
    log_verbose(verbose > 0, f"Found {len(csv_files)} CSV files")

    # Map files to domains
    domain_files = _discover_domain_files(csv_files, supported_domains, verbose > 0)

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

    # Count total files to process
    total_files = sum(len(files) for files in domain_files.values())
    console.print(f"\n[bold]Study: {study_id}[/bold]")
    console.print(
        f"[bold]Found {len(domain_files)} domains ({total_files} files) to process[/bold]"
    )
    console.print(f"[bold]Output format:[/bold] {output_format.upper()}")
    if generate_define:
        console.print("[bold]Define-XML:[/bold] Will be generated")
    if generate_sas:
        console.print("[bold]SAS programs:[/bold] Will be generated")

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
        console.print(f"\n[bold]Synthesizing {domain_code}[/bold]: {reason}")
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
                    dataset_href = str(result["xpt_path"].relative_to(output_dir))
                elif output_format in ("xml", "both") and result.get("xml_path"):
                    dataset_href = str(result["xml_path"].relative_to(output_dir))

                study_datasets.append(
                    StudyDataset(
                        domain_code=domain_code,
                        dataset=dataset,
                        config=result.get("config"),
                        dataset_href=dataset_href,
                        is_reference_data=is_reference_data,
                        has_no_data=has_no_data,
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
                    supp_href = str(supp["xpt_path"].relative_to(output_dir))
                elif output_format in ("xml", "both") and supp.get("xml_path"):
                    supp_href = str(supp["xml_path"].relative_to(output_dir))
                if generate_define:
                    study_datasets.append(
                        StudyDataset(
                            domain_code=supp_code or "",
                            dataset=supp_dataset,
                            config=supp.get("config"),
                            dataset_href=supp_href,
                            has_no_data=supp_has_no_data,
                        )
                    )

            record_count = result.get("records")
            if record_count is None and isinstance(dataset, pd.DataFrame):
                record_count = len(dataset)
            log_success(
                f"Generated {domain_code} scaffold (records={record_count or 0})"
            )
        except Exception as exc:
            log_error(f"{domain_code}: {exc}")
            errors.append((domain_code, str(exc)))

    ordered_domains = sorted(domain_files.keys(), key=lambda code: (code != "DM", code))
    reference_starts: dict[str, str] = {}

    for domain_code in ordered_domains:
        files_for_domain = domain_files[domain_code]

        # Merge all variants into one domain file (SDTM compliant)
        variant_names = [v for _, v in files_for_domain]
        if len(files_for_domain) == 1:
            display_name = domain_code
        else:
            display_name = f"{domain_code} (merging {', '.join(variant_names)})"

        console.print(f"\n[bold]Processing {display_name}[/bold]")
        for input_file, variant_name in files_for_domain:
            console.print(f"  - {input_file.name}")

        try:
            result = _process_and_merge_domain(
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
                        dataset_href = str(supp["xpt_path"].relative_to(output_dir))
                    elif (
                        xml_dir
                        and output_format in ("xml", "both")
                        and supp.get("xml_path")
                    ):
                        dataset_href = str(supp["xml_path"].relative_to(output_dir))
                    else:
                        base_name = get_domain(supp_domain_code).resolved_dataset_name()
                        dataset_href = f"{base_name.lower()}.xpt"

                    study_datasets.append(
                        StudyDataset(
                            domain_code=supp_domain_code,
                            dataset=supp_df,
                            config=supp_config,
                            dataset_href=dataset_href,
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
                    dataset_href = str(dataset_path.relative_to(output_dir))
                else:
                    dataset_path = xml_dir / f"{disk_name}.xml"
                    dataset_href = str(dataset_path.relative_to(output_dir))

                study_datasets.append(
                    StudyDataset(
                        domain_code=domain_code,
                        dataset=result["domain_dataframe"],
                        config=result["config"],
                        dataset_href=dataset_href,
                    )
                )

            # Phase 3: Increment progress tracker after successful domain processing
            progress_tracker.increment()

        except ParseError as exc:
            log_error(f"{display_name}: Parse error - {exc}")
            errors.append((display_name, str(exc)))
            progress_tracker.increment()  # Count failed domains too
        except Exception as exc:
            log_error(f"{display_name}: {exc}")
            errors.append((display_name, str(exc)))
            progress_tracker.increment()  # Count failed domains too

    # Synthesize core observation domains when the study provides no source data
    for missing_domain in [
        d for d in ["AE", "LB", "VS", "EX"] if d not in processed_domains
    ]:
        _register_synthesized_domain(
            domain_code=missing_domain,
            reason="No source files found",
            builder=lambda md=missing_domain: _synthesize_empty_observation_domain(
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

    for td_domain in trial_design_domains:
        if td_domain in processed_domains:
            continue
        _register_synthesized_domain(
            domain_code=td_domain,
            reason="Trial design scaffold",
            builder=lambda td_domain=td_domain: _synthesize_trial_design_domain(
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
        console.print(f"\n[bold]Synthesizing RELREC[/bold]: (relationship scaffold)")
        try:
            result = _synthesize_relrec(
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
                    dataset_href = str(result["xpt_path"].relative_to(output_dir))
                elif output_format in ("xml", "both") and result.get("xml_path"):
                    dataset_href = str(result["xml_path"].relative_to(output_dir))
                else:
                    dataset_href = f"{disk_name}.xpt"
                study_datasets.append(
                    StudyDataset(
                        domain_code="RELREC",
                        dataset=result["domain_dataframe"],
                        config=result["config"],
                        dataset_href=dataset_href,
                    )
                )
            log_success("Generated RELREC")
        except Exception as exc:
            log_error(f"RELREC: {exc}")
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
            log_success(f"Generated Define-XML 2.1 at {define_path}")
        except Exception as exc:
            log_error(f"Define-XML generation failed: {exc}")
            errors.append(("Define-XML", str(exc)))

    # Print summary
    print_study_summary(
        results, errors, output_dir, output_format, generate_define, generate_sas
    )


# Helper functions


def _discover_domain_files(
    csv_files: list[Path],
    supported_domains: list[str],
    verbose: bool,
) -> dict[str, list[tuple[Path, str]]]:
    """Discover and map CSV files to SDTM domains."""
    domain_files: dict[str, list[tuple[Path, str]]] = {}

    for csv_file in csv_files:
        filename = csv_file.stem.upper()
        # Skip metadata files
        if any(
            skip in filename
            for skip in ["CODELISTS", "CODELIST", "ITEMS", "README", "METADATA"]
        ):
            log_verbose(verbose, f"Skipping metadata file: {csv_file.name}")
            continue

        parts = filename.split("_")
        matched_domain = None
        variant_name = None

        for i, part in enumerate(parts):
            if part in supported_domains:
                matched_domain = part
                if i < len(parts) - 1:
                    variant_name = "_".join(parts[i:])
                else:
                    variant_name = part
                break

            for domain_code in supported_domains:
                if part.startswith(domain_code) and len(part) > len(domain_code):
                    matched_domain = domain_code
                    variant_name = part
                    break

            if matched_domain:
                break

        if matched_domain:
            if matched_domain not in domain_files:
                domain_files[matched_domain] = []
            domain_files[matched_domain].append(
                (csv_file, variant_name or matched_domain)
            )
            log_verbose(
                verbose,
                f"Matched {csv_file.name} -> {matched_domain} (variant: {variant_name})",
            )
        else:
            log_verbose(verbose, f"No domain match for: {csv_file.name}")

    return domain_files


def _process_and_merge_domain(
    files_for_domain: list[tuple[Path, str]],
    domain_code: str,
    study_id: str,
    output_format: str,
    xpt_dir: Path | None,
    xml_dir: Path | None,
    sas_dir: Path | None,
    min_confidence: float,
    streaming: bool,
    chunk_size: int,
    generate_sas: bool,
    verbose: bool,
    metadata: StudyMetadata | None = None,
    reference_starts: dict[str, str] | None = None,
    common_column_counts: dict[str, int] | None = None,
    total_input_files: int | None = None,
) -> dict:
    """Process multiple domain variant files and merge them into one output file."""
    from .dataset_xml import generate_dataset_xml_streaming, write_dataset_xml
    from .xpt import build_domain_dataframe

    domain = get_domain(domain_code)
    all_dataframes = []
    variant_frames: list[tuple[str, pd.DataFrame]] = []
    last_config = None
    supp_frames: list[pd.DataFrame] = []
    lb_long = False

    # Process each input file and collect dataframes
    for input_file, variant_name in files_for_domain:
        display_name = (
            f"{domain_code}"
            if variant_name == domain_code
            else f"{domain_code} ({variant_name})"
        )

        # Load input data
        frame = load_input_dataset(input_file)
        log_verbose(verbose, f"  Loaded {len(frame)} rows from {input_file.name}")

        # Special handling for VS visit status (VSTAT) helper files:
        # convert to VS rows with VSTAT test.
        is_vstat = (
            domain_code.upper() == "VS"
            and variant_name
            and "VSTAT" in variant_name.upper()
        )
        if is_vstat:
            log_verbose(
                verbose,
                f"  Skipping {input_file.name} (VSTAT helper not mapped to SDTM)",
            )
            continue

        vs_long = False
        if domain_code.upper() == "VS":
            frame = _reshape_vs_to_long(frame, study_id)
            vs_long = True
            log_verbose(
                verbose,
                f"  Normalized VS wide data to {len(frame)} long-form rows",
            )
            if frame.empty:
                console.print(
                    f"[yellow]⚠[/yellow] {display_name}: No vital signs records after reshaping"
                )
                continue
        lb_long_current = False
        if domain_code.upper() == "LB":
            reshaped = _reshape_lb_to_long(frame, study_id)
            if "LBTESTCD" in reshaped.columns:
                frame = reshaped
                lb_long_current = True
                lb_long = True
                log_verbose(
                    verbose,
                    f"  Normalized LB wide data to {len(frame)} long-form rows",
                )
                if frame.empty:
                    console.print(
                        f"[yellow]⚠[/yellow] {display_name}: No laboratory records after reshaping"
                    )
                    continue
            else:
                log_verbose(verbose, "  Skipping LB reshape (no recognizable tests)")
                continue

        # Build config using metadata-aware mapper if available; for VSTAT,
        # use identity mapping since we already shaped SDTM columns.
        if vs_long or lb_long_current:
            mappings = [
                ColumnMapping(
                    source_column=col,
                    target_variable=col,
                    transformation=None,
                    confidence_score=1.0,
                )
                for col in frame.columns
            ]
            config = build_config(domain_code, mappings)
        else:
            column_hints = build_column_hints(frame)
            engine = create_mapper(
                domain_code,
                metadata=metadata,
                min_confidence=min_confidence,
                column_hints=column_hints,
            )
            suggestions = engine.suggest(frame)

            if not suggestions.mappings:
                console.print(
                    f"[yellow]⚠[/yellow] {display_name}: No mappings found, skipping"
                )
                continue

            config = build_config(domain_code, suggestions.mappings)

        config.study_id = study_id
        last_config = config

        # Build domain dataframe with metadata for codelist transformations
        domain_dataframe = build_domain_dataframe(
            frame,
            config,
            lenient=True,
            metadata=metadata,
            reference_starts=reference_starts,
        )
        all_dataframes.append(domain_dataframe)
        variant_frames.append((variant_name or domain_code, domain_dataframe))
        log_verbose(
            verbose, f"  Processed {len(domain_dataframe)} rows for {variant_name}"
        )

        # Supplemental qualifiers: capture non-mapped columns from source
        if domain_code.upper() != "LB":  # LB reshaped wide→long; skip SUPP noise
            used_source_columns: set[str] = set()
            if config and config.mappings:
                for m in config.mappings:
                    used_source_columns.add(unquote_safe(m.source_column))
                    if getattr(m, "use_code_column", None):
                        used_source_columns.add(unquote_safe(m.use_code_column))
            supp_df, used_cols = build_suppqual(
                domain_code,
                frame,
                domain_dataframe,
                used_source_columns,
                study_id=study_id,
                common_column_counts=common_column_counts,
                total_files=total_input_files,
            )
            if supp_df is not None and not supp_df.empty:
                supp_frames.append(supp_df)
            # For AE, synthesize AETRTEM=Y in SUPP to satisfy treatment-emergent info
            if domain_code.upper() == "AE" and domain_dataframe is not None:
                if "AESEQ" in domain_dataframe.columns:
                    trt_records = []
                    for _, r in domain_dataframe.iterrows():
                        seq_val = r.get("AESEQ", "")
                        try:
                            seq_str = (
                                str(int(seq_val))
                                if pd.notna(seq_val)
                                and str(seq_val).strip() != ""
                                and float(seq_val).is_integer()
                                else str(seq_val)
                            )
                        except Exception:
                            seq_str = str(seq_val)
                        trt_records.append(
                            {
                                "STUDYID": study_id,
                                "RDOMAIN": "AE",
                                "USUBJID": r.get("USUBJID", ""),
                                "IDVAR": "AESEQ",
                                "IDVARVAL": seq_str,
                                "QNAM": "AETRTEM",
                                "QLABEL": "Treatment Emergent Flag",
                                "QVAL": "Y",
                                "QORIG": "DERIVED",
                                "QEVAL": "",
                            }
                        )
                    if trt_records:
                        supp_frames.append(pd.DataFrame(trt_records))

    # Merge supplemental frames per domain
    supp_results: list[dict] = []
    if supp_frames:
        merged_supp = (
            supp_frames[0]
            if len(supp_frames) == 1
            else pd.concat(supp_frames, ignore_index=True)
        )
        supp_domain_code = f"SUPP{domain_code.upper()}"
        supp_config = build_config(
            supp_domain_code,
            [
                ColumnMapping(
                    source_column=col,
                    target_variable=col,
                    transformation=None,
                    confidence_score=1.0,
                )
                for col in merged_supp.columns
            ],
        )
        supp_config.study_id = study_id
        base_filename = get_domain(supp_domain_code).resolved_dataset_name()
        disk_name = base_filename.lower()
        supp_result = {
            "domain_code": supp_domain_code,
            "records": len(merged_supp),
            "domain_dataframe": merged_supp,
            "config": supp_config,
            "xpt_path": None,
            "xml_path": None,
            "sas_path": None,
        }
        if xpt_dir and output_format in ("xpt", "both"):
            xpt_path = xpt_dir / f"{disk_name}.xpt"
            file_label = f"Supplemental Qualifiers for {domain_code.upper()}"
            write_xpt_file(
                merged_supp, supp_domain_code, xpt_path, file_label=file_label
            )
            supp_result["xpt_path"] = xpt_path
            supp_result["xpt_filename"] = xpt_path.name
        if xml_dir and output_format in ("xml", "both"):
            xml_path = xml_dir / f"{disk_name}.xml"
            write_dataset_xml(merged_supp, supp_domain_code, supp_config, xml_path)
            supp_result["xml_path"] = xml_path
            supp_result["xml_filename"] = xml_path.name
        supp_results.append(supp_result)
    else:
        # Remove stale supplemental files if none were generated this run
        supp_domain_code = f"SUPP{domain_code.upper()}"
        if xpt_dir and output_format in ("xpt", "both"):
            stale_xpt = xpt_dir / f"{supp_domain_code.lower()}.xpt"
            if stale_xpt.exists():
                stale_xpt.unlink()
        if xml_dir and output_format in ("xml", "both"):
            stale_xml = xml_dir / f"{supp_domain_code.lower()}.xml"
            if stale_xml.exists():
                stale_xml.unlink()

    if not all_dataframes:
        raise ValueError(f"No data could be processed for {domain_code}")

    # Ensure we have a config (should always be true if we have dataframes)
    assert last_config is not None, "Config should be set if we have dataframes"

    # Merge all dataframes
    if len(all_dataframes) == 1:
        merged_dataframe = all_dataframes[0]
    else:
        merged_dataframe = pd.concat(all_dataframes, ignore_index=True)
        # Re-assign sequence numbers per subject after merge (SD0005 compliance)
        seq_col = f"{domain_code}SEQ"
        if (
            seq_col in merged_dataframe.columns
            and "USUBJID" in merged_dataframe.columns
        ):
            merged_dataframe[seq_col] = (
                merged_dataframe.groupby("USUBJID").cumcount() + 1
            )
        log_verbose(
            verbose,
            f"Merged {len(all_dataframes)} files into {len(merged_dataframe)} rows",
        )
    # Additional de-duplication for merged LB data to avoid SD1152 noise
    if domain_code.upper() == "LB":
        dedup_keys = [
            key
            for key in ("USUBJID", "LBTESTCD", "LBDTC")
            if key in merged_dataframe.columns
        ]
        if dedup_keys:
            merged_dataframe = (
                merged_dataframe.copy()
                .sort_values(by=dedup_keys)
                .drop_duplicates(subset=dedup_keys, keep="first")
                .reset_index(drop=True)
            )
            seq_col = f"{domain_code}SEQ"
            if (
                seq_col in merged_dataframe.columns
                and "USUBJID" in merged_dataframe.columns
            ):
                merged_dataframe[seq_col] = (
                    merged_dataframe.groupby("USUBJID").cumcount() + 1
                )

    # Check for missing required values
    required_vars = {
        var.name
        for var in domain.variables
        if (var.core or "").strip().lower() == "req"
    }
    auto_populated = {"STUDYID", "DOMAIN"}
    missing_required = (required_vars - auto_populated) - set(merged_dataframe.columns)
    if missing_required:
        console.print(
            f"[yellow]⚠[/yellow] {domain_code}: Missing recommended variables: {', '.join(sorted(missing_required))}"
        )

    # Use domain code as base filename (SDTM compliant) with lower-case disk names
    base_filename = domain.resolved_dataset_name()
    disk_name = base_filename.lower()

    result = {
        "domain_code": domain_code,
        "records": len(merged_dataframe),
        "domain_dataframe": merged_dataframe,
        "config": last_config,
        "xpt_path": None,
        "xml_path": None,
        "sas_path": None,
        "split_xpt_paths": [],
        "supplementals": supp_results,
    }

    # Generate XPT file
    if xpt_dir and output_format in ("xpt", "both"):
        xpt_filename = f"{disk_name}.xpt"
        xpt_path = xpt_dir / xpt_filename
        write_xpt_file(merged_dataframe, domain_code, xpt_path)
        result["xpt_path"] = xpt_path
        result["xpt_filename"] = xpt_filename
        log_success(f"Generated XPT: {xpt_path}")
        split_paths: list[Path] = []
        if domain_code.upper() == "LB" and len(variant_frames) > 1:
            split_paths = write_variant_splits(
                merged_dataframe, variant_frames, domain, xpt_dir, console
            )
            result["split_xpt_paths"] = split_paths

    # Generate Dataset-XML file
    if xml_dir and output_format in ("xml", "both"):
        xml_filename = f"{disk_name}.xml"
        xml_path = xml_dir / xml_filename
        if streaming:
            generate_dataset_xml_streaming(
                merged_dataframe,
                domain_code,
                last_config,
                xml_path,
                chunk_size=chunk_size,
            )
        else:
            write_dataset_xml(
                merged_dataframe,
                domain_code,
                last_config,
                xml_path,
            )
        result["xml_path"] = xml_path
        result["xml_filename"] = xml_filename
        log_success(f"Generated Dataset-XML: {xml_path}")

    # Generate SAS program (use first input file as reference)
    if sas_dir and generate_sas:
        sas_filename = f"{disk_name}.sas"
        sas_path = sas_dir / sas_filename
        first_input_file = files_for_domain[0][0]
        sas_code = generate_sas_program(
            domain_code, last_config, first_input_file.stem, base_filename
        )
        write_sas_file(sas_code, sas_path)
        result["sas_path"] = sas_path
        log_success(f"Generated SAS: {sas_path}")

    return result


def _synthesize_trial_design_domain(
    domain_code: str,
    study_id: str,
    output_format: str,
    xpt_dir: Path | None,
    xml_dir: Path | None,
    sas_dir: Path | None,
    generate_sas: bool,
    reference_starts: dict[str, str] | None = None,
) -> dict:
    """Synthesize a trial design domain (TS, TA, TE, SE, DS) with minimal required data."""
    from .xpt import build_domain_dataframe, write_xpt_file
    from .dataset_xml import write_dataset_xml

    def _pick_subject(ref_starts: dict[str, str] | None) -> tuple[str, str]:
        if ref_starts:
            first_id = sorted(ref_starts.keys())[0]
            return first_id, ref_starts.get(first_id) or "2023-01-01"
        return "SYNTH001", "2023-01-01"

    subject_id, base_date = _pick_subject(reference_starts)
    domain = get_domain(domain_code)

    def _default_value(var: SDTMVariable) -> object:
        name = var.name.upper()
        if name in {"STUDYID", "DOMAIN"}:
            return None
        if name == "USUBJID":
            return subject_id
        if name.endswith("SEQ"):
            return 1
        if name == "TAETORD":
            return 1
        if name.endswith("DY"):
            return 1
        if name.endswith("DTC") or name.endswith("STDTC") or name.endswith("ENDTC"):
            return base_date
        if var.type == "Num":
            return 0
        return ""

    def _base_row() -> dict[str, object]:
        row = {var.name: _default_value(var) for var in domain.variables}
        row["STUDYID"] = study_id if "STUDYID" in row else None
        row["DOMAIN"] = domain_code
        return row

    upper = domain_code.upper()
    rows: list[dict[str, object]] = []

    if upper == "TS":
        row = _base_row()
        row.update(
            {
                "TSPARMCD": "TITLE",
                "TSPARM": "Study Title",
                "TSVAL": "Synthetic Trial",
                "TSVCDREF": "",
                "TSVCDVER": "",
            }
        )
        rows.append(row)
    elif upper == "TA":
        ta_elements = [("SCRN", "SCREENING", 0), ("TRT", "TREATMENT", 1)]
        for etcd, element, order in ta_elements:
            row = _base_row()
            row.update(
                {
                    "ARMCD": "ARM1",
                    "ARM": "Treatment Arm",
                    "ETCD": etcd,
                    "ELEMENT": element,
                    "TAETORD": order,
                    "EPOCH": element,
                }
            )
            rows.append(row)
    elif upper == "TE":
        te_elements = [
            ("SCRN", "SCREENING", base_date, base_date),
            ("TRT", "TREATMENT", base_date, base_date),
        ]
        for etcd, element, start, end in te_elements:
            row = _base_row()
            row.update(
                {
                    "ETCD": etcd,
                    "ELEMENT": element,
                    "TESTRL": start,
                    "TEENRL": end,
                    "TEDUR": "P1D",
                }
            )
            rows.append(row)
    elif upper == "SE":
        se_elements = [
            ("SCRN", "SCREENING", "SCREENING"),
            ("TRT", "TREATMENT", "TREATMENT"),
        ]
        for etcd, element, epoch in se_elements:
            row = _base_row()
            row.update(
                {
                    "ETCD": etcd,
                    "ELEMENT": element,
                    "EPOCH": epoch,
                    "SESTDTC": base_date,
                    "SEENDTC": base_date,
                    "SESTDY": 1,
                    "SEENDY": 1,
                }
            )
            rows.append(row)
    elif upper == "DS":
        subjects = reference_starts.keys() if reference_starts else [subject_id]
        for usubjid in subjects:
            start_date = (
                reference_starts.get(usubjid, base_date)
                if reference_starts
                else base_date
            )
            # Informed consent
            row = _base_row()
            row.update(
                {
                    "USUBJID": usubjid,
                    "DSDECOD": "INFORMED CONSENT OBTAINED",
                    "DSTERM": "INFORMED CONSENT OBTAINED",
                    "DSCAT": "PROTOCOL MILESTONE",
                    "DSSTDTC": start_date,
                    "DSSEQ": None,
                    "EPOCH": "SCREENING",
                    "DSSTDY": 1,
                    "DSDY": 1,
                }
            )
            rows.append(row)
            # Disposition event
            row = _base_row()
            row.update(
                {
                    "USUBJID": usubjid,
                    "DSDECOD": "COMPLETED",
                    "DSTERM": "COMPLETED",
                    "DSCAT": "DISPOSITION EVENT",
                    "DSSTDTC": start_date,
                    "DSSEQ": None,
                    "EPOCH": "TREATMENT",
                    "DSSTDY": 1,
                    "DSDY": 1,
                }
            )
            rows.append(row)
    else:
        rows.append(_base_row())

    frame = pd.DataFrame(rows)

    # Build minimal config
    mappings = []
    for col in frame.columns:
        mappings.append(
            ColumnMapping(
                source_column=col,
                target_variable=col,
                transformation=None,
                confidence_score=1.0,
            )
        )
    config = build_config(domain_code, mappings)
    config.study_id = study_id

    # Build domain dataframe
    domain_dataframe = build_domain_dataframe(
        frame, config, lenient=True, reference_starts=reference_starts
    )
    if domain_code.upper() == "SE":
        for dy_col in ("SESTDY", "SEENDY"):
            if dy_col in domain_dataframe.columns:
                numeric = pd.to_numeric(domain_dataframe[dy_col], errors="coerce")
                domain_dataframe[dy_col] = (
                    numeric.replace(0, 1).fillna(1).astype("Int64")
                )
    if domain_code.upper() == "DS":
        for dy_col in ("DSSTDY", "DSDY"):
            if dy_col in domain_dataframe.columns:
                numeric = pd.to_numeric(domain_dataframe[dy_col], errors="coerce")
                domain_dataframe[dy_col] = (
                    numeric.replace(0, 1).fillna(1).astype("Int64")
                )

    result = {
        "domain_code": domain_code,
        "records": len(domain_dataframe),
        "domain_dataframe": domain_dataframe,
        "config": config,
        "xpt_path": None,
        "xml_path": None,
        "sas_path": None,
        "has_no_data": True,
    }

    base_filename = domain.resolved_dataset_name()
    disk_name = base_filename.lower()

    # Generate XPT file
    if xpt_dir and output_format in ("xpt", "both"):
        xpt_filename = f"{disk_name}.xpt"
        xpt_path = xpt_dir / xpt_filename
        write_xpt_file(domain_dataframe, domain_code, xpt_path)
        result["xpt_path"] = xpt_path
        result["xpt_filename"] = xpt_filename

    # Generate Dataset-XML file
    if xml_dir and output_format in ("xml", "both"):
        xml_filename = f"{disk_name}.xml"
        xml_path = xml_dir / xml_filename
        write_dataset_xml(
            domain_dataframe,
            domain_code,
            config,
            xml_path,
        )
        result["xml_path"] = xml_path
        result["xml_filename"] = xml_filename

    # Generate SAS program
    if sas_dir and generate_sas:
        sas_filename = f"{disk_name}.sas"
        sas_path = sas_dir / sas_filename
        sas_code = generate_sas_program(
            domain_code,
            config,
            input_dataset=f"work.{base_filename.lower()}",
            output_dataset=f"sdtm.{base_filename}",
        )
        write_sas_file(sas_code, sas_path)
        result["sas_path"] = sas_path

    return result


def _synthesize_empty_observation_domain(
    *,
    domain_code: str,
    study_id: str,
    output_format: str,
    xpt_dir: Path | None,
    xml_dir: Path | None,
    sas_dir: Path | None,
    generate_sas: bool,
    reference_starts: dict[str, str] | None = None,
) -> dict:
    """Generate an empty-but-structured observation class domain (AE/LB/VS/EX)."""
    from .xpt import build_domain_dataframe, write_xpt_file
    from .dataset_xml import write_dataset_xml

    def _pick_subject(ref_starts: dict[str, str] | None) -> tuple[str, str]:
        if ref_starts:
            first_id = sorted(ref_starts.keys())[0]
            return first_id, ref_starts.get(first_id) or "2023-01-01"
        return "SYNTH001", "2023-01-01"

    subject_id, base_date = _pick_subject(reference_starts)
    domain = get_domain(domain_code)

    def _default_value(var: SDTMVariable) -> object:
        name = var.name.upper()
        if name in {"STUDYID", "DOMAIN"}:
            return None
        if name == "USUBJID":
            return subject_id
        if name.endswith("SEQ"):
            return 1
        if name.endswith("DY"):
            return 1
        if name.endswith("DTC") or name.endswith("STDTC") or name.endswith("ENDTC"):
            return base_date
        if var.type == "Num":
            return 0
        return ""

    base_row = {var.name: _default_value(var) for var in domain.variables}
    if "STUDYID" in base_row:
        base_row["STUDYID"] = study_id
    base_row["DOMAIN"] = domain_code
    if domain_code.upper() == "AE":
        base_row["AETERM"] = base_row.get("AETERM", "") or "SYNTHETIC ADVERSE EVENT"
        base_row["AEDECOD"] = base_row.get("AEDECOD", "") or base_row["AETERM"]
        base_row["AEACN"] = base_row.get("AEACN", "") or "DOSE NOT CHANGED"
        base_row["AESEV"] = base_row.get("AESEV", "") or "MILD"
        base_row["AESER"] = base_row.get("AESER", "") or "N"
        base_row["AEREL"] = base_row.get("AEREL", "") or "NOT RELATED"
        base_row["EPOCH"] = base_row.get("EPOCH", "") or "TREATMENT"
    if domain_code.upper() == "EX":
        base_row["EXTRT"] = base_row.get("EXTRT", "") or "PLACEBO"
        base_row["EXDOSE"] = base_row.get("EXDOSE", 0)
        base_row["EXDOSU"] = base_row.get("EXDOSU", "") or "mg"
        base_row["EXDUR"] = base_row.get("EXDUR", "") or "P1D"
        base_row["EXCAT"] = base_row.get("EXCAT", "") or "INVESTIGATIONAL PRODUCT"
        base_row["EPOCH"] = base_row.get("EPOCH", "") or "TREATMENT"
        base_row["EXSTDY"] = base_row.get("EXSTDY", 1) or 1
        base_row["EXENDY"] = base_row.get("EXENDY", 1) or 1
        base_row["EXTPTNUM"] = base_row.get("EXTPTNUM", 1) or 1
        base_row["EXTPT"] = base_row.get("EXTPT", "") or "VISIT 1"
        base_row["EXTPTREF"] = base_row.get("EXTPTREF", "") or "VISIT"
    if domain_code.upper() == "LB":
        base_row["LBTESTCD"] = base_row.get("LBTESTCD", "") or "CHOL"
        base_row["LBTEST"] = base_row.get("LBTEST", "") or "Cholesterol"
        base_row["LBORRES"] = base_row.get("LBORRES", "") or "0"
        base_row["LBORRESU"] = base_row.get("LBORRESU", "") or "mg/dL"
        base_row["LBORNRLO"] = base_row.get("LBORNRLO", "") or "0"
        base_row["LBORNRHI"] = base_row.get("LBORNRHI", "") or "0"
        base_row["LBSTRESC"] = base_row.get("LBSTRESC", "") or "0"
        base_row["EPOCH"] = base_row.get("EPOCH", "") or "SCREENING"
        base_row["LBDY"] = base_row.get("LBDY", 1) or 1
        base_row["LBENDY"] = base_row.get("LBENDY", 1) or 1
        base_row["VISIT"] = base_row.get("VISIT", "") or "Visit 1"
        base_row["VISITNUM"] = base_row.get("VISITNUM", 1) or 1
        base_row["VISITDY"] = base_row.get("VISITDY", 1) or 1
        base_row["LBTPTNUM"] = base_row.get("LBTPTNUM", 1) or 1
        base_row["LBTPT"] = base_row.get("LBTPT", "") or "VISIT 1"
    if domain_code.upper() == "VS":
        base_row["VSTESTCD"] = base_row.get("VSTESTCD", "") or "SYSBP"
        base_row["VSTEST"] = base_row.get("VSTEST", "") or "Systolic Blood Pressure"
        base_row["VSORRES"] = base_row.get("VSORRES", "") or "120"
        base_row["VSORRESU"] = base_row.get("VSORRESU", "") or "mmHg"
        base_row["VSDY"] = base_row.get("VSDY", 1) or 1
        base_row["VISIT"] = base_row.get("VISIT", "") or "Visit 1"
        base_row["VISITNUM"] = base_row.get("VISITNUM", 1) or 1
        base_row["VISITDY"] = base_row.get("VISITDY", 1) or 1
        base_row["EPOCH"] = base_row.get("EPOCH", "") or "TREATMENT"

    empty_frame = pd.DataFrame([base_row])
    supplemental_results: list[dict] = []

    mappings = [
        ColumnMapping(
            source_column=col,
            target_variable=col,
            transformation=None,
            confidence_score=1.0,
        )
        for col in empty_frame.columns
    ]
    config = build_config(domain_code, mappings)
    config.study_id = study_id

    domain_dataframe = build_domain_dataframe(
        empty_frame,
        config,
        lenient=True,
        reference_starts=reference_starts,
    )
    if domain_code.upper() == "LB":
        for dy_col in ("LBDY", "LBENDY", "VISITDY"):
            if dy_col in domain_dataframe.columns:
                domain_dataframe[dy_col] = (
                    pd.to_numeric(domain_dataframe[dy_col], errors="coerce")
                    .replace(0, 1)
                    .fillna(1)
                    .astype("Int64")
                )
    if domain_code.upper() == "EX":
        for dy_col in ("EXSTDY", "EXENDY"):
            if dy_col in domain_dataframe.columns:
                domain_dataframe[dy_col] = (
                    pd.to_numeric(domain_dataframe[dy_col], errors="coerce")
                    .replace(0, 1)
                    .fillna(1)
                    .astype("Int64")
                )
    if domain_code.upper() == "VS":
        for dy_col in ("VSDY", "VISITDY"):
            if dy_col in domain_dataframe.columns:
                domain_dataframe[dy_col] = (
                    pd.to_numeric(domain_dataframe[dy_col], errors="coerce")
                    .replace(0, 1)
                    .fillna(1)
                    .astype("Int64")
                )

    result = {
        "domain_code": domain_code,
        "records": len(domain_dataframe),
        "domain_dataframe": domain_dataframe,
        "config": config,
        "xpt_path": None,
        "xml_path": None,
        "sas_path": None,
        "has_no_data": True,
        "supplementals": supplemental_results,
    }

    base_filename = domain.resolved_dataset_name()
    disk_name = base_filename.lower()

    if xpt_dir and output_format in ("xpt", "both"):
        xpt_filename = f"{disk_name}.xpt"
        xpt_path = xpt_dir / xpt_filename
        write_xpt_file(domain_dataframe, domain_code, xpt_path)
        result["xpt_path"] = xpt_path
        result["xpt_filename"] = xpt_filename

    if xml_dir and output_format in ("xml", "both"):
        xml_filename = f"{disk_name}.xml"
        xml_path = xml_dir / xml_filename
        write_dataset_xml(
            domain_dataframe,
            domain_code,
            config,
            xml_path,
        )
        result["xml_path"] = xml_path
        result["xml_filename"] = xml_filename

    if sas_dir and generate_sas:
        sas_filename = f"{disk_name}.sas"
        sas_path = sas_dir / sas_filename
        sas_code = generate_sas_program(
            domain_code,
            config,
            input_dataset=f"work.{base_filename.lower()}",
            output_dataset=f"sdtm.{base_filename}",
        )
        write_sas_file(sas_code, sas_path)
        result["sas_path"] = sas_path

    # Generate supplemental qualifiers for AE (treatment-emergent flag)
    if domain_code.upper() == "AE":
        supp_domain_code = f"SUPP{domain_code.upper()}"
        supp_domain = get_domain(supp_domain_code)
        supp_row = {var.name: "" for var in supp_domain.variables}
        supp_row.update(
            {
                "STUDYID": study_id,
                "RDOMAIN": "AE",
                "USUBJID": base_row.get("USUBJID", "SYNTH001"),
                "IDVAR": "AESEQ",
                "IDVARVAL": str(base_row.get("AESEQ", 1) or 1),
                "QNAM": "AETRTEM",
                "QLABEL": "Treatment Emergent Flag",
                "QVAL": "Y",
                "QORIG": "DERIVED",
                "QEVAL": "",
            }
        )
        supp_df = pd.DataFrame([supp_row])
        supp_config = build_config(
            supp_domain_code,
            [
                ColumnMapping(
                    source_column=col,
                    target_variable=col,
                    transformation=None,
                    confidence_score=1.0,
                )
                for col in supp_df.columns
            ],
        )
        supp_config.study_id = study_id
        supp_disk = supp_domain.resolved_dataset_name().lower()
        supp_entry = {
            "domain_code": supp_domain_code,
            "records": len(supp_df),
            "domain_dataframe": supp_df,
            "config": supp_config,
            "xpt_path": None,
            "xml_path": None,
            "sas_path": None,
        }
        if xpt_dir and output_format in ("xpt", "both"):
            supp_xpt = xpt_dir / f"{supp_disk}.xpt"
            write_xpt_file(
                supp_df,
                supp_domain_code,
                supp_xpt,
                file_label="Supplemental Qualifiers for AE",
            )
            supp_entry["xpt_path"] = supp_xpt
            supp_entry["xpt_filename"] = supp_xpt.name
        if xml_dir and output_format in ("xml", "both"):
            supp_xml = xml_dir / f"{supp_disk}.xml"
            write_dataset_xml(supp_df, supp_domain_code, supp_config, supp_xml)
            supp_entry["xml_path"] = supp_xml
            supp_entry["xml_filename"] = supp_xml.name
        supplemental_results.append(supp_entry)

    return result


def _synthesize_relrec(
    *,
    study_id: str,
    output_format: str,
    xpt_dir: Path | None,
    xml_dir: Path | None,
    sas_dir: Path | None,
    generate_sas: bool,
    domain_results: list[dict],
) -> dict:
    """Create a populated RELREC dataset based on existing domain data."""
    from .xpt import build_domain_dataframe, write_xpt_file
    from .dataset_xml import write_dataset_xml

    domain = get_domain("RELREC")

    relrec_records = _build_relrec_records(domain_results, study_id)
    if relrec_records.empty:
        frame = pd.DataFrame(
            {var.name: pd.Series(dtype=var.pandas_dtype()) for var in domain.variables}
        )
    else:
        frame = relrec_records

    mappings = [
        ColumnMapping(
            source_column=col,
            target_variable=col,
            transformation=None,
            confidence_score=1.0,
        )
        for col in frame.columns
    ]
    config = build_config("RELREC", mappings)
    config.study_id = study_id

    domain_dataframe = build_domain_dataframe(frame, config, lenient=True)

    result = {
        "domain_code": "RELREC",
        "records": len(domain_dataframe),
        "domain_dataframe": domain_dataframe,
        "config": config,
        "xpt_path": None,
        "xml_path": None,
        "sas_path": None,
        "has_no_data": relrec_records.empty,
    }

    base_filename = domain.resolved_dataset_name()
    disk_name = base_filename.lower()

    # Generate XPT file
    if xpt_dir and output_format in ("xpt", "both"):
        xpt_filename = f"{disk_name}.xpt"
        xpt_path = xpt_dir / xpt_filename
        write_xpt_file(domain_dataframe, "RELREC", xpt_path)
        result["xpt_path"] = xpt_path
        result["xpt_filename"] = xpt_filename

    # Generate Dataset-XML file
    if xml_dir and output_format in ("xml", "both"):
        xml_filename = f"{disk_name}.xml"
        xml_path = xml_dir / xml_filename
        write_dataset_xml(domain_dataframe, "RELREC", config, xml_path)
        result["xml_path"] = xml_path
        result["xml_filename"] = xml_filename

    # Generate SAS program
    if sas_dir and generate_sas:
        sas_filename = f"{disk_name}.sas"
        sas_path = sas_dir / sas_filename
        sas_code = generate_sas_program(
            "RELREC",
            config,
            input_dataset="work.relrec",
            output_dataset=f"sdtm.{base_filename}",
        )
        write_sas_file(sas_code, sas_path)
        result["sas_path"] = sas_path

    return result


def _build_relrec_records(domain_results: list[dict], study_id: str) -> pd.DataFrame:
    """Build RELREC rows linking AE/EX records back to DS by subject."""

    def _get_domain_df(code: str) -> pd.DataFrame | None:
        for entry in domain_results:
            if entry.get("domain_code", "").upper() == code.upper():
                df = entry.get("domain_dataframe")
                if isinstance(df, pd.DataFrame) and not df.empty:
                    return df
        return None

    ae_df = _get_domain_df("AE")
    ds_df = _get_domain_df("DS")
    ex_df = _get_domain_df("EX")

    records: list[dict] = []

    def _seq_map(df: pd.DataFrame, seq_col: str) -> dict:
        if seq_col not in df.columns:
            return {}
        numeric = pd.to_numeric(df[seq_col], errors="coerce")
        return (
            pd.DataFrame({"USUBJID": df["USUBJID"], seq_col: numeric})
            .dropna(subset=["USUBJID", seq_col])
            .groupby("USUBJID")[seq_col]
            .min()
            .to_dict()
        )

    ds_seq_map = _seq_map(ds_df, "DSSEQ") if ds_df is not None else {}

    def _stringify(val: object, fallback_index: int) -> str:
        if pd.isna(val):
            return str(fallback_index)
        try:
            numeric = pd.to_numeric(val)
            if pd.isna(numeric):
                return str(val)
            if float(numeric).is_integer():
                return str(int(numeric))
            return str(numeric)
        except Exception:
            return str(val)

    def _add_pair(
        rdomain: str,
        usubjid: str,
        idvar: str,
        idvarval: str,
        relid: str,
        reltype: str | None = None,
    ) -> None:
        records.append(
            {
                "STUDYID": study_id,
                "RDOMAIN": rdomain,
                "USUBJID": usubjid,
                "IDVAR": idvar,
                "IDVARVAL": idvarval,
                "RELTYPE": reltype or "",
                "RELID": relid,
            }
        )

    if ae_df is not None and ds_seq_map:
        for idx, row in ae_df.iterrows():
            usubjid = str(row.get("USUBJID", "") or "").strip()
            if not usubjid:
                continue
            aeseq = _stringify(row.get("AESEQ"), idx + 1)
            relid = f"AE_DS_{usubjid}_{aeseq}"
            _add_pair("AE", usubjid, "AESEQ", aeseq, relid, None)
            ds_seq = ds_seq_map.get(usubjid)
            if ds_seq is not None:
                _add_pair("DS", usubjid, "DSSEQ", _stringify(ds_seq, 1), relid, None)

    if ex_df is not None and ds_seq_map:
        for idx, row in ex_df.iterrows():
            usubjid = str(row.get("USUBJID", "") or "").strip()
            if not usubjid:
                continue
            exseq = _stringify(row.get("EXSEQ"), idx + 1)
            relid = f"EX_DS_{usubjid}_{exseq}"
            _add_pair("EX", usubjid, "EXSEQ", exseq, relid, None)
            ds_seq = ds_seq_map.get(usubjid)
            if ds_seq is not None:
                _add_pair("DS", usubjid, "DSSEQ", _stringify(ds_seq, 1), relid, None)

    if not records and ds_df is not None:
        # Fallback: relate first two DS records per subject if nothing else available
        ds_seq_map = _seq_map(ds_df, "DSSEQ")
        for usubjid, ds_seq in ds_seq_map.items():
            relid = f"DS_ONLY_{usubjid}"
            _add_pair("DS", str(usubjid), "DSSEQ", _stringify(ds_seq, 1), relid, None)

    return pd.DataFrame(records)


def _reshape_vs_to_long(frame: pd.DataFrame, study_id: str) -> pd.DataFrame:
    """Convert source VS wide data to SDTM-compliant long rows using dynamic tests."""

    df = frame.copy()
    rename_map = {
        "Subject Id": "USUBJID",
        "SubjectId": "USUBJID",
        "Event name": "VISIT",
        "Event Name": "VISIT",
        "EventName": "VISIT",
        "Event sequence number": "VISITNUM",
        "Event Sequence Number": "VISITNUM",
        "EventSeq": "VISITNUM",
        "Event date": "VSDTC",
        "Event Date": "VSDTC",
        "EventDate": "VSDTC",
    }
    df = df.rename(columns=rename_map)

    # Normalize visit identifiers
    if "VISITNUM" in df.columns:
        df["VISITNUM"] = pd.to_numeric(df["VISITNUM"], errors="coerce")
    if "VISIT" not in df.columns and "VISITNUM" in df.columns:
        df["VISIT"] = df["VISITNUM"].apply(
            lambda n: f"Visit {int(n)}" if pd.notna(n) else ""
        )

    # Discover tests dynamically from ORRES_* columns
    tests = []
    for col in df.columns:
        m = re.match(r"^ORRES_([A-Za-z0-9]+)$", col, re.I)
        if m:
            testcd = m.group(1).upper()
            if testcd.endswith("CD"):
                continue
            tests.append(testcd)
    tests = sorted(set(tests))

    if not tests:
        return pd.DataFrame()

    # Map aliases to standard CDISC CT
    alias_map = {
        "PLS": "HR",
        "HR": "HR",
        "SYSBP": "SYSBP",
        "DIABP": "DIABP",
        "TEMP": "TEMP",
        "WEIGHT": "WEIGHT",
        "HEIGHT": "HEIGHT",
        "BMI": "BMI",
    }
    label_map = {
        "HR": "Heart Rate",
        "SYSBP": "Systolic Blood Pressure",
        "DIABP": "Diastolic Blood Pressure",
        "TEMP": "Temperature",
        "WEIGHT": "Weight",
        "HEIGHT": "Height",
        "BMI": "Body Mass Index",
    }
    unit_defaults = {
        "HR": "beats/min",
        "SYSBP": "mmHg",
        "DIABP": "mmHg",
        "TEMP": "C",
        "WEIGHT": "kg",
        "HEIGHT": "cm",
        "BMI": "kg/m2",
    }

    records: list[dict] = []
    for _, row in df.iterrows():
        usubjid = str(row.get("USUBJID", "") or "").strip()
        if not usubjid:
            continue
        visitnum = row.get("VISITNUM", pd.NA)
        visit = str(row.get("VISIT", "") or "").strip()
        if not visit and pd.notna(visitnum):
            visit = f"Visit {int(visitnum)}" if float(visitnum).is_integer() else ""

        vsdtc = row.get("VSDTC", "")
        status_cd = str(row.get("VSPERFCD", "") or "").strip().upper()
        reason = str(row.get("VSREASND", "") or "").strip()

        for testcd_raw in tests:
            std_testcd = alias_map.get(testcd_raw, None)
            if not std_testcd:
                continue
            value = row.get(f"ORRES_{testcd_raw}", pd.NA)
            if status_cd != "N" and pd.isna(value):
                continue
            unit_val = row.get(f"ORRESU_{testcd_raw}", "")
            pos_val = row.get(f"POS_{testcd_raw}", "")
            label_val = row.get(f"TEST_{testcd_raw}", "")
            stat_val = "NOT DONE" if status_cd == "N" else ""
            if stat_val != "NOT DONE" and (
                unit_val is None or str(unit_val).strip() == ""
            ):
                unit_val = unit_defaults.get(std_testcd, "")
            records.append(
                {
                    "STUDYID": study_id,
                    "DOMAIN": "VS",
                    "USUBJID": usubjid,
                    "VSTESTCD": std_testcd[:8],
                    "VSTEST": str(label_map.get(std_testcd, label_val or std_testcd)),
                    "VSORRES": "" if stat_val else value,
                    "VSORRESU": "" if stat_val else unit_val,
                    "VSSTAT": stat_val,
                    "VSREASND": reason if stat_val else "",
                    "VISITNUM": visitnum,
                    "VISIT": visit,
                    "VSDTC": vsdtc,
                    "VSPOS": pos_val,
                }
            )

    if not records:
        return pd.DataFrame()
    return pd.DataFrame(records)


def _reshape_lb_to_long(frame: pd.DataFrame, study_id: str) -> pd.DataFrame:
    """Convert wide LB source data to long-form SDTM rows."""

    df = frame.copy()
    allowed_tests = {
        "CHOL": "Cholesterol",
        "AST": "Aspartate Aminotransferase",
        "ALT": "Alanine Aminotransferase",
        "GLUC": "Glucose",
        "HGB": "Hemoglobin",
        "HCT": "Hematocrit",
        "RBC": "Erythrocytes",
        "WBC": "Leukocytes",
        "PLAT": "Platelets",
    }
    rename_map = {
        "Subject Id": "USUBJID",
        "SubjectId": "USUBJID",
        "Event date": "LBDTC",
        "Event Date": "LBDTC",
        "EventDate": "LBDTC",
        "Date of blood sample": "LBDTC",
        "Date ofstool sample": "LBDTC",
        "Date ofurine sample": "LBDTC",
        "Date of pregnancy test": "LBDTC",
        "EventDate": "LBDTC",
    }
    df = df.rename(columns=rename_map)

    usubjid_col = "USUBJID" if "USUBJID" in df.columns else None
    dtc_candidates = [
        c
        for c in df.columns
        if str(c).upper() == "LBDTC" or str(c).upper().endswith("DAT")
    ]

    # Collect test-specific columns by prefix
    test_defs: dict[str, dict[str, str]] = {}
    for col in df.columns:
        m = re.match(
            r"^([A-Za-z0-9]+)\s+result or finding in original units$", col, re.I
        )
        if m:
            test = m.group(1).upper()
            test_defs.setdefault(test, {})["orres"] = col
            continue
        m = re.match(r"^TEST_([A-Za-z0-9]+)$", col, re.I)
        if m:
            test = m.group(1).upper()
            test_defs.setdefault(test, {})["label"] = col
            continue
        m = re.match(r"^ORRES_([A-Za-z0-9]+)$", col, re.I)
        if m:
            test = m.group(1).upper()
            if test.endswith("CD"):
                continue
            test_defs.setdefault(test, {})["orres"] = col
            continue
        m = re.match(r"^([A-Za-z0-9]+)ORRES$", col, re.I)
        if m:
            test = m.group(1).upper()
            if test.endswith("CD"):
                continue
            test_defs.setdefault(test, {})["orres"] = col
            continue
        m = re.match(r"^([A-Za-z0-9]+)\s+unit(?:\s*-.*)?$", col, re.I)
        if m:
            test = m.group(1).upper()
            test_defs.setdefault(test, {})["unit"] = col
            continue
        m = re.match(r"^ORRESU_([A-Za-z0-9]+)$", col, re.I)
        if m:
            test = m.group(1).upper()
            if test.endswith("CD"):
                continue
            test_defs.setdefault(test, {})["unit"] = col
            continue
        m = re.match(r"^([A-Za-z0-9]+)\s+range \(lower limit\)$", col, re.I)
        if m:
            test = m.group(1).upper()
            test_defs.setdefault(test, {})["nrlo"] = col
            continue
        m = re.match(r"^ORNR_([A-Za-z0-9]+)_Lower$", col, re.I)
        if m:
            test = m.group(1).upper()
            test_defs.setdefault(test, {})["nrlo"] = col
            continue
        m = re.match(r"^([A-Za-z0-9]+)\s+range \(upper limit\)$", col, re.I)
        if m:
            test = m.group(1).upper()
            test_defs.setdefault(test, {})["nrhi"] = col
            continue
        m = re.match(r"^ORNR_([A-Za-z0-9]+)_Upper$", col, re.I)
        if m:
            test = m.group(1).upper()
            test_defs.setdefault(test, {})["nrhi"] = col
            continue

    if not test_defs:
        return df

    def _pick_first(val: object) -> object:
        if isinstance(val, pd.Series):
            for v in val:
                if pd.isna(v):
                    continue
                if str(v).strip():
                    return v
            return None
        return val

    records: list[dict] = []
    for _, row in df.iterrows():
        usubjid = str(row.get(usubjid_col, "") or "").strip() if usubjid_col else ""
        if not usubjid or usubjid.lower() == "subjectid":
            continue
        lbdtc = ""
        for cand in dtc_candidates:
            val = _pick_first(row.get(cand, ""))
            if val is None:
                continue
            sval = str(val).strip()
            if sval:
                lbdtc = sval
                break

        for testcd, cols in test_defs.items():
            # Map/normalize test code; skip unsupported tests
            norm_testcd = testcd.upper()
            if norm_testcd == "GLUCU":
                norm_testcd = "GLUC"
            if norm_testcd not in allowed_tests:
                continue
            orres_col = cols.get("orres")
            if not orres_col:
                continue
            value = _pick_first(row.get(orres_col, ""))
            if value is None:
                continue
            value_str = str(value).strip()
            if not value_str or value_str.upper().startswith("ORRES"):
                continue
            unit_val = (
                _pick_first(row.get(cols.get("unit"), "")) if cols.get("unit") else ""
            )
            nrlo_val = (
                _pick_first(row.get(cols.get("nrlo"), "")) if cols.get("nrlo") else ""
            )
            nrhi_val = (
                _pick_first(row.get(cols.get("nrhi"), "")) if cols.get("nrhi") else ""
            )
            label_val = (
                _pick_first(row.get(cols.get("label"), "")) if cols.get("label") else ""
            )
            records.append(
                {
                    "STUDYID": study_id,
                    "DOMAIN": "LB",
                    "USUBJID": usubjid,
                    "LBTESTCD": norm_testcd[:8],
                    "LBTEST": allowed_tests.get(norm_testcd, label_val or norm_testcd),
                    "LBORRES": value_str,
                    "LBORRESU": unit_val,
                    "LBORNRLO": nrlo_val,
                    "LBORNRHI": nrhi_val,
                    "LBDTC": lbdtc,
                }
            )

    if not records:
        return pd.DataFrame()
    return pd.DataFrame(records)
