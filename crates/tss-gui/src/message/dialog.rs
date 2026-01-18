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
#[allow(dead_code, clippy::enum_variant_names)]
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
#[allow(dead_code, clippy::enum_variant_names)]
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
/// Simplified enum with 11 variants (down from 17).
/// User actions trigger async tasks, which send back result messages.
#[derive(Debug, Clone)]
pub enum UpdateMessage {
    // -------------------------------------------------------------------------
    // User Actions
    // -------------------------------------------------------------------------
    /// Open the update dialog AND trigger check (no idle state)
    Open,

    /// Close the update dialog
    Close,

    /// User confirmed download after seeing update available
    ConfirmDownload,

    /// User confirmed install after verification
    ConfirmInstall,

    /// User clicked restart to apply update
    RestartNow,

    /// User chose to skip this version
    SkipVersion,

    /// Toggle changelog expanded/collapsed in Available state
    ToggleChangelog,

    /// Retry after error (uses RetryContext from state)
    Retry,

    // -------------------------------------------------------------------------
    // Async Operation Results
    // -------------------------------------------------------------------------
    /// Result of checking for updates
    CheckResult(std::result::Result<Option<tss_updater::UpdateInfo>, String>),

    /// Download progress update (from streaming download)
    DownloadProgress(tss_updater::DownloadProgress),

    /// Download complete with data
    DownloadComplete(std::result::Result<tss_updater::DownloadResult, String>),

    /// Verification complete (verified: bool, data: `Vec<u8>`)
    VerifyResult(std::result::Result<VerifyOutcome, String>),

    /// Installation complete
    InstallResult(std::result::Result<String, String>),
}

/// Outcome of SHA256 verification.
#[derive(Debug, Clone)]
pub struct VerifyOutcome {
    /// Whether verification passed.
    pub verified: bool,
    /// The downloaded data.
    pub data: Vec<u8>,
}
