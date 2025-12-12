"""Dataset-XML 1.0 generation module.

This module generates Dataset-XML 1.0 compliant files following the
CDISC Dataset-XML 1.0 specification for SDTM data exchange.

Dataset-XML 1.0 Key Features:
- Uses ODM 1.3.2 as the base with Dataset-XML extensions
- data:DatasetXMLVersion attribute on ODM root (required)
- ClinicalData container for subject-level data
- ReferenceData container for trial design data
- ItemGroupData elements with data:ItemGroupDataSeq attribute
- ItemData elements with ItemOID and Value attributes
- Supports streaming/chunked output for large datasets

Namespace definitions:
- ODM: http://www.cdisc.org/ns/odm/v1.3
- data: http://www.cdisc.org/ns/Dataset-XML/v1.0

Reference:
- CDISC Dataset-XML 1.0 specification
- docs/Dataset-XML_1-0/Example/ for examples
"""

from __future__ import annotations

import xml.etree.ElementTree as ET
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import TYPE_CHECKING, Iterator

if TYPE_CHECKING:
    import pandas as pd

    from cdisc_transpiler.mapping import MappingConfig

from cdisc_transpiler.domains import SDTMDomain, get_domain

# XML Namespaces for Dataset-XML 1.0
ODM_NS = "http://www.cdisc.org/ns/odm/v1.3"
DATA_NS = "http://www.cdisc.org/ns/Dataset-XML/v1.0"
XLINK_NS = "http://www.w3.org/1999/xlink"

# Dataset-XML version
DATASET_XML_VERSION = "1.0.0"

# Define-XML version for PriorFileOID reference
DEFINE_XML_VERSION = "2.1.0"
DEFAULT_SDTM_VERSION = "3.4"

# Shared variables that use IT.{VARIABLE} OID pattern (no domain prefix)
# RDOMAIN follows the shared pattern per CDISC examples (see SUPP-- datasets).
SHARED_VARIABLE_OIDS = {"STUDYID", "USUBJID", "RDOMAIN"}


class DatasetXMLError(Exception):
    """Error during Dataset-XML generation."""


@dataclass
class DatasetXMLConfig:
    """Configuration for Dataset-XML generation."""

    study_oid: str
    metadata_version_oid: str
    originator: str = "CDISC-Transpiler"
    source_system: str = "CDISC-Transpiler"
    source_system_version: str = "1.0"
    file_type: str = "Snapshot"
    granularity: str = "All"  # "All", "Metadata", "AdminData", "ReferenceData", "AllClinicalData", "SingleSite"


def _tag(namespace: str, name: str) -> str:
    """Return a fully qualified XML tag."""
    return f"{{{namespace}}}{name}"


def _attr(namespace: str, name: str) -> str:
    """Return a fully qualified XML attribute."""
    return f"{{{namespace}}}{name}"


