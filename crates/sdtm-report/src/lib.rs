use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use chrono::{SecondsFormat, Utc};
use polars::prelude::{AnyValue, DataFrame};
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};

use sdtm_core::{DomainFrame, order_variables_by_role, standard_columns};
use sdtm_ingest::{any_to_f64_for_output, any_to_string_for_output, any_to_string_non_empty};
use sdtm_model::ct::{Codelist, TerminologyCatalog, TerminologyRegistry};
use sdtm_model::{Domain, MappingConfig, Variable, VariableType};
use sdtm_standards::load_default_ct_registry;
use sdtm_xpt::{XptColumn, XptDataset, XptType, XptValue, XptWriterOptions, write_xpt};

const SAS_NUMERIC_LEN: u16 = 8;
const ODM_NS: &str = "http://www.cdisc.org/ns/odm/v1.3";
const DATASET_XML_NS: &str = "http://www.cdisc.org/ns/Dataset-XML/v1.0";
const DEFINE_XML_NS: &str = "http://www.cdisc.org/ns/def/v2.1";
const XLINK_NS: &str = "http://www.w3.org/1999/xlink";
const DATASET_XML_VERSION: &str = "1.0";
const DEFINE_XML_VERSION: &str = "2.1";

#[derive(Debug, Clone, Default)]
pub struct DatasetXmlOptions {
    pub dataset_name: Option<String>,
    pub metadata_version_oid: Option<String>,
    pub is_reference_data: Option<bool>,
}

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

#[derive(Debug, Clone, Default)]
pub struct SasProgramOptions {
    pub input_dataset: Option<String>,
    pub output_dataset: Option<String>,
}

pub fn write_xpt_outputs(
    output_dir: &Path,
    domains: &[Domain],
    frames: &[DomainFrame],
    options: &XptWriterOptions,
) -> Result<Vec<PathBuf>> {
    let mut domain_map = BTreeMap::new();
    for domain in domains {
        domain_map.insert(domain.code.to_uppercase(), domain);
    }

    let mut frames_sorted: Vec<&DomainFrame> = frames.iter().collect();
    frames_sorted.sort_by(|a, b| a.domain_code.cmp(&b.domain_code));

    let xpt_dir = output_dir.join("xpt");
    std::fs::create_dir_all(&xpt_dir).with_context(|| format!("create {}", xpt_dir.display()))?;

    let mut outputs = Vec::new();
    for frame in frames_sorted {
        let code = frame.domain_code.to_uppercase();
        let domain = domain_map
            .get(&code)
            .ok_or_else(|| anyhow!("missing domain definition for {code}"))?;
        // Use frame's dataset name (from metadata) for split domains, falling back to domain.code
        let output_dataset_name = frame.dataset_name();
        let dataset = build_xpt_dataset_with_name(domain, frame, &output_dataset_name)?;
        let disk_name = output_dataset_name.to_lowercase();
        let filename = format!("{disk_name}.xpt");
        let path = xpt_dir.join(filename);
        write_xpt(&path, &dataset, options)?;
        outputs.push(path);
    }
    Ok(outputs)
}

pub fn write_dataset_xml_outputs(
    output_dir: &Path,
    domains: &[Domain],
    frames: &[DomainFrame],
    study_id: &str,
    sdtm_ig_version: &str,
) -> Result<Vec<PathBuf>> {
    let domain_map = domain_map(domains);
    let mut frames_sorted: Vec<&DomainFrame> = frames.iter().collect();
    frames_sorted.sort_by(|a, b| a.domain_code.cmp(&b.domain_code));

    let xml_dir = output_dir.join("dataset-xml");
    std::fs::create_dir_all(&xml_dir).with_context(|| format!("create {}", xml_dir.display()))?;

    let mut outputs = Vec::new();
    for frame in frames_sorted {
        let code = frame.domain_code.to_uppercase();
        let domain = domain_map
            .get(&code)
            .ok_or_else(|| anyhow!("missing domain definition for {code}"))?;
        // Use frame's dataset name (from metadata) for split domains
        let output_dataset_name = frame.dataset_name();
        let disk_name = output_dataset_name.to_lowercase();
        let path = xml_dir.join(format!("{disk_name}.xml"));
        let options = DatasetXmlOptions {
            dataset_name: Some(output_dataset_name),
            ..Default::default()
        };
        write_dataset_xml(
            &path,
            domain,
            frame,
            study_id,
            sdtm_ig_version,
            Some(&options),
        )?;
        outputs.push(path);
    }
    Ok(outputs)
}

