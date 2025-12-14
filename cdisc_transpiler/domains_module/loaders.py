"""CSV loaders for SDTM metadata."""

from __future__ import annotations

import csv
from pathlib import Path
from typing import Any


def load_csv_rows(
    path: Path, dataset_field: str = "Dataset Name"
) -> dict[str, list[dict[str, Any]]]:
    """Load SDTM metadata rows keyed by dataset/domain code."""
    if not path.exists():
        return {}
    data: dict[str, list[dict[str, Any]]] = {}
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        for row in reader:
            code = (row.get(dataset_field) or "").strip().upper()
            var = (row.get("Variable Name") or "").strip()
            if not code or not var:
                continue
            data.setdefault(code, []).append(row)
    return data


def load_dataset_attributes(path: Path) -> dict[str, dict[str, str]]:
    """Load dataset-level attributes (class/label/structure) from Datasets.csv."""
    if not path.exists():
        return {}
    attributes: dict[str, dict[str, str]] = {}
    
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        for row in reader:
            dataset_name = (row.get("Dataset Name") or "").strip().upper()
            if not dataset_name:
                continue
            attributes[dataset_name] = {
                "class": (row.get("Class") or "").strip(),
                "label": (row.get("Dataset Label") or "").strip(),
                "structure": (row.get("Structure") or "").strip(),
            }
    return attributes
