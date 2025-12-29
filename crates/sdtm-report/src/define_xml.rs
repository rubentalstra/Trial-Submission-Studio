//! Define-XML output generation.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use chrono::{SecondsFormat, Utc};
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};

use sdtm_model::ct::{Codelist, TerminologyCatalog, TerminologyRegistry};
use sdtm_model::{Domain, Variable, VariableType};
use sdtm_standards::load_default_ct_registry;
use sdtm_transform::frame::DomainFrame;

use sdtm_transform::domain_sets::domain_map_by_code;

use crate::common::{
    DEFINE_XML_NS, DEFINE_XML_VERSION, ODM_NS, VariableTypeExt, XLINK_NS, has_collected_data,
    is_expected, is_identifier, is_reference_domain, is_required, normalize_study_id,
    variable_length, write_text_element, write_translated_text,
};

/// Options for Define-XML output.
#[derive(Debug, Clone)]
pub struct DefineXmlOptions {
    pub sdtm_ig_version: String,
    pub context: String,
}

impl DefineXmlOptions {
    pub fn new(sdtm_ig_version: impl Into<String>, context: impl Into<String>) -> Self {
        Self {
            sdtm_ig_version: sdtm_ig_version.into(),
            context: context.into(),
        }
    }
}

/// Item definition specification for Define-XML.
#[derive(Debug, Clone)]
struct ItemDefSpec {
    oid: String,
    name: String,
    label: Option<String>,
    data_type: VariableType,
    length: Option<u16>,
    codelist_oid: Option<String>,
    core: Option<String>,
    has_data: bool,
}

/// Codelist specification for Define-XML.
#[derive(Debug, Clone)]
struct CodeListSpec {
    name: String,
    values: Vec<String>,
    extensible: bool,
    standard_oid: Option<String>,
}

/// CT Standard definition for Define-XML def:Standards section.
#[derive(Debug, Clone)]
struct CtStandard {
    oid: String,
    name: String,
    publishing_set: String,
    version: String,
}

