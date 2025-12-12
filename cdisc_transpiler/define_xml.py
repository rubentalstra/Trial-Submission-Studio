"""Define-XML 2.1 generation helpers.

This module generates Define-XML documents compliant with CDISC Define-XML 2.1.0
specification. The implementation follows the schema structure defined in:
- define2-1-0.xsd
- define-extension.xsd
- define-ns.xsd

Key Define-XML 2.1 features supported:
- def:Context attribute (required on ODM element)
- def:Standards element with Standard definitions
- def:DefineVersion attribute
- def:Structure, def:Class, def:ArchiveLocationID on ItemGroupDef
- def:Origin with Type and Source attributes on ItemDef
- def:leaf elements with xlink:href for dataset locations
- KeySequence tracking on ItemRef
- MethodDef for derivation algorithms
- CommentDef for documentation
- CodeList with def:StandardOID and NCI code aliases
"""

from __future__ import annotations

from datetime import UTC, datetime
from pathlib import Path
from dataclasses import dataclass, replace
from typing import Iterable
from xml.etree import ElementTree as ET

import pandas as pd

from .mapping import MappingConfig
from .terminology import (
    get_nci_code as ct_get_nci_code,
    get_controlled_terminology,
)
from .domains import CT_VERSION, SDTMVariable, SDTMDomain, get_domain

# Define-XML 2.1 namespace declarations per specification
ODM_NS = "http://www.cdisc.org/ns/odm/v1.3"
DEF_NS = "http://www.cdisc.org/ns/def/v2.1"
XLINK_NS = "http://www.w3.org/1999/xlink"
XML_NS = "http://www.w3.org/XML/1998/namespace"

# Register namespaces for proper prefix handling
ET.register_namespace("", ODM_NS)
ET.register_namespace("def", DEF_NS)
ET.register_namespace("xlink", XLINK_NS)

# Define-XML 2.1 version identifier
DEFINE_VERSION = "2.1.0"

# Default SDTM standards aligned to SDTM-MSG v2.0 sample package
DEFAULT_SDTM_VERSION = "3.4"
DEFAULT_SDTM_MD_VERSION = "1.1"
DEFAULT_CT_PUBLISHING_SET = "SDTM"
DEFAULT_CT_DEFINE_PUBLISHING_SET = "DEFINE-XML"

IG_STANDARD_OID = "STD.1"
MD_STANDARD_OID = "STD.2_1"
CT_STANDARD_OID_SDTM = "STD.3"
CT_STANDARD_OID_DEFINE = "STD.4"

# Default supporting document references
ACRF_LEAF_ID = "LF.acrf"
ACRF_HREF = "acrf.pdf"
ACRF_TITLE = "Annotated CRF"
DEFAULT_CRF_PAGE_REFS = "1"
CSDRG_LEAF_ID = "LF.csdrg"
CSDRG_HREF = "csdrg.pdf"
CSDRG_TITLE = "Reviewers Guide"
DEFAULT_MEDDRA_VERSION = "26.1"
MEDDRA_HREF = "https://www.meddra.org/"
MEDDRA_CODELIST_NAME = "MedDRA Dictionary"

# Context values per Define-XML 2.1 spec
CONTEXT_SUBMISSION = "Submission"
CONTEXT_OTHER = "Other"


class DefineGenerationError(RuntimeError):
    """Raised when Define-XML export fails."""


@dataclass(frozen=True)
class StandardDefinition:
    """Represents a def:Standard element in Define-XML 2.1."""

    oid: str
    name: str
    type: str  # IG, CT
    version: str
    status: str = "Final"
    publishing_set: str | None = None  # For CT: SDTM, ADaM, SEND, etc.
    comment_oid: str | None = None


@dataclass(frozen=True)
class OriginDefinition:
    """Represents def:Origin metadata for a variable."""

    type: str  # Collected, Derived, Assigned, Protocol, Predecessor
    source: str | None = None  # Sponsor, Investigator, Vendor, Subject
    description: str | None = None
    document_ref: str | None = None
    page_refs: str | None = None


@dataclass(frozen=True)
class MethodDefinition:
    """Represents a MethodDef element for derivation algorithms."""

    oid: str
    name: str
    type: str  # Computation, Imputation, etc.
    description: str
    document_refs: tuple[str, ...] = ()


@dataclass(frozen=True)
class CommentDefinition:
    """Represents a def:CommentDef element."""

    oid: str
    text: str


def _default_standard_comments() -> list[CommentDefinition]:
    """Default comments used by the MSG sample package standards."""
    return [
        CommentDefinition(
            oid="COM.ST1",
            text=(
                "Study Data Tabulation Model Implementation Guide: "
                "Human Clinical Trials Version 3.4"
            ),
        ),
        CommentDefinition(
            oid="COM.ST2",
            text="Study Data Tabulation Model Implementation Guide for Medical Devices Version 1.0",
        ),
        CommentDefinition(
            oid="COM.ST3",
            text=(
                "This was the latest release of CDISC CT available when this sample "
                "submission was completed."
            ),
        ),
        CommentDefinition(
            oid="COM.ST4",
            text=(
                "This was the CDISC CT Package associated to the CDISC Define-XML "
                "Specification Version 2.1 when this sample submission was completed."
            ),
        ),
    ]


@dataclass(frozen=True)
class WhereClauseDefinition:
    """Represents a def:WhereClauseDef element for value-level metadata.

    WhereClauseDef specifies conditions under which a ValueListDef item applies.
    Used for domains with test results (LB, VS, etc.) where different tests
    have different properties.

    Per Define-XML 2.1:
    - OID format: WC.{domain}.{variable}.{testcd}
    - Contains RangeCheck elements with def:ItemOID
    """

    oid: str
    item_oid: str  # The item being constrained (e.g., IT.LB.LBTESTCD)
    comparator: str  # EQ, NE, IN, NOTIN, LT, LE, GT, GE
    check_values: tuple[str, ...]  # Values for the check
    soft_hard: str = "Soft"  # Soft or Hard


