"""XPT file writing functionality.

This module provides clean, validated XPT (SAS transport v5) file writing
using pyreadstat. It handles path validation, metadata generation, and
proper SDTM v5 compliance.
"""

from pathlib import Path

import pandas as pd
import pyreadstat

from ..domains_module import get_domain


class XportGenerationError(RuntimeError):
    """Raised when XPT export cannot be completed."""


def write_xpt_file(
    dataset: pd.DataFrame,
    domain_code: str,
    path: str | Path,
    *,
    file_label: str | None = None,
    table_name: str | None = None,
) -> None:
    """Persist the DataFrame as a SAS v5 transport file.
    
    Args:
        dataset: DataFrame to write
        domain_code: SDTM domain code (e.g., "DM", "AE")
        path: Output file path
        file_label: Optional file label (default: domain label, max 40 chars)
        table_name: Optional table name (default: domain dataset name, max 8 chars)
        
    Raises:
        XportGenerationError: If file cannot be written or validation fails
        
    Example:
        >>> write_xpt_file(dm_df, "DM", "output/xpt/dm.xpt")
        >>> write_xpt_file(ae_df, "AE", "output/xpt/ae.xpt", file_label="Adverse Events")
    """
    output_path = Path(path)
    
    # Force lower-case disk names to match MSG sample package convention
    output_path = output_path.with_name(output_path.name.lower())
    
    # Validate filename length (SDTM v5 requirement)
    if len(output_path.stem) > 8:
        raise XportGenerationError(
            f"XPT filename stem must be <=8 characters to satisfy SDTM v5: {output_path.name}"
        )
    
    # Create parent directory if needed
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    # Remove pre-existing file (including case-insensitive collisions)
    if output_path.exists():
        output_path.unlink()
    
    # Get domain metadata for labels
    domain = get_domain(domain_code)
    dataset_name = (table_name or domain.resolved_dataset_name()).upper()[:8]
    
    # Generate column labels from domain variables (max 40 chars per label)
    label_lookup = {variable.name: variable.label for variable in domain.variables}
    column_labels = [str(label_lookup.get(col, col))[:40] for col in dataset.columns]
    
    # Generate file label (max 40 chars)
    default_label = (domain.label or domain.description or dataset_name).strip()
    # Allow caller to suppress label by passing an empty string
    label = default_label if file_label is None else file_label
    label = (label or "").strip()[:40] or None  # v5 metadata cap
    
    # Write XPT file using pyreadstat
    try:
        pyreadstat.write_xport(
            dataset,
            str(output_path),
            file_label=label,
            column_labels=column_labels,
            table_name=dataset_name,
            file_format_version=5,
        )
    except Exception as exc:  # pragma: no cover - pyreadstat error surface
        raise XportGenerationError(f"Failed to write XPT file: {exc}") from exc
