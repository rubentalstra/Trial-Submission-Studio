"""General Observation Class variable management (infrastructure).

This builds Identifier/Timing templates grouped by the three General
Observation Classes, using SDTMIG/SDTM v2 CSV metadata.
"""

from typing import Any

from ...domain.entities.sdtm_classes import GENERAL_OBSERVATION_CLASSES
from ...domain.entities.sdtm_domain import SDTMVariable
from ...domain.entities.variable import variable_from_row
from .utils import core_priority, normalize_class, normalize_general_class


def is_preferred_variable(
    candidate: SDTMVariable, existing: SDTMVariable | None
) -> bool:
    if existing is None:
        return True
    cand_rank = core_priority(candidate.core)
    exist_rank = core_priority(existing.core)
    if cand_rank != exist_rank:
        return cand_rank > exist_rank
    cand_order = candidate.variable_order or 1_000_000
    exist_order = existing.variable_order or 1_000_000
    if cand_order != exist_order:
        return cand_order < exist_order
    return True


def build_general_class_variables(
    sdtmig_cache: dict[str, list[dict[str, Any]]],
    sdtm_v2_cache: dict[str, list[dict[str, Any]]],
) -> tuple[dict[str, dict[str, SDTMVariable]], dict[str, dict[str, set[str]]]]:
    """Collect Identifier/Timing templates grouped by General Observation Class."""
    registry: dict[str, dict[str, SDTMVariable]] = {}
    usage: dict[str, dict[str, set[str]]] = {}

    caches = [sdtm_v2_cache, sdtmig_cache]

    for cache in caches:
        for code, rows in cache.items():
            if not rows:
                continue
            class_name = normalize_class(rows[0].get("Class"))
            general_class = normalize_general_class(class_name)
            if general_class not in GENERAL_OBSERVATION_CLASSES:
                continue
            for row in rows:
                role = (row.get("Role") or "").strip().lower()
                if role not in ("identifier", "timing"):
                    continue
                variable = variable_from_row(row, code, class_name)
                implements = variable.implements
                if not implements:
                    continue
                usage.setdefault(general_class, {}).setdefault(implements, set()).add(
                    code
                )
                existing = registry.setdefault(general_class, {}).get(implements)
                if is_preferred_variable(variable, existing):
                    registry[general_class][implements] = variable

    return registry, usage


__all__ = ["build_general_class_variables", "is_preferred_variable"]
