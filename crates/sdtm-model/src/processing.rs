//! Study processing request and response types.
//!
//! This module provides the data structures for requesting and receiving
//! results from study processing operations.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::conformance::ValidationReport;

/// Supported output formats for SDTM data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum OutputFormat {
    /// SAS Transport (XPT) v5 format.
    Xpt,
    /// CDISC Dataset-XML format.
    Xml,
    /// SAS program output.
    Sas,
}

/// Paths to generated output files for a domain.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutputPaths {
    /// Path to XPT file (if generated).
    pub xpt: Option<PathBuf>,
    /// Path to Dataset-XML file (if generated).
    pub dataset_xml: Option<PathBuf>,
    /// Path to SAS program file (if generated).
    pub sas: Option<PathBuf>,
}

/// Result of processing a single domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainResult {
    /// Domain code (e.g., "AE", "DM").
    pub domain_code: String,
    /// Number of records in the output dataset.
    pub records: usize,
    /// Paths to generated output files.
    pub output_paths: OutputPaths,
    /// Validation report (if validation was performed).
    pub validation_report: Option<ValidationReport>,
    /// Errors encountered during processing.
    pub errors: Vec<String>,
}

/// Error information for a specific domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyError {
    /// Domain code where error occurred.
    pub domain_code: String,
    /// Error message.
    pub message: String,
}

/// Request to process a study.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStudyRequest {
    /// Path to study data folder.
    pub study_folder: PathBuf,
    /// Output directory for generated files.
    pub output_dir: PathBuf,
    /// Study identifier.
    pub study_id: String,
    /// Desired output formats.
    pub output_formats: Vec<OutputFormat>,
    /// Verbosity level (0-3).
    pub verbose: u8,
}

/// Response from study processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStudyResponse {
    /// True if processing completed successfully.
    pub success: bool,
    /// Study identifier.
    pub study_id: String,
    /// Output directory where files were written.
    pub output_dir: PathBuf,
    /// Results for each processed domain.
    pub domain_results: Vec<DomainResult>,
    /// Errors that occurred during processing.
    pub errors: Vec<StudyError>,
}
