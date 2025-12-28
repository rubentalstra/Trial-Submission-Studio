use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::conformance::ValidationReport;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Xpt,
    Xml,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutputPaths {
    pub xpt: Option<PathBuf>,
    pub dataset_xml: Option<PathBuf>,
    pub sas: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainResult {
    pub domain_code: String,
    pub records: usize,
    pub output_paths: OutputPaths,
    pub validation_report: Option<ValidationReport>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyError {
    pub domain_code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStudyRequest {
    pub study_folder: PathBuf,
    pub output_dir: PathBuf,
    pub study_id: String,
    pub output_formats: Vec<OutputFormat>,
    pub verbose: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStudyResponse {
    pub success: bool,
    pub study_id: String,
    pub output_dir: PathBuf,
    pub domain_results: Vec<DomainResult>,
    pub errors: Vec<StudyError>,
}
