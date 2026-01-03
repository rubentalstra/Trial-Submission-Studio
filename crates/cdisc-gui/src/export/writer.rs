//! Background export thread.
//!
//! Handles the actual export process in a background thread with cancellation support.

use super::supp::{build_supp_frame, has_supp_columns};
use super::types::{
    ExportConfig, ExportError, ExportHandle, ExportResult, ExportStep, ExportUpdate,
};
use crate::settings::ExportFormat;
use crate::state::StudyState;
use cdisc_output::types::DomainFrame;
use crossbeam_channel::Sender;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

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
        write_data_file(&path, &frame, config.format)?;

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
                write_data_file(&supp_path, &supp_frame, config.format)?;

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
    format: ExportFormat,
) -> Result<(), ExportError> {
    match format {
        ExportFormat::Xpt => write_xpt_file(path, frame),
        ExportFormat::DatasetXml => write_dataset_xml_file(path, frame),
    }
}

/// Write an XPT file.
fn write_xpt_file(path: &Path, frame: &DomainFrame) -> Result<(), ExportError> {
    use cdisc_xpt::{
        MissingValue, NumericValue, XptColumn, XptDataset, XptType, XptValue, write_xpt,
    };
    use polars::prelude::{AnyValue, DataType};

    // Build columns
    let mut columns = Vec::new();
    for col in frame.data.get_columns() {
        let name = col.name().to_uppercase();
        let dtype = col.dtype();

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
                        .into_iter()
                        .filter_map(|s| s.map(|s| s.len()))
                        .max()
                        .unwrap_or(8)
                        .max(1)
                        .min(200) // Cap at 200 for XPT
                } else {
                    8
                };
                (XptType::Char, max_len as u16)
            }
        };

        columns.push(XptColumn {
            name: name.clone(),
            label: Some(name), // Use name as label for now
            data_type,
            length,
            format: None,
            format_length: 0,
            format_decimals: 0,
            informat: None,
            informat_length: 0,
            informat_decimals: 0,
            justification: Default::default(),
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
fn write_dataset_xml_file(path: &Path, frame: &DomainFrame) -> Result<(), ExportError> {
    // TODO: Implement Dataset-XML writing
    // For now, create a placeholder file
    std::fs::write(
        path,
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!-- Dataset-XML for {} -->\n",
            frame.domain_code
        ),
    )
    .map_err(|e| ExportError::new(format!("Failed to write Dataset-XML file: {}", e)))?;

    Ok(())
}

/// Write Define-XML file.
fn write_define_xml(
    path: &Path,
    _study: &StudyState,
    _domain_frames: &[DomainFrame],
    _supp_frames: &[DomainFrame],
) -> Result<(), ExportError> {
    // TODO: Implement Define-XML generation
    // For now, create a placeholder file
    std::fs::write(
        path,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!-- Define-XML placeholder -->\n",
    )
    .map_err(|e| ExportError::new(format!("Failed to write Define-XML file: {}", e)))?;

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
