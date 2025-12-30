//! Study-level runtime state

use super::TransformState;
use crate::services::MappingState;
use polars::prelude::DataFrame;
use sdtm_ingest::StudyMetadata;
use sdtm_validate::ValidationReport;
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
    /// Source metadata (Items.csv, CodeLists.csv)
    pub metadata: Option<StudyMetadata>,
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
            metadata: None,
        }
    }

    /// Get the label for a source column from Items.csv metadata
    pub fn get_column_label(&self, column_id: &str) -> Option<&str> {
        self.metadata
            .as_ref()?
            .items
            .get(&column_id.to_uppercase())
            .map(|item| item.label.as_str())
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
    /// Path to source CSV file
    pub source_file: PathBuf,
    /// Source data as DataFrame
    pub source_data: DataFrame,
    /// Current status
    pub status: DomainStatus,
    /// Interactive mapping state (for GUI)
    pub mapping_state: Option<MappingState>,
    /// Transform display state (read-only, shows what will be applied)
    pub transform_state: Option<TransformState>,
    /// Validation results
    pub validation: Option<ValidationReport>,
    /// Selected validation issue index (for detail view)
    pub validation_selected_idx: Option<usize>,
    /// Preview of processed data
    pub preview_data: Option<DataFrame>,
}

impl DomainState {
    /// Create a new domain state
    pub fn new(source_file: PathBuf, source_data: DataFrame) -> Self {
        Self {
            source_file,
            source_data,
            status: DomainStatus::NotStarted,
            mapping_state: None,
            transform_state: None,
            validation: None,
            validation_selected_idx: None,
            preview_data: None,
        }
    }

    /// Invalidate cached data that depends on mappings.
    ///
    /// Call this whenever accepted mappings change to ensure:
    /// - Validation is re-run on the new mapped data
    /// - Transform state is regenerated
    /// - Preview data is regenerated
    pub fn invalidate_mapping_dependents(&mut self) {
        self.validation = None;
        self.validation_selected_idx = None;
        self.transform_state = None;
        self.preview_data = None;
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
#[allow(dead_code)] // Variants used in pattern matching, constructed when features are implemented
pub enum DomainStatus {
    /// Not yet started
    #[default]
    NotStarted,
    /// Loading/initializing mapping
    Loading,
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
    /// Get status icon (phosphor icon)
    pub fn icon(&self) -> &'static str {
        match self {
            Self::NotStarted => egui_phosphor::regular::CIRCLE,
            Self::Loading => egui_phosphor::regular::SPINNER_GAP,
            Self::MappingInProgress => egui_phosphor::regular::CIRCLE_HALF,
            Self::MappingComplete => egui_phosphor::regular::CIRCLE_WAVY_CHECK,
            Self::ValidationFailed => egui_phosphor::regular::X_CIRCLE,
            Self::ReadyForExport => egui_phosphor::regular::CHECK_CIRCLE,
        }
    }
}
