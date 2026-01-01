//! Settings types and configuration for the CDISC Transpiler GUI.
//!
//! This module defines all user-configurable settings including:
//! - General preferences (dark mode, CT version)
//! - Validation settings (mode, XPT version, custom rules)
//! - Developer settings (bypass rules, allow export with errors)
//! - Export defaults (output directory, format)
//! - Display settings (preview rows, decimal precision)
//! - Keyboard shortcuts

mod persistence;
pub mod ui;

pub use persistence::{load_settings, save_settings};

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

// ============================================================================
// Main Settings Struct
// ============================================================================

/// Application settings (persisted to disk as TOML).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub general: GeneralSettings,
    pub validation: ValidationSettings,
    pub developer: DeveloperSettings,
    pub export: ExportSettings,
    pub display: DisplaySettings,
    pub shortcuts: ShortcutSettings,

    /// Recent study folders (persisted for convenience).
    #[serde(default)]
    pub recent_studies: Vec<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
            validation: ValidationSettings::default(),
            developer: DeveloperSettings::default(),
            export: ExportSettings::default(),
            display: DisplaySettings::default(),
            shortcuts: ShortcutSettings::default(),
            recent_studies: Vec::new(),
        }
    }
}

// ============================================================================
// General Settings
// ============================================================================

/// General application preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralSettings {
    /// Enable dark mode theme.
    pub dark_mode: bool,
    /// Controlled Terminology version to use.
    pub ct_version: CtVersionSetting,
    /// Number of header rows in source CSV files.
    pub header_rows: usize,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            dark_mode: false,
            ct_version: CtVersionSetting::default(),
            header_rows: 2,
        }
    }
}

/// Serializable wrapper for CT version selection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum CtVersionSetting {
    /// CT version 2024-03-29 (production default).
    #[default]
    #[serde(rename = "2024-03-29")]
    V2024_03_29,
    /// CT version 2025-09-26 (latest).
    #[serde(rename = "2025-09-26")]
    V2025_09_26,
}

impl CtVersionSetting {
    /// Get all available CT versions.
    pub const fn all() -> &'static [CtVersionSetting] {
        &[Self::V2024_03_29, Self::V2025_09_26]
    }

    /// Get the display name for UI.
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::V2024_03_29 => "2024-03-29 (Default)",
            Self::V2025_09_26 => "2025-09-26 (Latest)",
        }
    }
}

impl From<CtVersionSetting> for sdtm_standards::ct::CtVersion {
    fn from(s: CtVersionSetting) -> Self {
        match s {
            CtVersionSetting::V2024_03_29 => Self::V2024_03_29,
            CtVersionSetting::V2025_09_26 => Self::V2025_09_26,
        }
    }
}

// ============================================================================
// Validation Settings
// ============================================================================

/// Validation mode selection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum ValidationModeSetting {
    /// Basic XPT format validation only.
    #[default]
    Basic,
    /// Full FDA compliance validation (stricter).
    FdaCompliant,
    /// Custom validation with specific rules enabled.
    Custom,
}

impl ValidationModeSetting {
    /// Get all available modes.
    pub const fn all() -> &'static [ValidationModeSetting] {
        &[Self::Basic, Self::FdaCompliant, Self::Custom]
    }

    /// Get the display name for UI.
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::FdaCompliant => "FDA Compliant",
            Self::Custom => "Custom",
        }
    }

    /// Get description for UI.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Basic => "Basic XPT format validation only",
            Self::FdaCompliant => "Strict FDA submission requirements",
            Self::Custom => "Select specific validation rules",
        }
    }
}

/// XPT format version selection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum XptVersionSetting {
    /// SAS Transport V5 (FDA standard, 8-char names).
    #[default]
    V5,
    /// SAS Transport V8 (extended, 32-char names).
    V8,
}

impl XptVersionSetting {
    /// Get all available versions.
    pub const fn all() -> &'static [XptVersionSetting] {
        &[Self::V5, Self::V8]
    }

    /// Get the display name for UI.
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::V5 => "V5 (FDA Standard)",
            Self::V8 => "V8 (Extended)",
        }
    }

    /// Get description for UI.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::V5 => "8-char names, 40-char labels (FDA required)",
            Self::V8 => "32-char names, 256-char labels",
        }
    }
}

/// Validation rule identifiers (matches sdtm_xpt validation rules).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum XptValidationRule {
    DatasetName,
    VariableName,
    DatasetLabel,
    VariableLabel,
    FormatName,
    DuplicateVariable,
    VariableLength,
    FdaVersion,
    FdaAscii,
}

