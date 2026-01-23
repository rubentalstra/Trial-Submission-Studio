//! Root project file type.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use rkyv::{Archive, Deserialize, Serialize};

use super::{DomainSnapshot, ProjectPlaceholders, SourceAssignment};

/// Root project file structure.
///
/// This is the top-level type that gets serialized to .tss files.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub struct ProjectFile {
    /// Schema version (for future migrations).
    pub schema_version: u32,

    /// When the project was created.
    pub created_at: String,

    /// When the project was last saved.
    pub last_saved_at: String,

    /// Study metadata.
    pub study: StudyMetadata,

    /// Source file assignments (CSV path -> domain code).
    pub source_assignments: Vec<SourceAssignment>,

    /// Per-domain snapshots (indexed by domain code).
    pub domains: BTreeMap<String, DomainSnapshot>,

    /// Placeholders for future features.
    pub placeholders: ProjectPlaceholders,
}

impl ProjectFile {
    /// Create a new project file with the given study metadata.
    pub fn new(study: StudyMetadata) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            schema_version: super::CURRENT_SCHEMA_VERSION,
            created_at: now.clone(),
            last_saved_at: now,
            study,
            source_assignments: Vec::new(),
            domains: BTreeMap::new(),
            placeholders: ProjectPlaceholders::default(),
        }
    }

    /// Update the last saved timestamp.
    pub fn touch(&mut self) {
        self.last_saved_at = Utc::now().to_rfc3339();
    }

    /// Parse the created_at timestamp.
    pub fn created_at(&self) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.created_at)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }

    /// Parse the last_saved_at timestamp.
    pub fn last_saved_at(&self) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.last_saved_at)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }
}

/// Study-level metadata.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub struct StudyMetadata {
    /// Study identifier (e.g., "DEMO_CDISC").
    pub study_id: String,

    /// Path to the study folder (for display purposes).
    pub study_folder: String,

    /// Workflow type (SDTM, ADaM, SEND).
    pub workflow_type: WorkflowTypeSnapshot,

    /// Implementation Guide version used (e.g., "3.4" for SDTM-IG 3.4).
    /// Applies to SDTM, ADaM, and SEND workflows.
    pub ig_version: String,

    /// Controlled Terminology version used.
    pub ct_version: Option<String>,
}

impl StudyMetadata {
    /// Create new study metadata.
    pub fn new(
        study_id: impl Into<String>,
        study_folder: impl Into<String>,
        workflow_type: WorkflowTypeSnapshot,
    ) -> Self {
        Self {
            study_id: study_id.into(),
            study_folder: study_folder.into(),
            workflow_type,
            ig_version: "3.4".to_string(),
            ct_version: None,
        }
    }
}

/// Workflow type for serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Archive, Serialize, Deserialize)]
#[rkyv(compare(PartialEq))]
pub enum WorkflowTypeSnapshot {
    #[default]
    Sdtm,
    Adam,
    Send,
}
