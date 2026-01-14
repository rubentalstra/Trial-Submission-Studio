//! Application settings - persisted user preferences.
//!
//! Settings are loaded from disk at startup and saved when changed.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// =============================================================================
// ROOT SETTINGS
// =============================================================================

/// Application settings.
///
/// Serialized to TOML and stored in the user's config directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// General application settings.
    pub general: GeneralSettings,

    /// Export settings.
    pub export: ExportSettings,

    /// Developer settings (advanced options).
    pub developer: DeveloperSettings,

    /// Update settings.
    pub updates: tss_updater::UpdateSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
            export: ExportSettings::default(),
            developer: DeveloperSettings::default(),
            updates: tss_updater::UpdateSettings::default(),
        }
    }
}

impl Settings {
    /// Load settings from the default path.
    pub fn load() -> Self {
        let mut settings = Self::load_from(Self::config_path());
        settings.migrate();
        settings
    }

    /// Load settings from a specific path.
    pub fn load_from(path: PathBuf) -> Self {
        std::fs::read_to_string(&path)
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
        self.save_to(Self::config_path())
    }

    /// Save settings to a specific path.
    pub fn save_to(&self, path: PathBuf) -> Result<(), String> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        std::fs::write(&path, content).map_err(|e| format!("Failed to write settings: {}", e))
    }

    /// Get the default config file path.
    pub fn config_path() -> PathBuf {
        directories::ProjectDirs::from("com", "TrialSubmissionStudio", "TSS")
            .map(|dirs| dirs.config_dir().join("settings.toml"))
            .unwrap_or_else(|| PathBuf::from("settings.toml"))
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

    /// Recent study folders (most recent first).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recent_studies: Vec<PathBuf>,

    /// Maximum number of recent studies to remember.
    #[serde(skip)]
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
    /// Add a study folder to recent list.
    ///
    /// Moves to front if already present, adds to front if new.
    pub fn add_recent(&mut self, folder: PathBuf) {
        // Remove if already present
        self.recent_studies.retain(|p| p != &folder);

        // Add to front
        self.recent_studies.insert(0, folder);

        // Trim to max
        self.recent_studies.truncate(self.max_recent);
    }

    /// Remove a study from recent list.
    pub fn remove_recent(&mut self, folder: &PathBuf) {
        self.recent_studies.retain(|p| p != folder);
    }

    /// Clear all recent studies.
    pub fn clear_recent(&mut self) {
        self.recent_studies.clear();
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
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            default_format: ExportFormat::Xpt,
            last_export_dir: None,
            include_define_xml: true,
            xpt_version: XptVersion::V8,
        }
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
    /// All available formats.
    pub const ALL: [ExportFormat; 2] = [Self::Xpt, Self::DatasetXml];

    /// Get display name (alias for label).
    pub fn display_name(&self) -> &'static str {
        self.label()
    }

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for DeveloperSettings {
    fn default() -> Self {
        Self {
            bypass_validation: false,
            developer_mode: false,
        }
    }
}
