"""Define-XML generator adapter.

This module provides an adapter implementation for generating Define-XML files.
It conforms to the DefineXMLGeneratorPort protocol.

The adapter accepts application-layer DTOs (DefineDatasetDTO) and converts
them to infrastructure-specific models (StudyDataset) before generating
the Define-XML file.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from ...application.ports import DefineXMLGeneratorPort

if TYPE_CHECKING:
    from collections.abc import Iterable

    from cdisc_transpiler.application.models import DefineDatasetDTO


class DefineXMLGenerator(DefineXMLGeneratorPort):
    """Adapter for generating Define-XML 2.1 files.

    This class implements the DefineXMLGeneratorPort protocol by accepting
    application-layer DTOs and converting them to infrastructure models
    before delegating to the infrastructure Define-XML writer.

    The adapter provides a clean boundary between the application layer
    (which knows nothing about infrastructure models) and the infrastructure
    layer (which handles the actual XML generation).

    Example:
        >>> generator = DefineXMLGenerator()
        >>> datasets = [DefineDatasetDTO(...), DefineDatasetDTO(...)]
        >>> # Prefer canonical defaults:
        >>> # sdtm_version=SDTMVersions.DEFAULT_VERSION
        >>> # context=SDTMVersions.DEFINE_CONTEXT_SUBMISSION
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
            sdtm_version: SDTM-IG version (e.g., SDTMVersions.DEFAULT_VERSION)
            context: Define-XML context (e.g., SDTMVersions.DEFINE_CONTEXT_SUBMISSION)

        Raises:
            Exception: If generation or writing fails

        Example:
            >>> generator = DefineXMLGenerator()
            >>> datasets = [DefineDatasetDTO(...)]
            >>> generator.generate(datasets, Path("define.xml"), sdtm_version="3.4", context="Submission")
        """
        # Import at runtime to avoid circular import
        from .define_xml.models import StudyDataset
        from .define_xml.xml_writer import write_study_define_file

        # Convert application DTOs to infrastructure models
        infra_datasets = [
            StudyDataset(
                domain_code=dto.domain_code,
                dataframe=dto.dataframe,
                config=dto.config,
                label=dto.label,
                structure=dto.structure,
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
