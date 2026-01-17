//! Dialog messages.
//!
//! Messages for modal dialogs: About, Settings, ThirdParty, Update.

/// Messages for dialog interactions.
#[derive(Debug, Clone)]
pub enum DialogMessage {
    /// About dialog messages
    About(AboutMessage),

    /// Settings dialog messages
    Settings(SettingsMessage),

    /// Third-party licenses dialog messages
    ThirdParty(ThirdPartyMessage),

    /// Update dialog messages
    Update(UpdateMessage),

    /// Close any open dialog
    CloseAll,
}

// =============================================================================
// ABOUT DIALOG
// =============================================================================

/// Messages for the About dialog.
#[derive(Debug, Clone)]
pub enum AboutMessage {
    /// Open the About dialog
    Open,

    /// Close the About dialog
    Close,

    /// Copy system info to clipboard and close
    CopyAndClose,

    /// Open the website
    OpenWebsite,

    /// Open the GitHub repository
    OpenGitHub,

    /// Open the open-source licenses link
    OpenOpenSource,
}

// =============================================================================
// SETTINGS DIALOG
// =============================================================================

/// Messages for the Settings dialog.
#[derive(Debug, Clone)]
pub enum SettingsMessage {
    /// Open the Settings dialog
    Open,

    /// Close the Settings dialog (discard changes)
    Close,

    /// Apply changes and close
    Apply,

    /// Reset to default settings
    ResetToDefaults,

    /// Switch settings category/tab
    CategorySelected(SettingsCategory),

    // General settings
    General(GeneralSettingsMessage),

    // Validation settings
    Validation(ValidationSettingsMessage),

    // Developer settings
    Developer(DeveloperSettingsMessage),

    // Export settings
    Export(ExportSettingsMessage),

    // Display settings
    Display(DisplaySettingsMessage),

    // Update settings
    Updates(UpdateSettingsMessage),
}

/// Settings category/tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsCategory {
    #[default]
    General,
    Validation,
    Export,
    Display,
    Updates,
    Developer,
}

impl SettingsCategory {
    /// Returns a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Validation => "Validation",
            Self::Export => "Export",
            Self::Display => "Display",
            Self::Updates => "Updates",
            Self::Developer => "Developer",
        }
    }
}

/// General settings messages.
#[derive(Debug, Clone)]
pub enum GeneralSettingsMessage {
    /// Change controlled terminology version
    CtVersionChanged(String),

    /// Change header rows setting
    HeaderRowsChanged(usize),

    /// Change mapping confidence threshold (0.0 to 1.0)
    ConfidenceThresholdChanged(f32),
}

/// Validation settings messages.
#[derive(Debug, Clone)]
pub enum ValidationSettingsMessage {
    /// Toggle strict validation mode
    StrictModeToggled(bool),

    /// Toggle a specific validation rule
    RuleToggled { rule_id: String, enabled: bool },
}

/// Developer settings messages.
#[derive(Debug, Clone)]
pub enum DeveloperSettingsMessage {
    /// Toggle bypass validation errors for export
    BypassValidationToggled(bool),

    /// Toggle developer mode
    DeveloperModeToggled(bool),
}

/// Export settings messages.
#[derive(Debug, Clone)]
pub enum ExportSettingsMessage {
    /// Change default output directory
    DefaultOutputDirChanged(String),

    /// Change default export format
    DefaultFormatChanged(crate::state::ExportFormat),

    /// Change default XPT version
    DefaultXptVersionChanged(crate::state::XptVersion),

    /// Change SDTM-IG version
    SdtmIgVersionChanged(crate::state::SdtmIgVersion),
}

/// Display settings messages.
#[derive(Debug, Clone)]
pub enum DisplaySettingsMessage {
    /// Change preview rows per page
    PreviewRowsChanged(usize),

    /// Change decimal precision
    DecimalPrecisionChanged(usize),
}

/// Update settings messages.
#[derive(Debug, Clone)]
pub enum UpdateSettingsMessage {
    /// Toggle automatic update check on startup
    CheckOnStartupToggled(bool),

    /// Change update channel (Stable/ReleaseCandidate/Beta/Alpha)
    ChannelChanged(tss_updater::UpdateChannel),

    /// Clear skipped version
    ClearSkippedVersion,
}

// =============================================================================
// THIRD-PARTY DIALOG
// =============================================================================

/// Messages for the Third-Party Licenses dialog.
#[derive(Debug, Clone)]
pub enum ThirdPartyMessage {
    /// Open the dialog
    Open,

    /// Close the dialog
    Close,

    /// Scroll position changed
    ScrollTo(f32),
}

// =============================================================================
// UPDATE DIALOG
// =============================================================================

/// Messages for the Update dialog.
///
/// This enum contains both user actions and async operation results.
/// User actions trigger async tasks, which send back result messages.
#[derive(Debug, Clone)]
pub enum UpdateMessage {
    // -------------------------------------------------------------------------
    // User Actions
    // -------------------------------------------------------------------------
    /// Open the update dialog
    Open,

    /// Close the update dialog
    Close,

    /// User initiated update check
    CheckForUpdates,

    /// User confirmed download after seeing update available
    ConfirmDownload,

    /// User confirmed install after verification
    ConfirmInstall,

    /// User clicked restart to apply update
    Restart,

    /// User chose to skip this version
    SkipVersion(String),

    /// User chose to be reminded later
    RemindLater,

    /// User cancelled the current operation
    Cancel,

    // -------------------------------------------------------------------------
    // Async Operation Results (from Task::perform / Task::sip)
    // -------------------------------------------------------------------------
    /// Result of checking for updates
    CheckComplete(std::result::Result<Option<tss_updater::UpdateInfo>, String>),

    /// Download progress update (from Task::sip stream)
    DownloadProgress(tss_updater::DownloadProgress),

    /// Download complete with data (from Task::sip stream completion)
    DownloadComplete(std::result::Result<tss_updater::DownloadResult, String>),

    /// Verification complete
    VerifyComplete(std::result::Result<VerifyResult, String>),

    /// Installation complete
    InstallComplete(std::result::Result<(), String>),
}

/// Result of SHA256 verification.
#[derive(Debug, Clone)]
pub struct VerifyResult {
    /// Whether verification passed.
    pub verified: bool,
    /// The SHA256 hash (if computed).
    pub sha256: Option<String>,
}
