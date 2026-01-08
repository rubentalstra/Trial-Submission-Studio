//! Background export thread.
//!
//! Handles the actual export process in a background thread with cancellation support.

use super::supp::{build_supp_domain_definition, build_supp_frame, has_supp_columns};
use super::types::{
    ExportConfig, ExportError, ExportHandle, ExportResult, ExportStep, ExportUpdate,
};
use crate::settings::ExportFormat;
use crate::state::StudyState;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tss_model::Domain;
use tss_output::types::DomainFrame;
use tss_output::{
    DatasetXmlOptions, DefineXmlOptions, write_dataset_xml as write_dataset_xml_impl,
    write_define_xml as write_define_xml_impl,
};

/// Spawn a background export thread.
///
/// Returns a handle that can be used to cancel the export.
pub fn spawn_export(
    config: ExportConfig,
    study: StudyState,
    sender: Sender<ExportUpdate>,
) -> ExportHandle {
    let handle = ExportHandle::new();
    let cancel_flag = handle.cancel_flag();
    let written_files = handle.written_files();

    std::thread::spawn(move || {
        let result = execute_export(&config, &study, &sender, &cancel_flag, &written_files);

        match result {
            Ok(result) => {
                let _ = sender.send(ExportUpdate::Complete { result });
            }
            Err(_) if cancel_flag.load(Ordering::SeqCst) => {
                // Cleanup partial files on cancel
                if let Ok(files) = written_files.lock() {
                    for path in files.iter() {
                        let _ = std::fs::remove_file(path);
                    }
                }
                let _ = sender.send(ExportUpdate::Cancelled);
            }
            Err(error) => {
                let _ = sender.send(ExportUpdate::Error { error });
            }
        }
    });

    handle
}

