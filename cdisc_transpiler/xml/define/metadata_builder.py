"""Metadata builder for Define-XML generation.

This module provides high-level orchestration for building complete Define-XML
documents by coordinating all the individual builders.
"""

from __future__ import annotations

from datetime import UTC, datetime
from typing import Iterable
from xml.etree import ElementTree as ET

from .models import DefineGenerationError, StudyDataset
from .constants import (
    ODM_NS,
    DEF_NS,
    DEFINE_VERSION,
    DEFAULT_SDTM_VERSION,
    CONTEXT_SUBMISSION,
    ACRF_LEAF_ID,
)
from ..utils import tag, attr
from .standards import get_default_standards


def build_define_tree(
    _dataset,
    domain_code: str,
    config,
    *,
    dataset_href: str | None = None,
    sdtm_version: str = DEFAULT_SDTM_VERSION,
    context: str = CONTEXT_SUBMISSION,
) -> ET.Element:
    """Return the Define-XML 2.1 document root for a single domain.

    Args:
        _dataset: DataFrame containing the domain data
        domain_code: SDTM domain code
        config: Mapping configuration
        dataset_href: Optional dataset file reference
        sdtm_version: SDTM-IG version
        context: Define-XML context

    Returns:
        Root Element of the Define-XML document
    """
    study_dataset = StudyDataset(
        domain_code=domain_code,
        dataframe=_dataset,
        config=config,
        archive_location=dataset_href,
    )
    return build_study_define_tree(
        [study_dataset],
        study_id=config.study_id or "STUDY",
        sdtm_version=sdtm_version,
        context=context,
    )


