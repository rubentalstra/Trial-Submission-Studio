//! Application settings - persisted user preferences.
//!
//! Settings are loaded from disk at startup and saved when changed.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

// =============================================================================
// ROOT SETTINGS
// =============================================================================

/// Application settings.
///
/// Serialized to TOML and stored in the user's config directory.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// General application settings.
    pub general: GeneralSettings,

    /// Export settings.
    pub export: ExportSettings,

    /// Validation settings.
    pub validation: ValidationSettings,

    /// Display settings.
    pub display: DisplaySettings,

    /// Developer settings (advanced options).
    pub developer: DeveloperSettings,

    /// Update settings.
    pub updates: tss_updater::UpdateSettings,
}

impl Settings {
    /// Load settings from the default path.
    pub fn load() -> Self {
        let mut settings = Self::load_from(&Self::config_path());
        settings.migrate();
        settings
    }

    /// Load settings from a specific path.
    pub fn load_from(path: &PathBuf) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Migrate old settings values to current defaults.
    ///
    /// This handles breaking changes in settings, such as:
    /// - `header_rows` changed from default 1 to 2 (for label + column name rows)
    fn migrate(&mut self) {
        // Migration: header_rows was incorrectly defaulting to 1
        // Clinical data CSVs typically have 2 header rows (labels + column names)
        if self.general.header_rows == 1 {
            self.general.header_rows = 2;
            // Save the migrated settings
            let _ = self.save();
        }
    }

    /// Save settings to the default path.
    pub fn save(&self) -> Result<(), String> {
        self.save_to(&Self::config_path())
    }

    /// Save settings to a specific path.
    pub fn save_to(&self, path: &PathBuf) -> Result<(), String> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        std::fs::write(path, content).map_err(|e| format!("Failed to write settings: {}", e))
    }

    /// Get the default config file path.
    pub fn config_path() -> PathBuf {
        directories::ProjectDirs::from("com", "TrialSubmissionStudio", "TSS")
            .map(|dirs| dirs.config_dir().join("settings.toml"))
            .unwrap_or_else(|| PathBuf::from("settings.toml"))
    }
}

// =============================================================================
// RECENT STUDY
// =============================================================================

/// Rich metadata for a recently opened study.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentStudy {
    /// Unique identifier for this recent study entry.
    ///
    /// Used for robust identification in menu actions instead of path encoding.
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,

    /// Path to the study folder (canonical/absolute).
    pub path: PathBuf,

    /// Display name (derived study ID, e.g., "DEMO_GDISC").
    pub display_name: String,

    /// Workflow type used when last opened.
    pub workflow_type: WorkflowType,

    /// When the study was last successfully opened.
    pub last_opened: DateTime<Utc>,

    /// Number of domains when last opened.
    pub domain_count: usize,

    /// Total row count when last opened.
    pub total_rows: usize,
}

impl RecentStudy {
    /// Create a new recent study entry.
    pub fn new(
        path: PathBuf,
        display_name: String,
        workflow_type: WorkflowType,
        domain_count: usize,
        total_rows: usize,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            path,
            display_name,
            workflow_type,
            last_opened: Utc::now(),
            domain_count,
            total_rows,
        }
    }

    /// Check if the study folder still exists.
    pub fn exists(&self) -> bool {
        self.path.exists() && self.path.is_dir()
    }

    /// Get relative time string (e.g., "2 hours ago", "Yesterday").
    pub fn relative_time(&self) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.last_opened);

        if duration.num_minutes() < 1 {
            "Just now".to_string()
        } else if duration.num_minutes() < 60 {
            let mins = duration.num_minutes();
            format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
        } else if duration.num_hours() < 24 {
            let hours = duration.num_hours();
            format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
        } else if duration.num_days() == 1 {
            "Yesterday".to_string()
        } else if duration.num_days() < 7 {
            let days = duration.num_days();
            format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
        } else if duration.num_weeks() < 4 {
            let weeks = duration.num_weeks();
            format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
        } else {
            self.last_opened.format("%b %d, %Y").to_string()
        }
    }

    /// Get stats string (e.g., "12 domains, 45,320 rows").
    pub fn stats_string(&self) -> String {
        let domains = if self.domain_count == 1 {
            "1 domain".to_string()
        } else {
            format!("{} domains", self.domain_count)
        };

        let rows = format_number(self.total_rows);
        let rows_label = if self.total_rows == 1 { "row" } else { "rows" };

        format!("{}, {} {}", domains, rows, rows_label)
    }
}

