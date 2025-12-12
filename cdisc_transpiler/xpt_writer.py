"""Optimized XPT file writer.

This module handles writing SAS transport files with validation
and optimization for large datasets.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

import pyreadstat

if TYPE_CHECKING:
    import pandas as pd
    from .domains import SDTMDomain


class XPTWriter:
    """Optimized XPT file writer."""
    
    def __init__(self, domain: SDTMDomain):
        """Initialize writer.
        
        Args:
            domain: SDTM domain definition
        """
        self.domain = domain
        self._label_lookup = {var.name: var.label for var in domain.variables}
    
    def write(
        self,
        dataset: pd.DataFrame,
        path: str | Path,
        *,
        file_label: str | None = None,
        table_name: str | None = None,
    ) -> None:
        """Write dataset to XPT file.
        
        Args:
            dataset: Domain dataframe
            path: Output path
            file_label: Optional file label
            table_name: Optional table name
            
        Raises:
            ValueError: If filename stem > 8 characters
            IOError: If write fails
        """
        output_path = Path(path)
        
        # Validate path
        self._validate_path(output_path)
        
        # Prepare metadata
        dataset_name = (table_name or self.domain.resolved_dataset_name()).upper()[:8]
        column_labels = self._get_column_labels(dataset)
        file_label = self._get_file_label(file_label)
        
        # Ensure output directory exists
        output_path.parent.mkdir(parents=True, exist_ok=True)
        
        # Remove existing file if present
        if output_path.exists():
            output_path.unlink()
        
        # Write XPT file
        try:
            pyreadstat.write_xport(
                dataset,
                str(output_path),
                file_label=file_label,
                column_labels=column_labels,
                table_name=dataset_name,
                file_format_version=5,
            )
        except Exception as exc:
            raise IOError(f"Failed to write XPT file {output_path}: {exc}") from exc
    
    def _validate_path(self, path: Path) -> None:
        """Validate output path.
        
        Args:
            path: Output path
            
        Raises:
            ValueError: If path is invalid
        """
        # Force lowercase for consistency
        if path.name != path.name.lower():
            raise ValueError(
                f"XPT filename must be lowercase: {path.name} -> {path.name.lower()}"
            )
        
        # Check stem length (SAS v5 requirement)
        if len(path.stem) > 8:
            raise ValueError(
                f"XPT filename stem must be ≤8 characters (SAS v5): {path.name}"
            )
    
    def _get_column_labels(self, dataset: pd.DataFrame) -> list[str]:
        """Get column labels for dataset.
        
        Args:
            dataset: Domain dataframe
            
        Returns:
            List of column labels (max 40 chars each)
        """
        labels = []
        for col in dataset.columns:
            label = str(self._label_lookup.get(col, col))
            labels.append(label[:40])  # SAS v5 limit
        return labels
    
    def _get_file_label(self, label: str | None) -> str | None:
        """Get file label.
        
        Args:
            label: Optional label (None uses default, "" suppresses)
            
        Returns:
            File label or None
        """
        if label is None:
            # Use default from domain
            default = (
                self.domain.label 
                or self.domain.description 
                or self.domain.code
            ).strip()
            label = default[:40]  # SAS v5 limit
        elif label:
            label = label.strip()[:40]
        else:
            label = None
        
        return label if label else None


def write_xpt_file(
    dataset: pd.DataFrame,
    domain_code: str,
    path: str | Path,
    *,
    file_label: str | None = None,
    table_name: str | None = None,
) -> None:
    """Write dataset to XPT file (convenience function).
    
    Args:
        dataset: Domain dataframe
        domain_code: SDTM domain code
        path: Output path
        file_label: Optional file label
        table_name: Optional table name
    """
    from .domains import get_domain
    
    domain = get_domain(domain_code)
    writer = XPTWriter(domain)
    writer.write(dataset, path, file_label=file_label, table_name=table_name)
