"""General Observation Class variable management."""

from __future__ import annotations

from .constants import ALWAYS_PROPAGATE_GENERAL, GENERAL_OBSERVATION_CLASSES
from .models import SDTMVariable
from .utils import core_priority, normalize_class, normalize_general_class
from .variable_builder import variable_from_row


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
    sdtmig_cache: dict[str, list[dict]], sdtm_v2_cache: dict[str, list[dict]]
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


def should_propagate_general(
    implements: str, general_class: str, usage: dict[str, dict[str, set[str]]]
) -> bool:
    """Decide whether to add a generalized variable to all domains in its class.
    
    Variables are propagated if they:
    - Start with '--' (domain-specific placeholders like --SEQ)
    - Are in ALWAYS_PROPAGATE_GENERAL (core variables like STUDYID)
    - Are used in multiple domains within the same class
    
    Args:
        implements: Generalized placeholder (e.g., '--SEQ', 'STUDYID')
        general_class: General Observation Class (e.g., 'FINDINGS', 'EVENTS')
        usage: Mapping of {class -> {implements -> set of domain codes}} tracking
               which domains use each generalized variable
    
    Returns:
        bool: True if the variable should be propagated to all domains in the class
    """
    if implements.startswith("--"):
        return True
    if implements in ALWAYS_PROPAGATE_GENERAL:
        return True
    domains = usage.get(general_class, {}).get(implements, set())
    return len(domains) > 1


def augment_general_class_variables(
    variables: list[SDTMVariable],
    class_name: str,
    code: str,
    general_class_variables: dict[str, dict[str, SDTMVariable]],
    general_class_usage: dict[str, dict[str, set[str]]],
) -> list[SDTMVariable]:
    """Add missing Identifier/Timing variables shared within the class."""
    general_class = normalize_general_class(class_name)
    templates = general_class_variables.get(general_class)
    if not templates:
        return variables

    existing = {v.name for v in variables}
    for implements, template in templates.items():
        if not should_propagate_general(implements, general_class, general_class_usage):
            continue
        target_name = (
            f"{code}{implements[2:]}" if implements.startswith("--") else implements
        )
        if target_name in existing:
            continue
        variables.append(
            SDTMVariable(
                name=target_name,
                label=template.label,
                type=template.type,
                length=template.length,
                core=template.core or "Perm",
                codelist_code=template.codelist_code,
                variable_order=None,
                role=template.role,
                value_list=template.value_list,
                described_value_domain=template.described_value_domain,
                codelist_submission_values=template.codelist_submission_values,
                usage_restrictions=template.usage_restrictions,
                definition=template.definition,
                notes=template.notes,
                variables_qualified=template.variables_qualified,
                source_dataset=code,
                source_version=template.source_version,
                implements=implements,
            )
        )
        existing.add(target_name)
    return variables