/// Execute the export process.
fn execute_export(
    config: &ExportConfig,
    study: &StudyState,
    sender: &Sender<ExportUpdate>,
    cancel_flag: &Arc<AtomicBool>,
    written_files: &Arc<Mutex<Vec<PathBuf>>>,
) -> Result<ExportResult, ExportError> {
    let start = Instant::now();

    // Create output directory structure: {output_dir}/datasets/
    let datasets_dir = config.output_dir.join("datasets");
    std::fs::create_dir_all(&datasets_dir)
        .map_err(|e| ExportError::new(format!("Failed to create output directory: {}", e)))?;

    // Extract study_id from the first selected domain's STUDYID column
    let study_id = config
        .selected_domains
        .iter()
        .next()
        .and_then(|code| study.get_domain(code))
        .and_then(|d| d.derived.preview.as_ref())
        .and_then(|df| df.column("STUDYID").ok())
        .and_then(|col| col.get(0).ok())
        .map(|v| format!("{}", v).trim_matches('"').to_string())
        .unwrap_or_else(|| "STUDY".to_string());

    let mut domain_frames: Vec<DomainFrame> = Vec::new();
    let mut supp_frames: Vec<DomainFrame> = Vec::new();

    // Process each selected domain
    for domain_code in &config.selected_domains {
        // Check for cancellation
        if cancel_flag.load(Ordering::SeqCst) {
            return Err(ExportError::new("Cancelled"));
        }

        // Get domain state
        let domain = study.get_domain(domain_code).ok_or_else(|| {
            ExportError::for_domain(domain_code, ExportStep::Preparing, "Domain not found")
        })?;

        // Get the preview data
        let preview_df = domain.derived.preview.as_ref().ok_or_else(|| {
            ExportError::for_domain(
                domain_code,
                ExportStep::Preparing,
                "No preview data - run preview first",
            )
        })?;

        // Build domain frame
        sender
            .send(ExportUpdate::Progress {
                domain: domain_code.clone(),
                step: ExportStep::ApplyingMappings,
            })
            .ok();

        let frame = DomainFrame::new(domain_code.clone(), preview_df.clone());

        // Build variable metadata from domain definition
        let variable_metadata = build_variable_metadata(domain.mapping.domain());

        // Write data file (lowercase filename)
        sender
            .send(ExportUpdate::Progress {
                domain: domain_code.clone(),
                step: ExportStep::WritingFile,
            })
            .ok();

        let filename = format!(
            "{}.{}",
            domain_code.to_lowercase(),
            config.format.extension()
        );
        let path = datasets_dir.join(&filename);
        let domain_def = domain.mapping.domain();
        write_data_file(
            &path,
            &frame,
            domain_def,
            &study_id,
            config.format,
            &variable_metadata,
        )?;

        // Track written file
        if let Ok(mut files) = written_files.lock() {
            files.push(path.clone());
        }
        sender.send(ExportUpdate::FileWritten { path }).ok();

        // Generate SUPP if needed
        if has_supp_columns(domain) {
            sender
                .send(ExportUpdate::Progress {
                    domain: domain_code.clone(),
                    step: ExportStep::GeneratingSUPP,
                })
                .ok();

            if let Some(supp_frame) = build_supp_frame(domain_code, domain, preview_df) {
                let supp_filename = format!(
                    "supp{}.{}",
                    domain_code.to_lowercase(),
                    config.format.extension()
                );
                let supp_path = datasets_dir.join(&supp_filename);
                // Use SUPP-specific metadata (standard SUPP variables)
                let supp_metadata = build_supp_variable_metadata();
                // Get SUPP domain definition from standards for Dataset-XML
                let supp_domain = build_supp_domain_definition(domain_code);
                if let Some(ref supp_def) = supp_domain {
                    write_data_file(
                        &supp_path,
                        &supp_frame,
                        supp_def,
                        &study_id,
                        config.format,
                        &supp_metadata,
                    )?;
                } else {
                    // Fallback: XPT doesn't need domain definition
                    if config.format == ExportFormat::Xpt {
                        write_xpt_file(&supp_path, &supp_frame, &supp_metadata)?;
                    }
                }

                if let Ok(mut files) = written_files.lock() {
                    files.push(supp_path.clone());
                }
                sender
                    .send(ExportUpdate::FileWritten { path: supp_path })
                    .ok();

                supp_frames.push(supp_frame);
            }
        }

        domain_frames.push(frame);
    }

    // Check for cancellation before Define-XML
    if cancel_flag.load(Ordering::SeqCst) {
        return Err(ExportError::new("Cancelled"));
    }

    // Write Define-XML (always generated)
    sender
        .send(ExportUpdate::Progress {
            domain: "Define-XML".to_string(),
            step: ExportStep::WritingDefineXml,
        })
        .ok();

    let define_path = datasets_dir.join("define.xml");
    write_define_xml(&define_path, study, &domain_frames, &supp_frames)?;

    if let Ok(mut files) = written_files.lock() {
        files.push(define_path.clone());
    }
    sender
        .send(ExportUpdate::FileWritten { path: define_path })
        .ok();

    // Build result
    let written = written_files.lock().map(|f| f.clone()).unwrap_or_default();

    Ok(ExportResult {
        output_dir: config.output_dir.clone(),
        written_files: written,
        elapsed_ms: start.elapsed().as_millis() as u64,
    })
}

/// Write a data file in the specified format.
fn write_data_file(
    path: &Path,
    frame: &DomainFrame,
    domain: &Domain,
    study_id: &str,
    format: ExportFormat,
    variable_metadata: &HashMap<String, VariableMetadata>,
) -> Result<(), ExportError> {
    match format {
        ExportFormat::Xpt => write_xpt_file(path, frame, variable_metadata),
        ExportFormat::DatasetXml => write_dataset_xml_file(path, frame, domain, study_id),
    }
}

/// Variable metadata for XPT export.
#[derive(Debug, Clone, Default)]
struct VariableMetadata {
    /// Human-readable label (max 40 chars).
    label: Option<String>,
    /// SAS format (e.g., "$200.", "8.", "DATE9.").
    format: Option<String>,
}

