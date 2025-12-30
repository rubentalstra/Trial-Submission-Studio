//! Study-level runtime state

use polars::prelude::DataFrame;
use sdtm_model::{MappingConfig, ValidationReport};
use std::collections::HashMap;
use std::path::PathBuf;

/// Runtime state for a loaded study
pub struct StudyState {
    /// Study identifier (derived from folder name)
    pub study_id: String,
    /// Path to study folder
    pub study_folder: PathBuf,
    /// State for each discovered domain
    pub domains: HashMap<String, DomainState>,
}

impl StudyState {
    /// Create a new study state from a folder path
    pub fn new(study_folder: PathBuf) -> Self {
        let study_id = study_folder
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown Study")
            .to_string();

        Self {
            study_id,
            study_folder,
            domains: HashMap::new(),
        }
    }

    /// Get a domain by code
    pub fn get_domain(&self, code: &str) -> Option<&DomainState> {
        self.domains.get(code)
    }

    /// Get a mutable domain by code
    pub fn get_domain_mut(&mut self, code: &str) -> Option<&mut DomainState> {
        self.domains.get_mut(code)
    }

    /// Get all domain codes sorted alphabetically
    pub fn domain_codes(&self) -> Vec<&str> {
        let mut codes: Vec<&str> = self.domains.keys().map(String::as_str).collect();
        codes.sort();
        codes
    }
}

/// State for a single domain
pub struct DomainState {
    /// Domain code (e.g., "DM", "AE")
    pub code: String,
    /// Path to source CSV file
    pub source_file: PathBuf,
    /// Source data as DataFrame
    pub source_data: DataFrame,
    /// Current status
    pub status: DomainStatus,
    /// Column mapping configuration
    pub mapping: Option<MappingConfig>,
    /// Validation results
    pub validation: Option<ValidationReport>,
    /// Preview of processed data
    pub preview_data: Option<DataFrame>,
}

impl DomainState {
    /// Create a new domain state
    pub fn new(code: String, source_file: PathBuf, source_data: DataFrame) -> Self {
        Self {
            code,
            source_file,
            source_data,
            status: DomainStatus::NotStarted,
            mapping: None,
            validation: None,
            preview_data: None,
        }
    }

    /// Get source column names
    pub fn source_columns(&self) -> Vec<String> {
        self.source_data
            .get_column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Get row count
    pub fn row_count(&self) -> usize {
        self.source_data.height()
    }
}

/// Status of domain processing
#[derive(Default, Clone, Copy, PartialEq)]
pub enum DomainStatus {
    /// Not yet started
    #[default]
    NotStarted,
    /// Mapping in progress
    MappingInProgress,
    /// Mapping complete
    MappingComplete,
    /// Validation failed
    ValidationFailed,
    /// Ready for export
    ReadyForExport,
}

impl DomainStatus {
    /// Get display label
    pub fn label(&self) -> &'static str {
        match self {
            Self::NotStarted => "Not Started",
            Self::MappingInProgress => "Mapping...",
            Self::MappingComplete => "Mapped",
            Self::ValidationFailed => "Errors",
            Self::ReadyForExport => "Ready",
        }
    }

    /// Get status icon
    pub fn icon(&self) -> &'static str {
        match self {
            Self::NotStarted => "○",
            Self::MappingInProgress => "◐",
            Self::MappingComplete => "●",
            Self::ValidationFailed => "✕",
            Self::ReadyForExport => "✓",
        }
    }
}