impl XptValidationRule {
    /// Get all available rules.
    pub const fn all() -> &'static [XptValidationRule] {
        &[
            Self::DatasetName,
            Self::VariableName,
            Self::DatasetLabel,
            Self::VariableLabel,
            Self::FormatName,
            Self::DuplicateVariable,
            Self::VariableLength,
            Self::FdaVersion,
            Self::FdaAscii,
        ]
    }

    /// Get the display name for UI.
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::DatasetName => "Dataset Name",
            Self::VariableName => "Variable Name",
            Self::DatasetLabel => "Dataset Label",
            Self::VariableLabel => "Variable Label",
            Self::FormatName => "Format Name",
            Self::DuplicateVariable => "Duplicate Variable",
            Self::VariableLength => "Variable Length",
            Self::FdaVersion => "FDA Version",
            Self::FdaAscii => "FDA ASCII",
        }
    }

    /// Get description for UI.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::DatasetName => "Validates dataset names (8 chars for V5, 32 for V8)",
            Self::VariableName => "Validates variable names (A-Z, 0-9, _, starts with letter)",
            Self::DatasetLabel => "Validates dataset label length (40 for V5, 256 for V8)",
            Self::VariableLabel => "Validates variable label length",
            Self::FormatName => "Validates format/informat names",
            Self::DuplicateVariable => "Checks for duplicate variable names",
            Self::VariableLength => "Validates character field lengths",
            Self::FdaVersion => "Requires V5 format for FDA submissions",
            Self::FdaAscii => "Requires ASCII-only characters for FDA",
        }
    }

    /// Check if this rule is FDA-specific.
    pub const fn is_fda_only(&self) -> bool {
        matches!(self, Self::FdaVersion | Self::FdaAscii)
    }
}

/// Validation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ValidationSettings {
    /// Validation mode.
    pub mode: ValidationModeSetting,
    /// XPT format version.
    pub xpt_version: XptVersionSetting,
    /// Enabled rules when mode is Custom.
    pub custom_enabled_rules: HashSet<XptValidationRule>,
}

impl Default for ValidationSettings {
    fn default() -> Self {
        // Default: all non-FDA rules enabled
        let enabled: HashSet<_> = XptValidationRule::all()
            .iter()
            .filter(|r| !r.is_fda_only())
            .copied()
            .collect();

        Self {
            mode: ValidationModeSetting::Basic,
            xpt_version: XptVersionSetting::V5,
            custom_enabled_rules: enabled,
        }
    }
}

// ============================================================================
// Developer Settings
// ============================================================================

/// Developer mode settings for testing and debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DeveloperSettings {
    /// Enable developer mode.
    pub enabled: bool,
    /// Rules to bypass when developer mode is enabled.
    pub bypassed_rules: HashSet<XptValidationRule>,
    /// Allow export even with validation errors.
    pub allow_export_with_errors: bool,
    /// Show extra debug information in the UI.
    pub show_debug_info: bool,
}

impl Default for DeveloperSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            bypassed_rules: HashSet::new(),
            allow_export_with_errors: false,
            show_debug_info: false,
        }
    }
}

// ============================================================================
// Export Settings
// ============================================================================

/// Export data format selection (XPT or Dataset-XML).
/// Note: Define-XML is always generated alongside the data format.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum ExportFormat {
    /// XPT (SAS Transport) - FDA standard format.
    #[default]
    Xpt,
    /// CDISC Dataset-XML format.
    DatasetXml,
}

impl ExportFormat {
    /// Get all available formats.
    pub const fn all() -> &'static [ExportFormat] {
        &[Self::Xpt, Self::DatasetXml]
    }

    /// Get the display name for UI.
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Xpt => "XPT (SAS Transport)",
            Self::DatasetXml => "Dataset-XML",
        }
    }

    /// Get description for UI.
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Xpt => "FDA-required SAS Transport format",
            Self::DatasetXml => "CDISC XML format for datasets",
        }
    }
}

/// Export default settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportSettings {
    /// Default output directory.
    pub default_output_dir: Option<PathBuf>,
    /// Default export format (XPT or Dataset-XML).
    pub default_format: ExportFormat,
    /// Generate Define-XML alongside data export (always recommended).
    pub generate_define_xml: bool,
    /// Filename template (e.g., "{domain}").
    pub filename_template: String,
    /// Overwrite existing files without prompting.
    pub overwrite_without_prompt: bool,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            default_output_dir: None,
            default_format: ExportFormat::Xpt,
            generate_define_xml: true, // Always generate Define-XML by default
            filename_template: "{domain}".to_string(),
            overwrite_without_prompt: false,
        }
    }
}

// ============================================================================
// Display Settings
// ============================================================================

/// Preview row limit options.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum PreviewRowLimit {
    /// Show 100 rows.
    Rows100,
    /// Show 500 rows (default).
    #[default]
    Rows500,
    /// Show 1000 rows.
    Rows1000,
    /// Show all rows.
    All,
}

