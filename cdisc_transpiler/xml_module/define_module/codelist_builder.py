"""CodeList builder for Define-XML generation.

This module handles the construction of CodeList elements for Define-XML,
including controlled terminology integration, NCI code aliases, and MedDRA
external dictionary references.
"""

from __future__ import annotations

from typing import Iterable
from xml.etree import ElementTree as ET

import pandas as pd

from ...domains_module import SDTMVariable
from ...terminology_module import get_controlled_terminology
from .constants import (
    ODM_NS,
    DEF_NS,
    XML_NS,
    CT_STANDARD_OID_SDTM,
    DEFAULT_MEDDRA_VERSION,
    MEDDRA_HREF,
    MEDDRA_CODELIST_NAME,
)
from ..utils import tag, attr


# MedDRA variables that reference external MedDRA dictionary
_MEDDRA_VARIABLES = {
    "AEDECOD",
    "AEPTCD",
    "AELLT",
    "AELLTCD",
    "AEHLT",
    "AEHLTCD",
    "AEHLGT",
    "AEHLGTCD",
    "AEBODSYS",
    "AEBDSYCD",
    "AESOC",
    "AESOCCD",
}

# Variables that should have only "Y" value in codelist
_YES_ONLY_VARS = {
    "DTHFL",
    "LBLOBXFL",
    "QSLOBXFL",
    "VSLOBXFL",
}


def append_code_lists(
    parent: ET.Element, domain_code: str, variables: Iterable[SDTMVariable]
) -> None:
    """Append CodeList elements per Define-XML 2.1 specification.

    Args:
        parent: Parent XML element to append to
        domain_code: Domain code (e.g., "DM", "AE")
        variables: Variables with codelist requirements
    """
    for variable in variables:
        if variable.codelist_code or needs_meddra(variable.name):
            parent.append(build_code_list_element(variable, domain_code))


def build_code_list_element(
    variable: SDTMVariable,
    domain_code: str,
    oid_override: str | None = None,
    extended_values: Iterable[str] | None = None,
) -> ET.Element:
    """Create a CodeList element with CT values and NCI aliases.

    Args:
        variable: SDTM variable with codelist
        domain_code: Domain code
        oid_override: Optional OID override
        extended_values: Additional non-standard values to include

    Returns:
        CodeList XML element
    """
    from .variable_builder import get_datatype

    is_meddra = needs_meddra(variable.name)
    data_type = "text" if is_meddra else get_datatype(variable)
    attrib: dict[str, str] = {
        "OID": oid_override or get_code_list_oid(variable, domain_code),
        "Name": MEDDRA_CODELIST_NAME
        if is_meddra
        else f"{domain_code}.{variable.name} Controlled Terms",
        "DataType": "text" if data_type == "text" else data_type,
    }

    if is_meddra:
        attrib[attr(DEF_NS, "IsNonStandard")] = "Yes"
    else:
        # Only CT-based lists should reference the CT standard
        attrib[attr(DEF_NS, "StandardOID")] = CT_STANDARD_OID_SDTM

    code_list = ET.Element(tag(ODM_NS, "CodeList"), attrib=attrib)

    use_enumerated = should_use_enumerated_item(variable.name)
    ct = get_controlled_terminology(variable=variable.name)
    extended_set = {
        str(val).strip() for val in (extended_values or []) if str(val).strip()
    }
    if variable.name.upper() in _YES_ONLY_VARS:
        ct_values = ["Y"]
    else:
        ct_values = sorted(ct.submission_values) if ct else []

    all_values: list[tuple[str, bool]] = []
    seen: set[str] = set()
    for value in ct_values:
        if value in seen:
            continue
        all_values.append((value, False))
        seen.add(value)
    for value in sorted(extended_set):
        if value in seen:
            continue
        all_values.append((value, True))
        seen.add(value)

    for value, is_extended in all_values:
        if use_enumerated:
            enum_item = ET.SubElement(
                code_list,
                tag(ODM_NS, "EnumeratedItem"),
                attrib={"CodedValue": value},
            )
            if is_extended:
                enum_item.set(attr(DEF_NS, "ExtendedValue"), "Yes")
            nci_code = get_nci_code(variable.name, value)
            if nci_code:
                ET.SubElement(
                    enum_item,
                    tag(ODM_NS, "Alias"),
                    attrib={"Context": "nci:ExtCodeID", "Name": nci_code},
                )
        else:
            cli_attrib = {"CodedValue": value}
            if is_extended:
                cli_attrib[attr(DEF_NS, "ExtendedValue")] = "Yes"
            cli = ET.SubElement(
                code_list,
                tag(ODM_NS, "CodeListItem"),
                attrib=cli_attrib,
            )
            decode = ET.SubElement(cli, tag(ODM_NS, "Decode"))
            ET.SubElement(
                decode,
                tag(ODM_NS, "TranslatedText"),
                attrib={attr(XML_NS, "lang"): "en"},
            ).text = get_decode_value(variable.name, value)

            nci_code = get_nci_code(variable.name, value)
            if nci_code:
                ET.SubElement(
                    cli,
                    tag(ODM_NS, "Alias"),
                    attrib={"Context": "nci:ExtCodeID", "Name": nci_code},
                )

    if is_meddra:
        ET.SubElement(
            code_list,
            tag(ODM_NS, "ExternalCodeList"),
            attrib={
                "Dictionary": "MedDRA",
                "Version": DEFAULT_MEDDRA_VERSION,
                "href": MEDDRA_HREF,
            },
        )

    # Append top-level Alias with the CT codelist code
    if variable.codelist_code and not is_meddra:
        ET.SubElement(
            code_list,
            tag(ODM_NS, "Alias"),
            attrib={"Context": "nci:ExtCodeID", "Name": variable.codelist_code},
        )

    return code_list


