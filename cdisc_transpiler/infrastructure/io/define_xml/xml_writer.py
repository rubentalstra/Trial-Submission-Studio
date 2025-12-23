"""Define-XML writer + builders.

This module contains XML serialization and the concrete builder implementation
for Define-XML documents.

Rationale: the Define-XML implementation was previously split across many small
builder modules. Consolidating them here reduces file count and makes the
infrastructure I/O layer easier to navigate.
"""

from datetime import UTC, datetime
from typing import TYPE_CHECKING, TypeAlias
from xml.etree import ElementTree as ET
from xml.etree.ElementTree import Element

import pandas as pd

from cdisc_transpiler.domain.entities.sdtm_domain import SDTMVariable
from cdisc_transpiler.infrastructure.repositories.ct_repository import (
    get_default_ct_repository,
)
from cdisc_transpiler.infrastructure.sdtm_spec.constants import CT_VERSION
from cdisc_transpiler.infrastructure.sdtm_spec.registry import get_domain

from ..xml_utils import attr, safe_href, tag
from .constants import (
    ACRF_LEAF_ID,
    CT_STANDARD_OID_SDTM,
    DEF_NS,
    DEFAULT_MEDDRA_VERSION,
    DEFINE_VERSION,
    MEDDRA_CODELIST_NAME,
    MEDDRA_HREF,
    ODM_NS,
    XLINK_NS,
    XML_NS,
)
from .models import (
    DefineGenerationError,
    ValueListDefinition,
    ValueListItemDefinition,
    WhereClauseDefinition,
)
from .standards import get_default_standard_comments, get_default_standards

if TYPE_CHECKING:
    from collections.abc import Iterable, Sequence
    from pathlib import Path

    from cdisc_transpiler.application.ports.repositories import CTRepositoryPort
    from cdisc_transpiler.domain.entities.controlled_terminology import (
        ControlledTerminology,
    )
    from cdisc_transpiler.domain.entities.sdtm_domain import SDTMDomain

    from .models import (
        CommentDefinition,
        MethodDefinition,
        StandardDefinition,
        StudyDataset,
    )

    XmlElement: TypeAlias = Element[str]
else:
    XmlElement: TypeAlias = Element
ItemDefSpec: TypeAlias = tuple[
    SDTMVariable, str, ValueListDefinition | None, WhereClauseDefinition | None
]
ValueListItemSpec: TypeAlias = tuple[SDTMVariable, str]
CodeListSpec: TypeAlias = tuple[SDTMVariable, str, set[str]]
DTC_DATETIME_MIN_LENGTH = 19


def write_study_define_file(
    datasets: Iterable[StudyDataset],
    output: Path,
    *,
    sdtm_version: str,
    context: str,
) -> None:
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


