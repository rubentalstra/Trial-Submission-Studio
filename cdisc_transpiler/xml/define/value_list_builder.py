"""Value list builder for Define-XML generation.

This module handles the construction of ValueListDef and WhereClauseDef elements
for value-level metadata, primarily used in SUPP-- datasets and findings domains.
"""

from __future__ import annotations

from xml.etree import ElementTree as ET

import pandas as pd

from ...domains_module import SDTMDomain, SDTMVariable
from .models import ValueListDefinition, ValueListItemDefinition, WhereClauseDefinition
from .constants import ODM_NS, DEF_NS
from ..utils import tag, attr


def build_supp_value_lists(
    dataset: pd.DataFrame | None, domain: SDTMDomain
) -> tuple[
    list[ValueListDefinition],
    list[WhereClauseDefinition],
    dict[str, tuple[SDTMVariable, str]],
    str | None,
]:
    """Build ValueListDef/WhereClauseDef for SUPP-- datasets based on QNAM values.

    Args:
        dataset: DataFrame containing SUPP data
        domain: SDTM domain definition

    Returns:
        Tuple of (value_lists, where_clauses, vl_item_defs, value_list_oid)
    """
    code = domain.code.upper()
    if not code.startswith("SUPP") or dataset is None or "QNAM" not in dataset.columns:
        return ([], [], {}, None)

    qnames = sorted(
        {
            str(val).strip()
            for val in dataset["QNAM"].dropna().unique()
            if str(val).strip()
        }
    )
    if not qnames:
        return ([], [], {}, None)

    qlabels: dict[str, str] = {}
    if "QLABEL" in dataset.columns:
        for qnam in qnames:
            try:
                label_val = (
                    dataset.loc[dataset["QNAM"] == qnam, "QLABEL"].dropna().iloc[0]
                )
                label_text = str(label_val).strip()
                if label_text:
                    qlabels[qnam] = label_text
            except Exception:
                continue

    vl_items: list[ValueListItemDefinition] = []
    wc_defs: list[WhereClauseDefinition] = []
    vl_item_defs: dict[str, tuple[SDTMVariable, str]] = {}

    for order, qnam in enumerate(qnames, start=1):
        wc_oid = f"WC.{code}.QNAM.{qnam}"
        wc_defs.append(
            WhereClauseDefinition(
                oid=wc_oid,
                dataset_name=domain.dataset_name,
                variable_name="QNAM",
                variable_oid=f"IT.{code}.QNAM",
                comparator="EQ",
                check_values=(qnam,),
            )
        )

        item_oid = f"IT.{code}.QVAL.{qnam}"
        vl_items.append(
            ValueListItemDefinition(
                item_oid=item_oid,
                where_clause_oid=wc_oid,
                order_number=order,
                mandatory=False,
            )
        )

        label = qlabels.get(qnam) or f"Supplemental value for {qnam}"
        qval_len = (
            dataset.loc[dataset["QNAM"] == qnam, "QVAL"].astype(str).str.len().max()
        )
        if pd.isna(qval_len):
            qval_len = 1
        qval_len = max(1, min(int(qval_len), 200))
        vl_item_defs[item_oid] = (
            SDTMVariable(
                name=qnam,
                label=label,
                type="Char",
                length=qval_len,
                core="Perm",
                source_dataset=domain.code,
                source_version=domain.description,
            ),
            domain.code,
        )

    value_list = ValueListDefinition(oid=f"VL.{code}.QVAL", items=tuple(vl_items))
    return ([value_list], wc_defs, vl_item_defs, value_list.oid)


def append_value_list_defs(
    parent: ET.Element,
    value_lists: list[ValueListDefinition],
) -> None:
    """Append ValueListDef elements for value-level metadata.

    Per Define-XML 2.1, ValueListDef appears after ItemGroupDef elements
    and contains ItemRef elements with def:WhereClauseOID attributes.

    Args:
        parent: Parent XML element
        value_lists: List of ValueListDefinition objects
    """
    for vl in value_lists:
        vl_elem = ET.SubElement(
            parent,
            tag(DEF_NS, "ValueListDef"),
            attrib={"OID": vl.oid},
        )

        for item in vl.items:
            item_ref = ET.SubElement(
                vl_elem,
                tag(ODM_NS, "ItemRef"),
                attrib={
                    "ItemOID": item.item_oid,
                    "OrderNumber": str(item.order_number),
                    "Mandatory": "Yes" if item.mandatory else "No",
                },
            )
            ET.SubElement(
                item_ref,
                tag(DEF_NS, "WhereClauseRef"),
                attrib={"WhereClauseOID": item.where_clause_oid},
            )

            if getattr(item, 'method_oid', None):
                item_ref.set("MethodOID", item.method_oid)