def collect_extended_codelist_values(
    dataset: pd.DataFrame | None, variable: SDTMVariable
) -> set[str]:
    """Return dataset values that are not part of the standard CT list.

    Args:
        dataset: DataFrame containing the data
        variable: SDTM variable to check

    Returns:
        Set of extended (non-standard) values found in the data
    """
    if dataset is None or variable.name not in dataset.columns:
        return set()
    if needs_meddra(variable.name):
        return set()

    ct = get_controlled_terminology(variable=variable.name) or (
        get_controlled_terminology(codelist_code=variable.codelist_code)
        if variable.codelist_code
        else None
    )
    if ct is None:
        return set()

    extras: set[str] = set()
    series = pd.Series(dataset[variable.name])
    for raw_value in series.dropna().unique():
        if isinstance(raw_value, (bytes, bytearray)):
            raw_value = raw_value.decode(errors="ignore")
        text = str(raw_value).strip()
        if not text:
            continue
        normalized = ct.normalize(raw_value)
        canonical = normalized if normalized is not None else text
        if canonical in ct.submission_values:
            continue
        extras.add(canonical)
    return extras


def should_use_enumerated_item(variable_name: str) -> bool:
    """Determine if EnumeratedItem should be used instead of CodeListItem.

    EnumeratedItem is used for extensible code lists where the
    submission value equals the decode value.

    Args:
        variable_name: Name of the variable

    Returns:
        True if EnumeratedItem should be used
    """
    name = variable_name.upper()

    # Variables that typically use CodeListItem with different decodes
    codelist_item_vars = {
        "SEX",
        "RACE",
        "ETHNIC",
        "COUNTRY",
        "AEOUT",
        "AESEV",
        "AEREL",
        "AESCAN",
        "AESCONG",
        "AESDISAB",
        "AESDTH",
        "AESHOSP",
        "AESLIFE",
        "AECONTRT",
        "AEACN",
        "NY",
        "EPOCH",
        "ARM",
        "ARMCD",
    }

    # Use EnumeratedItem for most test codes and unit codes
    if name.endswith(("TESTCD", "UNIT", "STRESU", "CAT", "SCAT", "STAT")):
        return True

    return name not in codelist_item_vars


def needs_meddra(variable_name: str) -> bool:
    """Return True when the variable should point to MedDRA terminology.

    Args:
        variable_name: Name of the variable

    Returns:
        True if variable uses MedDRA dictionary
    """
    return variable_name.upper() in _MEDDRA_VARIABLES


def get_decode_value(variable_name: str, coded_value: str) -> str:
    """Return the decode value for a coded value.

    For common SDTM controlled terminology, provides the proper decode.
    Falls back to the coded value if no specific decode is defined.

    Args:
        variable_name: Name of the variable
        coded_value: Coded value to decode

    Returns:
        Decoded value or original coded value
    """
    # Common SDTM decodes
    sex_decodes = {"M": "Male", "F": "Female", "U": "Unknown"}
    ny_decodes = {"Y": "Yes", "N": "No"}
    severity_decodes = {"MILD": "Mild", "MODERATE": "Moderate", "SEVERE": "Severe"}
    outcome_decodes = {
        "RECOVERED/RESOLVED": "Recovered/Resolved",
        "RECOVERING/RESOLVING": "Recovering/Resolving",
        "NOT RECOVERED/NOT RESOLVED": "Not Recovered/Not Resolved",
        "RECOVERED/RESOLVED WITH SEQUELAE": "Recovered/Resolved With Sequelae",
        "FATAL": "Fatal",
        "UNKNOWN": "Unknown",
    }

    name = variable_name.upper()
    value_upper = coded_value.upper()

    if name == "SEX":
        return sex_decodes.get(value_upper, coded_value)
    if name in (
        "AESCAN",
        "AESCONG",
        "AESDISAB",
        "AESDTH",
        "AESHOSP",
        "AESLIFE",
        "AECONTRT",
    ):
        return ny_decodes.get(value_upper, coded_value)
    if name == "AESEV":
        return severity_decodes.get(value_upper, coded_value)
    if name == "AEOUT":
        return outcome_decodes.get(value_upper, coded_value)

    return coded_value


def get_nci_code(variable_name: str, coded_value: str) -> str | None:
    """Return the NCI code for a controlled term if available.

    NCI codes (C-codes) are used as external code identifiers in
    CDISC controlled terminology. This function uses the controlled
    terminology registry for accurate code lookup.

    Args:
        variable_name: Name of the variable
        coded_value: Coded value to look up

    Returns:
        NCI C-code or None if not found
    """
    # Get controlled terminology for the variable and lookup NCI code
    ct = get_controlled_terminology(variable=variable_name)
    if ct:
        nci_code = ct.get_nci_code(coded_value)
        if nci_code:
            return nci_code

    # Fallback for common codes not in the registry
    fallback_codes = {
        ("NY", "Y"): "C49488",
        ("NY", "N"): "C49487",
    }

    key = (variable_name.upper(), coded_value.upper())
    return fallback_codes.get(key)


def get_code_list_oid(variable: SDTMVariable, domain_code: str) -> str:
    """Return the CodeList OID, consolidating MedDRA references.

    Args:
        variable: SDTM variable
        domain_code: Domain code

    Returns:
        CodeList OID string
    """
    name = variable.name.upper()
    if name == "RDOMAIN":
        return "CL.RDOMAIN"
    if needs_meddra(variable.name):
        domain = (domain_code or "GEN").upper()
        return f"CL.{domain}.MEDDRA"
    return f"CL.{domain_code.upper()}.{variable.name}"