@dataclass(frozen=True)
class ValueListItemDefinition:
    """Represents an ItemRef within a ValueListDef.

    Each item in a ValueListDef describes metadata for a specific
    subset of data identified by a WhereClauseDef.
    """

    item_oid: str  # The ItemDef OID this refers to
    where_clause_oid: str  # The WhereClauseDef OID for this item
    order_number: int
    mandatory: bool = False
    method_oid: str | None = None


@dataclass(frozen=True)
class ValueListDefinition:
    """Represents a def:ValueListDef element for value-level metadata.

    ValueListDef provides variable-level metadata that differs by value
    of another variable (typically TESTCD for findings domains).

    Per Define-XML 2.1:
    - OID format: VL.{domain}.{variable}
    - Contains ItemRef elements pointing to ItemDefs
    - ItemRef has def:WhereClauseOID attribute
    """

    oid: str
    items: tuple[ValueListItemDefinition, ...]


@dataclass
class StudyDataset:
    """Container for a domain dataset to be included in Define-XML."""

    domain_code: str
    dataset: pd.DataFrame | None
    config: MappingConfig
    dataset_href: str | None
    standard_oid: str | None = None
    comment_oid: str | None = None
    is_reference_data: bool = False
    has_no_data: bool = False


def _get_default_standards(
    sdtm_version: str,
    ct_version: str,
    *,
    md_version: str = DEFAULT_SDTM_MD_VERSION,
) -> list[StandardDefinition]:
    """Return the default standard definitions for SDTM submissions."""
    return [
        StandardDefinition(
            oid=IG_STANDARD_OID,
            name="SDTMIG",
            type="IG",
            version=sdtm_version,
            status="Final",
            comment_oid="COM.ST1",
        ),
        StandardDefinition(
            oid=MD_STANDARD_OID,
            name="SDTMIG-MD",
            type="IG",
            version=md_version,
            status="Final",
            comment_oid="COM.ST2",
        ),
        StandardDefinition(
            oid=CT_STANDARD_OID_SDTM,
            name="CDISC/NCI",
            type="CT",
            version=ct_version,
            status="Final",
            publishing_set=DEFAULT_CT_PUBLISHING_SET,
            comment_oid="COM.ST3",
        ),
        StandardDefinition(
            oid=CT_STANDARD_OID_DEFINE,
            name="CDISC/NCI",
            type="CT",
            version=ct_version,
            status="Final",
            publishing_set=DEFAULT_CT_DEFINE_PUBLISHING_SET,
            comment_oid="COM.ST4",
        ),
    ]


def write_define_file(
    dataset: pd.DataFrame,
    domain_code: str,
    config: MappingConfig,
    path: str | Path,
    *,
    dataset_href: str | None = None,
    sdtm_version: str = DEFAULT_SDTM_VERSION,
    context: str = CONTEXT_SUBMISSION,
) -> None:
    """Render and persist a Define-XML 2.1 document.

    Args:
        dataset: DataFrame containing the domain data
        domain_code: SDTM domain code (e.g., 'DM', 'AE')
        config: Mapping configuration with study metadata
        path: Output file path for the Define-XML
        dataset_href: Optional href for the dataset file reference
        sdtm_version: SDTM-IG version (default: 3.4)
        context: Define-XML context - 'Submission' or 'Other'
    """
    root = build_define_tree(
        dataset,
        domain_code,
        config,
        dataset_href=dataset_href,
        sdtm_version=sdtm_version,
        context=context,
    )
    file_path = Path(path)
    file_path.parent.mkdir(parents=True, exist_ok=True)
    tree = ET.ElementTree(root)
    try:
        tree.write(file_path, encoding="UTF-8", xml_declaration=True)
    except OSError as exc:  # pragma: no cover - filesystem failures
        raise DefineGenerationError(f"Failed to write Define-XML: {exc}") from exc


def write_study_define_file(
    datasets: Iterable[StudyDataset],
    output: Path,
    *,
    sdtm_version: str,
    context: str,
) -> None:
    """Write a study-level Define-XML 2.1 document containing multiple datasets.

    Args:
        datasets: Iterable of StudyDataset objects
        output: Output file path
        sdtm_version: SDTM-IG version
        context: Define-XML context - 'Submission' or 'Other'
    """
    datasets = list(datasets)
    if not datasets:
        raise DefineGenerationError(
            "No datasets supplied for study-level Define generation"
        )

    study_id = datasets[0].config.study_id or "STUDY"
    root = build_study_define_tree(
        datasets,
        study_id=study_id,
        sdtm_version=sdtm_version,
        context=context,
    )
    tree = ET.ElementTree(root)
    output.parent.mkdir(parents=True, exist_ok=True)
    tree.write(output, xml_declaration=True, encoding="UTF-8")


