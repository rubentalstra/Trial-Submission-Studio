use std::path::PathBuf;

use sdtm_model::ValidationReport;

#[derive(Debug)]
pub struct StudyResult {
    pub study_id: String,
    pub output_dir: PathBuf,
    pub domains: Vec<DomainSummary>,
    pub data_checks: Vec<DomainDataCheck>,
    pub errors: Vec<String>,
    pub define_xml: Option<PathBuf>,
    pub has_errors: bool,
}

#[derive(Debug)]
pub struct DomainSummary {
    pub domain_code: String,
    pub description: String,
    pub records: usize,
    pub outputs: sdtm_model::OutputPaths,
    pub conformance: Option<ValidationReport>,
}

#[derive(Debug, Clone)]
pub struct DomainDataCheck {
    pub domain_code: String,
    pub csv_rows: usize,
    pub xpt_rows: Option<usize>,
}
