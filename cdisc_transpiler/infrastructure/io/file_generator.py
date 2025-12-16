"""Unified file generator for consistent output across formats.

This module consolidates file generation logic that was duplicated
across multiple services (domain_processing_coordinator,
domain_synthesis_coordinator, study_orchestration_service).

Key Features:
- Single source of truth for XPT/XML/SAS generation
- Consistent error handling and logging
- Configurable output via OutputRequest/OutputResult DTOs
- No duplicate code
- Dependency injection of writer adapters
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

import pandas as pd

from ...application.models import OutputRequest, OutputResult

if TYPE_CHECKING:
    from ...application.ports import (
        DatasetXMLWriterPort,
        SASWriterPort,
        XPTWriterPort,
    )
    from ...domain.entities.mapping import MappingConfig

# Import domain helper (still needed for resolving dataset names)
from ...domains_module import get_domain


class FileGenerator:
    """Centralized file generation for all formats.

    This class replaces duplicated file generation logic across:
    - domain_processing_coordinator.py (3 copies of similar logic)
    - domain_synthesis_coordinator.py (3 copies)
    - study_orchestration_service.py (2 copies)

    The FileGenerator now accepts writer adapters via dependency injection,
    following the Ports & Adapters architecture pattern. This allows for
    flexible writer implementations and better testability.

    Example:
        >>> from cdisc_transpiler.infrastructure.io import (
        ...     FileGenerator, OutputRequest, OutputDirs,
        ...     XPTWriter, DatasetXMLWriter, SASWriter
        ... )
        >>>
        >>> generator = FileGenerator(
        ...     xpt_writer=XPTWriter(),
        ...     xml_writer=DatasetXMLWriter(),
        ...     sas_writer=SASWriter(),
        ... )
        >>> request = OutputRequest(
        ...     dataframe=dm_df,
        ...     domain_code="DM",
        ...     config=config,
        ...     output_dirs=OutputDirs(xpt_dir=Path("output/xpt")),
        ...     formats={"xpt"},
        ... )
        >>> result = generator.generate(request)
        >>> if result.success:
        ...     print(f"Generated: {result.xpt_path}")
    """

    def __init__(
        self,
        xpt_writer: XPTWriterPort,
        xml_writer: DatasetXMLWriterPort,
        sas_writer: SASWriterPort,
    ):
        """Initialize the FileGenerator with writer adapters.

        Args:
            xpt_writer: Adapter for writing XPT files
            xml_writer: Adapter for writing Dataset-XML files
            sas_writer: Adapter for writing SAS programs

        Example:
            >>> from cdisc_transpiler.infrastructure.io import (
            ...     FileGenerator, XPTWriter, DatasetXMLWriter, SASWriter
            ... )
            >>> generator = FileGenerator(
            ...     xpt_writer=XPTWriter(),
            ...     xml_writer=DatasetXMLWriter(),
            ...     sas_writer=SASWriter(),
            ... )
        """
        self._xpt_writer = xpt_writer
        self._xml_writer = xml_writer
        self._sas_writer = sas_writer

    def generate(self, request: OutputRequest) -> OutputResult:
        """Generate all requested output files.

        Args:
            request: Output generation request with dataframe and configuration

        Returns:
            OutputResult with paths to generated files and any errors
        """
        result = OutputResult()

        # Determine base filename
        base_filename = request.base_filename
        if base_filename is None:
            domain = get_domain(request.domain_code)
            base_filename = domain.resolved_dataset_name()
        disk_name = base_filename.lower()

        # Generate XPT file
        if "xpt" in request.formats and request.output_dirs.xpt_dir:
            try:
                result.xpt_path = self._generate_xpt(
                    request.dataframe,
                    request.domain_code,
                    request.output_dirs.xpt_dir,
                    disk_name,
                )
            except Exception as e:
                result.errors.append(f"XPT generation failed: {e}")

        # Generate Dataset-XML file
        if "xml" in request.formats and request.output_dirs.xml_dir:
            try:
                result.xml_path = self._generate_xml(
                    request.dataframe,
                    request.domain_code,
                    request.config,
                    request.output_dirs.xml_dir,
                    disk_name,
                )
            except Exception as e:
                result.errors.append(f"XML generation failed: {e}")

        # Generate SAS program
        if "sas" in request.formats and request.output_dirs.sas_dir:
            try:
                result.sas_path = self._generate_sas(
                    request.domain_code,
                    request.config,
                    request.output_dirs.sas_dir,
                    disk_name,
                    base_filename,
                    request.input_dataset,
                    request.output_dataset,
                )
            except Exception as e:
                result.errors.append(f"SAS generation failed: {e}")

        return result

    def _generate_xpt(
        self,
        dataframe: pd.DataFrame,
        domain_code: str,
        xpt_dir: Path,
        disk_name: str,
    ) -> Path:
        """Generate XPT file using the injected XPT writer.

        Args:
            dataframe: Data to write
            domain_code: Domain code
            xpt_dir: Output directory
            disk_name: Base filename (without extension)

        Returns:
            Path to generated XPT file
        """
        xpt_path = xpt_dir / f"{disk_name}.xpt"
        self._xpt_writer.write(dataframe, domain_code, xpt_path)
        return xpt_path

    def _generate_xml(
        self,
        dataframe: pd.DataFrame,
        domain_code: str,
        config: MappingConfig,
        xml_dir: Path,
        disk_name: str,
    ) -> Path:
        """Generate Dataset-XML file using the injected XML writer.

        Args:
            dataframe: Data to write
            domain_code: Domain code
            config: Mapping configuration
            xml_dir: Output directory
            disk_name: Base filename (without extension)

        Returns:
            Path to generated XML file
        """
        xml_path = xml_dir / f"{disk_name}.xml"
        self._xml_writer.write(dataframe, domain_code, config, xml_path)
        return xml_path

    def _generate_sas(
        self,
        domain_code: str,
        config: MappingConfig,
        sas_dir: Path,
        disk_name: str,
        base_filename: str,
        input_dataset: str | None,
        output_dataset: str | None,
    ) -> Path:
        """Generate SAS program using the injected SAS writer.

        Args:
            domain_code: Domain code
            config: Mapping configuration
            sas_dir: Output directory
            disk_name: Base filename (without extension)
            base_filename: Base filename for output dataset
            input_dataset: Input dataset name (e.g., "work.dm")
            output_dataset: Output dataset name (e.g., "sdtm.dm")

        Returns:
            Path to generated SAS file
        """
        sas_path = sas_dir / f"{disk_name}.sas"

        # Determine input/output dataset names
        if input_dataset is None:
            input_dataset = f"work.{domain_code.lower()}"
        if output_dataset is None:
            output_dataset = f"sdtm.{base_filename}"

        # Use the injected SAS writer
        self._sas_writer.write(
            domain_code,
            config,
            sas_path,
            input_dataset=input_dataset,
            output_dataset=output_dataset,
        )

        return sas_path
