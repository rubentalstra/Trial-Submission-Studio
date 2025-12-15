"""Builder for Dataset-XML 1.0 document trees.

This module handles the construction of Dataset-XML document structures.
"""

from __future__ import annotations

from datetime import UTC, datetime
from typing import TYPE_CHECKING
from xml.etree import ElementTree as ET

if TYPE_CHECKING:
    import pandas as pd
    from cdisc_transpiler.mapping_module import MappingConfig

from cdisc_transpiler.domains_module import SDTMDomain, get_domain
from .constants import (
    ODM_NS,
    DATA_NS,
    XLINK_NS,
    DATASET_XML_VERSION,
    DEFINE_XML_VERSION,
    DEFAULT_SDTM_VERSION,
)
from .utils import tag, attr, generate_item_oid, is_null, format_value


def build_dataset_xml_tree(
    data: pd.DataFrame,
    domain_code: str,
    config: MappingConfig,
    *,
    metadata_version_oid: str | None = None,
    is_reference_data: bool = False,
) -> ET.Element:
    """Build a Dataset-XML 1.0 document tree for a single domain.

    Args:
        data: The pandas DataFrame containing the domain data
        domain_code: The SDTM domain code (e.g., "DM", "AE")
        config: The mapping configuration containing study metadata
        metadata_version_oid: The MetaDataVersionOID to reference Define-XML
        is_reference_data: Whether this is reference data (trial design)

    Returns:
        The root Element of the Dataset-XML 1.0 document
    """
    domain = get_domain(domain_code)
    study_id = (config.study_id or "STUDY").strip() or "STUDY"
    study_oid = f"STDY.{study_id}"
    dataset_name = domain.resolved_dataset_name()
    timestamp = datetime.now(UTC).isoformat(timespec="seconds")

    # Determine MetaDataVersionOID
    mdv_oid = metadata_version_oid or f"MDV.{study_oid}.SDTMIG.{DEFAULT_SDTM_VERSION}"

    # Define-XML FileOID for PriorFileOID reference
    define_file_oid = f"{study_oid}.Define-XML_{DEFINE_XML_VERSION}"

    # Build ODM root element
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

    # Choose container based on data type
    container_tag_name = "ReferenceData" if is_reference_data else "ClinicalData"
    container = ET.SubElement(
        root,
        tag(ODM_NS, container_tag_name),
        attrib={
            "StudyOID": study_oid,
            "MetaDataVersionOID": mdv_oid,
        },
    )

    # Add ItemGroupData elements
    append_item_group_data(container, data, domain, dataset_name)

    return root


def append_item_group_data(
    parent: ET.Element,
    data: pd.DataFrame,
    domain: SDTMDomain,
    dataset_name: str,
) -> None:
    """Append ItemGroupData elements for each row in the DataFrame.

    Args:
        parent: Parent XML element
        data: DataFrame with domain data
        domain: SDTM domain definition
        dataset_name: Name of the dataset
    """
    for seq, (_, row) in enumerate(data.iterrows(), start=1):
        item_group_data = ET.SubElement(
            parent,
            tag(ODM_NS, "ItemGroupData"),
            attrib={"ItemGroupOID": f"IG.{dataset_name}"},
        )
        item_group_data.set(attr(DATA_NS, "ItemGroupDataSeq"), str(seq))

        # Add ItemData for each column
        for col_name in data.columns:
            value = row[col_name]

            if is_null(value):
                continue

            formatted_value = format_value(value, col_name)
            item_oid = generate_item_oid(col_name, domain.code)

            ET.SubElement(
                item_group_data,
                tag(ODM_NS, "ItemData"),
                attrib={
                    "ItemOID": item_oid,
                    "Value": formatted_value,
                },
            )