/// Build variable metadata map from domain definition.
fn build_variable_metadata(domain: &Domain) -> HashMap<String, VariableMetadata> {
    domain
        .variables
        .iter()
        .map(|v| {
            let name = v.name.to_uppercase();
            let label = v.label.clone();
            // Derive SAS format from data type and length
            let format = match v.data_type {
                tss_model::VariableType::Char => {
                    let len = v.length.unwrap_or(200).min(200);
                    Some(format!("${len}."))
                }
                tss_model::VariableType::Num => Some("8.".to_string()),
            };
            (name, VariableMetadata { label, format })
        })
        .collect()
}

/// Parse a SAS format string into components.
///
/// Examples:
/// - "$200." -> (Some("$"), 200, 0)
/// - "8." -> (None, 8, 0)
/// - "DATE9." -> (Some("DATE"), 9, 0)
/// - "12.2" -> (None, 12, 2)
fn parse_sas_format(format: &Option<String>) -> (Option<String>, u16, u16) {
    let Some(fmt) = format else {
        return (None, 0, 0);
    };

    let fmt = fmt.trim();
    if fmt.is_empty() {
        return (None, 0, 0);
    }

    // Character format: $<length>.
    if fmt.starts_with('$') {
        let len_str = fmt.trim_start_matches('$').trim_end_matches('.');
        let length = len_str.parse::<u16>().unwrap_or(8);
        return (Some("$".to_string()), length, 0);
    }

    // Named format: NAME<length>. or NAME<length>.<decimals>
    // Find where digits start
    let name_end = fmt.chars().position(|c| c.is_ascii_digit()).unwrap_or(0);

    if name_end > 0 {
        // Named format (e.g., DATE9., BEST12.)
        let name = fmt[..name_end].to_string();
        let rest = fmt[name_end..].trim_end_matches('.');
        let parts: Vec<&str> = rest.split('.').collect();
        let length = parts.first().and_then(|s| s.parse().ok()).unwrap_or(8);
        let decimals = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        (Some(name), length, decimals)
    } else {
        // Numeric format: <length>. or <length>.<decimals>
        let rest = fmt.trim_end_matches('.');
        let parts: Vec<&str> = rest.split('.').collect();
        let length = parts.first().and_then(|s| s.parse().ok()).unwrap_or(8);
        let decimals = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        (None, length, decimals)
    }
}

/// Build variable metadata for SUPP domains.
///
/// SUPP domains have standard variables per SDTM IG 3.4 Section 8.4.
fn build_supp_variable_metadata() -> HashMap<String, VariableMetadata> {
    let supp_vars = [
        ("STUDYID", "Study Identifier", "$20."),
        ("RDOMAIN", "Related Domain Abbreviation", "$2."),
        ("USUBJID", "Unique Subject Identifier", "$40."),
        ("IDVAR", "Identifying Variable", "$8."),
        ("IDVARVAL", "Identifying Variable Value", "$40."),
        ("QNAM", "Qualifier Variable Name", "$8."),
        ("QLABEL", "Qualifier Variable Label", "$40."),
        ("QVAL", "Data Value", "$200."),
        ("QORIG", "Origin", "$40."),
        ("QEVAL", "Evaluator", "$40."),
    ];

    supp_vars
        .iter()
        .map(|(name, label, format)| {
            (
                name.to_string(),
                VariableMetadata {
                    label: Some(label.to_string()),
                    format: Some(format.to_string()),
                },
            )
        })
        .collect()
}

