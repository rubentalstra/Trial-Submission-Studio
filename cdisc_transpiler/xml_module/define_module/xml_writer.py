"""XML writer for Define-XML generation.

This module handles XML serialization and file I/O for Define-XML documents.
"""

from __future__ import annotations

from pathlib import Path
from typing import Iterable
from xml.etree import ElementTree as ET


from .models import DefineGenerationError, StudyDataset

from .metadata_builder import build_study_define_tree


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
