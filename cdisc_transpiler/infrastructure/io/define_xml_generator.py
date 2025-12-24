from typing import TYPE_CHECKING, override

from ...application.ports.services import DefineXMLGeneratorPort
from .define_xml.models import StudyDataset
from .define_xml.xml_writer import write_study_define_file

if TYPE_CHECKING:
    from collections.abc import Iterable
    from pathlib import Path

    from cdisc_transpiler.application.models import DefineDatasetDTO


class DefineXMLGenerator(DefineXMLGeneratorPort):
    pass

    @override
    def generate(
        self,
        datasets: Iterable[DefineDatasetDTO],
        output_path: Path,
        *,
        sdtm_version: str,
        context: str,
    ) -> None:
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
            infra_datasets, output_path, sdtm_version=sdtm_version, context=context
        )
