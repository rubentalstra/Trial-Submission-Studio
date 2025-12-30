//! Dataset-XML output generation.

use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use chrono::{SecondsFormat, Utc};
use polars::prelude::AnyValue;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};

use sdtm_ingest::any_to_string_non_empty;
use sdtm_model::Domain;
use sdtm_transform::{DomainFrame, domain_map_by_code};

use crate::common::{
    DATASET_XML_NS, DATASET_XML_VERSION, DEFINE_XML_VERSION, ODM_NS, XLINK_NS, ensure_output_dir,
    ensure_parent_dir, is_reference_domain, normalize_study_id,
};

/// Options for Dataset-XML output.
#[derive(Debug, Clone, Default)]
pub struct DatasetXmlOptions {
    pub dataset_name: Option<String>,
    pub metadata_version_oid: Option<String>,
    pub is_reference_data: Option<bool>,
}

/// Write Dataset-XML outputs for all domains.
pub fn write_dataset_xml_outputs(
    output_dir: &Path,
    domains: &[Domain],
    frames: &[DomainFrame],
    study_id: &str,
    sdtm_ig_version: &str,
) -> Result<Vec<PathBuf>> {
    let domain_lookup = domain_map_by_code(domains);
    let mut frames_sorted: Vec<&DomainFrame> = frames.iter().collect();
    frames_sorted.sort_by(|a, b| a.domain_code.cmp(&b.domain_code));

    let xml_dir = ensure_output_dir(output_dir, "dataset-xml")?;

    let mut outputs = Vec::new();
    for frame in frames_sorted {
        let code = frame.domain_code.to_uppercase();
        let domain = domain_lookup
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

/// Write a single Dataset-XML file.
pub fn write_dataset_xml(
    output_path: &Path,
    domain: &Domain,
    frame: &DomainFrame,
    study_id: &str,
    sdtm_ig_version: &str,
    options: Option<&DatasetXmlOptions>,
) -> Result<()> {
    let options = options.cloned().unwrap_or_default();
    let dataset_name = options
        .dataset_name
        .unwrap_or_else(|| crate::common::dataset_name(domain));
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
    ensure_parent_dir(output_path)?;
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
