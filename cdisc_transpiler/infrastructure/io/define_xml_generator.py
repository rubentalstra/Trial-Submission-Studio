"""Define-XML generator adapter.

This module provides an adapter implementation for generating Define-XML files.
It wraps the existing xml_module.define_module functionality while conforming
to the DefineXmlGeneratorPort protocol.

The adapter accepts application-layer DTOs (DefineDatasetDTO) and converts
them to infrastructure-specific models (StudyDataset) before generating
the Define-XML file.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from typing import Iterable
    from cdisc_transpiler.application.models import DefineDatasetDTO


class DefineXmlGenerator:
    """Adapter for generating Define-XML 2.1 files.

    This class implements the DefineXmlGeneratorPort protocol by accepting
    application-layer DTOs and converting them to infrastructure models
    before delegating to the xml_module.define_module for XML generation.

    The adapter provides a clean boundary between the application layer
    (which knows nothing about infrastructure models) and the infrastructure
    layer (which handles the actual XML generation).

    Example:
        >>> generator = DefineXmlGenerator()
        >>> datasets = [DefineDatasetDTO(...), DefineDatasetDTO(...)]
        >>> generator.generate(datasets, Path("define.xml"), sdtm_version="3.4", context="Submission")
    """

    def generate(
        self,
        datasets: Iterable[DefineDatasetDTO],
        output_path: Path,
        *,
        sdtm_version: str,
        context: str,
    ) -> None:
        """Generate a Define-XML 2.1 file for the given study datasets.

        This method converts application-layer DTOs to infrastructure models
        and delegates to the XML generation module.

        Args:
            datasets: Iterable of DefineDatasetDTO objects (application-layer)
            output_path: Path where Define-XML file should be written
            sdtm_version: SDTM-IG version (e.g., "3.4")
            context: Define-XML context - 'Submission' or 'Other'

        Raises:
            Exception: If generation or writing fails (propagated from xml_module)

        Example:
            >>> generator = DefineXmlGenerator()
            >>> datasets = [DefineDatasetDTO(...)]
            >>> generator.generate(datasets, Path("define.xml"), sdtm_version="3.4", context="Submission")
        """
        # Import at runtime to avoid circular import
        from .define_xml.xml_writer import write_study_define_file
        from .define_xml.models import StudyDataset

        # Convert application DTOs to infrastructure models
        infra_datasets = [
            StudyDataset(
                domain_code=dto.domain_code,
                dataframe=dto.dataframe,
                config=dto.config,
                label=dto.label,
                structure=dto.structure,
                is_split=dto.is_split,
                split_suffix=dto.split_suffix,
                archive_location=dto.archive_location,
            )
            for dto in datasets
        ]

        write_study_define_file(
            infra_datasets,
            output_path,
            sdtm_version=sdtm_version,
            context=context,
        )
