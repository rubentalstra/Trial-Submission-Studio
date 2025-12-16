"""Dataset builder for Define-XML generation.

This module handles the construction of ItemGroupDef elements (dataset definitions)
and ItemRef elements (variable references within datasets).
"""

from __future__ import annotations

from typing import Iterable
from xml.etree import ElementTree as ET

import pandas as pd

from cdisc_transpiler.domain.entities.sdtm_domain import SDTMDomain, SDTMVariable
from cdisc_transpiler.infrastructure.sdtm_spec.registry import get_domain
from .constants import ODM_NS
from .constants import tag
from .variable_builder import get_item_oid


def append_item_refs(
    parent: ET.Element, variables: Iterable[SDTMVariable], domain_code: str
) -> None:
    """Append ItemRef elements with KeySequence support per Define-XML 2.1.

    Args:
        parent: Parent XML element
        variables: SDTM variables to reference
        domain_code: Domain code
    """
    key_sequences = get_key_sequence(domain_code)

    for order, variable in enumerate(variables, start=1):
        attrib = {
            "ItemOID": get_item_oid(variable, domain_code),
            "OrderNumber": str(order),
            "Mandatory": (
                "Yes" if (variable.core or "").strip().lower() == "req" else "No"
            ),
        }

        if variable.name in key_sequences:
            attrib["KeySequence"] = str(key_sequences[variable.name])

        role = get_variable_role(variable.name, domain_code, variable.role)
        if role:
            attrib["Role"] = role

        parent.append(ET.Element(tag(ODM_NS, "ItemRef"), attrib=attrib))


def get_key_sequence(domain_code: str) -> dict[str, int]:
    """Infer key sequence mapping for a domain.

    This is used to emit Define-XML 2.1 `ItemRef/@KeySequence`.

    We intentionally avoid hardcoded per-domain tables here. The SDTMIG
    Variables.csv already contains variable role/core/order metadata, and the
    SDTM spec registry builds `SDTMDomain` definitions from it.

    Heuristic:
    - Consider only variables with Core == "Req".
    - Exclude DOMAIN (it is required, but not treated as a dataset key).
    - Prefer non-sequence Identifiers first (ordered by Variable Order).
    - Then Topic keys commonly used to identify findings/trial summary rows
      (e.g., *TESTCD, *PARMCD).
    - Then sequence/group identifiers (*SEQ, *GRPID) last.

    Args:
        domain_code: Domain code

    Returns:
        Dictionary mapping variable names to key sequence numbers (1-based)
    """

    try:
        domain = get_domain(domain_code)
    except KeyError:
        # Split datasets (SDTMIG v3.4 Section 4.1.7) such as LBCC/LBHM/VSRESP
        # should inherit metadata (including keys) from the parent 2-char domain.
        code = (domain_code or "").upper()
        if len(code) > 2:
            domain = get_domain(code[:2])
        else:
            raise

    def _is_req(variable: SDTMVariable) -> bool:
        return (variable.core or "").strip().lower() == "req"

    def _role(variable: SDTMVariable) -> str:
        return (variable.role or "").strip().lower()

    def _is_seq_like(name: str) -> bool:
        upper = name.upper()
        if upper in {"SEQ", "GRPID"}:
            return False
        return upper.endswith(("SEQ", "GRPID"))

    def _is_topic_key_like(name: str) -> bool:
        upper = name.upper()
        return upper.endswith(("TESTCD", "PARMCD"))

    required = [
        v for v in domain.variables if _is_req(v) and v.name.upper() != "DOMAIN"
    ]

    identifiers_non_seq = [
        v for v in required if _role(v) == "identifier" and not _is_seq_like(v.name)
    ]
    topic_keys = [
        v for v in required if _role(v) == "topic" and _is_topic_key_like(v.name)
    ]
    identifiers_seq = [
        v for v in required if _role(v) == "identifier" and _is_seq_like(v.name)
    ]

    ordered_keys = [*identifiers_non_seq, *topic_keys, *identifiers_seq]

    # De-duplicate while preserving order.
    seen: set[str] = set()
    unique_keys: list[str] = []
    for var in ordered_keys:
        if var.name in seen:
            continue
        seen.add(var.name)
        unique_keys.append(var.name)

    return {name: idx for idx, name in enumerate(unique_keys, start=1)}


def get_variable_role(
    variable_name: str, domain_code: str, role_hint: str | None = None
) -> str | None:
    """Return the Role attribute value for a variable if applicable.

    Args:
        variable_name: Name of the variable
        domain_code: Domain code
        role_hint: Optional role hint from variable metadata

    Returns:
        Role string or None
    """
    if role_hint:
        return role_hint

    name = variable_name.upper()

    if name in ("STUDYID", "DOMAIN", "RDOMAIN", "USUBJID", "SUBJID"):
        return "Identifier"

    if name.endswith(("DTC", "DY", "DUR", "STDY", "ENDY")):
        return "Timing"

    if name == "QVAL" and domain_code.upper().startswith("SUPP"):
        return "Record Qualifier"

    return None


def get_active_domain_variables(
    domain: SDTMDomain, dataset: pd.DataFrame | None
) -> tuple[SDTMVariable, ...]:
    """Return only required domain variables, those present in the dataset, plus extras.

    Args:
        domain: SDTM domain definition
        dataset: Optional DataFrame with actual data

    Returns:
        Tuple of active variables for the domain
    """
    if dataset is None:
        return domain.variables

    available = set(dataset.columns)
    required = {
        var.name
        for var in domain.variables
        if (var.core or "").strip().lower() == "req"
    }

    active: list[SDTMVariable] = []
    for var in domain.variables:
        if var.name in available or var.name in required:
            active.append(var)

    known = {var.name for var in active}
    extras = available - known
    for name in sorted(extras):
        active.append(
            SDTMVariable(
                name=name,
                label=name,
                type="Char",
                length=200,
                core="Perm",
            )
        )
    return tuple(active)


def get_domain_description_alias(domain: SDTMDomain) -> str | None:
    """Return the DomainDescription alias text.

    For SUPP-- domains, uses base domain label when available.
    For split datasets (SDTMIG v3.4 Section 4.1.7), returns parent domain label.

    Args:
        domain: SDTM domain

    Returns:
        Domain description or None
    """
    code = (domain.code or "").upper()

    # Handle SUPP-- domains
    if code.startswith("SUPP") and len(code) == 6:
        base_code = code[4:]
        try:
            base_domain = get_domain(base_code)
            if base_domain.label:
                return base_domain.label
        except Exception:
            pass

    # Handle split datasets (e.g., LBHM → LB, VSRESP → VS)
    # Per SDTMIG v3.4 Section 4.1.7, split datasets should reference parent domain
    if len(code) > 2:
        # Try to find parent domain by checking 2-character prefix
        potential_parent = code[:2]
        try:
            parent_domain = get_domain(potential_parent)
            # Only use parent if this appears to be a split (starts with parent code)
            if code.startswith(potential_parent) and code != potential_parent:
                return parent_domain.label
        except Exception:
            pass

    return domain.label or None
