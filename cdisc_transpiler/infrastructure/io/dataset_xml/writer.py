"""Writer for Dataset-XML 1.0 files.

This module handles XML serialization and file I/O for Dataset-XML documents.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING
from xml.etree import ElementTree as ET

if TYPE_CHECKING:
    import pandas as pd
    from cdisc_transpiler.domain.entities.mapping import MappingConfig

from cdisc_transpiler.domains_module import get_domain
from .builder import build_dataset_xml_tree


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
        data: The pandas DataFrame containing the domain data
        domain_code: The SDTM domain code (e.g., "DM", "AE")
        config: The mapping configuration containing study metadata
        output: The output file path
        metadata_version_oid: The MetaDataVersionOID to reference Define-XML
        is_reference_data: Override for reference data detection
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