def build_study_define_tree(
    datasets: Iterable[StudyDataset],
    *,
    study_id: str,
    sdtm_version: str,
    context: str,
) -> ET.Element:
    """Build a study-level Define-XML 2.1 document tree with proper ordering.

    This is the main orchestration function that coordinates all builders
    to create a complete Define-XML document.

    Args:
        datasets: Iterable of StudyDataset objects
        study_id: Study identifier
        sdtm_version: SDTM-IG version
        context: Define-XML context ('Submission' or 'Other')

    Returns:
        Root Element of the Define-XML document
    """
    # Import here to avoid circular dependencies
    from ...domains import CT_VERSION, get_domain
    from .codelist_builder import append_code_lists, collect_extended_codelist_values
    from .variable_builder import append_item_defs
    from .dataset_builder import (
        append_item_refs,
        get_active_domain_variables,
        get_domain_description_alias,
    )
    from .value_list_builder import (
        build_supp_value_lists,
        append_value_list_defs,
        append_where_clause_defs,
        append_method_defs,
        append_comment_defs,
    )
    from .standards import get_default_standard_comments

    datasets = list(datasets)
    if not datasets:
        raise DefineGenerationError(
            "No datasets supplied for study-level Define generation"
        )

    timestamp = datetime.now(UTC).isoformat(timespec="seconds")
    study_id = (study_id or "STUDY").strip() or "STUDY"
    study_oid = f"STDY.{study_id}"

    define_file_oid = f"{study_oid}.Define-XML_{DEFINE_VERSION}"
    root = ET.Element(
        tag(ODM_NS, "ODM"),
        attrib={
            "FileType": "Snapshot",
            "FileOID": define_file_oid,
            "ODMVersion": "1.3.2",
            "CreationDateTime": timestamp,
            "Originator": "CDISC-Transpiler",
            "SourceSystem": "CDISC-Transpiler",
            "SourceSystemVersion": "1.0",
        },
    )
    root.set(attr(DEF_NS, "Context"), context)

    study = ET.SubElement(root, tag(ODM_NS, "Study"), attrib={"OID": study_oid})
    globals_node = ET.SubElement(study, tag(ODM_NS, "GlobalVariables"))
    ET.SubElement(globals_node, tag(ODM_NS, "StudyName")).text = study_id
    ET.SubElement(
        globals_node, tag(ODM_NS, "StudyDescription")
    ).text = f"SDTM submission for {study_id}"
    ET.SubElement(globals_node, tag(ODM_NS, "ProtocolName")).text = study_id

    metadata = ET.SubElement(
        study,
        tag(ODM_NS, "MetaDataVersion"),
        attrib={
            "OID": f"MDV.{study_oid}.SDTMIG.{sdtm_version}",
            "Name": f"Study {study_id}, Data Definitions",
            "Description": f"SDTM {sdtm_version} metadata definitions for {study_id}",
        },
    )
    metadata.set(attr(DEF_NS, "DefineVersion"), DEFINE_VERSION)

    standards = get_default_standards(sdtm_version, CT_VERSION)
    append_standards(metadata, standards)

    # Supporting annotated CRF reference
    acrf = ET.SubElement(metadata, tag(DEF_NS, "AnnotatedCRF"))
    ET.SubElement(acrf, tag(DEF_NS, "DocumentRef"), attrib={"leafID": ACRF_LEAF_ID})

    # Collections to avoid duplicate OIDs and enforce proper ordering
    item_groups: list[ET.Element] = []
    item_def_specs: dict[str, tuple] = {}
    vl_item_def_specs: dict[str, tuple] = {}
    code_list_specs: dict[str, tuple] = {}
    value_list_defs = []
    where_clause_defs = []

    # Process each dataset
    for ds in datasets:
        domain = get_domain(ds.domain_code)
        active_vars = get_active_domain_variables(domain, ds.dataframe)

        # Determine parent domain for split datasets (SDTMIG v3.4 Section 4.1.7)
        # Split datasets should have Domain attribute set to parent domain code
        parent_domain_code = ds.domain_code
        if ds.is_split and len(ds.domain_code) > 2:
            # Extract parent domain code (e.g., LBHM → LB, VSRESP → VS)
            # Try 2-character prefix first (most common)
            potential_parent = ds.domain_code[:2]
            try:
                get_domain(potential_parent)
                parent_domain_code = potential_parent
            except Exception:
                # If 2-char doesn't work, the dataset code itself is the domain
                parent_domain_code = ds.domain_code
        
        # Build ItemGroupDef
        ig_attrib = {
            "OID": f"IG.{ds.domain_code}",
            "Name": ds.domain_code,
            "Repeating": "Yes",
            "Domain": parent_domain_code,  # Use parent domain for split datasets
            "SASDatasetName": ds.domain_code[:8],
        }
        ig_attrib[attr(DEF_NS, "Structure")] = (
            ds.structure or "One record per subject per domain-specific entity"
        )
        ig_attrib[attr(DEF_NS, "Class")] = domain.class_name or "EVENTS"

        if ds.archive_location:
            from ..utils import safe_href

            ig_attrib[attr(DEF_NS, "ArchiveLocationID")] = f"LF.{ds.domain_code}"

        ig = ET.Element(tag(ODM_NS, "ItemGroupDef"), attrib=ig_attrib)

        # Add description
        desc = ET.SubElement(ig, tag(ODM_NS, "Description"))
        ET.SubElement(
            desc,
            tag(ODM_NS, "TranslatedText"),
            attrib={attr(DEF_NS.replace("def", "xml"), "lang"): "en"},
        ).text = ds.label or domain.label or ds.domain_code

        # Add domain description alias for split datasets
        alias_text = get_domain_description_alias(domain)
        if alias_text and ds.is_split:
            ET.SubElement(
                ig,
                tag(ODM_NS, "Alias"),
                attrib={"Context": "DomainDescription", "Name": alias_text},
            )

        # Add ItemRefs
        append_item_refs(ig, active_vars, ds.domain_code)

        # Add leaf for dataset file
        if ds.archive_location:
            from ..utils import safe_href

            leaf = ET.SubElement(
                ig,
                tag(DEF_NS, "leaf"),
                attrib={
                    "ID": f"LF.{ds.domain_code}",
                    attr(DEF_NS.replace("def", "xlink"), "href"): safe_href(
                        str(ds.archive_location)
                    ),
                },
            )
            ET.SubElement(leaf, tag(DEF_NS, "title")).text = f"{ds.domain_code}.xpt"

        item_groups.append(ig)

        # Collect ItemDef specs
        for var in active_vars:
            oid = f"IT.{ds.domain_code}.{var.name}"
            if oid not in item_def_specs:
                item_def_specs[oid] = (var, ds.domain_code, None, None)
            if var.codelist_code:
                cl_oid = f"CL.{ds.domain_code}.{var.name}"
                if cl_oid not in code_list_specs:
                    extended = collect_extended_codelist_values(ds.dataframe, var)
                    code_list_specs[cl_oid] = (var, ds.domain_code, extended)

        # Handle SUPP-- value lists
        vl_defs, wc_defs, vl_items, vl_oid = build_supp_value_lists(
            ds.dataframe, domain
        )
        value_list_defs.extend(vl_defs)
        where_clause_defs.extend(wc_defs)
        vl_item_def_specs.update(vl_items)

    # Append ItemGroupDefs
    for ig in item_groups:
        metadata.append(ig)

    # Append ItemDefs
    item_def_parent = ET.Element("temp")
    for oid, (var, domain_code, _, _) in sorted(item_def_specs.items()):
        append_item_defs(item_def_parent, [var], domain_code)
    for item_def in item_def_parent:
        metadata.append(item_def)

    # Append value-level ItemDefs
    for oid, (var, domain_code) in sorted(vl_item_def_specs.items()):
        append_item_defs(metadata, [var], domain_code)

    # Append CodeLists
    cl_parent = ET.Element("temp")
    for cl_oid, (var, domain_code, extended) in sorted(code_list_specs.items()):
        from .codelist_builder import build_code_list_element

        cl_parent.append(
            build_code_list_element(var, domain_code, extended_values=extended)
        )
    for cl in cl_parent:
        metadata.append(cl)

    # Append ValueListDefs
    append_value_list_defs(metadata, value_list_defs)

    # Append WhereClauseDefs
    append_where_clause_defs(metadata, where_clause_defs)

    # Append MethodDefs (empty for now)
    append_method_defs(metadata, [])

    # Append CommentDefs
    comments = get_default_standard_comments()
    append_comment_defs(metadata, comments)

    return root


def append_standards(parent: ET.Element, standards: list) -> None:
    """Append def:Standards element with Standard definitions.

    Args:
        parent: Parent XML element (MetaDataVersion)
        standards: List of StandardDefinition objects
    """
    from .models import StandardDefinition

    standards_elem = ET.SubElement(parent, tag(DEF_NS, "Standards"))

    for std in standards:
        std_attrib = {
            "OID": std.oid,
            "Name": std.name,
            "Type": std.type,
            "Version": std.version,
            "Status": std.status,
        }
        if std.publishing_set:
            std_attrib["PublishingSet"] = std.publishing_set

        std_elem = ET.SubElement(
            standards_elem,
            tag(DEF_NS, "Standard"),
            attrib=std_attrib,
        )

        if std.comment_oid:
            std_elem.set(attr(DEF_NS, "CommentOID"), std.comment_oid)