def _register_namespaces() -> None:
    """Register XML namespaces for pretty output."""
    ET.register_namespace("", ODM_NS)
    ET.register_namespace("xlink", XLINK_NS)
    ET.register_namespace("data", DATA_NS)


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
        data: The pandas DataFrame containing the domain data.
        domain_code: The SDTM domain code (e.g., "DM", "AE").
        config: The mapping configuration containing study metadata.
        metadata_version_oid: The MetaDataVersionOID to reference Define-XML.
        is_reference_data: Whether this is reference data (trial design).

    Returns:
        The root Element of the Dataset-XML 1.0 document.

    Per CDISC Dataset-XML 1.0 standard:
    - FileOID references the ItemGroupDef in the Define-XML
    - PriorFileOID references the Define-XML document
    - StudyOID matches the Study/@OID in Define-XML
    - MetaDataVersionOID matches the MetaDataVersion/@OID in Define-XML
    """
    _register_namespaces()

    domain = get_domain(domain_code)
    study_id = (config.study_id or "STUDY").strip() or "STUDY"
    study_oid = f"STDY.{study_id}"
    dataset_name = domain.resolved_dataset_name()
    timestamp = datetime.now(UTC).isoformat(timespec="seconds")

    # Determine MetaDataVersionOID - must match Define-XML MetaDataVersion/@OID
    mdv_oid = metadata_version_oid or f"MDV.{study_oid}.SDTMIG.{DEFAULT_SDTM_VERSION}"

    # Define-XML FileOID for PriorFileOID reference
    define_file_oid = f"{study_oid}.Define-XML_{DEFINE_XML_VERSION}"

    # Build ODM root element per Dataset-XML 1.0 spec
    root = ET.Element(
        _tag(ODM_NS, "ODM"),
        attrib={
            "FileType": "Snapshot",
            # FileOID identifies this dataset file and references ItemGroupDef
            "FileOID": f"{define_file_oid}(IG.{dataset_name})",
            # PriorFileOID references the Define-XML document
            "PriorFileOID": define_file_oid,
            "ODMVersion": "1.3.2",
            "CreationDateTime": timestamp,
            "Originator": "CDISC-Transpiler",
        },
    )
    # Explicitly declare xlink namespace for completeness (matches CDISC examples)
    root.set("xmlns:xlink", XLINK_NS)
    # data:DatasetXMLVersion is REQUIRED in Dataset-XML 1.0
    root.set(_attr(DATA_NS, "DatasetXMLVersion"), DATASET_XML_VERSION)

    # Choose container based on data type
    if is_reference_data:
        # ReferenceData for trial design domains (TA, TE, TV, TI, TS, SE, SV)
        container = ET.SubElement(
            root,
            _tag(ODM_NS, "ReferenceData"),
            attrib={
                # StudyOID must match Study/@OID in Define-XML (without STDY. prefix per standard examples)
                "StudyOID": study_oid,
                "MetaDataVersionOID": mdv_oid,
            },
        )
    else:
        # ClinicalData for subject-level domains
        container = ET.SubElement(
            root,
            _tag(ODM_NS, "ClinicalData"),
            attrib={
                # StudyOID must match Study/@OID in Define-XML (without STDY. prefix per standard examples)
                "StudyOID": study_oid,
                "MetaDataVersionOID": mdv_oid,
            },
        )

    # Add ItemGroupData elements
    _append_item_group_data(container, data, domain, dataset_name)

    return root


def _append_item_group_data(
    parent: ET.Element,
    data: pd.DataFrame,
    domain: SDTMDomain,
    dataset_name: str,
) -> None:
    """Append ItemGroupData elements for each row in the DataFrame.

    Per Dataset-XML 1.0:
    - ItemGroupData has data:ItemGroupDataSeq attribute (sequence number)
    - ItemGroupData has ItemGroupOID pointing to ItemGroupDef
    - ItemData elements contain ItemOID and Value

    ItemOID conventions per CDISC standard:
    - Shared variables (STUDYID, USUBJID): IT.{VARIABLE}
    - Domain-specific variables: IT.{DOMAIN}.{VARIABLE}
    """
    for seq, (_, row) in enumerate(data.iterrows(), start=1):
        # Create ItemGroupData with sequence number
        item_group_data = ET.SubElement(
            parent,
            _tag(ODM_NS, "ItemGroupData"),
            attrib={
                "ItemGroupOID": f"IG.{dataset_name}",
            },
        )
        # data:ItemGroupDataSeq is the row sequence number
        item_group_data.set(_attr(DATA_NS, "ItemGroupDataSeq"), str(seq))

        # Add ItemData for each column
        for col_name in data.columns:
            value = row[col_name]

            # Skip null/NaN values
            if _is_null(value):
                continue

            # Format value appropriately
            formatted_value = _format_value(value, col_name)

            # Generate ItemOID following CDISC standard conventions
            item_oid = _generate_item_oid(col_name, domain.code)

            ET.SubElement(
                item_group_data,
                _tag(ODM_NS, "ItemData"),
                attrib={
                    "ItemOID": item_oid,
                    "Value": formatted_value,
                },
            )


def _generate_item_oid(variable_name: str, domain_code: str) -> str:
    """Generate ItemOID following CDISC standard conventions.

    Per CDISC Dataset-XML 1.0 standard:
    - Shared variables (STUDYID, USUBJID) use IT.{VARIABLE} without domain prefix
    - Domain-specific variables use IT.{DOMAIN}.{VARIABLE}
    """
    name = variable_name.upper()
    if name in SHARED_VARIABLE_OIDS:
        return f"IT.{name}"
    return f"IT.{domain_code.upper()}.{variable_name}"


def _is_null(value: object) -> bool:
    """Check if a value is null/NaN/empty."""
    import pandas as pd

    if value is None:
        return True
    if isinstance(value, float) and pd.isna(value):
        return True
    if isinstance(value, str) and value.strip() == "":
        return True
    return False


def _format_value(
    value: object,
    column_name: str,
) -> str:
    """Format a value for Dataset-XML output.

    Handles proper formatting of dates, numbers, and strings.
    """
    import pandas as pd

    # Handle datetime
    if isinstance(value, (pd.Timestamp, datetime)):
        # ISO 8601 format for datetime columns
        if column_name.endswith("DTC"):
            return value.isoformat() if hasattr(value, "isoformat") else str(value)
        return str(value)

    # Handle numeric values
    if isinstance(value, float):
        # Remove trailing zeros for cleaner output
        if value == int(value):
            return str(int(value))
        return f"{value:g}"

    if isinstance(value, int):
        return str(value)

    # Default string conversion
    return str(value)


def write_dataset_xml(
    data: pd.DataFrame,
    domain_code: str,
    config: MappingConfig,
    output: Path,
    *,
    metadata_version_oid: str | None = None,
    is_reference_data: bool | None = None,
) -> None:
    """Write a Dataset-XML 1.0 file for a single domain.

    Args:
        data: The pandas DataFrame containing the domain data.
        domain_code: The SDTM domain code (e.g., "DM", "AE").
        config: The mapping configuration containing study metadata.
        output: The output file path.
        metadata_version_oid: The MetaDataVersionOID to reference Define-XML.
        is_reference_data: Override for reference data detection.
    """
    domain = get_domain(domain_code)

    # Auto-detect reference data if not specified
    class_name = (domain.class_name or "").replace("-", " ").strip().upper()
    if is_reference_data is None:
        is_reference_data = class_name in ("TRIAL DESIGN", "STUDY REFERENCE")

    root = build_dataset_xml_tree(
        data,
        domain_code,
        config,
        metadata_version_oid=metadata_version_oid,
        is_reference_data=is_reference_data,
    )

    tree = ET.ElementTree(root)
    output.parent.mkdir(parents=True, exist_ok=True)
    tree.write(output, xml_declaration=True, encoding="utf-8")


def generate_dataset_xml_streaming(
    data: pd.DataFrame,
    domain_code: str,
    config: MappingConfig,
    output: Path,
    *,
    chunk_size: int = 1000,
    metadata_version_oid: str | None = None,
    is_reference_data: bool | None = None,
) -> None:
    """Generate Dataset-XML with streaming for large datasets.

    This method writes the XML incrementally to handle large datasets
    without loading the entire document into memory.

    Args:
        data: The pandas DataFrame containing the domain data.
        domain_code: The SDTM domain code.
        config: The mapping configuration.
        output: The output file path.
        chunk_size: Number of rows to process at a time.
        metadata_version_oid: The MetaDataVersionOID to reference Define-XML.
        is_reference_data: Override for reference data detection.
    """
    domain = get_domain(domain_code)
    study_id = (config.study_id or "STUDY").strip() or "STUDY"
    study_oid = f"STDY.{study_id}"
    dataset_name = domain.resolved_dataset_name()
    timestamp = datetime.now(UTC).isoformat(timespec="seconds")
    mdv_oid = metadata_version_oid or f"MDV.{study_oid}.SDTMIG.{DEFAULT_SDTM_VERSION}"
    define_file_oid = f"{study_oid}.Define-XML_{DEFINE_XML_VERSION}"

    # Auto-detect reference data
    class_name = (domain.class_name or "").replace("-", " ").strip().upper()
    if is_reference_data is None:
        is_reference_data = class_name in ("TRIAL DESIGN", "STUDY REFERENCE")

    output.parent.mkdir(parents=True, exist_ok=True)

    with output.open("w", encoding="utf-8") as f:
        # Write XML declaration
        f.write('<?xml version="1.0" encoding="UTF-8"?>\n')

        # Write ODM opening tag with namespaces
        f.write(f'<ODM xmlns="{ODM_NS}" ')
        f.write(f'xmlns:xlink="{XLINK_NS}" ')
        f.write(f'xmlns:data="{DATA_NS}" ')
        f.write('FileType="Snapshot" ')
        f.write(f'FileOID="{define_file_oid}(IG.{dataset_name})" ')
        f.write(f'PriorFileOID="{define_file_oid}" ')
        f.write('Granularity="All" ')
        f.write('ODMVersion="1.3.2" ')
        f.write(f'CreationDateTime="{timestamp}" ')
        f.write('Originator="CDISC-Transpiler" ')
        f.write('SourceSystem="CDISC-Transpiler" ')
        f.write('SourceSystemVersion="1.0" ')
        f.write(f'data:DatasetXMLVersion="{DATASET_XML_VERSION}">\n')

        # Write data container opening
        container_tag = "ReferenceData" if is_reference_data else "ClinicalData"
        f.write(f'  <{container_tag} StudyOID="{study_oid}" ')
        f.write(f'MetaDataVersionOID="{mdv_oid}">\n')

        # Stream data in chunks
        seq = 0

        for chunk_start in range(0, len(data), chunk_size):
            chunk = data.iloc[chunk_start : chunk_start + chunk_size]

            for _, row in chunk.iterrows():
                seq += 1
                _write_item_group_data_streaming(f, row, domain, dataset_name, seq)

        # Close tags
        f.write(f"  </{container_tag}>\n")
        f.write("</ODM>\n")


def _write_item_group_data_streaming(
    f: object,
    row: object,
    domain: SDTMDomain,
    dataset_name: str,
    seq: int,
) -> None:
    """Write a single ItemGroupData element to the file stream."""
    f.write(f'    <ItemGroupData ItemGroupOID="IG.{dataset_name}" ')
    f.write(f'data:ItemGroupDataSeq="{seq}">\n')

    for col_name, value in row.items():
        if _is_null(value):
            continue

        formatted_value = _format_value(value, col_name)
        # Escape XML special characters
        escaped_value = _escape_xml(formatted_value)

        item_oid = _generate_item_oid(col_name, domain.code)
        f.write(f'      <ItemData ItemOID="{item_oid}" ')
        f.write(f'Value="{escaped_value}"/>\n')

    f.write("    </ItemGroupData>\n")


def _escape_xml(value: str) -> str:
    """Escape special XML characters in attribute values."""
    return (
        value.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&apos;")
    )


def iter_dataset_xml_records(
    data: pd.DataFrame,
    domain_code: str,
) -> Iterator[ET.Element]:
    """Yield ItemGroupData elements for each record.

    This generator can be used to build custom Dataset-XML documents
    or to stream records to other formats.

    Args:
        data: The pandas DataFrame containing the domain data.
        domain_code: The SDTM domain code.

    Yields:
        ItemGroupData Element for each row in the DataFrame.
    """
    domain = get_domain(domain_code)
    dataset_name = domain.resolved_dataset_name()

    for seq, (_, row) in enumerate(data.iterrows(), start=1):
        item_group_data = ET.Element(
            _tag(ODM_NS, "ItemGroupData"),
            attrib={"ItemGroupOID": f"IG.{dataset_name}"},
        )
        item_group_data.set(_attr(DATA_NS, "ItemGroupDataSeq"), str(seq))

        for col_name in data.columns:
            value = row[col_name]
            if _is_null(value):
                continue

            formatted_value = _format_value(value, col_name)
            ET.SubElement(
                item_group_data,
                _tag(ODM_NS, "ItemData"),
                attrib={
                    "ItemOID": f"IT.{domain.code}.{col_name}",
                    "Value": formatted_value,
                },
            )

        yield item_group_data
