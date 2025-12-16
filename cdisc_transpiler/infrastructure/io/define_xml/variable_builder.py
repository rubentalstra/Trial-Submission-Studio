"""Variable builder for Define-XML generation.

This module handles the construction of ItemDef elements (variable definitions)
for Define-XML documents.
"""

from __future__ import annotations

from typing import Iterable
from xml.etree import ElementTree as ET

from cdisc_transpiler.domain.entities.sdtm_domain import SDTMVariable
from .constants import ODM_NS, DEF_NS, XML_NS
from ..xml_utils import tag, attr


def append_item_defs(
    parent: ET.Element, variables: Iterable[SDTMVariable], domain_code: str
) -> None:
    """Append ItemDef elements per Define-XML 2.1 specification.

    Args:
        parent: Parent XML element
        variables: SDTM variables to define
        domain_code: Domain code
    """
    # Local import to avoid circular dependency with codelist_builder
    from .codelist_builder import get_code_list_oid

    for variable in variables:
        data_type = get_datatype(variable)

        attrib = {
            "OID": get_item_oid(variable, domain_code),
            "Name": variable.name,
            "DataType": data_type,
            "SASFieldName": variable.name[:8],
        }

        if data_type in ("text", "integer"):
            attrib["Length"] = str(variable.length)
        elif data_type == "float":
            attrib["Length"] = str(variable.length)
            attrib["SignificantDigits"] = "2"

        item = ET.SubElement(parent, tag(ODM_NS, "ItemDef"), attrib=attrib)

        if data_type == "float":
            item.set(attr(DEF_NS, "DisplayFormat"), f"{variable.length}.2")

        description = ET.SubElement(item, tag(ODM_NS, "Description"))
        ET.SubElement(
            description,
            tag(ODM_NS, "TranslatedText"),
            attrib={attr(XML_NS, "lang"): "en"},
        ).text = variable.label

        if variable.codelist_code:
            ET.SubElement(
                item,
                tag(ODM_NS, "CodeListRef"),
                attrib={"CodeListOID": get_code_list_oid(variable, domain_code)},
            )

        origin_type, origin_source = get_origin(
            variable.name, domain_code, role=variable.role
        )
        origin_attrib = {"Type": origin_type}
        if origin_source:
            origin_attrib["Source"] = origin_source
        ET.SubElement(item, tag(DEF_NS, "Origin"), attrib=origin_attrib)


def get_datatype(variable: SDTMVariable) -> str:
    """Return the proper Define-XML 2.1 DataType for a variable."""
    name = variable.name.upper()
    var_type = variable.type.lower()

    if name.endswith("DTC"):
        if variable.length >= 19:
            return "datetime"
        return "date"

    if name.endswith(("DUR", "ELTM")):
        return "durationDatetime"

    if var_type == "num":
        integer_patterns = ("SEQ", "NUM", "CD", "DY", "ORD", "TPT")
        integer_names = ("AGE", "VISITNUM", "VISITDY", "TAETORD", "DOSE", "NARMS")
        if any(name.endswith(p) for p in integer_patterns) or name in integer_names:
            return "integer"
        return "float"

    return "text"


def get_origin(
    variable_name: str, domain_code: str, *, role: str | None = None
) -> tuple[str, str | None]:
    """Return (Type, Source) for def:Origin element."""
    name = variable_name.upper()
    code = domain_code.upper()
    role_hint = (role or "").strip().lower()

    if name == "DOMAIN":
        return ("Assigned", "Sponsor")
    if name == "STUDYID":
        return ("Protocol", "Sponsor")

    if name == "USUBJID" or name.endswith(("SEQ", "DY")):
        return ("Derived", "Sponsor")

    if name in ("EPOCH", "QORIG", "RDOMAIN") or name.endswith(("CD", "FLG")):
        return ("Assigned", "Sponsor")

    if code == "TS" or name in ("VISITNUM", "VISITDY", "TAETORD"):
        return ("Protocol", "Sponsor")

    if role_hint == "identifier":
        return ("Assigned", "Sponsor")
    if role_hint == "timing":
        return ("Derived", "Sponsor")
    if role_hint == "topic":
        return ("Collected", "Investigator")

    return ("Collected", "Investigator")


def get_item_oid(variable: SDTMVariable, domain_code: str | None) -> str:
    """Generate ItemOID following CDISC standard conventions."""
    name = variable.name.upper()

    SHARED_VARIABLES = {"STUDYID", "USUBJID", "RDOMAIN"}

    if name in SHARED_VARIABLES:
        return f"IT.{name}"

    code = (domain_code or "VAR").upper()
    return f"IT.{code}.{variable.name}"