/// Write Define-XML output.
pub fn write_define_xml(
    output_path: &Path,
    study_id: &str,
    domains: &[Domain],
    frames: &[DomainFrame],
    options: &DefineXmlOptions,
) -> Result<()> {
    if frames.is_empty() {
        return Err(anyhow!("no datasets supplied for define-xml"));
    }
    let study_id = normalize_study_id(study_id);
    let study_oid = format!("STDY.{study_id}");
    let file_oid = format!("{study_oid}.Define-XML_{DEFINE_XML_VERSION}");
    let mdv_oid = format!("MDV.{study_oid}.SDTMIG.{}", options.sdtm_ig_version);
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    let domain_lookup = domain_map_by_code(domains);
    let mut entries: Vec<(&Domain, &DomainFrame)> = Vec::new();
    for frame in frames {
        let code = frame.domain_code.to_uppercase();
        let domain = domain_lookup
            .get(&code)
            .ok_or_else(|| anyhow!("missing domain definition for {code}"))?;
        entries.push((domain, frame));
    }
    entries.sort_by(|a, b| a.0.code.cmp(&b.0.code));

    let ct_registry = load_default_ct_registry().context("load ct registry")?;
    let mut item_defs: BTreeMap<String, ItemDefSpec> = BTreeMap::new();
    let mut code_lists: BTreeMap<String, CodeListSpec> = BTreeMap::new();
    let mut ct_standards: BTreeMap<String, CtStandard> = BTreeMap::new();

    for (domain, frame) in &entries {
        let output_dataset_name = frame.dataset_name();
        for variable in &domain.variables {
            let oid = format!("IT.{}.{}", output_dataset_name, variable.name);
            let length = match variable.data_type {
                VariableType::Char => Some(variable_length(variable, &frame.data)?),
                VariableType::Num => None,
                // Future types default to having a length
                _ => Some(variable_length(variable, &frame.data)?),
            };
            let codelist_oid = resolve_codelist(
                domain,
                variable,
                &ct_registry,
                &mut code_lists,
                &mut ct_standards,
            )?;
            let has_data = has_collected_data(&frame.data, &variable.name);

            item_defs.insert(
                oid.clone(),
                ItemDefSpec {
                    oid,
                    name: variable.name.clone(),
                    label: variable.label.clone(),
                    data_type: variable.data_type,
                    length,
                    codelist_oid,
                    core: variable.core.clone(),
                    has_data,
                },
            );
        }
    }

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
    }
    let file =
        File::create(output_path).with_context(|| format!("create {}", output_path.display()))?;
    let writer = BufWriter::new(file);
    let mut xml = Writer::new_with_indent(writer, b' ', 2);

    xml.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let mut root = BytesStart::new("ODM");
    root.push_attribute(("xmlns", ODM_NS));
    root.push_attribute(("xmlns:def", DEFINE_XML_NS));
    root.push_attribute(("xmlns:xlink", XLINK_NS));
    root.push_attribute(("FileType", "Snapshot"));
    root.push_attribute(("FileOID", file_oid.as_str()));
    root.push_attribute(("ODMVersion", "1.3.2"));
    root.push_attribute(("CreationDateTime", timestamp.as_str()));
    root.push_attribute(("Originator", "CDISC-Transpiler"));
    root.push_attribute(("SourceSystem", "CDISC-Transpiler"));
    root.push_attribute(("SourceSystemVersion", "1.0"));
    root.push_attribute(("def:Context", options.context.as_str()));
    xml.write_event(Event::Start(root))?;

    let mut study = BytesStart::new("Study");
    study.push_attribute(("OID", study_oid.as_str()));
    xml.write_event(Event::Start(study))?;

    xml.write_event(Event::Start(BytesStart::new("GlobalVariables")))?;
    write_text_element(&mut xml, "StudyName", &study_id)?;
    write_text_element(
        &mut xml,
        "StudyDescription",
        &format!("SDTM submission for {study_id}"),
    )?;
    write_text_element(&mut xml, "ProtocolName", &study_id)?;
    xml.write_event(Event::End(BytesEnd::new("GlobalVariables")))?;

    let mut metadata = BytesStart::new("MetaDataVersion");
    metadata.push_attribute(("OID", mdv_oid.as_str()));
    let mdv_name = format!("Study {study_id}, Data Definitions");
    let mdv_desc = format!(
        "SDTM {} metadata definitions for {study_id}",
        options.sdtm_ig_version
    );
    metadata.push_attribute(("Name", mdv_name.as_str()));
    metadata.push_attribute(("Description", mdv_desc.as_str()));
    metadata.push_attribute(("def:DefineVersion", DEFINE_XML_VERSION));
    xml.write_event(Event::Start(metadata))?;

    // Write def:Standards section with CT versions
    if !ct_standards.is_empty() {
        xml.write_event(Event::Start(BytesStart::new("def:Standards")))?;
        for standard in ct_standards.values() {
            let mut std_node = BytesStart::new("def:Standard");
            std_node.push_attribute(("OID", standard.oid.as_str()));
            std_node.push_attribute(("Name", standard.name.as_str()));
            std_node.push_attribute(("Type", "CT"));
            std_node.push_attribute(("PublishingSet", standard.publishing_set.as_str()));
            std_node.push_attribute(("Version", standard.version.as_str()));
            std_node.push_attribute(("Status", "Final"));
            xml.write_event(Event::Empty(std_node))?;
        }
        xml.write_event(Event::End(BytesEnd::new("def:Standards")))?;
    }

    for (domain, frame) in &entries {
        let output_dataset_name = frame.dataset_name();
        let base_domain_code = frame.base_domain_code();
        let mut ig = BytesStart::new("ItemGroupDef");
        let ig_oid = format!("IG.{}", output_dataset_name);
        let sas_dataset_name: String = output_dataset_name.chars().take(8).collect();
        ig.push_attribute(("OID", ig_oid.as_str()));
        ig.push_attribute(("Name", output_dataset_name.as_str()));
        ig.push_attribute(("Repeating", "Yes"));
        ig.push_attribute(("Domain", base_domain_code));
        ig.push_attribute(("SASDatasetName", sas_dataset_name.as_str()));
        if let Some(label) = domain.label.as_ref() {
            ig.push_attribute(("def:Label", label.as_str()));
        }
        if let Some(class_name) = domain.class_name.as_ref() {
            ig.push_attribute(("def:Class", class_name.as_str()));
        }
        if let Some(structure) = domain.structure.as_ref() {
            ig.push_attribute(("def:Structure", structure.as_str()));
        }
        if is_reference_domain(domain) {
            ig.push_attribute(("def:IsReferenceData", "Yes"));
        }
        xml.write_event(Event::Start(ig))?;

        let ordered_vars = domain.variables_by_role();

        let mut key_sequence = 1usize;
        for (idx, variable) in ordered_vars.iter().enumerate() {
            let mut item_ref = BytesStart::new("ItemRef");
            let item_oid = format!("IT.{}.{}", output_dataset_name, variable.name);
            let order_number = format!("{}", idx + 1);
            item_ref.push_attribute(("ItemOID", item_oid.as_str()));
            item_ref.push_attribute(("OrderNumber", order_number.as_str()));
            item_ref.push_attribute((
                "Mandatory",
                if is_required(variable) { "Yes" } else { "No" },
            ));
            if is_identifier(variable) {
                let seq = format!("{key_sequence}");
                item_ref.push_attribute(("KeySequence", seq.as_str()));
                key_sequence += 1;
            }
            xml.write_event(Event::Empty(item_ref))?;
        }
        xml.write_event(Event::End(BytesEnd::new("ItemGroupDef")))?;
    }

    for item_def in item_defs.values() {
        let mut item = BytesStart::new("ItemDef");
        item.push_attribute(("OID", item_def.oid.as_str()));
        item.push_attribute(("Name", item_def.name.as_str()));
        item.push_attribute(("DataType", item_def.data_type.as_define_type()));
        if let Some(length) = item_def.length {
            let length_text = format!("{length}");
            item.push_attribute(("Length", length_text.as_str()));
        }
        xml.write_event(Event::Start(item))?;
        if let Some(label) = item_def.label.as_ref() {
            write_translated_text(&mut xml, "Description", label)?;
        }
        if let Some(codelist_oid) = item_def.codelist_oid.as_ref() {
            let mut ref_node = BytesStart::new("CodeListRef");
            ref_node.push_attribute(("CodeListOID", codelist_oid.as_str()));
            xml.write_event(Event::Empty(ref_node))?;
        }

        let origin_type = if is_expected(item_def.core.as_deref()) && !item_def.has_data {
            "Not Collected"
        } else if item_def.has_data {
            "Collected"
        } else {
            "Derived"
        };
        let mut origin = BytesStart::new("def:Origin");
        origin.push_attribute(("Type", origin_type));
        xml.write_event(Event::Empty(origin))?;

        xml.write_event(Event::End(BytesEnd::new("ItemDef")))?;
    }

    for (oid, list) in code_lists {
        let mut node = BytesStart::new("CodeList");
        node.push_attribute(("OID", oid.as_str()));
        node.push_attribute(("Name", list.name.as_str()));
        node.push_attribute(("DataType", "text"));
        if let Some(std_oid) = list.standard_oid.as_ref() {
            node.push_attribute(("def:StandardOID", std_oid.as_str()));
        }
        if list.extensible {
            node.push_attribute(("def:Extensible", "Yes"));
        }
        xml.write_event(Event::Start(node))?;
        for value in list.values {
            let mut item = BytesStart::new("CodeListItem");
            item.push_attribute(("CodedValue", value.as_str()));
            xml.write_event(Event::Start(item))?;
            write_translated_text(&mut xml, "Decode", &value)?;
            xml.write_event(Event::End(BytesEnd::new("CodeListItem")))?;
        }
        xml.write_event(Event::End(BytesEnd::new("CodeList")))?;
    }

    xml.write_event(Event::End(BytesEnd::new("MetaDataVersion")))?;
    xml.write_event(Event::End(BytesEnd::new("Study")))?;
    xml.write_event(Event::End(BytesEnd::new("ODM")))?;
    Ok(())
}