def build_study_define_tree(
    datasets: Iterable[StudyDataset],
    *,
    study_id: str,
    sdtm_version: str,
    context: str,
) -> XmlElement:
    datasets = list(datasets)
    if not datasets:
        raise DefineGenerationError(
            "No datasets supplied for study-level Define generation"
        )

    timestamp = datetime.now(UTC).isoformat(timespec="seconds")
    study_id = (study_id or "STUDY").strip() or "STUDY"
    study_oid = f"STDY.{study_id}"

    define_file_oid = f"{study_oid}.Define-XML_{DEFINE_VERSION}"
    root: XmlElement = ET.Element(
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
    _append_standards(metadata, standards)

    acrf = ET.SubElement(metadata, tag(DEF_NS, "AnnotatedCRF"))
    ET.SubElement(acrf, tag(DEF_NS, "DocumentRef"), attrib={"leafID": ACRF_LEAF_ID})

    item_groups: list[XmlElement] = []
    item_def_specs: dict[str, ItemDefSpec] = {}
    vl_item_def_specs: dict[str, ValueListItemSpec] = {}
    code_list_specs: dict[str, CodeListSpec] = {}
    value_list_defs: list[ValueListDefinition] = []
    where_clause_defs: list[WhereClauseDefinition] = []

    for ds in datasets:
        domain = get_domain(ds.domain_code)
        parent_domain_code = ds.domain_code

        active_vars = get_active_domain_variables(domain, ds.dataframe)

        ig_attrib: dict[str, str] = {
            "OID": f"IG.{ds.domain_code}",
            "Name": ds.domain_code,
            "Repeating": "Yes",
            "Domain": parent_domain_code,
            "SASDatasetName": ds.domain_code[:8],
        }
        ig_attrib[attr(DEF_NS, "Structure")] = (
            ds.structure or "One record per subject per domain-specific entity"
        )
        ig_attrib[attr(DEF_NS, "Class")] = domain.class_name or "EVENTS"

        if ds.archive_location:
            ig_attrib[attr(DEF_NS, "ArchiveLocationID")] = f"LF.{ds.domain_code}"

        ig: XmlElement = ET.Element(tag(ODM_NS, "ItemGroupDef"), attrib=ig_attrib)

        desc = ET.SubElement(ig, tag(ODM_NS, "Description"))
        ET.SubElement(
            desc,
            tag(ODM_NS, "TranslatedText"),
            attrib={attr(XML_NS, "lang"): "en"},
        ).text = ds.label or domain.label or ds.domain_code

        append_item_refs(ig, active_vars, ds.domain_code)

        if ds.archive_location:
            leaf = ET.SubElement(
                ig,
                tag(DEF_NS, "leaf"),
                attrib={
                    "ID": f"LF.{ds.domain_code}",
                    attr(XLINK_NS, "href"): safe_href(str(ds.archive_location)),
                },
            )
            ET.SubElement(leaf, tag(DEF_NS, "title")).text = f"{ds.domain_code}.xpt"

        item_groups.append(ig)

        for var in active_vars:
            oid = f"IT.{ds.domain_code}.{var.name}"
            if oid not in item_def_specs:
                item_def_specs[oid] = (var, ds.domain_code, None, None)
            if var.codelist_code:
                cl_oid = f"CL.{ds.domain_code}.{var.name}"
                if cl_oid not in code_list_specs:
                    extended = collect_extended_codelist_values(ds.dataframe, var)
                    code_list_specs[cl_oid] = (var, ds.domain_code, extended)

        vl_defs, wc_defs, vl_items, _vl_oid = build_supp_value_lists(
            ds.dataframe, domain
        )
        value_list_defs.extend(vl_defs)
        where_clause_defs.extend(wc_defs)
        vl_item_def_specs.update(vl_items)

    for ig in item_groups:
        metadata.append(ig)

    item_def_parent: XmlElement = ET.Element("temp")
    for _oid, (var, domain_code, _, _) in sorted(item_def_specs.items()):
        append_item_defs(item_def_parent, [var], domain_code)
    for item_def in item_def_parent:
        metadata.append(item_def)

    for _oid, (var, domain_code) in sorted(vl_item_def_specs.items()):
        append_item_defs(metadata, [var], domain_code)

    cl_parent: XmlElement = ET.Element("temp")
    for _cl_oid, (var, domain_code, extended) in sorted(code_list_specs.items()):
        cl_parent.append(
            build_code_list_element(var, domain_code, extended_values=extended)
        )
    for cl in cl_parent:
        metadata.append(cl)

    append_value_list_defs(metadata, value_list_defs)
    append_where_clause_defs(metadata, where_clause_defs)
    append_method_defs(metadata, [])

    comments = get_default_standard_comments()
    append_comment_defs(metadata, comments)

    return root


def _append_standards(
    parent: XmlElement, standards: Sequence[StandardDefinition]
) -> None:
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


def append_item_refs(
    parent: XmlElement, variables: Iterable[SDTMVariable], domain_code: str
) -> None:
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
    code = (domain_code or "").upper()

    # RELREC uniqueness is defined by RELID groups in practice.
    # The SDTMIG spec marks RELID as required, but not as an Identifier role.
    # P21 flags duplicate records when keys are underspecified, so include RELID.
    if code == "RELREC":
        return {"STUDYID": 1, "RDOMAIN": 2, "IDVAR": 3, "RELID": 4}

    # SUPPQUAL uniqueness is defined by (STUDYID, RDOMAIN, USUBJID, IDVAR, IDVARVAL, QNAM).
    # P21 flags duplicate records when key variables are underspecified.
    if code.startswith("SUPP"):
        return {
            "STUDYID": 1,
            "RDOMAIN": 2,
            "USUBJID": 3,
            "IDVAR": 4,
            "IDVARVAL": 5,
            "QNAM": 6,
        }

    try:
        domain = get_domain(domain_code)
    except KeyError:
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
    if dataset is None:
        return domain.variables

    available = set(dataset.columns)
    required = {
        var.name
        for var in domain.variables
        if (var.core or "").strip().lower() == "req"
    }

    qval_length: int | None = None
    if (domain.code or "").upper().startswith("SUPP") and "QVAL" in available:
        observed = dataset["QVAL"].astype("string").fillna("")
        max_len = int(observed.str.len().max() or 0)
        qval_length = min(max(1, max_len), 200)

    active: list[SDTMVariable] = []
    for var in domain.variables:
        if var.name in available or var.name in required:
            if qval_length is not None and var.name.upper() == "QVAL":
                active.append(
                    SDTMVariable(
                        name=var.name,
                        label=var.label,
                        type=var.type,
                        length=qval_length,
                        core=var.core,
                        role=var.role,
                        codelist_code=var.codelist_code,
                    )
                )
            else:
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
    code = (domain.code or "").upper()

    if code.startswith("SUPP") and len(code) == 6:
        base_code = code[4:]
        try:
            base_domain = get_domain(base_code)
            if base_domain.label:
                return base_domain.label
        except Exception:
            pass

    if len(code) > 2:
        potential_parent = code[:2]
        try:
            parent_domain = get_domain(potential_parent)
            if code.startswith(potential_parent) and code != potential_parent:
                return parent_domain.label
        except Exception:
            pass

    return domain.label or None


def append_item_defs(
    parent: XmlElement, variables: Iterable[SDTMVariable], domain_code: str
) -> None:
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
    name = variable.name.upper()
    var_type = variable.type.lower()

    if name.endswith("DTC"):
        if variable.length >= DTC_DATETIME_MIN_LENGTH:
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
    name = variable.name.upper()

    shared_variables = {"STUDYID", "USUBJID", "RDOMAIN"}

    if name in shared_variables:
        return f"IT.{name}"

    code = (domain_code or "VAR").upper()
    return f"IT.{code}.{variable.name}"


def _get_ct(variable: SDTMVariable, domain_code: str) -> ControlledTerminology | None:
    ct_repository: CTRepositoryPort = get_default_ct_repository()
    if variable.codelist_code:
        ct = ct_repository.get_by_code(variable.codelist_code)
        if ct is not None:
            return ct

    try:
        domain = get_domain(domain_code)
        for var in domain.variables:
            if var.name.upper() == variable.name.upper() and var.codelist_code:
                return ct_repository.get_by_code(var.codelist_code)
    except Exception:
        pass

    return ct_repository.get_by_name(variable.name)


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
    "DTHFL",
    "LBLOBXFL",
    "QSLOBXFL",
    "VSLOBXFL",
}


