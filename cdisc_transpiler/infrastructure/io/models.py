"""Models for file generation."""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import pandas as pd
    from ..mapping_module import MappingConfig


@dataclass
class OutputDirs:
    """Output directory configuration.
    
    Attributes:
        xpt_dir: Directory for XPT files (None to skip)
        xml_dir: Directory for Dataset-XML files (None to skip)
        sas_dir: Directory for SAS programs (None to skip)
    """
    
    xpt_dir: Path | None = None
    xml_dir: Path | None = None
    sas_dir: Path | None = None


@dataclass
class OutputRequest:
    """Request for file generation.
    
    Attributes:
        dataframe: DataFrame to write
        domain_code: SDTM domain code (e.g., "DM", "AE")
        config: Mapping configuration for the domain
        output_dirs: Directory configuration
        formats: Set of formats to generate ({"xpt", "xml", "sas"})
        base_filename: Base filename (defaults to lowercase domain code)
        input_dataset: Input dataset name for SAS (e.g., "work.dm")
        output_dataset: Output dataset name for SAS (e.g., "sdtm.dm")
    """
    
    dataframe: pd.DataFrame
    domain_code: str
    config: MappingConfig
    output_dirs: OutputDirs
    formats: set[str]
    base_filename: str | None = None
    input_dataset: str | None = None
    output_dataset: str | None = None


@dataclass
class OutputResult:
    """Result of file generation.
    
    Attributes:
        xpt_path: Path to generated XPT file (None if not generated)
        xml_path: Path to generated Dataset-XML file (None if not generated)
        sas_path: Path to generated SAS program (None if not generated)
        errors: List of error messages encountered
    """
    
    xpt_path: Path | None = None
    xml_path: Path | None = None
    sas_path: Path | None = None
    errors: list[str] = field(default_factory=list)
    
    @property
    def success(self) -> bool:
        """Check if generation was successful (no errors)."""
        return len(self.errors) == 0
