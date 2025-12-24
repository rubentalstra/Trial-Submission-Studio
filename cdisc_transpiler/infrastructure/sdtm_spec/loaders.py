import csv
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path


def load_csv_rows(
    path: Path, dataset_field: str = "Dataset Name"
) -> dict[str, list[dict[str, Any]]]:
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
