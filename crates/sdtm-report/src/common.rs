//! Shared utilities and types for SDTM report generation.

use std::io::Write;

use anyhow::{Context, Result, anyhow};
use polars::prelude::{AnyValue, DataFrame};
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

use sdtm_ingest::any_to_string;
use sdtm_model::{Domain, Variable, VariableType};

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
        .unwrap_or_else(|| domain.code.clone())
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
    let class_name = match domain.class_name.as_ref() {
        Some(value) => value,
        None => return false,
    };
    let normalized = normalize_class(class_name);
    normalized == "TRIAL DESIGN" || normalized == "STUDY REFERENCE"
}

/// Normalize class name for comparison.
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
        VariableType::Char | _ => {
            // Treat Char and any future types as variable-length strings
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

/// Check if variable is required (Core = "Req").
pub fn is_required(variable: &Variable) -> bool {
    variable
        .core
        .as_deref()
        .map(|v| v.eq_ignore_ascii_case("req"))
        .unwrap_or(false)
}

/// Check if variable is an identifier.
pub fn is_identifier(variable: &Variable) -> bool {
    variable
        .role
        .as_deref()
        .map(|v| v.eq_ignore_ascii_case("identifier"))
        .unwrap_or(false)
}

/// Check if variable should be upcased.
pub fn should_upcase(variable: &Variable) -> bool {
    is_identifier(variable) || variable.codelist_code.is_some()
}

/// Check if variable is expected (Core = "Exp").
pub fn is_expected(core: Option<&str>) -> bool {
    core.map(|v| v.trim().eq_ignore_ascii_case("exp"))
        .unwrap_or(false)
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
            // Future types default to text
            _ => "text",
        }
    }
}

#[cfg(test)]
mod tests {
    use polars::prelude::{Column, IntoColumn, NamedFrom, Series};

    use super::*;

    fn test_df(columns: Vec<(&str, Vec<&str>)>) -> DataFrame {
        let cols: Vec<Column> = columns
            .into_iter()
            .map(|(name, values)| {
                Series::new(
                    name.into(),
                    values.iter().copied().map(String::from).collect::<Vec<_>>(),
                )
                .into_column()
            })
            .collect();
        DataFrame::new(cols).unwrap()
    }

    fn test_variable(name: &str, data_type: VariableType) -> Variable {
        Variable {
            name: name.to_string(),
            label: Some(format!("{} Label", name)),
            data_type,
            length: None,
            role: None,
            core: None,
            codelist_code: None,
            order: None,
        }
    }

    fn test_domain(code: &str, class_name: Option<&str>) -> Domain {
        Domain {
            code: code.to_string(),
            description: Some(format!("{} Domain", code)),
            class_name: class_name.map(String::from),
            dataset_class: None,
            label: Some(format!("{} Label", code)),
            structure: None,
            dataset_name: None,
            variables: vec![],
        }
    }

    #[test]
    fn test_dataset_name_uses_code_when_no_override() {
        let domain = test_domain("AE", None);
        assert_eq!(dataset_name(&domain), "AE");
    }

    #[test]
    fn test_dataset_name_uses_override() {
        let mut domain = test_domain("FA", None);
        domain.dataset_name = Some("FACM".to_string());
        assert_eq!(dataset_name(&domain), "FACM");
    }

    #[test]
    fn test_normalize_study_id_trims_whitespace() {
        assert_eq!(normalize_study_id("  STUDY01  "), "STUDY01");
    }

    #[test]
    fn test_normalize_study_id_defaults_empty() {
        assert_eq!(normalize_study_id(""), "STUDY");
        assert_eq!(normalize_study_id("   "), "STUDY");
    }

    #[test]
    fn test_is_reference_domain_trial_design() {
        let domain = test_domain("TA", Some("Trial Design"));
        assert!(is_reference_domain(&domain));
    }

    #[test]
    fn test_is_reference_domain_study_reference() {
        let domain = test_domain("OI", Some("Study Reference"));
        assert!(is_reference_domain(&domain));
    }

    #[test]
    fn test_is_reference_domain_findings() {
        let domain = test_domain("AE", Some("Events"));
        assert!(!is_reference_domain(&domain));
    }

    #[test]
    fn test_is_reference_domain_none_class() {
        let domain = test_domain("AE", None);
        assert!(!is_reference_domain(&domain));
    }

