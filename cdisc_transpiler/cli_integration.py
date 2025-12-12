"""CLI Integration Guide and Helper Functions.

This module demonstrates how to integrate the new service layer
with the existing CLI and provides helper functions for migration.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .metadata import StudyMetadata
    from .services import DomainProcessingService, FileGenerationService

# Example usage patterns for migrating CLI code


def example_process_domain_with_service(
    domain_code: str,
    source_file: Path,
    study_id: str,
    output_dir: Path,
    *,
    metadata: StudyMetadata | None = None,
    reference_starts: dict[str, str] | None = None,
) -> None:
    """Example: Process a domain using the service layer.
    
    This replaces the complex domain processing logic in CLI.
    
    Args:
        domain_code: SDTM domain code
        source_file: Path to source data
        study_id: Study identifier
        output_dir: Output directory
        metadata: Optional study metadata
        reference_starts: Optional reference dates
    """
    from .services import DomainProcessingService, FileGenerationService
    from .cli_utils import log_success, log_error, print_domain_header
    
    # Print header
    print_domain_header(domain_code, [source_file])
    
    try:
        # Step 1: Process domain
        domain_service = DomainProcessingService(
            study_id=study_id,
            metadata=metadata,
            reference_starts=reference_starts,
        )
        
        result = domain_service.process_domain(
            domain_code,
            source_file,
            transform_long=(domain_code.upper() in {"VS", "LB"}),
            generate_suppqual=True,
        )
        
        log_success(f"Processed {result.record_count} records")
        
        # Step 2: Generate files
        file_service = FileGenerationService(
            output_dir,
            generate_xpt=True,
            generate_sas=True,
        )
        
        files = file_service.generate_files(
            domain_code,
            result.dataframe,
            result.config,
        )
        
        if files.xpt_path:
            log_success(f"Generated XPT: {files.xpt_path.name}")
        if files.sas_path:
            log_success(f"Generated SAS: {files.sas_path.name}")
        
        # Step 3: Process supplementals
        if result.supplementals:
            for supp in result.supplementals:
                supp_files = file_service.generate_files(
                    supp.domain_code,
                    supp.dataframe,
                    supp.config,
                )
                if supp_files.xpt_path:
                    log_success(f"Generated SUPP XPT: {supp_files.xpt_path.name}")
        
    except Exception as exc:
        log_error(f"Failed to process {domain_code}: {exc}")
        raise


def example_merge_domain_variants(
    domain_code: str,
    source_files: list[Path],
    study_id: str,
    output_dir: Path,
    *,
    metadata: StudyMetadata | None = None,
    reference_starts: dict[str, str] | None = None,
) -> None:
    """Example: Merge domain variants using the service layer.
    
    This replaces complex merging logic in CLI.
    
    Args:
        domain_code: SDTM domain code
        source_files: List of variant files
        study_id: Study identifier
        output_dir: Output directory
        metadata: Optional study metadata
        reference_starts: Optional reference dates
    """
    from .services import DomainProcessingService, FileGenerationService
    from .cli_utils import log_success, print_domain_header
    
    print_domain_header(domain_code, source_files)
    
    # Step 1: Process each variant
    domain_service = DomainProcessingService(
        study_id=study_id,
        metadata=metadata,
        reference_starts=reference_starts,
    )
    
    variants = []
    for source_file in source_files:
        result = domain_service.process_domain(
            domain_code,
            source_file,
            transform_long=False,
            generate_suppqual=False,  # Generate after merging
        )
        variants.append(result)
        log_success(f"Processed variant: {source_file.name} ({result.record_count} records)")
    
    # Step 2: Merge variants
    merged = domain_service.merge_domain_variants(domain_code, variants)
    log_success(f"Merged into {merged.record_count} total records")
    
    # Step 3: Generate files
    file_service = FileGenerationService(
        output_dir,
        generate_xpt=True,
        generate_sas=True,
    )
    
    files = file_service.generate_files(
        domain_code,
        merged.dataframe,
        merged.config,
    )
    
    if files.xpt_path:
        log_success(f"Generated merged XPT: {files.xpt_path.name}")


def example_synthesize_trial_design(
    study_id: str,
    output_dir: Path,
    reference_starts: dict[str, str],
) -> None:
    """Example: Synthesize trial design domains.
    
    This replaces scattered trial design logic in CLI.
    
    Args:
        study_id: Study identifier
        output_dir: Output directory
        reference_starts: Subject reference dates
    """
    from .services import TrialDesignService, FileGenerationService
    from .cli_utils import log_success, log_info
    
    log_info("Synthesizing trial design domains...")
    
    # Create services
    trial_service = TrialDesignService(study_id, reference_starts)
    file_service = FileGenerationService(output_dir, generate_xpt=True)
    
    # Synthesize each domain
    domains_to_synthesize = [
        ("TS", trial_service.synthesize_ts),
        ("TA", trial_service.synthesize_ta),
        ("TE", trial_service.synthesize_te),
        ("SE", trial_service.synthesize_se),
        ("DS", trial_service.synthesize_ds),
    ]
    
    for domain_code, synthesize_func in domains_to_synthesize:
        df, config = synthesize_func()
        files = file_service.generate_files(domain_code, df, config)
        if files.xpt_path:
            log_success(f"Synthesized {domain_code}: {files.xpt_path.name}")


def example_progress_tracking():
    """Example: Use progress tracking in CLI."""
    from .cli_utils import ProgressTracker, progress_bar
    
    # Create tracker
    tracker = ProgressTracker(total_domains=10)
    
    # Process domains
    with progress_bar("Processing domains", total=10) as (progress, task):
        for i in range(10):
            # ... process domain ...
            error = (i == 5)  # Simulate error on domain 5
            tracker.increment(error=error)
            progress.update(task, advance=1)
    
    # Print summary
    tracker.print_summary()


# Migration helpers

def get_legacy_reference_starts(dm_df) -> dict[str, str]:
    """Extract reference starts from DM dataframe (legacy compatibility).
    
    Args:
        dm_df: Demographics dataframe
        
    Returns:
        USUBJID -> RFSTDTC mapping
    """
    if dm_df is None or dm_df.empty:
        return {}
    
    if "USUBJID" not in dm_df.columns or "RFSTDTC" not in dm_df.columns:
        return {}
    
    return dm_df.set_index("USUBJID")["RFSTDTC"].dropna().to_dict()


def discover_domain_files_helper(input_dir: Path) -> dict[str, list[Path]]:
    """Discover domain files in input directory (legacy compatibility).
    
    Args:
        input_dir: Input directory
        
    Returns:
        Dictionary of domain_code -> list of files
    """
    domain_files: dict[str, list[Path]] = {}
    
    for file_path in sorted(input_dir.glob("*.csv")):
        # Extract domain code from filename
        # Supports patterns like: dm.csv, DM.csv, demographics.csv, etc.
        name = file_path.stem.upper()
        
        # Try to match against known domains
        from .domains import list_domains
        domains = list_domains()
        
        for domain in domains:
            if name.startswith(domain.code.upper()):
                code = domain.code
                if code not in domain_files:
                    domain_files[code] = []
                domain_files[code].append(file_path)
                break
    
    return domain_files


# Validation integration example

def example_validate_after_processing():
    """Example: Integrate validation after processing."""
    from .validators import ValidationEngine, format_validation_report
    from .cli_utils import log_warning, log_error
    
    # Assuming we have processed domains stored in domain_results
    domain_results = {}  # domain_code -> (domain_obj, dataframe)
    study_id = "STUDY001"
    
    # Create validation engine
    engine = ValidationEngine()
    
    # Validate all domains
    issues_by_domain = engine.validate_study(
        study_id=study_id,
        domains=domain_results,
        controlled_terminology=None,  # Add CT if available
        reference_starts={},
    )
    
    # Report issues
    if issues_by_domain:
        log_warning(f"Found validation issues in {len(issues_by_domain)} domain(s)")
        report = format_validation_report(issues_by_domain)
        print(report)
    else:
        from .cli_utils import log_success
        log_success("All domains passed validation")
