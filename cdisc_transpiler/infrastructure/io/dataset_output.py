from typing import TYPE_CHECKING

from ...application.models import DatasetOutputResult
from .dataset_xml_writer import DatasetXMLError
from .sas_writer import SASWriterError
from .xpt_writer import XportGenerationError

if TYPE_CHECKING:
    from pathlib import Path

    import pandas as pd

    from ...application.models import DatasetOutputRequest
    from ...application.ports.services import (
        DatasetXMLWriterPort,
        SASWriterPort,
        XPTWriterPort,
    )
    from ...domain.entities.mapping import MappingConfig
from ..sdtm_spec.registry import get_domain


class DatasetOutputAdapter:
    pass

    def __init__(
        self,
        xpt_writer: XPTWriterPort,
        xml_writer: DatasetXMLWriterPort,
        sas_writer: SASWriterPort,
    ) -> None:
        super().__init__()
        self._xpt_writer = xpt_writer
        self._xml_writer = xml_writer
        self._sas_writer = sas_writer

    def generate(self, request: DatasetOutputRequest) -> DatasetOutputResult:
        result = DatasetOutputResult()
        base_filename = request.base_filename
        if base_filename is None:
            domain = get_domain(request.domain_code)
            base_filename = domain.resolved_dataset_name()
        disk_name = base_filename.lower()
        if "xpt" in request.formats and request.output_dirs.xpt_dir:
            try:
                result.xpt_path = self._generate_xpt(
                    request.dataframe,
                    request.domain_code,
                    request.output_dirs.xpt_dir,
                    disk_name,
                )
            except (OSError, ValueError, XportGenerationError) as exc:
                result.errors.append(f"XPT generation failed: {exc}")
            except Exception as exc:
                result.errors.append(f"XPT generation failed: {exc}")
        if "xml" in request.formats and request.output_dirs.xml_dir:
            try:
                result.xml_path = self._generate_xml(
                    request.dataframe,
                    request.domain_code,
                    request.config,
                    request.output_dirs.xml_dir,
                    disk_name,
                )
            except (OSError, TypeError, ValueError, DatasetXMLError) as exc:
                result.errors.append(f"XML generation failed: {exc}")
            except Exception as exc:
                result.errors.append(f"XML generation failed: {exc}")
        if "sas" in request.formats and request.output_dirs.sas_dir:
            try:
                result.sas_path = self._generate_sas(request, disk_name, base_filename)
            except (OSError, TypeError, ValueError, KeyError, SASWriterError) as exc:
                result.errors.append(f"SAS generation failed: {exc}")
            except Exception as exc:
                result.errors.append(f"SAS generation failed: {exc}")
        return result

    def _generate_xpt(
        self, dataframe: pd.DataFrame, domain_code: str, xpt_dir: Path, disk_name: str
    ) -> Path:
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
        xml_path = xml_dir / f"{disk_name}.xml"
        self._xml_writer.write(dataframe, domain_code, config, xml_path)
        return xml_path

    def _generate_sas(
        self, request: DatasetOutputRequest, disk_name: str, base_filename: str
    ) -> Path:
        sas_dir = request.output_dirs.sas_dir
        if sas_dir is None:
            raise ValueError("SAS output directory is not configured")
        sas_path = sas_dir / f"{disk_name}.sas"
        input_dataset = request.input_dataset
        output_dataset = request.output_dataset
        if input_dataset is None:
            input_dataset = f"work.{request.domain_code.lower()}"
        if output_dataset is None:
            output_dataset = f"sdtm.{base_filename}"
        self._sas_writer.write(
            request.domain_code,
            request.config,
            sas_path,
            input_dataset=input_dataset,
            output_dataset=output_dataset,
        )
        return sas_path