    #[test]
    fn test_is_reference_domain_normalizes_class_name() {
        let domain = test_domain("TA", Some("trial-design"));
        assert!(is_reference_domain(&domain));

        let domain2 = test_domain("TA", Some("TRIAL_DESIGN"));
        assert!(is_reference_domain(&domain2));
    }

    #[test]
    fn test_variable_length_uses_explicit_length() {
        let df = test_df(vec![("AETERM", vec!["short"])]);
        let mut variable = test_variable("AETERM", VariableType::Char);
        variable.length = Some(200);

        let length = variable_length(&variable, &df).unwrap();
        assert_eq!(length, 200);
    }

    #[test]
    fn test_variable_length_numeric_always_8() {
        let df = test_df(vec![("AESTDY", vec!["1"])]);
        let variable = test_variable("AESTDY", VariableType::Num);

        let length = variable_length(&variable, &df).unwrap();
        assert_eq!(length, SAS_NUMERIC_LEN);
    }

    #[test]
    fn test_variable_length_computes_max_from_data() {
        let df = test_df(vec![("AETERM", vec!["short", "medium length", "x"])]);
        let variable = test_variable("AETERM", VariableType::Char);

        let length = variable_length(&variable, &df).unwrap();
        assert_eq!(length, 13); // "medium length" has 13 chars
    }

    #[test]
    fn test_variable_length_minimum_one() {
        let df = test_df(vec![("AETERM", vec!["", "", ""])]);
        let variable = test_variable("AETERM", VariableType::Char);

        let length = variable_length(&variable, &df).unwrap();
        assert_eq!(length, 1);
    }

    #[test]
    fn test_is_required() {
        let mut variable = test_variable("USUBJID", VariableType::Char);
        variable.core = Some("Req".to_string());
        assert!(is_required(&variable));

        variable.core = Some("REQ".to_string());
        assert!(is_required(&variable));

        variable.core = Some("Exp".to_string());
        assert!(!is_required(&variable));

        variable.core = None;
        assert!(!is_required(&variable));
    }

    #[test]
    fn test_is_identifier() {
        let mut variable = test_variable("USUBJID", VariableType::Char);
        variable.role = Some("Identifier".to_string());
        assert!(is_identifier(&variable));

        variable.role = Some("IDENTIFIER".to_string());
        assert!(is_identifier(&variable));

        variable.role = Some("Topic".to_string());
        assert!(!is_identifier(&variable));

        variable.role = None;
        assert!(!is_identifier(&variable));
    }

    #[test]
    fn test_should_upcase_identifier() {
        let mut variable = test_variable("USUBJID", VariableType::Char);
        variable.role = Some("Identifier".to_string());
        assert!(should_upcase(&variable));
    }

    #[test]
    fn test_should_upcase_with_codelist() {
        let mut variable = test_variable("SEX", VariableType::Char);
        variable.codelist_code = Some("C66731".to_string());
        assert!(should_upcase(&variable));
    }

    #[test]
    fn test_should_upcase_neither() {
        let variable = test_variable("AETERM", VariableType::Char);
        assert!(!should_upcase(&variable));
    }

    #[test]
    fn test_is_expected() {
        assert!(is_expected(Some("Exp")));
        assert!(is_expected(Some("EXP")));
        assert!(is_expected(Some("  exp  ")));
        assert!(!is_expected(Some("Req")));
        assert!(!is_expected(None));
    }

    #[test]
    fn test_has_collected_data_with_values() {
        let df = test_df(vec![("AETERM", vec!["Headache", "", "Nausea"])]);
        assert!(has_collected_data(&df, "AETERM"));
    }

    #[test]
    fn test_has_collected_data_all_empty() {
        let df = test_df(vec![("AETERM", vec!["", "  ", ""])]);
        assert!(!has_collected_data(&df, "AETERM"));
    }

    #[test]
    fn test_has_collected_data_missing_column() {
        let df = test_df(vec![("AETERM", vec!["Headache"])]);
        assert!(!has_collected_data(&df, "AEOTHER"));
    }

    #[test]
    fn test_variable_type_ext() {
        assert_eq!(VariableType::Char.as_define_type(), "text");
        assert_eq!(VariableType::Num.as_define_type(), "float");
    }
}
