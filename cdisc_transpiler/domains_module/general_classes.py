"""General Observation Class variable management."""

from __future__ import annotations

from typing import Any

from .constants import GENERAL_OBSERVATION_CLASSES
from ..domain.entities.sdtm_domain import SDTMVariable
from .utils import core_priority, normalize_class, normalize_general_class
from ..domain.entities.variable import variable_from_row


def is_preferred_variable(
    candidate: SDTMVariable, existing: SDTMVariable | None
) -> bool:
    """Select the better variable template when duplicates exist."""
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
    sdtmig_cache: dict[str, list[dict[str, Any]]], sdtm_v2_cache: dict[str, list[dict[str, Any]]]
) -> tuple[dict[str, dict[str, SDTMVariable]], dict[str, dict[str, set[str]]]]:
    """Collect Identifier/Timing templates grouped by General Observation Class."""
    registry: dict[str, dict[str, SDTMVariable]] = {}
    usage: dict[str, dict[str, set[str]]] = {}
    # Process older standard first so SDTMIG (newer) wins ties
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