pub fn write_dataset_xml(
    output_path: &Path,
    domain: &Domain,
    frame: &DomainFrame,
    study_id: &str,
    sdtm_ig_version: &str,
    options: Option<&DatasetXmlOptions>,
) -> Result<()> {
    let options = options.cloned().unwrap_or_default();
    let dataset_name = options.dataset_name.unwrap_or_else(|| dataset_name(domain));
    let study_id = normalize_study_id(study_id);
    let study_oid = format!("STDY.{study_id}");
    let mdv_oid = options
        .metadata_version_oid
        .unwrap_or_else(|| format!("MDV.{study_oid}.SDTMIG.{sdtm_ig_version}"));
    let define_file_oid = format!("{study_oid}.Define-XML_{DEFINE_XML_VERSION}");
    let file_oid = format!("{define_file_oid}(IG.{dataset_name})");
    let is_reference = options
        .is_reference_data
        .unwrap_or_else(|| is_reference_domain(domain));
    let container_name = if is_reference {
        "ReferenceData"
    } else {
        "ClinicalData"
    };

    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let file =
        File::create(output_path).with_context(|| format!("create {}", output_path.display()))?;
    let writer = BufWriter::new(file);
    let mut xml = Writer::new_with_indent(writer, b' ', 2);

    xml.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let mut root = BytesStart::new("ODM");
    root.push_attribute(("xmlns", ODM_NS));
    root.push_attribute(("xmlns:xlink", XLINK_NS));
    root.push_attribute(("xmlns:data", DATASET_XML_NS));
    root.push_attribute(("data:DatasetXMLVersion", DATASET_XML_VERSION));
    root.push_attribute(("FileType", "Snapshot"));
    root.push_attribute(("FileOID", file_oid.as_str()));
    root.push_attribute(("PriorFileOID", define_file_oid.as_str()));
    root.push_attribute(("ODMVersion", "1.3.2"));
    root.push_attribute(("CreationDateTime", timestamp.as_str()));
    root.push_attribute(("Originator", "CDISC-Transpiler"));
    xml.write_event(Event::Start(root))?;

    let mut container = BytesStart::new(container_name);
    container.push_attribute(("StudyOID", study_oid.as_str()));
    container.push_attribute(("MetaDataVersionOID", mdv_oid.as_str()));
    xml.write_event(Event::Start(container))?;

    let df = &frame.data;
    let mut columns = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        let series = df
            .column(variable.name.as_str())
            .with_context(|| format!("missing column {}", variable.name))?;
        columns.push(series);
    }

    for row_idx in 0..df.height() {
        let mut group = BytesStart::new("ItemGroupData");
        let group_oid = format!("IG.{dataset_name}");
        let group_seq = format!("{}", row_idx + 1);
        group.push_attribute(("ItemGroupOID", group_oid.as_str()));
        group.push_attribute(("data:ItemGroupDataSeq", group_seq.as_str()));
        xml.write_event(Event::Start(group))?;
        for (variable, column) in domain.variables.iter().zip(columns.iter()) {
            let value = column.get(row_idx).unwrap_or(AnyValue::Null);
            if let Some(text) = any_to_string_non_empty(value) {
                let mut item = BytesStart::new("ItemData");
                let item_oid = format!("IT.{dataset_name}.{}", variable.name);
                item.push_attribute(("ItemOID", item_oid.as_str()));
                item.push_attribute(("Value", text.as_str()));
                xml.write_event(Event::Empty(item))?;
            }
        }
        xml.write_event(Event::End(BytesEnd::new("ItemGroupData")))?;
    }

    xml.write_event(Event::End(BytesEnd::new(container_name)))?;
    xml.write_event(Event::End(BytesEnd::new("ODM")))?;
    Ok(())
}

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

    let domain_map = domain_map(domains);
    let mut entries: Vec<(&Domain, &DomainFrame)> = Vec::new();
    for frame in frames {
        let code = frame.domain_code.to_uppercase();
        let domain = domain_map
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
        // Use frame's dataset name for split domains in OIDs
        let output_dataset_name = frame.dataset_name();
        for variable in &domain.variables {
            let oid = format!("IT.{}.{}", output_dataset_name, variable.name);
            let length = match variable.data_type {
                VariableType::Char => Some(variable_length(variable, &frame.data)?),
                VariableType::Num => None,
            };
            let codelist_oid = resolve_codelist(
                domain,
                variable,
                &ct_registry,
                &mut code_lists,
                &mut ct_standards,
            )?;

            // Check if variable has any non-null/non-empty values
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
        // Use frame's dataset name for split domains (e.g., LBCH, FAAE)
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

        // Order variables by SDTM role per SDTMIG v3.4 Chapter 2.1:
        // Identifiers, Topic, Qualifiers (Grouping, Result, Synonym, Record, Variable), Rule, Timing
        let ordered_vars = order_variables_by_role(&domain.variables);

        let mut key_sequence = 1usize;
        for (idx, variable) in ordered_vars.iter().enumerate() {
            let mut item_ref = BytesStart::new("ItemRef");
            // Use dataset name for ItemOID reference to match ItemDef OIDs
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

        // Per SDTMIG 2.5: Add def:Origin element to indicate data provenance.
        // For Expected variables that are uncollected, use Type="Not Collected"
        // to document that the variable was included per requirements but no data exists.
        let origin_type = if is_expected(item_def.core.as_deref()) && !item_def.has_data {
            "Not Collected"
        } else if item_def.has_data {
            "Collected"
        } else {
            "Derived" // Default for variables with no data that aren't Expected
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

pub fn write_sas_outputs(
    output_dir: &Path,
    domains: &[Domain],
    frames: &[DomainFrame],
    mappings: &BTreeMap<String, MappingConfig>,
    options: &SasProgramOptions,
) -> Result<Vec<PathBuf>> {
    let domain_map = domain_map(domains);
    let mut frames_sorted: Vec<&DomainFrame> = frames.iter().collect();
    frames_sorted.sort_by(|a, b| a.domain_code.cmp(&b.domain_code));

    let sas_dir = output_dir.join("sas");
    std::fs::create_dir_all(&sas_dir).with_context(|| format!("create {}", sas_dir.display()))?;

    let mut outputs = Vec::new();
    for frame in frames_sorted {
        let code = frame.domain_code.to_uppercase();
        let domain = domain_map
            .get(&code)
            .ok_or_else(|| anyhow!("missing domain definition for {code}"))?;
        let mapping = mappings
            .get(&code)
            .ok_or_else(|| anyhow!("missing mapping config for {code}"))?;
        // Use frame's dataset name (from metadata) for split domains
        let output_dataset_name = frame.dataset_name();
        let disk_name = output_dataset_name.to_lowercase();
        let path = sas_dir.join(format!("{disk_name}.sas"));
        let sas_options = SasProgramOptions {
            output_dataset: Some(format!("sdtm.{}", disk_name)),
            ..options.clone()
        };
        let program = generate_sas_program(domain, frame, mapping, &sas_options)?;
        std::fs::write(&path, program).with_context(|| format!("write {}", path.display()))?;
        outputs.push(path);
    }
    Ok(outputs)
}

pub fn generate_sas_program(
    domain: &Domain,
    frame: &DomainFrame,
    mapping: &MappingConfig,
    options: &SasProgramOptions,
) -> Result<String> {
    let input_dataset = options
        .input_dataset
        .clone()
        .unwrap_or_else(|| format!("work.{}", domain.code.to_lowercase()));
    let output_dataset = options
        .output_dataset
        .clone()
        .unwrap_or_else(|| format!("sdtm.{}", dataset_name(domain).to_lowercase()));
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    let mut lines = Vec::new();
    lines.push("/* Generated by CDISC Transpiler */".to_string());
    lines.push(format!("/* Domain: {} */", domain.code));
    lines.push(format!("/* Generated: {} */", timestamp));
    lines.push(String::new());
    lines.push(format!("DATA {output_dataset};"));
    lines.push(format!("    SET {input_dataset};"));
    lines.push("    length".to_string());
    for variable in &domain.variables {
        let length = variable_length(variable, &frame.data)?;
        let suffix = if variable.data_type == VariableType::Char {
            format!("${length}")
        } else {
            format!("{length}")
        };
        lines.push(format!("        {} {}", variable.name, suffix));
    }
    lines.push("    ;".to_string());
    lines.push(String::new());
    lines.push("    /* Column mappings */".to_string());
    let assignment_map = build_assignment_map(domain, mapping);
    for assignment in assignment_map {
        for line in assignment.lines() {
            lines.push(format!("    {line}"));
        }
    }
    lines.push(String::new());
    lines.push("    /* Defaulted required fields */".to_string());
    for default_line in default_assignments(domain, mapping) {
        lines.push(format!("    {default_line}"));
    }
    let standard = standard_columns(domain);
    if let Some(study_col) = standard.study_id.as_ref() {
        lines.push(format!("    {study_col} = \"{}\";", mapping.study_id));
    }
    if let Some(domain_col) = standard.domain.as_ref() {
        lines.push(format!("    {domain_col} = \"{}\";", domain.code));
    }
    lines.push(String::new());
    lines.push(format!("    KEEP {};", keep_clause(domain)));
    lines.push("RUN;".to_string());
    Ok(lines.join("\n"))
}

/// Build XPT dataset with an explicit dataset name.
///
/// This variant allows specifying the dataset name directly, useful for:
/// - Split domains (e.g., LBCH, FAAE) where the name comes from frame metadata
/// - Custom output naming requirements
pub fn build_xpt_dataset_with_name(
    domain: &Domain,
    frame: &DomainFrame,
    dataset_name: &str,
) -> Result<XptDataset> {
    let df = &frame.data;
    let columns = build_xpt_columns(domain, df)?;
    let rows = build_xpt_rows(domain, df)?;
    Ok(XptDataset {
        name: dataset_name.to_uppercase(),
        label: domain.label.clone(),
        columns,
        rows,
    })
}

fn build_xpt_columns(domain: &Domain, df: &DataFrame) -> Result<Vec<XptColumn>> {
    let mut columns = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        let length = variable_length(variable, df)?;
        columns.push(XptColumn {
            name: variable.name.clone(),
            label: variable.label.clone(),
            data_type: match variable.data_type {
                VariableType::Num => XptType::Num,
                VariableType::Char => XptType::Char,
            },
            length,
        });
    }
    Ok(columns)
}

fn build_xpt_rows(domain: &Domain, df: &DataFrame) -> Result<Vec<Vec<XptValue>>> {
    let mut series = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        let col = df
            .column(variable.name.as_str())
            .with_context(|| format!("missing column {}", variable.name))?;
        series.push(col);
    }

    let row_count = df.height();
    let mut rows = Vec::with_capacity(row_count);
    for row_idx in 0..row_count {
        let mut row = Vec::with_capacity(series.len());
        for (variable, column) in domain.variables.iter().zip(series.iter()) {
            let value = column.get(row_idx).unwrap_or(AnyValue::Null);
            let cell = match variable.data_type {
                VariableType::Num => XptValue::Num(any_to_f64_for_output(value)),
                VariableType::Char => XptValue::Char(any_to_string_for_output(value)),
            };
            row.push(cell);
        }
        rows.push(row);
    }
    Ok(rows)
}

fn variable_length(variable: &Variable, df: &DataFrame) -> Result<u16> {
    if let Some(length) = variable.length {
        if length == 0 {
            return Err(anyhow!("variable {} has zero length", variable.name));
        }
        return Ok(length.min(u16::MAX as u32) as u16);
    }
    match variable.data_type {
        VariableType::Num => Ok(SAS_NUMERIC_LEN),
        VariableType::Char => {
            let series = df
                .column(variable.name.as_str())
                .with_context(|| format!("missing column {}", variable.name))?;
            let mut max_len = 0usize;
            for idx in 0..df.height() {
                let value = series.get(idx).unwrap_or(AnyValue::Null);
                let text = any_to_string_for_output(value);
                let len = text.trim_end().len();
                if len > max_len {
                    max_len = len;
                }
            }
            let len = max_len.max(1);
            if len > u16::MAX as usize {
                return Err(anyhow!("variable {} length too large", variable.name));
            }
            Ok(len as u16)
        }
    }
}

trait VariableTypeExt {
    fn as_define_type(&self) -> &'static str;
}

impl VariableTypeExt for VariableType {
    fn as_define_type(&self) -> &'static str {
        match self {
            VariableType::Char => "text",
            VariableType::Num => "float",
        }
    }
}

#[derive(Debug, Clone)]
struct ItemDefSpec {
    oid: String,
    name: String,
    label: Option<String>,
    data_type: VariableType,
    length: Option<u16>,
    codelist_oid: Option<String>,
    /// Core designation: "Req", "Exp", or "Perm"
    core: Option<String>,
    /// True if the variable has any non-null values
    has_data: bool,
}

#[derive(Debug, Clone)]
struct CodeListSpec {
    name: String,
    values: Vec<String>,
    extensible: bool,
    /// Reference to CT standard OID (e.g., "STD.CT.SDTM.2024-03-29")
    standard_oid: Option<String>,
}

/// CT Standard definition for Define-XML def:Standards section
#[derive(Debug, Clone)]
struct CtStandard {
    oid: String,
    name: String,
    publishing_set: String,
    version: String,
}

fn domain_map(domains: &[Domain]) -> BTreeMap<String, &Domain> {
    let mut map = BTreeMap::new();
    for domain in domains {
        map.insert(domain.code.to_uppercase(), domain);
    }
    map
}

fn dataset_name(domain: &Domain) -> String {
    domain
        .dataset_name
        .clone()
        .unwrap_or_else(|| domain.code.clone())
}

fn normalize_study_id(study_id: &str) -> String {
    let trimmed = study_id.trim();
    if trimmed.is_empty() {
        "STUDY".to_string()
    } else {
        trimmed.to_string()
    }
}

fn is_reference_domain(domain: &Domain) -> bool {
    let class_name = match domain.class_name.as_ref() {
        Some(value) => value,
        None => return false,
    };
    let normalized = normalize_class(class_name);
    normalized == "TRIAL DESIGN" || normalized == "STUDY REFERENCE"
}

fn normalize_class(value: &str) -> String {
    let mut out = String::new();
    let mut last_space = false;
    for ch in value.chars() {
        let c = if ch == '-' || ch == '_' { ' ' } else { ch };
        let upper = c.to_ascii_uppercase();
        if upper == ' ' {
            if !last_space {
                out.push(' ');
                last_space = true;
            }
        } else {
            out.push(upper);
            last_space = false;
        }
    }
    out.trim().to_string()
}

fn write_text_element<W: Write>(writer: &mut Writer<W>, name: &str, text: &str) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new(name)))?;
    writer.write_event(Event::Text(BytesText::new(text)))?;
    writer.write_event(Event::End(BytesEnd::new(name)))?;
    Ok(())
}