/// Resolve codelist for a variable.
fn resolve_codelist(
    domain: &Domain,
    variable: &Variable,
    ct_registry: &TerminologyRegistry,
    code_lists: &mut BTreeMap<String, CodeListSpec>,
    ct_standards: &mut BTreeMap<String, CtStandard>,
) -> Result<Option<String>> {
    let mut ct_entries: Vec<(&Codelist, Option<&TerminologyCatalog>)> = Vec::new();

    if let Some(raw) = variable.codelist_code.as_ref() {
        let codes = parse_codelist_codes(raw);
        for code in codes {
            if let Some(resolved) = ct_registry.resolve(&code, None) {
                ct_entries.push((resolved.codelist, Some(resolved.catalog)));
            }
        }
    }
    if ct_entries.is_empty() {
        if let Some(raw) = variable.codelist_code.as_ref() {
            let code = raw.split(';').next().unwrap_or("").trim();
            if !code.is_empty() {
                if let Some(resolved) = ct_registry.resolve(code, None) {
                    ct_entries.push((resolved.codelist, Some(resolved.catalog)));
                } else {
                    return Err(anyhow!(
                        "missing codelist {} for {}.{}",
                        raw,
                        domain.code,
                        variable.name
                    ));
                }
            }
        }
        if ct_entries.is_empty() {
            return Ok(None);
        }
    }

    let standard_oid =
        ct_entries
            .first()
            .and_then(|(_, catalog): &(&Codelist, Option<&TerminologyCatalog>)| {
                catalog.and_then(|cat| {
                    let publishing_set = cat.publishing_set.as_ref()?;
                    let version = cat.version.as_ref()?;
                    let oid = format!("STD.CT.{}.{}", publishing_set, version);

                    ct_standards
                        .entry(oid.clone())
                        .or_insert_with(|| CtStandard {
                            oid: oid.clone(),
                            name: "CDISC/NCI".to_string(),
                            publishing_set: publishing_set.clone(),
                            version: version.clone(),
                        });

                    Some(oid)
                })
            });

    let oid = format!("CL.{}.{}", domain.code, variable.name);
    if !code_lists.contains_key(&oid) {
        let mut values = BTreeSet::new();
        let mut names = BTreeSet::new();
        let mut extensible = false;
        for (ct, _) in &ct_entries {
            names.insert(ct.name.clone());
            extensible |= ct.extensible;
            for value in ct.submission_values() {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    values.insert(trimmed.to_string());
                }
            }
        }
        let name = names.into_iter().collect::<Vec<_>>().join("; ");
        code_lists.insert(
            oid.clone(),
            CodeListSpec {
                name,
                values: values.into_iter().collect(),
                extensible,
                standard_oid,
            },
        );
    }
    Ok(Some(oid))
}

/// Parse codelist codes from semicolon/comma-separated string.
fn parse_codelist_codes(raw: &str) -> Vec<String> {
    raw.split([';', ','])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(std::string::ToString::to_string)
        .collect()
}
