use std::path::PathBuf;

use sdtm_model::ConformanceReport;

#[derive(Debug)]
pub struct StudyResult {
    pub study_id: String,
    pub output_dir: PathBuf,
    pub domains: Vec<DomainSummary>,
    pub errors: Vec<String>,
    pub conformance_report: Option<PathBuf>,
    pub define_xml: Option<PathBuf>,
    pub has_errors: bool,
}

#[derive(Debug)]
pub struct DomainSummary {
    pub domain_code: String,
    pub description: String,
    pub records: usize,
    pub outputs: sdtm_model::OutputPaths,
    pub conformance: Option<ConformanceReport>,
}
