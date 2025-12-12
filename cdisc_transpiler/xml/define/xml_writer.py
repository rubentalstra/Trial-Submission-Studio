"""XML writer for Define-XML generation.

This module handles XML serialization and file I/O for Define-XML documents.
"""

from __future__ import annotations

from pathlib import Path
from typing import Iterable
from xml.etree import ElementTree as ET

from .models import DefineGenerationError, StudyDataset
from .constants import DEFAULT_SDTM_VERSION, CONTEXT_SUBMISSION


def write_define_file(
    dataset,
    domain_code: str,
    config,
    path: str | Path,
    *,
    dataset_href: str | None = None,
    sdtm_version: str = DEFAULT_SDTM_VERSION,
    context: str = CONTEXT_SUBMISSION,
) -> None:
    """Render and persist a Define-XML 2.1 document.
    
    Args:
        dataset: DataFrame containing the domain data
        domain_code: SDTM domain code (e.g., 'DM', 'AE')
        config: Mapping configuration with study metadata
        path: Output file path for the Define-XML
        dataset_href: Optional href for the dataset file reference
        sdtm_version: SDTM-IG version (default: 3.4)
        context: Define-XML context - 'Submission' or 'Other'
    """
    # Import here to avoid circular dependency
    from ..define_xml import build_define_tree
    
    root = build_define_tree(
        dataset,
        domain_code,
        config,
        dataset_href=dataset_href,
        sdtm_version=sdtm_version,
        context=context,
    )
    file_path = Path(path)
    file_path.parent.mkdir(parents=True, exist_ok=True)
    tree = ET.ElementTree(root)
    try:
        tree.write(file_path, encoding="UTF-8", xml_declaration=True)
    except OSError as exc:
        raise DefineGenerationError(f"Failed to write Define-XML: {exc}") from exc


def write_study_define_file(
    datasets: Iterable[StudyDataset],
    output: Path,
    *,
    sdtm_version: str,
    context: str,
) -> None:
    """Write a study-level Define-XML 2.1 document containing multiple datasets.
    
    Args:
        datasets: Iterable of StudyDataset objects
        output: Output file path
        sdtm_version: SDTM-IG version
        context: Define-XML context - 'Submission' or 'Other'
    """
    # Import here to avoid circular dependency
    from ..define_xml import build_study_define_tree
    
    datasets = list(datasets)
    if not datasets:
        raise DefineGenerationError(
            "No datasets supplied for study-level Define generation"
        )

    study_id = datasets[0].config.study_id or "STUDY"
    root = build_study_define_tree(
        datasets,
        study_id=study_id,
        sdtm_version=sdtm_version,
        context=context,
    )
    tree = ET.ElementTree(root)
    output.parent.mkdir(parents=True, exist_ok=True)
    tree.write(output, xml_declaration=True, encoding="UTF-8")