fn write_translated_text<W: Write>(
    writer: &mut Writer<W>,
    wrapper: &str,
    text: &str,
) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new(wrapper)))?;
    let mut translated = BytesStart::new("TranslatedText");
    translated.push_attribute(("xml:lang", "en"));
    writer.write_event(Event::Start(translated))?;
    writer.write_event(Event::Text(BytesText::new(text)))?;
    writer.write_event(Event::End(BytesEnd::new("TranslatedText")))?;
    writer.write_event(Event::End(BytesEnd::new(wrapper)))?;
    Ok(())
}

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
        // Try to resolve by first codelist code
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

    // Determine the standard OID for this codelist
    let standard_oid =
        ct_entries
            .first()
            .and_then(|(_, catalog): &(&Codelist, Option<&TerminologyCatalog>)| {
                catalog.and_then(|cat| {
                    let publishing_set = cat.publishing_set.as_ref()?;
                    let version = cat.version.as_ref()?;
                    let oid = format!("STD.CT.{}.{}", publishing_set, version);

                    // Register the CT standard if not already present
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

fn parse_codelist_codes(raw: &str) -> Vec<String> {
    raw.split([';', ','])
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect()
}

fn build_assignment_map(domain: &Domain, mapping: &MappingConfig) -> Vec<String> {
    let mut variable_lookup = BTreeMap::new();
    for variable in &domain.variables {
        variable_lookup.insert(variable.name.to_uppercase(), variable);
    }
    let mut assignments = Vec::new();
    for item in &mapping.mappings {
        let variable = variable_lookup
            .get(&item.target_variable.to_uppercase())
            .copied();
        assignments.push(render_assignment(item, variable));
    }
    assignments
}

fn render_assignment(
    mapping: &sdtm_model::MappingSuggestion,
    variable: Option<&Variable>,
) -> String {
    let mut expr = mapping
        .transformation
        .clone()
        .unwrap_or_else(|| mapping.source_column.clone());
    if let Some(var) = variable
        && var.data_type == VariableType::Char
    {
        expr = format!("strip(coalescec({}, ''))", expr);
        if should_upcase(var) {
            expr = format!("upcase({expr})");
        }
    }
    format!("{} = {};", mapping.target_variable, expr)
}

fn default_assignments(domain: &Domain, mapping: &MappingConfig) -> Vec<String> {
    let mut mapped: BTreeSet<String> = mapping
        .mappings
        .iter()
        .map(|item| item.target_variable.to_uppercase())
        .collect();
    let mut defaults = Vec::new();
    for variable in &domain.variables {
        if !is_required(variable) {
            continue;
        }
        if mapped.contains(&variable.name.to_uppercase()) {
            continue;
        }
        defaults.push(default_assignment(variable));
        mapped.insert(variable.name.to_uppercase());
    }
    defaults
}

fn default_assignment(variable: &Variable) -> String {
    match variable.data_type {
        VariableType::Num => format!("{} = .;", variable.name),
        VariableType::Char => format!("{} = '';", variable.name),
    }
}

fn keep_clause(domain: &Domain) -> String {
    domain
        .variables
        .iter()
        .map(|var| var.name.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_required(variable: &Variable) -> bool {
    variable
        .core
        .as_deref()
        .map(|v| v.eq_ignore_ascii_case("req"))
        .unwrap_or(false)
}

fn is_identifier(variable: &Variable) -> bool {
    variable
        .role
        .as_deref()
        .map(|v| v.eq_ignore_ascii_case("identifier"))
        .unwrap_or(false)
}

fn should_upcase(variable: &Variable) -> bool {
    is_identifier(variable) || variable.codelist_code.is_some()
}

fn is_expected(core: Option<&str>) -> bool {
    core.map(|v| v.trim().eq_ignore_ascii_case("exp"))
        .unwrap_or(false)
}

/// Check if a variable column has any non-null/non-empty values (i.e., was "collected").
fn has_collected_data(df: &DataFrame, variable_name: &str) -> bool {
    let series = match df.column(variable_name) {
        Ok(s) => s,
        Err(_) => return false,
    };

    for idx in 0..df.height() {
        if let Ok(value) = series.get(idx) {
            match value {
                AnyValue::Null => continue,
                AnyValue::String(s) if s.trim().is_empty() => continue,
                AnyValue::StringOwned(ref s) if s.as_str().trim().is_empty() => continue,
                _ => return true, // Found a non-null, non-empty value
            }
        }
    }
    false
}