/// Format a number with thousand separators.
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Workflow type for recent studies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowType {
    #[default]
    Sdtm,
    Adam,
    Send,
}

impl WorkflowType {
    /// Get display label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Sdtm => "SDTM",
            Self::Adam => "ADaM",
            Self::Send => "SEND",
        }
    }
}

// =============================================================================
// GENERAL SETTINGS
// =============================================================================

/// General application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralSettings {
    /// Number of header rows in CSV files (default: 2 for label + column names).
    pub header_rows: usize,

    /// Recent studies with full metadata.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recent_studies: Vec<RecentStudy>,

    /// Maximum number of recent studies to remember.
    pub max_recent: usize,

    /// Minimum confidence score (0.0 to 1.0) for displaying mapping suggestions.
    ///
    /// Higher values show only high-confidence matches, lower values show more suggestions.
    pub mapping_confidence_threshold: f32,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            header_rows: 2, // Default to double-header (row 1 = labels, row 2 = column names)
            recent_studies: Vec::new(),
            max_recent: 10,
            mapping_confidence_threshold: 0.6, // Default threshold for mapping suggestions
        }
    }
}

impl GeneralSettings {
    /// Add or update a study with full metadata.
    ///
    /// If the study exists, updates its metadata and moves it to front.
    /// If new, adds to front. Respects max_recent limit.
    pub fn add_recent_study(&mut self, study: RecentStudy) {
        // Check if already present (by path)
        let existing_idx = self
            .recent_studies
            .iter()
            .position(|s| s.path == study.path);

        if let Some(idx) = existing_idx {
            // Update existing and move to front
            let mut existing = self.recent_studies.remove(idx);
            existing.display_name = study.display_name;
            existing.workflow_type = study.workflow_type;
            existing.last_opened = study.last_opened;
            existing.domain_count = study.domain_count;
            existing.total_rows = study.total_rows;
            self.recent_studies.insert(0, existing);
        } else {
            // Add new study to front
            self.recent_studies.insert(0, study);
        }

        // Trim to max_recent
        self.enforce_max_recent();
    }

    /// Enforce the max_recent limit by removing oldest studies.
    fn enforce_max_recent(&mut self) {
        if self.recent_studies.len() > self.max_recent {
            self.recent_studies.truncate(self.max_recent);
        }
    }

    /// Remove a study from recent list by path.
    pub fn remove_recent(&mut self, path: &PathBuf) {
        self.recent_studies.retain(|s| &s.path != path);
    }

    /// Clear all recent studies.
    pub fn clear_all_recent(&mut self) {
        self.recent_studies.clear();
    }

    /// Remove stale studies (those with missing paths).
    pub fn prune_stale(&mut self) {
        self.recent_studies.retain(RecentStudy::exists);
    }

    /// Get recent studies sorted by last_opened (most recent first).
    pub fn recent_sorted(&self) -> Vec<&RecentStudy> {
        let mut sorted: Vec<&RecentStudy> = self.recent_studies.iter().collect();
        sorted.sort_by(|a, b| b.last_opened.cmp(&a.last_opened));
        sorted
    }
}

// =============================================================================
// EXPORT SETTINGS
// =============================================================================

/// Export settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportSettings {
    /// Default export format.
    pub default_format: ExportFormat,

    /// Last used export directory.
    pub last_export_dir: Option<PathBuf>,

    /// Include Define-XML in exports.
    pub include_define_xml: bool,

    /// XPT version for SAS transport files.
    pub xpt_version: XptVersion,

    /// SDTM-IG version for Dataset-XML and Define-XML.
    pub sdtm_ig_version: SdtmIgVersion,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            default_format: ExportFormat::Xpt,
            last_export_dir: None,
            include_define_xml: true,
            xpt_version: XptVersion::default(),
            sdtm_ig_version: SdtmIgVersion::default(),
        }
    }
}

