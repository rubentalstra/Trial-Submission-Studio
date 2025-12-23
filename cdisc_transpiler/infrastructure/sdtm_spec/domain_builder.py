"""Domain construction from CSV metadata.

This is infrastructure code: it loads SDTMIG/SDTM v2 CSV metadata and builds
domain entities used by the application/domain layers.
"""

from __future__ import annotations

from collections.abc import Mapping, Sequence

from ...domain.entities.sdtm_domain import SDTMDomain, SDTMVariable
from ...domain.entities.variable import variable_from_row
from .utils import normalize_class


def compute_row_order(row: Mapping[str, str], idx: int) -> tuple[int, int]:
    """Compute ordering key for a CSV row based on Variable Order field."""
    raw = (row.get("Variable Order") or "").strip()
    try:
        return (int(raw), idx)
    except ValueError:
        return (1_000_000, idx)


def build_domain_from_rows(
    code: str,
    rows: Sequence[Mapping[str, str]],
    source: str,
    dataset_attributes: dict[str, dict[str, str]],
) -> SDTMDomain | None:
    """Construct a domain definition from CSV rows."""
    if not rows:
        return None

    rows_list = list(rows)
    ordered = [
        r
        for _, r in sorted(
            ((compute_row_order(r, i), r) for i, r in enumerate(rows_list)),
            key=lambda x: x[0],
        )
    ]

    dataset_meta = dataset_attributes.get(code, {})
    dataset_label = dataset_meta.get("label")
    dataset_structure = dataset_meta.get("structure", "")
    class_name = normalize_class(
        dataset_meta.get("class") or (ordered[0].get("Class") or "GENERAL")
    )

    variables: list[SDTMVariable] = [
        variable_from_row(row, code, class_name) for row in ordered
    ]
    description = dataset_label or f"{code} domain (from {source})"

    return SDTMDomain(
        code=code,
        description=description,
        class_name=class_name,
        structure=dataset_structure,
        label=dataset_label,
        variables=tuple(variables),
        dataset_name=code,
    )


def build_supp_domain(code: str, suppqual_base: SDTMDomain | None) -> SDTMDomain:
    """Create a SUPP-- domain definition based on SUPPQUAL metadata."""
    dataset_name = code.upper()
    if suppqual_base is None:
        raise KeyError(
            "SUPPQUAL metadata not available; cannot build supplemental domain."
        )
    return SDTMDomain(
        code=dataset_name,
        description=f"Supplemental Qualifiers for {dataset_name[4:]}",
        class_name=suppqual_base.class_name,
        structure=suppqual_base.structure,
        label=suppqual_base.label,
        variables=suppqual_base.variables,
        dataset_name=dataset_name,
    )


__all__ = ["build_domain_from_rows", "build_supp_domain", "compute_row_order"]
