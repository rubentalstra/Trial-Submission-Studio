"""Helper functions for CLI operations.

This module contains utility functions extracted from the main CLI module
to improve code organization and reusability.

SDTM Reference:
    These utilities support SDTM-compliant output generation as defined
    in SDTMIG v3.4. The module handles PDF generation for Define-XML
    and split dataset management per Section 4.1.7.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Any

import pandas as pd
from rich.console import Console

from ..xpt_module import write_xpt_file

if TYPE_CHECKING:
    from ..domains_module import SDTMDomain

console = Console()


def write_variant_splits(
    variant_frames: list[tuple[str, pd.DataFrame]],
    domain: SDTMDomain,
    xpt_dir: Path,
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

    Returns:
        Tuple of (list of paths, list of (split_name, dataframe, path) tuples)
    """
    from .logging_config import get_logger

    logger = get_logger()
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
            logger.warning(
                f"Warning: Split dataset '{table}' does not start "
                f"with domain code '{domain_code}'. Skipping."
            )
            continue

        if len(table) > 8:
            logger.warning(
                f"Warning: Split dataset name '{table}' exceeds "
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
        split_suffix = table[len(domain_code) :]
        file_label = (
            f"{domain.description} - {split_suffix}"
            if split_suffix
            else domain.description
        )

        write_xpt_file(
            variant_df, domain.code, split_path, file_label=file_label, table_name=table
        )
        split_paths.append(split_path)
        split_datasets.append((table, variant_df, split_path))
        logger.success(
            f"Split dataset: {split_path} (DOMAIN={domain_code}, table={table})"
        )

    return split_paths, split_datasets


def print_study_summary(
    results: list[dict[str, Any]],
    errors: list[tuple[str, str]],
    output_dir: Path,
    output_format: str,
    generate_define: bool,
    generate_sas: bool,
) -> None:
    """Print summary of study processing results with detailed table.

    This function is maintained for backward compatibility but now delegates
    to the SummaryPresenter class for actual formatting and display.

    Args:
        results: List of processing results
        errors: List of (domain, error) tuples
        output_dir: Output directory path
        output_format: Output format (xpt, xml, both)
        generate_define: Whether Define-XML was generated
        generate_sas: Whether SAS programs were generated
    """
    from .presenters import SummaryPresenter

    presenter = SummaryPresenter(console)
    presenter.present(
        results=results,
        errors=errors,
        output_dir=output_dir,
        output_format=output_format,
        generate_define=generate_define,
        generate_sas=generate_sas,
    )