def append_where_clause_defs(
    parent: ET.Element,
    where_clauses: list[WhereClauseDefinition],
) -> None:
    """Append WhereClauseDef elements for value-level metadata conditions.

    Per Define-XML 2.1, WhereClauseDef appears after ValueListDef elements
    and contains RangeCheck elements that specify conditions.

    Args:
        parent: Parent XML element
        where_clauses: List of WhereClauseDefinition objects
    """
    for wc in where_clauses:
        wc_elem = ET.SubElement(
            parent,
            tag(DEF_NS, "WhereClauseDef"),
            attrib={"OID": wc.oid},
        )

        range_check = ET.SubElement(
            wc_elem,
            tag(ODM_NS, "RangeCheck"),
            attrib={
                "Comparator": wc.comparator,
                "SoftHard": getattr(wc, 'soft_hard', 'Soft'),
            },
        )
        item_oid = getattr(wc, 'item_oid', None) or wc.variable_oid
        range_check.set(attr(DEF_NS, "ItemOID"), item_oid)

        for value in wc.check_values:
            ET.SubElement(
                range_check,
                tag(ODM_NS, "CheckValue"),
            ).text = value


def generate_vlm_for_findings_domain(
    domain_code: str,
    test_codes: list[str],
    result_variable: str = "ORRES",
) -> tuple[list[ValueListDefinition], list[WhereClauseDefinition]]:
    """Generate value-level metadata for a findings domain.

    Findings domains (LB, VS, EG, etc.) typically have different metadata
    for each test code. This function generates the VLM structure for
    a given set of test codes.

    Args:
        domain_code: The domain code (e.g., "LB", "VS")
        test_codes: List of test codes (e.g., ["GLUC", "HGB", "WBC"])
        result_variable: The result variable name (default: "ORRES")

    Returns:
        Tuple of (value_lists, where_clauses)
    """
    value_lists: list[ValueListDefinition] = []
    where_clauses: list[WhereClauseDefinition] = []

    # Implementation note: This is a template function that can be extended
    # when full VLM support for findings domains is needed

    return (value_lists, where_clauses)


def append_method_defs(
    parent: ET.Element,
    methods: list,
) -> None:
    """Append MethodDef elements for computation/derivation algorithms.

    Per Define-XML 2.1, MethodDef provides documentation for
    how derived variables are computed.

    Args:
        parent: Parent XML element
        methods: List of MethodDefinition objects
    """
    from .models import MethodDefinition

    for method in methods:
        method_elem = ET.SubElement(
            parent,
            tag(ODM_NS, "MethodDef"),
            attrib={
                "OID": method.oid,
                "Name": method.name,
                "Type": method.type,
            },
        )

        description = ET.SubElement(method_elem, tag(ODM_NS, "Description"))
        ET.SubElement(
            description,
            tag(ODM_NS, "TranslatedText"),
            attrib={attr(DEF_NS.replace("def", "xml"), "lang"): "en"},
        ).text = method.description

        for doc_ref in method.document_refs:
            ET.SubElement(
                method_elem,
                tag(DEF_NS, "DocumentRef"),
                attrib={"leafID": doc_ref},
            )


def append_comment_defs(
    parent: ET.Element,
    comments: list,
) -> None:
    """Append def:CommentDef elements.

    Per Define-XML 2.1, CommentDef provides additional comments
    that can be referenced by other elements.

    Args:
        parent: Parent XML element
        comments: List of CommentDefinition objects
    """
    from .models import CommentDefinition

    for comment in comments:
        comment_elem = ET.SubElement(
            parent,
            tag(DEF_NS, "CommentDef"),
            attrib={"OID": comment.oid},
        )

        description = ET.SubElement(comment_elem, tag(ODM_NS, "Description"))
        ET.SubElement(
            description,
            tag(ODM_NS, "TranslatedText"),
            attrib={attr(DEF_NS.replace("def", "xml"), "lang"): "en"},
        ).text = comment.text