def build_code_list_element(
    variable: SDTMVariable,
    domain_code: str,
    oid_override: str | None = None,
    extended_values: Iterable[str] | None = None,
) -> XmlElement:
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
        attrib[attr(DEF_NS, "StandardOID")] = CT_STANDARD_OID_SDTM

    code_list = ET.Element(tag(ODM_NS, "CodeList"), attrib=attrib)

    use_enumerated = should_use_enumerated_item(variable.name)
    ct = _get_ct(variable, domain_code)
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
        code = ct.lookup_code(value) if ct else None

        if use_enumerated:
            enum_item = ET.SubElement(
                code_list,
                tag(ODM_NS, "EnumeratedItem"),
                attrib={"CodedValue": value},
            )
            if is_extended:
                enum_item.set(attr(DEF_NS, "ExtendedValue"), "Yes")
            if code:
                ET.SubElement(
                    enum_item,
                    tag(ODM_NS, "Alias"),
                    attrib={"Context": "nci:ExtCodeID", "Name": code},
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

            if code:
                ET.SubElement(
                    cli,
                    tag(ODM_NS, "Alias"),
                    attrib={"Context": "nci:ExtCodeID", "Name": code},
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
    if dataset is None or variable.name not in dataset.columns:
        return set()
    if needs_meddra(variable.name):
        return set()

    ct_repository: CTRepositoryPort = get_default_ct_repository()
    ct = (
        ct_repository.get_by_code(variable.codelist_code)
        if variable.codelist_code
        else ct_repository.get_by_name(variable.name)
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
        canonical = normalized or text
        if canonical in ct.submission_values:
            continue
        extras.add(canonical)
    return extras


def should_use_enumerated_item(variable_name: str) -> bool:
    name = variable_name.upper()

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

    if name.endswith(("TESTCD", "UNIT", "STRESU", "CAT", "SCAT", "STAT")):
        return True

    return name not in codelist_item_vars


def needs_meddra(variable_name: str) -> bool:
    return variable_name.upper() in _MEDDRA_VARIABLES


def get_decode_value(variable_name: str, coded_value: str) -> str:
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


def get_code_list_oid(variable: SDTMVariable, domain_code: str) -> str:
    name = variable.name.upper()
    if name == "RDOMAIN":
        return "CL.RDOMAIN"
    if needs_meddra(variable.name):
        domain = (domain_code or "GEN").upper()
        return f"CL.{domain}.MEDDRA"
    return f"CL.{domain_code.upper()}.{variable.name}"


def build_supp_value_lists(
    dataset: pd.DataFrame | None, domain: SDTMDomain
) -> tuple[
    list[ValueListDefinition],
    list[WhereClauseDefinition],
    dict[str, tuple[SDTMVariable, str]],
    str | None,
]:
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
                dataset_name=domain.dataset_name or domain.code,
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
                mandatory="No",
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
    parent: XmlElement, value_lists: Sequence[ValueListDefinition]
) -> None:
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
                    "OrderNumber": str(item.order_number or ""),
                    "Mandatory": item.mandatory or "No",
                },
            )
            ET.SubElement(
                item_ref,
                tag(DEF_NS, "WhereClauseRef"),
                attrib={"WhereClauseOID": item.where_clause_oid or ""},
            )

            method_oid = getattr(item, "method_oid", None)
            if method_oid:
                item_ref.set("MethodOID", method_oid)


def append_where_clause_defs(
    parent: XmlElement, where_clauses: Sequence[WhereClauseDefinition]
) -> None:
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
                "SoftHard": getattr(wc, "soft_hard", "Soft"),
            },
        )
        item_oid = getattr(wc, "item_oid", None) or wc.variable_oid
        range_check.set(attr(DEF_NS, "ItemOID"), item_oid)

        for value in wc.check_values:
            ET.SubElement(range_check, tag(ODM_NS, "CheckValue")).text = value


def append_method_defs(parent: XmlElement, methods: Sequence[MethodDefinition]) -> None:
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
            attrib={attr(XML_NS, "lang"): "en"},
        ).text = method.description

        for doc_ref in method.document_refs:
            ET.SubElement(
                method_elem,
                tag(DEF_NS, "DocumentRef"),
                attrib={"leafID": doc_ref},
            )


def append_comment_defs(
    parent: XmlElement, comments: Sequence[CommentDefinition]
) -> None:
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
            attrib={attr(XML_NS, "lang"): "en"},
        ).text = comment.text