/// Write an XPT file.
fn write_xpt_file(
    path: &Path,
    frame: &DomainFrame,
    variable_metadata: &HashMap<String, VariableMetadata>,
) -> Result<(), ExportError> {
    use polars::prelude::{AnyValue, DataType};
    use xportrs::{
        Justification, MissingValue, NumericValue, XptColumn, XptDataset, XptType, XptValue,
        write_xpt,
    };

    // Build columns
    let mut columns = Vec::new();
    for col in frame.data.get_columns() {
        let name = col.name().to_uppercase();
        let dtype = col.dtype();

        // Look up variable metadata for label and format
        let metadata = variable_metadata.get(&name);

        // Determine type and length
        let (data_type, length) = match dtype {
            DataType::Float64
            | DataType::Float32
            | DataType::Int64
            | DataType::Int32
            | DataType::Int16
            | DataType::Int8
            | DataType::UInt64
            | DataType::UInt32
            | DataType::UInt16
            | DataType::UInt8 => (XptType::Num, 8),
            _ => {
                // String type - calculate max length
                let max_len = if let Ok(str_col) = col.str() {
                    str_col
                        .iter()
                        .flatten()
                        .map(|s| s.len())
                        .max()
                        .unwrap_or(8)
                        .clamp(1, 200) // Cap at 200 for XPT
                } else {
                    8
                };
                (XptType::Char, max_len as u16)
            }
        };

        // Get label from metadata, fall back to variable name
        let label = metadata
            .and_then(|m| m.label.clone())
            .unwrap_or_else(|| name.clone());

        // Get format from metadata, or derive default
        let format = metadata
            .and_then(|m| m.format.clone())
            .or_else(|| match data_type {
                XptType::Char => Some(format!("${length}.")),
                XptType::Num => Some("8.".to_string()),
                _ => Some("8.".to_string()), // Default for future XptType variants
            });

        // Parse format into name, length, decimals
        let (format_name, format_length, format_decimals) = parse_sas_format(&format);

        // Justification: Left for character, Right for numeric
        let justification = match data_type {
            XptType::Char => Justification::Left,
            XptType::Num => Justification::Right,
            _ => Justification::Right, // Default for future XptType variants
        };

        columns.push(XptColumn {
            name: name.clone(),
            label: Some(label),
            data_type,
            length,
            format: format_name,
            format_length,
            format_decimals,
            // Informat: Not typically needed for export (data already in memory)
            informat: None,
            informat_length: 0,
            informat_decimals: 0,
            justification,
        });
    }

    // Build rows
    let mut rows = Vec::with_capacity(frame.data.height());
    for row_idx in 0..frame.data.height() {
        let mut row = Vec::with_capacity(columns.len());
        for (col_idx, col) in frame.data.get_columns().iter().enumerate() {
            let value = col.get(row_idx).ok();
            let xpt_type = columns[col_idx].data_type;

            let xpt_value = match (xpt_type, value) {
                (XptType::Num, Some(AnyValue::Float64(n))) => XptValue::Num(NumericValue::Value(n)),
                (XptType::Num, Some(AnyValue::Float32(n))) => {
                    XptValue::Num(NumericValue::Value(n as f64))
                }
                (XptType::Num, Some(AnyValue::Int64(n))) => {
                    XptValue::Num(NumericValue::Value(n as f64))
                }
                (XptType::Num, Some(AnyValue::Int32(n))) => {
                    XptValue::Num(NumericValue::Value(n as f64))
                }
                (XptType::Num, Some(AnyValue::Int16(n))) => {
                    XptValue::Num(NumericValue::Value(n as f64))
                }
                (XptType::Num, Some(AnyValue::Int8(n))) => {
                    XptValue::Num(NumericValue::Value(n as f64))
                }
                (XptType::Num, Some(AnyValue::UInt64(n))) => {
                    XptValue::Num(NumericValue::Value(n as f64))
                }
                (XptType::Num, Some(AnyValue::UInt32(n))) => {
                    XptValue::Num(NumericValue::Value(n as f64))
                }
                (XptType::Num, Some(AnyValue::UInt16(n))) => {
                    XptValue::Num(NumericValue::Value(n as f64))
                }
                (XptType::Num, Some(AnyValue::UInt8(n))) => {
                    XptValue::Num(NumericValue::Value(n as f64))
                }
                (XptType::Num, _) => XptValue::Num(NumericValue::Missing(MissingValue::Standard)),
                (XptType::Char, Some(AnyValue::String(s))) => XptValue::Char(s.to_string()),
                (XptType::Char, Some(AnyValue::StringOwned(s))) => XptValue::Char(s.to_string()),
                (XptType::Char, Some(AnyValue::Null)) | (XptType::Char, None) => {
                    XptValue::Char(String::new())
                }
                (XptType::Char, Some(other)) => XptValue::Char(format!("{}", other)),
                (_, _) => XptValue::Num(NumericValue::Missing(MissingValue::Standard)), // Default for future variants
            };
            row.push(xpt_value);
        }
        rows.push(row);
    }

    // Create dataset
    let dataset = XptDataset {
        name: frame.dataset_name().to_uppercase(),
        label: Some(frame.domain_code.clone()),
        dataset_type: None,
        columns,
        rows,
    };

    // Write to file
    write_xpt(path, &dataset)
        .map_err(|e| ExportError::new(format!("Failed to write XPT file: {}", e)))?;

    Ok(())
}

