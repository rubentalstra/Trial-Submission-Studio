"""Dataset builder for Define-XML generation.

This module handles the construction of ItemGroupDef elements (dataset definitions)
and ItemRef elements (variable references within datasets).
"""

from __future__ import annotations

from typing import Iterable
from xml.etree import ElementTree as ET

import pandas as pd

from ...domains_module import SDTMDomain, SDTMVariable, get_domain
from .constants import ODM_NS
from ..utils import tag
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
    """Return key sequence mapping for a domain.

    Args:
        domain_code: Domain code

    Returns:
        Dictionary mapping variable names to key sequence numbers
    """
    code = domain_code.upper()

    base_keys = {"STUDYID": 1, "USUBJID": 2}

    domain_specific = {
        "DM": {"STUDYID": 1, "USUBJID": 2},
        "AE": {"STUDYID": 1, "USUBJID": 2, "AESEQ": 3},
        "CM": {"STUDYID": 1, "USUBJID": 2, "CMSEQ": 3},
        "DS": {"STUDYID": 1, "USUBJID": 2, "DSSEQ": 3},
        "EX": {"STUDYID": 1, "USUBJID": 2, "EXSEQ": 3},
        "LB": {"STUDYID": 1, "USUBJID": 2, "LBSEQ": 3, "LBTESTCD": 4},
        "VS": {"STUDYID": 1, "USUBJID": 2, "VSSEQ": 3, "VSTESTCD": 4},
        "TS": {"STUDYID": 1, "TSSEQ": 2, "TSPARMCD": 3},
        "DA": {"STUDYID": 1, "USUBJID": 2, "DASEQ": 3},
        "TA": {"STUDYID": 1, "ARMCD": 2, "TAETORD": 3},
        "TE": {"STUDYID": 1, "ETCD": 2},
        "SE": {"STUDYID": 1, "USUBJID": 2, "SESEQ": 3},
        "SUPP": {
            "STUDYID": 1,
            "RDOMAIN": 2,
            "USUBJID": 3,
            "IDVAR": 4,
            "IDVARVAL": 5,
            "QNAM": 6,
        },
        "RELREC": {
            "STUDYID": 1,
            "RDOMAIN": 2,
            "USUBJID": 3,
            "IDVAR": 4,
            "IDVARVAL": 5,
            "RELID": 6,
        },
    }

    if code.startswith("SUPP"):
        return domain_specific["SUPP"]
    return domain_specific.get(code, base_keys)


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