/// SDTM Implementation Guide version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SdtmIgVersion {
    /// SDTM-IG 3.2
    V32,
    /// SDTM-IG 3.3
    V33,
    /// SDTM-IG 3.4 (default/latest)
    #[default]
    V34,
}

impl SdtmIgVersion {
    /// Get version string for export (e.g., "3.4").
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V32 => "3.2",
            Self::V33 => "3.3",
            Self::V34 => "3.4",
        }
    }

    /// Get display label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::V32 => "SDTM-IG 3.2",
            Self::V33 => "SDTM-IG 3.3",
            Self::V34 => "SDTM-IG 3.4",
        }
    }

    /// All available versions.
    pub const ALL: [SdtmIgVersion; 3] = [Self::V32, Self::V33, Self::V34];
}

impl std::fmt::Display for SdtmIgVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Export file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExportFormat {
    /// SAS Transport (XPT) format.
    #[default]
    Xpt,
    /// Dataset-XML format.
    DatasetXml,
}

impl ExportFormat {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Xpt => "XPT (SAS Transport)",
            Self::DatasetXml => "Dataset-XML",
        }
    }

    /// Brief description of the format.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Xpt => "Industry standard format for FDA submissions",
            Self::DatasetXml => "XML representation of CDISC datasets",
        }
    }

    /// Get file extension.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Xpt => "xpt",
            Self::DatasetXml => "xml",
        }
    }
}

/// XPT version for SAS transport files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum XptVersion {
    /// Version 5.
    #[default]
    V5,
    /// Version 8.
    V8,
}

impl XptVersion {
    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::V5 => "Version 5 (FDA)",
            Self::V8 => "Version 8",
        }
    }
}

// =============================================================================
// DEVELOPER SETTINGS
// =============================================================================

/// Developer settings (advanced options).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DeveloperSettings {
    /// Bypass validation errors when exporting.
    ///
    /// When enabled, allows exporting even if validation errors exist.
    /// Use with caution - exported data may not be FDA-compliant.
    pub bypass_validation: bool,

    /// Enable developer mode (shows additional debug info).
    pub developer_mode: bool,
}

// =============================================================================
// VALIDATION SETTINGS
// =============================================================================

/// Validation settings for CDISC conformance checking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ValidationSettings {
    /// Enable/disable individual validation rules.
    pub rules: ValidationRuleSettings,

    /// Treat warnings as errors (strict mode).
    pub strict_mode: bool,

    /// Controlled Terminology version to use for validation (None = latest).
    pub ct_version: Option<String>,
}

/// Individual validation rule toggles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ValidationRuleSettings {
    /// Check that required variables are present.
    pub check_required_variables: bool,
    /// Check that expected variables are present.
    pub check_expected_variables: bool,
    /// Check data types match expected types.
    pub check_data_types: bool,
    /// Check ISO 8601 date/time format.
    pub check_iso8601_format: bool,
    /// Check sequence number uniqueness.
    pub check_sequence_uniqueness: bool,
    /// Check text length against CDISC limits.
    pub check_text_length: bool,
    /// Check identifier nulls (STUDYID, USUBJID).
    pub check_identifier_nulls: bool,
    /// Check controlled terminology values.
    pub check_controlled_terminology: bool,
}

impl Default for ValidationRuleSettings {
    fn default() -> Self {
        Self {
            check_required_variables: true,
            check_expected_variables: true,
            check_data_types: true,
            check_iso8601_format: true,
            check_sequence_uniqueness: true,
            check_text_length: true,
            check_identifier_nulls: true,
            check_controlled_terminology: true,
        }
    }
}

// =============================================================================
// DISPLAY SETTINGS
// =============================================================================

/// Display settings for the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplaySettings {
    /// Number of rows to show in data preview.
    pub preview_rows_per_page: usize,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            preview_rows_per_page: 50,
        }
    }
}
