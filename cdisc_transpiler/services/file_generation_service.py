"""File generation service.

This service coordinates the generation of various output file formats:
- XPT (SAS transport files)
- Dataset-XML
- Define-XML
- SAS programs
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import pandas as pd
    from ..mapping import MappingConfig

from ..xpt_module import write_xpt_file
from ..sas import generate_sas_program, write_sas_file
from ..domains import get_domain


@dataclass
class FileGenerationResult:
    """Result of file generation."""

    xpt_path: Path | None = None
    xml_path: Path | None = None
    sas_path: Path | None = None


class FileGenerationService:
    """Service for generating output files in various formats."""

    def __init__(
        self,
        output_dir: Path,
        *,
        generate_xpt: bool = True,
        generate_xml: bool = False,
        generate_sas: bool = True,
        streaming: bool = False,
        chunk_size: int = 1000,
    ):
        """Initialize the file generation service.

        Args:
            output_dir: Base output directory
            generate_xpt: Whether to generate XPT files
            generate_xml: Whether to generate Dataset-XML files
            generate_sas: Whether to generate SAS programs
            streaming: Use streaming mode for Dataset-XML
            chunk_size: Chunk size for streaming
        """
        self.output_dir = output_dir
        self.generate_xpt = generate_xpt
        self.generate_xml = generate_xml
        self.generate_sas = generate_sas
        self.streaming = streaming
        self.chunk_size = chunk_size

        # Create output directories
        if generate_xpt:
            self.xpt_dir = output_dir / "xpt"
            self.xpt_dir.mkdir(parents=True, exist_ok=True)
        else:
            self.xpt_dir = None

        if generate_xml:
            self.xml_dir = output_dir / "dataset-xml"
            self.xml_dir.mkdir(parents=True, exist_ok=True)
        else:
            self.xml_dir = None

        if generate_sas:
            self.sas_dir = output_dir / "sas"
            self.sas_dir.mkdir(parents=True, exist_ok=True)
        else:
            self.sas_dir = None

    def generate_files(
        self,
        domain_code: str,
        dataframe: pd.DataFrame,
        config: MappingConfig,
        *,
        file_label: str | None = None,
        table_name: str | None = None,
    ) -> FileGenerationResult:
        """Generate output files for a domain.

        Args:
            domain_code: SDTM domain code
            dataframe: Domain dataframe
            config: Mapping configuration
            file_label: Optional file label for XPT
            table_name: Optional table name for XPT

        Returns:
            FileGenerationResult with paths to generated files
        """
        domain = get_domain(domain_code)
        base_name = domain.resolved_dataset_name().lower()

        result = FileGenerationResult()

        # Generate XPT file
        if self.generate_xpt and self.xpt_dir:
            xpt_path = self.xpt_dir / f"{base_name}.xpt"
            write_xpt_file(
                dataframe,
                domain_code,
                xpt_path,
                file_label=file_label,
                table_name=table_name,
            )
            result.xpt_path = xpt_path

        # Generate Dataset-XML file
        if self.generate_xml and self.xml_dir:
            xml_path = self.xml_dir / f"{base_name}.xml"
            self._generate_dataset_xml(dataframe, domain_code, config, xml_path)
            result.xml_path = xml_path

        # Generate SAS program
        if self.generate_sas and self.sas_dir:
            sas_path = self.sas_dir / f"{base_name}.sas"
            self._generate_sas_program(domain_code, config, base_name, sas_path)
            result.sas_path = sas_path

        return result

    def _generate_dataset_xml(
        self,
        dataframe: pd.DataFrame,
        domain_code: str,
        config: MappingConfig,
        output_path: Path,
    ) -> None:
        """Generate Dataset-XML file.

        Args:
            dataframe: Domain dataframe
            domain_code: Domain code
            config: Mapping configuration
            output_path: Output file path
        """
        from ..xml.dataset import write_dataset_xml, generate_dataset_xml_streaming

        if self.streaming:
            generate_dataset_xml_streaming(
                dataframe,
                domain_code,
                config,
                output_path,
                chunk_size=self.chunk_size,
            )
        else:
            write_dataset_xml(dataframe, domain_code, config, output_path)

    def _generate_sas_program(
        self,
        domain_code: str,
        config: MappingConfig,
        base_name: str,
        output_path: Path,
    ) -> None:
        """Generate SAS program.

        Args:
            domain_code: Domain code
            config: Mapping configuration
            base_name: Base name for datasets
            output_path: Output file path
        """
        sas_code = generate_sas_program(
            domain_code,
            config,
            input_dataset=f"work.{base_name}",
            output_dataset=f"sdtm.{base_name.upper()}",
        )
        write_sas_file(sas_code, output_path)

    def write_split_xpt(
        self,
        dataframe: pd.DataFrame,
        domain_code: str,
        split_name: str,
        *,
        table_name: str | None = None,
    ) -> Path | None:
        """Write a split XPT file following SDTMIG v3.4 Section 4.1.7.

        According to SDTMIG v3.4 Section 4.1.7 "Splitting Domains":
        - Split datasets follow naming pattern: [DOMAIN][SPLIT]
        - All splits maintain the same DOMAIN variable value
        - Dataset names must be ≤ 8 characters
        
        Args:
            dataframe: Split dataframe
            domain_code: Domain code (e.g., "LB", "VS", "EG")
            split_name: Name for the split file (e.g., "lbhm", "lbcc")
            table_name: Optional table name (defaults to split_name.upper())

        Returns:
            Path to generated file or None
        """
        if not self.generate_xpt or not self.xpt_dir:
            return None

        split_dir = self.xpt_dir / "split"
        split_dir.mkdir(parents=True, exist_ok=True)

        # Validate and clean split name
        clean_split = split_name.upper().replace("_", "").replace(" ", "")
        
        # Ensure split name starts with domain code
        if not clean_split.startswith(domain_code.upper()):
            clean_split = f"{domain_code.upper()}{clean_split}"
        
        # Ensure ≤ 8 characters per SDTMIG v3.4
        if len(clean_split) > 8:
            clean_split = clean_split[:8]
        
        split_path = split_dir / f"{clean_split.lower()}.xpt"
        
        write_xpt_file(
            dataframe,
            domain_code,
            split_path,
            table_name=table_name or clean_split,
            file_label="",
        )

        return split_path
