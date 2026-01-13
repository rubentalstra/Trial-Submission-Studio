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
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
            export: ExportSettings::default(),
        }
    }
}

impl Settings {
    /// Load settings from the default path.
    pub fn load() -> Self {
        Self::load_from(Self::config_path())
    }

    /// Load settings from a specific path.
    pub fn load_from(path: PathBuf) -> Self {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default()
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
    /// Number of header rows in CSV files (default: 1).
    pub header_rows: usize,

    /// Auto-check for updates on startup.
    pub auto_check_updates: bool,

    /// Recent study folders (most recent first).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recent_studies: Vec<PathBuf>,

    /// Maximum number of recent studies to remember.
    #[serde(skip)]
    pub max_recent: usize,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            header_rows: 1,
            auto_check_updates: true,
            recent_studies: Vec::new(),
            max_recent: 10,
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
    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Xpt => "XPT (SAS Transport)",
            Self::DatasetXml => "Dataset-XML",
        }
    }

    /// Get file extension.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Xpt => "xpt",
            Self::DatasetXml => "xml",
        }
    }

    /// All formats.
    pub const ALL: [ExportFormat; 2] = [Self::Xpt, Self::DatasetXml];
}

/// XPT version for SAS transport files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum XptVersion {
    /// Version 5 (legacy, wider compatibility).
    V5,
    /// Version 8 (modern, recommended).
    #[default]
    V8,
}

impl XptVersion {
    /// Get display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::V5 => "Version 5 (Legacy)",
            Self::V8 => "Version 8 (Modern)",
        }
    }
}
