"""SUPPQUAL (Supplemental Qualifiers) building utilities.

This module provides functions for building SUPPQUAL DataFrames
for non-model columns in parent SDTM domains.

NOTE: This module is a compatibility wrapper. The actual implementation
has been moved to `cdisc_transpiler.domain.services.suppqual_service`.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Any

import pandas as pd

# Re-export from domain service for backwards compatibility
from ..domain.services.suppqual_service import (
    sanitize_qnam as _sanitize_qnam,
    build_suppqual as _build_suppqual_service,
    extract_used_columns as _extract_used_columns,
    finalize_suppqual,
)
from ..domains_module import get_domain

if TYPE_CHECKING:
    pass


# Re-export sanitize function
_sanitize_qnam = _sanitize_qnam


def extract_used_columns(config: Any) -> set[str]:
    return _extract_used_columns(config)


def build_suppqual(
    domain_code: str,
    source_df: pd.DataFrame,
    mapped_df: pd.DataFrame | None,
    used_source_columns: set[str] | None = None,
    *,
    study_id: str | None = None,
    common_column_counts: dict[str, int] | None = None,
    total_files: int | None = None,
) -> tuple[pd.DataFrame | None, set[str]]:
    """Build a SUPP-- DataFrame for non-model columns in a parent domain.

    Creates a SUPPQUAL domain containing supplemental qualifier data for
    columns that exist in the source data but are not part of the SDTM
    domain model.

    Args:
        domain_code: Two-character domain code (e.g., "DM", "AE")
        source_df: Source DataFrame with raw data
        mapped_df: Mapped DataFrame with processed domain data
        used_source_columns: Set of source columns already mapped
        study_id: Study identifier for STUDYID column
        common_column_counts: Count of column appearances across files
        total_files: Total number of input files

    Returns:
        Tuple of (supp_df, used_columns):
        - supp_df: SUPPQUAL DataFrame or None if no qualifiers found
        - used_columns: Set of source columns included in SUPPQUAL
    """
    # Get domain definition
    domain = domain_code.upper()
    domain_def = get_domain(domain)

    # Call domain service
    supp_df, used_cols = _build_suppqual_service(
        domain_code=domain_code,
        source_df=source_df,
        mapped_df=mapped_df,
        domain_def=domain_def,
        used_source_columns=used_source_columns,
        study_id=study_id,
        common_column_counts=common_column_counts,
        total_files=total_files,
    )

    if supp_df is None:
        return None, used_cols

    # Get SUPP domain definition for finalization
    supp_domain_code = f"SUPP{domain}"
    try:
        supp_domain_def = get_domain(supp_domain_code)
    except KeyError:
        supp_domain_def = None

    # Finalize with ordering and deduplication
    supp_df = finalize_suppqual(supp_df, supp_domain_def, domain_code)

    return supp_df, used_cols


def write_suppqual_files(
    supp_frames: list[pd.DataFrame],
    domain_code: str,
    study_id: str,
    output_format: str,
    xpt_dir: Path | None,
    xml_dir: Path | None,
) -> dict[str, Any]:
    """Generate supplemental qualifier files (XPT and/or XML).

    This function merges multiple SUPPQUAL dataframes, creates an identity
    mapping configuration, and writes the output files.

    Args:
        supp_frames: List of SUPPQUAL DataFrames to merge
        domain_code: Parent domain code (e.g., "AE", "DM")
        study_id: Study identifier
        output_format: Output format ("xpt", "xml", or "both")
        xpt_dir: Directory for XPT files (optional)
        xml_dir: Directory for XML files (optional)

    Returns:
        Dictionary with supplemental file metadata including paths and record counts
    """
    from ..mapping_module import ColumnMapping, build_config
    from ..infrastructure.io.xpt_write import write_xpt_file
    from ..infrastructure.io.dataset_xml.writer import write_dataset_xml

    # Merge SUPP dataframes
    merged_supp = (
        supp_frames[0]
        if len(supp_frames) == 1
        else pd.concat(supp_frames, ignore_index=True)
    )

    supp_domain_code = f"SUPP{domain_code.upper()}"

    # Build identity mapping config
    mappings = [
        ColumnMapping(
            source_column=col,
            target_variable=col,
            transformation=None,
            confidence_score=1.0,
        )
        for col in merged_supp.columns
    ]
    supp_config = build_config(supp_domain_code, mappings)
    supp_config.study_id = study_id

    base_filename = get_domain(supp_domain_code).resolved_dataset_name()
    disk_name = base_filename.lower()

    supp_result: dict[str, Any] = {
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
        write_xpt_file(merged_supp, supp_domain_code, xpt_path, file_label=file_label)
        supp_result["xpt_path"] = xpt_path
        supp_result["xpt_filename"] = xpt_path.name

    if xml_dir and output_format in ("xml", "both"):
        xml_path = xml_dir / f"{disk_name}.xml"
        write_dataset_xml(merged_supp, supp_domain_code, supp_config, xml_path)
        supp_result["xml_path"] = xml_path
        supp_result["xml_filename"] = xml_path.name

    return supp_result