def build_define_tree(
    _dataset: pd.DataFrame,
    domain_code: str,
    config: MappingConfig,
    *,
    dataset_href: str | None = None,
    sdtm_version: str = DEFAULT_SDTM_VERSION,
    context: str = CONTEXT_SUBMISSION,
) -> ET.Element:
    """Return the Define-XML 2.1 document root for a single domain."""
    study_dataset = StudyDataset(
        domain_code=domain_code,
        dataset=_dataset,
        config=config,
        dataset_href=dataset_href,
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
    """Build a study-level Define-XML 2.1 document tree with proper ordering."""
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
        _tag(ODM_NS, "ODM"),
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
    root.set(_attr(DEF_NS, "Context"), context)

    study = ET.SubElement(root, _tag(ODM_NS, "Study"), attrib={"OID": study_oid})
    globals_node = ET.SubElement(study, _tag(ODM_NS, "GlobalVariables"))
    ET.SubElement(globals_node, _tag(ODM_NS, "StudyName")).text = study_id
    ET.SubElement(
        globals_node, _tag(ODM_NS, "StudyDescription")
    ).text = f"SDTM submission for {study_id}"
    ET.SubElement(globals_node, _tag(ODM_NS, "ProtocolName")).text = study_id

    metadata = ET.SubElement(
        study,
        _tag(ODM_NS, "MetaDataVersion"),
        attrib={
            "OID": f"MDV.{study_oid}.SDTMIG.{sdtm_version}",
            "Name": f"Study {study_id}, Data Definitions",
            "Description": f"SDTM {sdtm_version} metadata definitions for {study_id}",
        },
    )
    metadata.set(_attr(DEF_NS, "DefineVersion"), DEFINE_VERSION)

    standards = _get_default_standards(sdtm_version, CT_VERSION)
    _append_standards(metadata, standards)

    # Supporting annotated CRF reference (required for Pages checks)
    acrf = ET.SubElement(metadata, _tag(DEF_NS, "AnnotatedCRF"))
    ET.SubElement(acrf, _tag(DEF_NS, "DocumentRef"), attrib={"leafID": ACRF_LEAF_ID})

    # Collections to avoid duplicate OIDs and to enforce proper ordering
    item_groups: list[ET.Element] = []
    item_def_specs: dict[str, tuple[SDTMVariable, str, str | None, str | None]] = {}
    vl_item_def_specs: dict[str, tuple[SDTMVariable, str]] = {}
    code_list_specs: dict[str, tuple[SDTMVariable, str]] = {}
    value_list_defs: list[ValueListDefinition] = []
    where_clause_defs: list[WhereClauseDefinition] = []
    value_list_ref_map: dict[str, str] = {}
    method_defs: dict[str, MethodDefinition] = {}
    code_list_extras: dict[str, set[str]] = {}
    comment_defs: list[CommentDefinition] = _default_standard_comments()
    seen_comment_oids: set[str] = {c.oid for c in comment_defs}

    for entry in datasets:
        domain = get_domain(entry.domain_code)
        dataset_name = domain.resolved_dataset_name()
        has_no_data = entry.has_no_data or entry.dataset is None
        if isinstance(entry.dataset, pd.DataFrame) and entry.dataset.empty:
            has_no_data = True
        dataset_href = (
            None
            if has_no_data
            else _safe_href(entry.dataset_href or f"{dataset_name}.xpt")
        )
        active_vars = _active_domain_variables(domain, entry.dataset)
        # For SUPP-- domains, clamp QVAL length to actual data max to avoid SD1082
        if (
            domain.code.upper().startswith("SUPP")
            and entry.dataset is not None
            and "QVAL" in entry.dataset.columns
        ):
            try:
                max_len = (
                    entry.dataset["QVAL"]
                    .astype(str)
                    .str.len()
                    .max()
                )
            except Exception:
                max_len = None
            if max_len and not pd.isna(max_len):
                max_len = int(min(max_len, 200))
                active_vars = tuple(
                    replace(var, length=max_len)
                    if var.name.upper() == "QVAL"
                    else var
                    for var in active_vars
                )

        empty_expected: set[str] = set()
        if entry.dataset is not None and not has_no_data:
            for var in domain.variables:
                if (var.core or "").strip().upper() != "EXP":
                    continue
                if var.name not in entry.dataset.columns:
                    continue
                if _is_all_missing(entry.dataset[var.name]):
                    empty_expected.add(var.name)

        comment_oids: dict[str, str] = {}
        for var_name in empty_expected:
            oid = f"COM.{domain.code}.{var_name}.NODATA"
            comment_oids[var_name] = oid
            if oid not in seen_comment_oids:
                seen_comment_oids.add(oid)
                text = (
                    f"{var_name} not populated; no collected values for this expected "
                    f"variable in {domain.code}."
                )
                if domain.code.upper() == "DM" and var_name.upper() == "ARMNRS":
                    text = (
                        "ARMNRS not populated because all subjects were assigned planned "
                        "and actual arms."
                    )
                comment_defs.append(CommentDefinition(oid=oid, text=text))

        class_name = (domain.class_name or "").replace("-", " ").strip().upper()
        is_reference_data = entry.is_reference_data or class_name in {
            "TRIAL DESIGN",
            "STUDY REFERENCE",
        }
        structure = (domain.structure or "").strip() or "Not Defined"
        non_repeating_domains = {"DM", "TA", "TE", "TI", "TS", "TV", "DI"}
        repeating = "No" if domain.code.upper() in non_repeating_domains else "Yes"

        item_group = ET.Element(
            _tag(ODM_NS, "ItemGroupDef"),
            attrib={
                "OID": f"IG.{dataset_name}",
                "Name": dataset_name,
                "Domain": domain.code,
                "SASDatasetName": dataset_name,
                "Repeating": repeating,
                "Purpose": "Tabulation",
                "IsReferenceData": "Yes" if is_reference_data else "No",
            },
        )
        item_group.set(_attr(DEF_NS, "Structure"), structure)
        item_group.set(
            _attr(DEF_NS, "StandardOID"), entry.standard_oid or IG_STANDARD_OID
        )
        if not has_no_data:
            item_group.set(_attr(DEF_NS, "ArchiveLocationID"), f"LF.{dataset_name}")
        if has_no_data:
            item_group.set(_attr(DEF_NS, "HasNoData"), "Yes")

        dataset_comment_oid = entry.comment_oid
        if has_no_data and not dataset_comment_oid:
            dataset_comment_oid = f"COM.{domain.code}.NODATA"
            if dataset_comment_oid not in seen_comment_oids:
                seen_comment_oids.add(dataset_comment_oid)
                comment_defs.append(
                    CommentDefinition(
                        oid=dataset_comment_oid,
                        text=(
                            f"No data were submitted for {domain.code}; "
                            f"{domain.description} not collected or not applicable."
                        ),
                    )
                )
        if dataset_comment_oid:
            item_group.set(_attr(DEF_NS, "CommentOID"), dataset_comment_oid)

        description = ET.SubElement(item_group, _tag(ODM_NS, "Description"))
        ET.SubElement(
            description,
            _tag(ODM_NS, "TranslatedText"),
            attrib={_attr(XML_NS, "lang"): "en"},
        ).text = domain.description

        key_sequences = _get_key_sequence(domain.code)
        for order, variable in enumerate(active_vars, start=1):
            attrib = {
                "ItemOID": _item_oid(variable, domain.code),
                "OrderNumber": str(order),
                "Mandatory": (
                    "Yes" if (variable.core or "").strip().lower() == "req" else "No"
                ),
            }
            if variable.name in key_sequences:
                attrib["KeySequence"] = str(key_sequences[variable.name])
            role = _get_variable_role(variable.name, domain.code, variable.role)
            if role:
                attrib["Role"] = role
            origin_type, _origin_source = _get_origin(
                variable.name, domain.code, role=variable.role
            )
            if origin_type == "Derived":
                attrib["MethodOID"] = "MT.DERIVED"
                method_defs.setdefault(
                    "MT.DERIVED",
                    MethodDefinition(
                        oid="MT.DERIVED",
                        name="Generic Derivation",
                        type="Computation",
                        description="Derived programmatically per SDTMIG conventions.",
                    ),
                )
            item_group.append(ET.Element(_tag(ODM_NS, "ItemRef"), attrib=attrib))

        alias_name = _domain_description_alias(domain)
        if alias_name:
            ET.SubElement(
                item_group,
                _tag(ODM_NS, "Alias"),
                attrib={"Context": "DomainDescription", "Name": alias_name},
            )

        ET.SubElement(
            item_group,
            _tag(DEF_NS, "Class"),
            attrib={"Name": domain.class_name},
        )
        if dataset_href:
            leaf = ET.SubElement(
                item_group,
                _tag(DEF_NS, "leaf"),
                attrib={"ID": f"LF.{dataset_name}"},
            )
            leaf.set(_attr(XLINK_NS, "href"), dataset_href)
            ET.SubElement(leaf, _tag(DEF_NS, "title")).text = dataset_href
        item_groups.append(item_group)

        # Collect ItemDef and CodeList specifications
        for variable in active_vars:
            oid = _item_oid(variable, domain.code)
            item_def_specs.setdefault(
                oid,
                (variable, domain.code, None, comment_oids.get(variable.name)),
            )
            if variable.codelist_code or _needs_meddra(variable.name):
                code_list_specs.setdefault(
                    _code_list_oid(variable, domain.code), (variable, domain.code)
                )
                extras = _collect_extended_codelist_values(entry.dataset, variable)
                if extras:
                    code_list_extras.setdefault(
                        _code_list_oid(variable, domain.code), set()
                    ).update(extras)

        # Supplemental qualifiers require value-level metadata for QVAL
        vl_defs, wc_defs, vl_items, vl_oid = _build_supp_value_lists(
            entry.dataset, domain
        )
        value_list_defs.extend(vl_defs)
        where_clause_defs.extend(wc_defs)
        for oid, var in vl_items.items():
            vl_item_def_specs.setdefault(oid, var)
        if vl_oid:
            value_list_ref_map[f"IT.{domain.code}.QVAL"] = vl_oid

    for oid, vl_oid in value_list_ref_map.items():
        if oid in item_def_specs:
            variable, dom_code, _, comment_oid = item_def_specs[oid]
            item_def_specs[oid] = (variable, dom_code, vl_oid, comment_oid)

    # Value-level metadata and where-clauses appear before ItemGroups
    if value_list_defs:
        _append_value_list_defs(metadata, value_list_defs)
    if where_clause_defs:
        _append_where_clause_defs(metadata, where_clause_defs)

    for item_group in item_groups:
        metadata.append(item_group)

    # ItemDefs (unique OIDs)
    all_item_defs: list[tuple[str, SDTMVariable, str, str | None, str | None]] = []
    for oid, (var, dom, vl_ref, comment_oid) in item_def_specs.items():
        all_item_defs.append((oid, var, dom, vl_ref, comment_oid))
    for oid, (var, dom_code) in vl_item_def_specs.items():
        all_item_defs.append((oid, var, dom_code, None, None))

    for oid, variable, dom_code, vl_ref, comment_oid in all_item_defs:
        metadata.append(
            _build_item_def_element(
                variable,
                dom_code,
                item_oid_override=oid,
                value_list_oid=vl_ref,
                comment_oid=comment_oid,
            )
        )

    # CodeLists (unique OIDs)
    for oid, (variable, dom_code) in code_list_specs.items():
        metadata.append(
            _build_code_list_element(
                variable,
                dom_code,
                oid_override=oid,
                extended_values=code_list_extras.get(oid),
            )
        )

    if method_defs:
        _append_method_defs(metadata, list(method_defs.values()))
    if comment_defs:
        _append_comment_defs(metadata, comment_defs)

    # Leaf definitions for supporting documents
    leaf_acrf = ET.SubElement(
        metadata, _tag(DEF_NS, "leaf"), attrib={"ID": ACRF_LEAF_ID}
    )
    leaf_acrf.set(_attr(XLINK_NS, "href"), ACRF_HREF)
    ET.SubElement(leaf_acrf, _tag(DEF_NS, "title")).text = ACRF_TITLE

    leaf_csdrg = ET.SubElement(
        metadata, _tag(DEF_NS, "leaf"), attrib={"ID": CSDRG_LEAF_ID}
    )
    leaf_csdrg.set(_attr(XLINK_NS, "href"), CSDRG_HREF)
    ET.SubElement(leaf_csdrg, _tag(DEF_NS, "title")).text = CSDRG_TITLE

    supplemental = ET.SubElement(metadata, _tag(DEF_NS, "SupplementalDoc"))
    ET.SubElement(
        supplemental, _tag(DEF_NS, "DocumentRef"), attrib={"leafID": CSDRG_LEAF_ID}
    )

    return root


def _append_standards(parent: ET.Element, standards: list[StandardDefinition]) -> None:
    """Append def:Standards element with Standard children."""
    standards_elem = ET.SubElement(parent, _tag(DEF_NS, "Standards"))
    for std in standards:
        attribs = {
            "OID": std.oid,
            "Name": std.name,
            "Type": std.type,
            "Version": std.version,
            "Status": std.status,
        }
        if std.publishing_set:
            attribs["PublishingSet"] = std.publishing_set
        std_elem = ET.SubElement(
            standards_elem, _tag(DEF_NS, "Standard"), attrib=attribs
        )
        if std.comment_oid:
            std_elem.set(_attr(DEF_NS, "CommentOID"), std.comment_oid)


def _append_item_refs(
    parent: ET.Element, variables: Iterable[SDTMVariable], domain_code: str
) -> None:
    """Append ItemRef elements with KeySequence support per Define-XML 2.1."""
    # Define key variables by domain for KeySequence
    key_sequences = _get_key_sequence(domain_code)

    for order, variable in enumerate(variables, start=1):
        attrib = {
            "ItemOID": _item_oid(variable, domain_code),
            "OrderNumber": str(order),
            "Mandatory": (
                "Yes" if (variable.core or "").strip().lower() == "req" else "No"
            ),
        }

        # Add KeySequence for key variables
        if variable.name in key_sequences:
            attrib["KeySequence"] = str(key_sequences[variable.name])

        # Add Role for specific variable types
        role = _get_variable_role(variable.name, domain_code, variable.role)
        if role:
            attrib["Role"] = role

        parent.append(ET.Element(_tag(ODM_NS, "ItemRef"), attrib=attrib))


def _get_key_sequence(domain_code: str) -> dict[str, int]:
    """Return key sequence mapping for a domain."""
    code = domain_code.upper()

    # Common key sequences per SDTM-IG
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


def _get_variable_role(
    variable_name: str, domain_code: str, role_hint: str | None = None
) -> str | None:
    """Return the Role attribute value for a variable if applicable."""
    if role_hint:
        return role_hint

    name = variable_name.upper()

    # Identifier variables
    if name in ("STUDYID", "DOMAIN", "RDOMAIN", "USUBJID", "SUBJID"):
        return "Identifier"

    # Timing variables
    if name.endswith(("DTC", "DY", "DUR", "STDY", "ENDY")):
        return "Timing"

    # Supplemental qualifier QVAL
    if name == "QVAL" and domain_code.upper().startswith("SUPP"):
        return "Record Qualifier"

    return None


def _active_domain_variables(
    domain: SDTMDomain, dataset: pd.DataFrame | None
) -> tuple[SDTMVariable, ...]:
    """Return only required domain variables, those present in the dataset, plus extras."""
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


def _append_item_defs(
    parent: ET.Element, variables: Iterable[SDTMVariable], domain_code: str
) -> None:
    """Append ItemDef elements per Define-XML 2.1 specification."""
    for variable in variables:
        # Determine proper DataType per Define-XML 2.1
        data_type = _get_datatype(variable)

        attrib = {
            "OID": _item_oid(variable, domain_code),
            "Name": variable.name,
            "DataType": data_type,
            "SASFieldName": variable.name[:8],  # SAS name max 8 chars
        }

        # Add Length for text/integer types
        if data_type in ("text", "integer"):
            attrib["Length"] = str(variable.length)
        elif data_type == "float":
            # For float, add Length and SignificantDigits
            attrib["Length"] = str(variable.length)
            attrib["SignificantDigits"] = "2"

        item = ET.SubElement(parent, _tag(ODM_NS, "ItemDef"), attrib=attrib)

        # Add def:DisplayFormat for numeric types
        if data_type == "float":
            item.set(_attr(DEF_NS, "DisplayFormat"), f"{variable.length}.2")

        # Description element
        description = ET.SubElement(item, _tag(ODM_NS, "Description"))
        ET.SubElement(
            description,
            _tag(ODM_NS, "TranslatedText"),
            attrib={_attr(XML_NS, "lang"): "en"},
        ).text = variable.label

        # CodeListRef if controlled terminology exists
        if variable.codelist_code:
            ET.SubElement(
                item,
                _tag(ODM_NS, "CodeListRef"),
                attrib={"CodeListOID": _code_list_oid(variable, domain_code)},
            )

        # def:Origin element with Type and Source
        origin_type, origin_source = _get_origin(
            variable.name, domain_code, role=variable.role
        )
        origin_attrib = {"Type": origin_type}
        if origin_source:
            origin_attrib["Source"] = origin_source
        ET.SubElement(item, _tag(DEF_NS, "Origin"), attrib=origin_attrib)


def _get_datatype(variable: SDTMVariable) -> str:
    """Return the proper Define-XML 2.1 DataType for a variable.

    Per Define-XML 2.1:
    - 'text' for character variables
    - 'integer' for whole number numeric variables
    - 'float' for decimal numeric variables
    - 'date' for ISO 8601 date-only values
    - 'datetime' for ISO 8601 datetime values
    - 'time' for ISO 8601 time-only values

    Per SDTM/XPT:
    - SAS Num maps to integer or float in Define-XML
    - SAS Char maps to text, date, datetime, or time in Define-XML
    """
    name = variable.name.upper()
    var_type = variable.type.lower()

    # Date/datetime/duration variables (always character in SDTM)
    if name.endswith("DTC"):
        # Full datetime format (2023-01-15T14:30:00)
        if variable.length >= 19:
            return "datetime"
        # Date only format (2023-01-15)
        return "date"

    # ISO 8601 duration/elapsed time (always character in SDTM)
    if name.endswith(("DUR", "ELTM")):
        return "durationDatetime"

    # Numeric types
    if var_type == "num":
        # Integer variables (sequence numbers, counts, codes, days, ordinals)
        integer_patterns = ("SEQ", "NUM", "CD", "DY", "ORD", "TPT")
        integer_names = ("AGE", "VISITNUM", "VISITDY", "TAETORD", "DOSE", "NARMS")
        if any(name.endswith(p) for p in integer_patterns) or name in integer_names:
            return "integer"
        # Float for other numeric values (results, ranges, etc.)
        return "float"

    # All other character variables
    return "text"


def _get_origin(
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

    # Derived variables
    if name == "USUBJID" or name.endswith(("SEQ", "DY")):
        return ("Derived", "Sponsor")

    # Assigned variables (constants)
    if name in ("EPOCH", "QORIG", "RDOMAIN") or name.endswith(("CD", "FLG")):
        return ("Assigned", "Sponsor")

    # Protocol-sourced variables
    if code == "TS" or name in ("VISITNUM", "VISITDY", "TAETORD"):
        return ("Protocol", "Sponsor")

    if role_hint == "identifier":
        return ("Assigned", "Sponsor")
    if role_hint == "timing":
        return ("Derived", "Sponsor")
    if role_hint == "topic":
        return ("Collected", "Investigator")

    # Default to Collected from Investigator
    return ("Collected", "Investigator")


def _is_all_missing(series: pd.Series) -> bool:
    """Return True when a series has no non-blank values."""
    if series.isna().all():
        return True
    as_str = series.astype("string").str.strip()
    return as_str.eq("").all()


def _append_code_lists(
    parent: ET.Element, domain_code: str, variables: Iterable[SDTMVariable]
) -> None:
    """Append CodeList elements per Define-XML 2.1 specification."""
    for variable in variables:
        if variable.codelist_code or _needs_meddra(variable.name):
            parent.append(_build_code_list_element(variable, domain_code))


def _build_code_list_element(
    variable: SDTMVariable,
    domain_code: str,
    oid_override: str | None = None,
    extended_values: Iterable[str] | None = None,
) -> ET.Element:
    """Create a CodeList element with CT values and NCI aliases."""
    is_meddra = _needs_meddra(variable.name)
    data_type = "text" if is_meddra else _get_datatype(variable)
    attrib: dict[str, str] = {
        "OID": oid_override or _code_list_oid(variable, domain_code),
        "Name": MEDDRA_CODELIST_NAME
        if is_meddra
        else f"{domain_code}.{variable.name} Controlled Terms",
        "DataType": "text" if data_type == "text" else data_type,
    }

    if is_meddra:
        attrib[_attr(DEF_NS, "IsNonStandard")] = "Yes"
    else:
        # Only CT-based lists should reference the CT standard; external dictionaries
        # like MedDRA do not use StandardOID.
        attrib[_attr(DEF_NS, "StandardOID")] = CT_STANDARD_OID_SDTM

    code_list = ET.Element(_tag(ODM_NS, "CodeList"), attrib=attrib)

    use_enumerated = _should_use_enumerated_item(variable.name)
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
                _tag(ODM_NS, "EnumeratedItem"),
                attrib={"CodedValue": value},
            )
            if is_extended:
                enum_item.set(_attr(DEF_NS, "ExtendedValue"), "Yes")
            nci_code = _get_nci_code(variable.name, value)
            if nci_code:
                ET.SubElement(
                    enum_item,
                    _tag(ODM_NS, "Alias"),
                    attrib={"Context": "nci:ExtCodeID", "Name": nci_code},
                )
        else:
            cli_attrib = {"CodedValue": value}
            if is_extended:
                cli_attrib[_attr(DEF_NS, "ExtendedValue")] = "Yes"
            cli = ET.SubElement(
                code_list,
                _tag(ODM_NS, "CodeListItem"),
                attrib=cli_attrib,
            )
            decode = ET.SubElement(cli, _tag(ODM_NS, "Decode"))
            ET.SubElement(
                decode,
                _tag(ODM_NS, "TranslatedText"),
                attrib={_attr(XML_NS, "lang"): "en"},
            ).text = _get_decode_value(variable.name, value)

            nci_code = _get_nci_code(variable.name, value)
            if nci_code:
                ET.SubElement(
                    cli,
                    _tag(ODM_NS, "Alias"),
                    attrib={"Context": "nci:ExtCodeID", "Name": nci_code},
                )

    if is_meddra:
        ET.SubElement(
            code_list,
            _tag(ODM_NS, "ExternalCodeList"),
            attrib={
                "Dictionary": "MedDRA",
                "Version": DEFAULT_MEDDRA_VERSION,
                "href": MEDDRA_HREF,
            },
        )

    # Append top-level Alias with the CT codelist code after items to satisfy ordering
    if variable.codelist_code and not is_meddra:
        ET.SubElement(
            code_list,
            _tag(ODM_NS, "Alias"),
            attrib={"Context": "nci:ExtCodeID", "Name": variable.codelist_code},
        )

    return code_list


def _collect_extended_codelist_values(
    dataset: pd.DataFrame | None, variable: SDTMVariable
) -> set[str]:
    """Return dataset values that are not part of the standard CT list."""
    if dataset is None or variable.name not in dataset.columns:
        return set()
    if _needs_meddra(variable.name):
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


def _should_use_enumerated_item(variable_name: str) -> bool:
    """Determine if EnumeratedItem should be used instead of CodeListItem.

    EnumeratedItem is used for extensible code lists where the
    submission value equals the decode value.
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

_YES_ONLY_VARS = {
    # P21 expects Yes-only subset for these SDTM flags
    "DTHFL",
    "LBLOBXFL",
    "QSLOBXFL",
    "VSLOBXFL",
}


def _needs_meddra(variable_name: str) -> bool:
    """Return True when the variable should point to MedDRA terminology."""
    return variable_name.upper() in _MEDDRA_VARIABLES


def _get_decode_value(variable_name: str, coded_value: str) -> str:
    """Return the decode value for a coded value.

    For common SDTM controlled terminology, provides the proper decode.
    Falls back to the coded value if no specific decode is defined.
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


def _get_nci_code(variable_name: str, coded_value: str) -> str | None:
    """Return the NCI code for a controlled term if available.

    NCI codes (C-codes) are used as external code identifiers in
    CDISC controlled terminology. This function uses the controlled
    terminology registry for accurate code lookup.
    """
    # First try the controlled terminology registry
    nci_code = ct_get_nci_code(variable_name, coded_value)
    if nci_code:
        return nci_code

    # Fallback for common codes not in the registry
    fallback_codes = {
        ("NY", "Y"): "C49488",
        ("NY", "N"): "C49487",
    }

    key = (variable_name.upper(), coded_value.upper())
    return fallback_codes.get(key)


def _item_oid(variable: SDTMVariable, domain_code: str | None) -> str:
    """Generate ItemOID following CDISC standard conventions.

    Per CDISC Dataset-XML 1.0 and Define-XML 2.1 standards:
    - Shared variables (STUDYID, USUBJID) use IT.{VARIABLE} without domain prefix
    - Domain-specific variables use IT.{DOMAIN}.{VARIABLE}

    This ensures consistent OIDs across all datasets in a study.
    """
    name = variable.name.upper()

    # Shared variables used identically across all domains
    # These get simple IT.{VARIABLE} OIDs
    SHARED_VARIABLES = {"STUDYID", "USUBJID", "RDOMAIN"}

    if name in SHARED_VARIABLES:
        return f"IT.{name}"

    # Domain-specific variables get IT.{DOMAIN}.{VARIABLE} OIDs
    code = (domain_code or "VAR").upper()
    return f"IT.{code}.{variable.name}"


def _code_list_oid(variable: SDTMVariable, domain_code: str) -> str:
    """Return the CodeList OID, consolidating MedDRA references."""
    name = variable.name.upper()
    if name == "RDOMAIN":
        return "CL.RDOMAIN"
    if _needs_meddra(variable.name):
        domain = (domain_code or "GEN").upper()
        return f"CL.{domain}.MEDDRA"
    return f"CL.{domain_code.upper()}.{variable.name}"


def _tag(namespace: str, name: str) -> str:
    return f"{{{namespace}}}{name}"


def _attr(namespace: str, name: str) -> str:
    return f"{{{namespace}}}{name}"


def _safe_href(href: str) -> str:
    """Clamp dataset href to SAS 8-char dataset name plus extension and length cap."""

    if not href:
        return href
    path = Path(href)
    stem = path.stem[:8]
    new_name = f"{stem}{path.suffix}".lower()
    safe = str(path.with_name(new_name))
    return safe[:64]


def _domain_description_alias(domain: SDTMDomain) -> str | None:
    """Return the DomainDescription alias text, with SUPP-- using base domain label."""
    code = (domain.code or "").upper()
    # Supplemental qualifiers: use base domain label when available
    if code.startswith("SUPP") and len(code) == 6:
        base_code = code[4:]
        try:
            base_domain = get_domain(base_code)
            if base_domain.label:
                return base_domain.label
        except Exception:
            pass
    return domain.label or None


# =============================================================================
# Value-Level Metadata (VLM) Support
# =============================================================================


def _build_item_def_element(
    variable: SDTMVariable,
    domain_code: str,
    *,
    item_oid_override: str | None = None,
    value_list_oid: str | None = None,
    comment_oid: str | None = None,
) -> ET.Element:
    """Create an ItemDef element with origin, codelist, and optional ValueListRef."""
    data_type = _get_datatype(variable)
    item_oid = item_oid_override or _item_oid(variable, domain_code)
    attrib = {
        "OID": item_oid,
        "Name": variable.name,
        "DataType": data_type,
        "SASFieldName": variable.name[:8],
    }
    if data_type in ("text", "integer"):
        attrib["Length"] = str(variable.length)
    elif data_type == "float":
        attrib["Length"] = str(variable.length)
        attrib["SignificantDigits"] = "2"

    item = ET.Element(_tag(ODM_NS, "ItemDef"), attrib=attrib)
    if data_type == "float":
        item.set(_attr(DEF_NS, "DisplayFormat"), f"{variable.length}.2")

    description = ET.SubElement(item, _tag(ODM_NS, "Description"))
    ET.SubElement(
        description,
        _tag(ODM_NS, "TranslatedText"),
        attrib={_attr(XML_NS, "lang"): "en"},
    ).text = variable.label

    if variable.codelist_code or _needs_meddra(variable.name):
        ET.SubElement(
            item,
            _tag(ODM_NS, "CodeListRef"),
            attrib={"CodeListOID": _code_list_oid(variable, domain_code)},
        )
    if comment_oid:
        item.set(_attr(DEF_NS, "CommentOID"), comment_oid)

    origin_type, origin_source = _get_origin(
        variable.name, domain_code, role=variable.role
    )
    origin_attrib = {"Type": origin_type}
    if origin_source:
        origin_attrib["Source"] = origin_source
    origin = ET.SubElement(item, _tag(DEF_NS, "Origin"), attrib=origin_attrib)

    if origin_type.lower() == "collected" and origin_source:
        doc_ref = ET.SubElement(
            origin, _tag(DEF_NS, "DocumentRef"), attrib={"leafID": ACRF_LEAF_ID}
        )
        ET.SubElement(
            doc_ref,
            _tag(DEF_NS, "PDFPageRef"),
            attrib={"PageRefs": DEFAULT_CRF_PAGE_REFS, "Type": "PhysicalRef"},
        )

    if value_list_oid:
        ET.SubElement(
            item,
            _tag(DEF_NS, "ValueListRef"),
            attrib={"ValueListOID": value_list_oid},
        )

    return item


def _build_supp_value_lists(
    dataset: pd.DataFrame | None, domain: SDTMDomain
) -> tuple[
    list[ValueListDefinition],
    list[WhereClauseDefinition],
    dict[str, tuple[SDTMVariable, str]],
    str | None,
]:
    """Build ValueListDef/WhereClauseDef for SUPP-- datasets based on QNAM values."""
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
                item_oid=f"IT.{code}.QNAM",
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
        # Use actual data length for QVAL to avoid SD1082 "variable length too long"
        qval_len = (
            dataset.loc[dataset["QNAM"] == qnam, "QVAL"]
            .astype(str)
            .str.len()
            .max()
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


def _append_value_list_defs(
    parent: ET.Element,
    value_lists: list[ValueListDefinition],
) -> None:
    """Append ValueListDef elements for value-level metadata.

    Per Define-XML 2.1, ValueListDef appears after ItemGroupDef elements
    and contains ItemRef elements with def:WhereClauseOID attributes.
    """
    for vl in value_lists:
        vl_elem = ET.SubElement(
            parent,
            _tag(DEF_NS, "ValueListDef"),
            attrib={"OID": vl.oid},
        )

        for item in vl.items:
            item_ref = ET.SubElement(
                vl_elem,
                _tag(ODM_NS, "ItemRef"),
                attrib={
                    "ItemOID": item.item_oid,
                    "OrderNumber": str(item.order_number),
                    "Mandatory": "Yes" if item.mandatory else "No",
                },
            )
            ET.SubElement(
                item_ref,
                _tag(DEF_NS, "WhereClauseRef"),
                attrib={"WhereClauseOID": item.where_clause_oid},
            )

            if item.method_oid:
                item_ref.set("MethodOID", item.method_oid)


def _append_where_clause_defs(
    parent: ET.Element,
    where_clauses: list[WhereClauseDefinition],
) -> None:
    """Append WhereClauseDef elements for value-level metadata conditions.

    Per Define-XML 2.1, WhereClauseDef appears after ValueListDef elements
    and contains RangeCheck elements that specify conditions.
    """
    for wc in where_clauses:
        wc_elem = ET.SubElement(
            parent,
            _tag(DEF_NS, "WhereClauseDef"),
            attrib={"OID": wc.oid},
        )

        range_check = ET.SubElement(
            wc_elem,
            _tag(ODM_NS, "RangeCheck"),
            attrib={
                "Comparator": wc.comparator,
                "SoftHard": wc.soft_hard,
            },
        )
        range_check.set(_attr(DEF_NS, "ItemOID"), wc.item_oid)

        # Add CheckValue elements
        for value in wc.check_values:
            ET.SubElement(
                range_check,
                _tag(ODM_NS, "CheckValue"),
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
        domain_code: The domain code (e.g., "LB", "VS").
        test_codes: List of test codes (e.g., ["GLUC", "HGB", "WBC"]).
        result_variable: The result variable name (default: "ORRES").

    Returns:
        Tuple of (ValueListDefinitions, WhereClauseDefinitions).
    """
    code = domain_code.upper()
    testcd_var = f"{code}TESTCD"
    orres_var = f"{code}{result_variable}"

    value_list_items: list[ValueListItemDefinition] = []
    where_clauses: list[WhereClauseDefinition] = []

    for order, testcd in enumerate(test_codes, start=1):
        # Create WhereClauseDef for this test code
        wc_oid = f"WC.{code}.{orres_var}.{testcd}"
        item_oid = f"IT.{code}.{orres_var}.{testcd}"

        where_clauses.append(
            WhereClauseDefinition(
                oid=wc_oid,
                item_oid=f"IT.{code}.{testcd_var}",
                comparator="EQ",
                check_values=(testcd,),
            )
        )

        # Create ValueListItemDefinition
        value_list_items.append(
            ValueListItemDefinition(
                item_oid=item_oid,
                where_clause_oid=wc_oid,
                order_number=order,
                mandatory=False,
            )
        )

    # Create the ValueListDef
    value_list = ValueListDefinition(
        oid=f"VL.{code}.{orres_var}",
        items=tuple(value_list_items),
    )

    return ([value_list], where_clauses)


def _append_method_defs(
    parent: ET.Element,
    methods: list[MethodDefinition],
) -> None:
    """Append MethodDef elements for computation/derivation algorithms.

    Per Define-XML 2.1, MethodDef provides documentation for
    how derived variables are computed.
    """
    for method in methods:
        method_elem = ET.SubElement(
            parent,
            _tag(ODM_NS, "MethodDef"),
            attrib={
                "OID": method.oid,
                "Name": method.name,
                "Type": method.type,
            },
        )

        description = ET.SubElement(method_elem, _tag(ODM_NS, "Description"))
        ET.SubElement(
            description,
            _tag(ODM_NS, "TranslatedText"),
            attrib={_attr(XML_NS, "lang"): "en"},
        ).text = method.description

        # Add document references if any
        for doc_ref in method.document_refs:
            ET.SubElement(
                method_elem,
                _tag(DEF_NS, "DocumentRef"),
                attrib={"leafID": doc_ref},
            )


def _append_comment_defs(
    parent: ET.Element,
    comments: list[CommentDefinition],
) -> None:
    """Append def:CommentDef elements.

    Per Define-XML 2.1, CommentDef provides additional comments
    that can be referenced by other elements.
    """
    for comment in comments:
        comment_elem = ET.SubElement(
            parent,
            _tag(DEF_NS, "CommentDef"),
            attrib={"OID": comment.oid},
        )

        description = ET.SubElement(comment_elem, _tag(ODM_NS, "Description"))
        ET.SubElement(
            description,
            _tag(ODM_NS, "TranslatedText"),
            attrib={_attr(XML_NS, "lang"): "en"},
        ).text = comment.text
