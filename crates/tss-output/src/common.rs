//! Shared utilities and types for SDTM report generation.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use polars::prelude::{AnyValue, DataFrame};
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

use tss_model::any_to_string;
use tss_model::{CoreDesignation, Domain, Variable, VariableRole, VariableType};

/// SAS numeric length constant (8 bytes).
pub const SAS_NUMERIC_LEN: u16 = 8;

/// ODM namespace.
pub const ODM_NS: &str = "http://www.cdisc.org/ns/odm/v1.3";

/// Dataset-XML namespace.
pub const DATASET_XML_NS: &str = "http://www.cdisc.org/ns/Dataset-XML/v1.0";

/// Define-XML namespace.
pub const DEFINE_XML_NS: &str = "http://www.cdisc.org/ns/def/v2.1";

/// XLink namespace.
pub const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

/// Dataset-XML version.
pub const DATASET_XML_VERSION: &str = "1.0";

/// Define-XML version.
pub const DEFINE_XML_VERSION: &str = "2.1";

/// Get dataset name from domain.
pub fn dataset_name(domain: &Domain) -> String {
    domain
        .dataset_name
        .clone()
        .unwrap_or_else(|| domain.name.clone())
}

/// Ensure a parent directory exists for a file path.
pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    Ok(())
}

/// Ensure an output subdirectory exists and return its path.
pub fn ensure_output_dir(base_dir: &Path, name: &str) -> Result<PathBuf> {
    let dir = base_dir.join(name);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    Ok(dir)
}

/// Normalize study ID, defaulting to "STUDY" if empty.
pub fn normalize_study_id(study_id: &str) -> String {
    let trimmed = study_id.trim();
    if trimmed.is_empty() {
        "STUDY".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Check if domain is a reference domain (Trial Design or Study Reference).
pub fn is_reference_domain(domain: &Domain) -> bool {
    matches!(
        domain.class,
        Some(tss_model::DatasetClass::TrialDesign) | Some(tss_model::DatasetClass::StudyReference)
    )
}

/// Calculate variable length from data.
pub fn variable_length(variable: &Variable, df: &DataFrame) -> Result<u16> {
    if let Some(length) = variable.length {
        if length == 0 {
            return Err(anyhow!("variable {} has zero length", variable.name));
        }
        return Ok(length.min(u16::MAX as u32) as u16);
    }
    match variable.data_type {
        VariableType::Num => Ok(SAS_NUMERIC_LEN),
        VariableType::Char => {
            // Treat Char as variable-length strings
            let series = df
                .column(variable.name.as_str())
                .with_context(|| format!("missing column {}", variable.name))?;
            let mut max_len = 0usize;
            for idx in 0..df.height() {
                let value = series.get(idx).unwrap_or(AnyValue::Null);
                let text = any_to_string(value);
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

/// Check if variable is required (Core = Required).
pub fn is_required(variable: &Variable) -> bool {
    variable.core == Some(CoreDesignation::Required)
}

/// Check if variable is an identifier.
pub fn is_identifier(variable: &Variable) -> bool {
    variable.role == Some(VariableRole::Identifier)
}

/// Check if variable should be upcased.
pub fn should_upcase(variable: &Variable) -> bool {
    is_identifier(variable) || variable.codelist_code.is_some()
}

/// Check if variable is expected (Core = Expected).
pub fn is_expected(core: Option<CoreDesignation>) -> bool {
    core == Some(CoreDesignation::Expected)
}

/// Check if a variable column has any non-null/non-empty values (i.e., was "collected").
pub fn has_collected_data(df: &DataFrame, variable_name: &str) -> bool {
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
                _ => return true,
            }
        }
    }
    false
}

/// Write a simple text element.
pub fn write_text_element<W: Write>(writer: &mut Writer<W>, name: &str, text: &str) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new(name)))?;
    writer.write_event(Event::Text(BytesText::new(text)))?;
    writer.write_event(Event::End(BytesEnd::new(name)))?;
    Ok(())
}

/// Write a translated text element (with xml:lang="en").
pub fn write_translated_text<W: Write>(
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

/// Extension trait for VariableType to get Define-XML type.
pub trait VariableTypeExt {
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
