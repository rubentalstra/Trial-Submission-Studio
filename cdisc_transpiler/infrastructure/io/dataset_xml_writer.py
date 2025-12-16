"""Dataset-XML writer adapter + implementation.

This module intentionally contains the full Dataset-XML implementation to keep
the I/O layer simpler and reduce file count.
"""

from __future__ import annotations

from datetime import UTC, datetime
from pathlib import Path
from typing import TYPE_CHECKING, Any, cast
from xml.etree import ElementTree as ET

import pandas as pd

from cdisc_transpiler.constants import Constraints, SDTMVersions
from cdisc_transpiler.infrastructure.sdtm_spec.registry import get_domain

if TYPE_CHECKING:
    from cdisc_transpiler.domain.entities.mapping import MappingConfig


# XML Namespaces for Dataset-XML 1.0
ODM_NS = "http://www.cdisc.org/ns/odm/v1.3"
DATA_NS = "http://www.cdisc.org/ns/Dataset-XML/v1.0"
XLINK_NS = "http://www.w3.org/1999/xlink"

# Register namespaces for output
ET.register_namespace("", ODM_NS)
ET.register_namespace("xlink", XLINK_NS)
ET.register_namespace("data", DATA_NS)

DATASET_XML_VERSION = Constraints.DATASET_XML_VERSION
DEFINE_XML_VERSION = Constraints.DEFINE_XML_VERSION
DEFAULT_SDTM_VERSION = SDTMVersions.DEFAULT_VERSION


class DatasetXMLError(RuntimeError):
    """Raised when Dataset-XML generation or writing fails."""


def tag(namespace: str, name: str) -> str:
    return f"{{{namespace}}}{name}"


def attr(namespace: str, name: str) -> str:
    return f"{{{namespace}}}{name}"


def _generate_item_oid(variable_name: str, dataset_name: str) -> str:
    return f"IT.{dataset_name.upper()}.{variable_name.upper()}"


def _is_null(value: object) -> bool:
    if value is None:
        return True
    if isinstance(value, float) and pd.isna(value):
        return True
    if isinstance(value, str) and value.strip() == "":
        return True
    return False


def _format_value(value: object) -> str:
    if isinstance(value, (pd.Series, pd.DataFrame)):
        try:
            value = value.iloc[0]  # type: ignore[index]
        except (AttributeError, IndexError, KeyError, TypeError):
            return ""
    try:
        if bool(pd.isna(cast(Any, value))):
            return ""
    except (TypeError, ValueError):
        pass

    if isinstance(value, (int, float)):
        if isinstance(value, float):
            return format(value, ".15g")
        return str(value)
    return str(value).strip()


def _build_dataset_xml_tree(
    data: pd.DataFrame,
    domain_code: str,
    config: MappingConfig,
    *,
    dataset_name: str | None = None,
    metadata_version_oid: str | None = None,
    is_reference_data: bool = False,
) -> ET.Element:
    domain = get_domain(domain_code)
    study_id = (config.study_id or "STUDY").strip() or "STUDY"
    study_oid = f"STDY.{study_id}"
    dataset_name = (
        dataset_name or domain.resolved_dataset_name()
    ).strip() or domain.code
    timestamp = datetime.now(UTC).isoformat(timespec="seconds")

    mdv_oid = metadata_version_oid or f"MDV.{study_oid}.SDTMIG.{DEFAULT_SDTM_VERSION}"
    define_file_oid = f"{study_oid}.Define-XML_{DEFINE_XML_VERSION}"

    root = ET.Element(
        tag(ODM_NS, "ODM"),
        attrib={
            "FileType": "Snapshot",
            "FileOID": f"{define_file_oid}(IG.{dataset_name})",
            "PriorFileOID": define_file_oid,
            "ODMVersion": "1.3.2",
            "CreationDateTime": timestamp,
            "Originator": "CDISC-Transpiler",
        },
    )
    root.set("xmlns:xlink", XLINK_NS)
    root.set(attr(DATA_NS, "DatasetXMLVersion"), DATASET_XML_VERSION)

    container_tag_name = "ReferenceData" if is_reference_data else "ClinicalData"
    container = ET.SubElement(
        root,
        tag(ODM_NS, container_tag_name),
        attrib={"StudyOID": study_oid, "MetaDataVersionOID": mdv_oid},
    )

    for seq, (_, row) in enumerate(data.iterrows(), start=1):
        item_group_data = ET.SubElement(
            container,
            tag(ODM_NS, "ItemGroupData"),
            attrib={"ItemGroupOID": f"IG.{dataset_name}"},
        )
        item_group_data.set(attr(DATA_NS, "ItemGroupDataSeq"), str(seq))

        for col_name in data.columns:
            value = row[col_name]
            if _is_null(value):
                continue
            ET.SubElement(
                item_group_data,
                tag(ODM_NS, "ItemData"),
                attrib={
                    "ItemOID": _generate_item_oid(col_name, dataset_name),
                    "Value": _format_value(value),
                },
            )

    return root


def write_dataset_xml(
    data: pd.DataFrame,
    domain_code: str,
    config: MappingConfig,
    output: Path,
    *,
    dataset_name: str | None = None,
    metadata_version_oid: str | None = None,
    is_reference_data: bool | None = None,
) -> None:
    """Write a Dataset-XML 1.0 file for a single domain."""
    try:
        domain = get_domain(domain_code)

        class_name = (domain.class_name or "").replace("-", " ").strip().upper()
        if is_reference_data is None:
            is_reference_data = class_name in ("TRIAL DESIGN", "STUDY REFERENCE")

        root = _build_dataset_xml_tree(
            data,
            domain_code,
            config,
            dataset_name=dataset_name,
            metadata_version_oid=metadata_version_oid,
            is_reference_data=is_reference_data,
        )
        tree = ET.ElementTree(root)
        output.parent.mkdir(parents=True, exist_ok=True)
        tree.write(output, xml_declaration=True, encoding="utf-8")
    except (OSError, TypeError, ValueError) as exc:
        raise DatasetXMLError(f"Failed to write Dataset-XML: {exc}") from exc


class DatasetXMLWriter:
    """Adapter for writing Dataset-XML files.

    This class implements the DatasetXMLWriterPort protocol and delegates to
    the concrete infrastructure writer in `infrastructure.io.dataset_xml`.

    Example:
        >>> writer = DatasetXMLWriter()
        >>> df = pd.DataFrame({"STUDYID": ["001"], "USUBJID": ["001-001"]})
        >>> writer.write(df, "DM", config, Path("output/dm.xml"))
    """

    def write(
        self,
        dataframe: pd.DataFrame,
        domain_code: str,
        config: MappingConfig,
        output_path: Path,
    ) -> None:
        """Write a DataFrame to a Dataset-XML file.

        Args:
            dataframe: Data to write
            domain_code: SDTM domain code (e.g., "DM", "AE")
            config: Mapping configuration with column metadata
            output_path: Path where XML file should be written

        Raises:
            Exception: If writing fails

        Example:
            >>> writer = DatasetXMLWriter()
            >>> df = pd.DataFrame({"STUDYID": ["001"]})
            >>> writer.write(df, "DM", config, Path("dm.xml"))
        """
        write_dataset_xml(dataframe, domain_code, config, output_path)