impl PreviewRowLimit {
    /// Get all available options.
    pub const fn all() -> &'static [PreviewRowLimit] {
        &[Self::Rows100, Self::Rows500, Self::Rows1000, Self::All]
    }

    /// Get the actual row limit value.
    pub const fn value(&self) -> Option<usize> {
        match self {
            Self::Rows100 => Some(100),
            Self::Rows500 => Some(500),
            Self::Rows1000 => Some(1000),
            Self::All => None,
        }
    }

    /// Get the display name for UI.
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Rows100 => "100 rows",
            Self::Rows500 => "500 rows",
            Self::Rows1000 => "1000 rows",
            Self::All => "All rows",
        }
    }
}

/// Display settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DisplaySettings {
    /// Maximum rows to show in preview tables.
    pub max_preview_rows: PreviewRowLimit,
    /// Decimal precision for numeric display.
    pub decimal_precision: u8,
    /// Show row numbers in preview tables.
    pub show_row_numbers: bool,
    /// Truncate text columns longer than this (in characters).
    pub truncate_long_text: usize,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            max_preview_rows: PreviewRowLimit::default(),
            decimal_precision: 4,
            show_row_numbers: true,
            truncate_long_text: 50,
        }
    }
}

// ============================================================================
// Keyboard Shortcuts
// ============================================================================

/// Shortcut actions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    OpenStudy,
    GoToExport,
    GoHome,
    OpenSettings,
    ToggleDarkMode,
    NextTab,
    PrevTab,
}

impl ShortcutAction {
    /// Get all available actions.
    pub const fn all() -> &'static [ShortcutAction] {
        &[
            Self::OpenStudy,
            Self::GoToExport,
            Self::GoHome,
            Self::OpenSettings,
            Self::ToggleDarkMode,
            Self::NextTab,
            Self::PrevTab,
        ]
    }

    /// Get the display name for UI.
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::OpenStudy => "Open Study",
            Self::GoToExport => "Go to Export",
            Self::GoHome => "Go Home",
            Self::OpenSettings => "Open Settings",
            Self::ToggleDarkMode => "Toggle Dark Mode",
            Self::NextTab => "Next Tab",
            Self::PrevTab => "Previous Tab",
        }
    }
}

/// Key binding definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyBinding {
    /// The key (e.g., "O", "E", ",").
    pub key: String,
    /// Ctrl modifier (Windows/Linux).
    pub ctrl: bool,
    /// Command modifier (macOS).
    pub command: bool,
    /// Shift modifier.
    pub shift: bool,
    /// Alt/Option modifier.
    pub alt: bool,
}

impl KeyBinding {
    /// Create a new key binding with Cmd (macOS) or Ctrl (other).
    pub fn cmd_or_ctrl(key: &str) -> Self {
        let is_macos = cfg!(target_os = "macos");
        Self {
            key: key.to_string(),
            ctrl: !is_macos,
            command: is_macos,
            shift: false,
            alt: false,
        }
    }

    /// Get display string for UI.
    pub fn display(&self) -> String {
        let mut parts = Vec::new();

        if cfg!(target_os = "macos") {
            if self.ctrl {
                parts.push("⌃");
            }
            if self.alt {
                parts.push("⌥");
            }
            if self.shift {
                parts.push("⇧");
            }
            if self.command {
                parts.push("⌘");
            }
        } else {
            if self.ctrl {
                parts.push("Ctrl");
            }
            if self.alt {
                parts.push("Alt");
            }
            if self.shift {
                parts.push("Shift");
            }
        }

        parts.push(&self.key);

        if cfg!(target_os = "macos") {
            parts.join("")
        } else {
            parts.join("+")
        }
    }
}

/// Keyboard shortcut settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShortcutSettings {
    /// Key bindings for each action.
    pub bindings: HashMap<ShortcutAction, KeyBinding>,
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        let mut bindings = HashMap::new();

        bindings.insert(ShortcutAction::OpenStudy, KeyBinding::cmd_or_ctrl("O"));
        bindings.insert(ShortcutAction::GoToExport, KeyBinding::cmd_or_ctrl("E"));
        bindings.insert(ShortcutAction::OpenSettings, KeyBinding::cmd_or_ctrl(","));
        bindings.insert(
            ShortcutAction::GoHome,
            KeyBinding {
                key: "Escape".to_string(),
                ctrl: false,
                command: false,
                shift: false,
                alt: false,
            },
        );
        bindings.insert(
            ShortcutAction::NextTab,
            KeyBinding {
                key: "→".to_string(),
                ctrl: false,
                command: false,
                shift: false,
                alt: false,
            },
        );
        bindings.insert(
            ShortcutAction::PrevTab,
            KeyBinding {
                key: "←".to_string(),
                ctrl: false,
                command: false,
                shift: false,
                alt: false,
            },
        );

        Self { bindings }
    }
}