/// Write a Dataset-XML file.
fn write_dataset_xml_file(
    path: &Path,
    frame: &DomainFrame,
    domain: &Domain,
    study_id: &str,
) -> Result<(), ExportError> {
    // TODO: SDTM-IG version (made configurable)
    let sdtm_ig_version = "3.4";

    let options = DatasetXmlOptions {
        dataset_name: Some(frame.dataset_name()),
        ..Default::default()
    };

    write_dataset_xml_impl(
        path,
        domain,
        frame,
        study_id,
        sdtm_ig_version,
        Some(&options),
    )
    .map_err(|e| ExportError::new(format!("Failed to write Dataset-XML: {}", e)))?;

    Ok(())
}

/// Write Define-XML file.
fn write_define_xml(
    path: &Path,
    study: &StudyState,
    domain_frames: &[DomainFrame],
    supp_frames: &[DomainFrame],
) -> Result<(), ExportError> {
    // Collect domain definitions from the study
    let mut domains: Vec<Domain> = Vec::new();
    let mut all_frames: Vec<DomainFrame> = Vec::new();

    // Add main domain definitions and frames
    for frame in domain_frames {
        if let Some(domain_state) = study.get_domain(&frame.domain_code) {
            domains.push(domain_state.mapping.domain().clone());
            all_frames.push(frame.clone());
        }
    }

    // Add SUPP domain definitions and frames
    for supp_frame in supp_frames {
        // Extract parent domain code from SUPP frame (e.g., "suppdm" -> "DM")
        let parent_code = supp_frame
            .domain_code
            .strip_prefix("SUPP")
            .or_else(|| supp_frame.domain_code.strip_prefix("supp"))
            .unwrap_or(&supp_frame.domain_code)
            .to_uppercase();

        // Create SUPP domain definition from standards
        if let Some(supp_domain) = build_supp_domain_definition(&parent_code) {
            domains.push(supp_domain);
        }
        all_frames.push(supp_frame.clone());
    }

    // Get study ID from the first domain's STUDYID column or use default
    let study_id = domain_frames
        .first()
        .and_then(|f| {
            f.data
                .column("STUDYID")
                .ok()
                .and_then(|col| col.get(0).ok())
                .map(|v| format!("{}", v).trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "STUDY".to_string());

    // Create options with SDTM IG version
    // TODO: SDTM-IG version (made configurable)
    let options = DefineXmlOptions::new("3.4", "Submission");

    // Call the actual Define-XML writer
    write_define_xml_impl(path, &study_id, &domains, &all_frames, &options)
        .map_err(|e| ExportError::new(format!("Failed to write Define-XML: {}", e)))?;

    Ok(())
}

/// Extension trait for ExportFormat.
impl ExportFormat {
    /// Get file extension for format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Xpt => "xpt",
            Self::DatasetXml => "xml",
        }
    }
}
