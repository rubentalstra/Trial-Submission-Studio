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

/// Workflow type for projects.
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
// RECENT PROJECT
// =============================================================================

/// Rich metadata for a recently opened project (.tss file).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    /// Unique identifier for this recent project entry.
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,

    /// Path to the .tss project file (canonical/absolute).
    pub path: PathBuf,

    /// Display name (study ID from the project).
    pub display_name: String,

    /// Workflow type used.
    pub workflow_type: WorkflowType,

    /// When the project was last successfully opened.
    pub last_opened: DateTime<Utc>,

    /// Number of domains in the project.
    pub domain_count: usize,
}

impl RecentProject {
    /// Create a new recent project entry.
    pub fn new(
        path: PathBuf,
        display_name: String,
        workflow_type: WorkflowType,
        domain_count: usize,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            path,
            display_name,
            workflow_type,
            last_opened: Utc::now(),
            domain_count,
        }
    }

    /// Check if the project file still exists.
    pub fn exists(&self) -> bool {
        self.path.exists() && self.path.is_file()
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

    /// Get stats string (e.g., "12 domains").
    pub fn stats_string(&self) -> String {
        if self.domain_count == 1 {
            "1 domain".to_string()
        } else {
            format!("{} domains", self.domain_count)
        }
    }
}

// =============================================================================
// GENERAL SETTINGS
// =============================================================================

/// Assignment mode for mapping source files to domains.
///
/// Users choose their preferred method in Settings > General.
/// Only one mode is active at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentMode {
    /// Drag files to domains.
    #[default]
    DragAndDrop,
    /// Click file to select, then click domain to assign.
    ClickToAssign,
}

#[allow(dead_code)] // Methods used by Settings dialog UI (to be implemented)
impl AssignmentMode {
    /// Get display label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::DragAndDrop => "Drag and Drop",
            Self::ClickToAssign => "Click to Assign",
        }
    }

    /// Get description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::DragAndDrop => "Drag files from the source panel to domains",
            Self::ClickToAssign => "Click a file to select it, then click a domain to assign",
        }
    }

    /// All available modes.
    pub const ALL: [AssignmentMode; 2] = [Self::DragAndDrop, Self::ClickToAssign];
}

/// General application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralSettings {
    /// Number of header rows in CSV files (default: 2 for label + column names).
    pub header_rows: usize,

    /// Recent projects with full metadata.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recent_projects: Vec<RecentProject>,

    /// Maximum number of recent projects to remember.
    pub max_recent: usize,

    /// Minimum confidence score (0.0 to 1.0) for displaying mapping suggestions.
    ///
    /// Higher values show only high-confidence matches, lower values show more suggestions.
    pub mapping_confidence_threshold: f32,

    /// Assignment mode for source-to-domain mapping.
    pub assignment_mode: AssignmentMode,

    /// Whether auto-save is enabled.
    ///
    /// When enabled, projects are automatically saved after a debounce period.
    pub auto_save_enabled: bool,

    /// Auto-save debounce time in milliseconds.
    ///
    /// After a change, the project will be auto-saved once this many milliseconds
    /// have passed without further changes.
    pub auto_save_debounce_ms: u64,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            header_rows: 2, // Default to double-header (row 1 = labels, row 2 = column names)
            recent_projects: Vec::new(),
            max_recent: 10,
            mapping_confidence_threshold: 0.6, // Default threshold for mapping suggestions
            assignment_mode: AssignmentMode::default(),
            auto_save_enabled: true,     // Auto-save enabled by default
            auto_save_debounce_ms: 2000, // 2 second debounce
        }
    }
}

impl GeneralSettings {
    /// Add or update a project with full metadata.
    ///
    /// If the project exists, updates its metadata and moves it to front.
    /// If new, adds to front. Respects max_recent limit.
    pub fn add_recent_project(&mut self, project: RecentProject) {
        // Check if already present (by path)
        let existing_idx = self
            .recent_projects
            .iter()
            .position(|p| p.path == project.path);

        if let Some(idx) = existing_idx {
            // Update existing and move to front
            let mut existing = self.recent_projects.remove(idx);
            existing.display_name = project.display_name;
            existing.workflow_type = project.workflow_type;
            existing.last_opened = project.last_opened;
            existing.domain_count = project.domain_count;
            self.recent_projects.insert(0, existing);
        } else {
            // Add new project to front
            self.recent_projects.insert(0, project);
        }

        // Trim to max_recent
        self.enforce_max_recent_projects();
    }

    /// Enforce the max_recent limit by removing oldest projects.
    fn enforce_max_recent_projects(&mut self) {
        if self.recent_projects.len() > self.max_recent {
            self.recent_projects.truncate(self.max_recent);
        }
    }

    /// Remove a project from recent list by path.
    pub fn remove_recent_project(&mut self, path: &PathBuf) {
        self.recent_projects.retain(|p| &p.path != path);
    }

    /// Clear all recent projects.
    pub fn clear_all_recent_projects(&mut self) {
        self.recent_projects.clear();
    }

    /// Remove stale projects (those with missing paths).
    pub fn prune_stale_projects(&mut self) {
        self.recent_projects.retain(RecentProject::exists);
    }

    /// Get recent projects sorted by last_opened (most recent first).
    pub fn recent_projects_sorted(&self) -> Vec<&RecentProject> {
        let mut sorted: Vec<&RecentProject> = self.recent_projects.iter().collect();
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

    /// Implementation Guide version for Dataset-XML and Define-XML.
    pub ig_version: SdtmIgVersion,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            default_format: ExportFormat::Xpt,
            last_export_dir: None,
            include_define_xml: true,
            xpt_version: XptVersion::default(),
            ig_version: SdtmIgVersion::default(),
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

use crate::theme::palette::{AccessibilityMode, ThemeMode};

/// Display settings for the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplaySettings {
    /// Number of rows to show in data preview.
    pub preview_rows_per_page: usize,
    /// Theme mode (light/dark/system).
    pub theme_mode: ThemeMode,
    /// Color vision accessibility mode.
    pub accessibility_mode: AccessibilityMode,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            preview_rows_per_page: 50,
            theme_mode: ThemeMode::Light,
            accessibility_mode: AccessibilityMode::Standard,
        }
    }
}
