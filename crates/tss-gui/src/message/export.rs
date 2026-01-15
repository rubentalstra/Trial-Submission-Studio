//! Export view messages.
//!
//! Messages for the export functionality including domain selection,
//! format selection, and export progress tracking.
//!
//! Uses types from `crate::state` for format and version enums to avoid
//! duplication and ensure consistency.

use std::path::PathBuf;

use crate::state::{ExportFormat, ExportResult, XptVersion};

/// Messages for the Export view.
#[derive(Debug, Clone)]
pub enum ExportMessage {
    // =========================================================================
    // Domain selection
    // =========================================================================
    /// Toggle a domain for export.
    DomainToggled(String),

    /// Select all exportable domains.
    SelectAll,

    /// Deselect all domains.
    DeselectAll,

    // =========================================================================
    // Export configuration
    // =========================================================================
    /// Change the export format.
    FormatChanged(ExportFormat),

    /// User clicked to change output directory.
    OutputDirChangeClicked,

    /// User selected an output directory.
    OutputDirSelected(PathBuf),

    /// Change XPT version (V5 or V8).
    XptVersionChanged(XptVersion),

    /// Toggle Define-XML generation.
    ToggleDefineXml,

    // =========================================================================
    // Export execution
    // =========================================================================
    /// Start the export process.
    StartExport,

    /// Cancel the export in progress.
    CancelExport,

    /// Export progress update (from background task).
    Progress(ExportProgress),

    /// Export completed (from background task).
    Complete(ExportResult),

    // =========================================================================
    // Post-export actions
    // =========================================================================
    /// Dismiss the completion modal.
    DismissCompletion,

    /// Retry the export after an error.
    RetryExport,

    /// Open the output folder in file manager.
    OpenOutputFolder,
}

/// Progress update during export.
#[derive(Debug, Clone)]
pub enum ExportProgress {
    /// Starting export for a domain.
    StartingDomain(String),

    /// Current step in the export process.
    Step(ExportStep),

    /// Domain export completed.
    DomainComplete(String),

    /// Overall progress percentage (0.0 - 1.0).
    OverallProgress(f32),
}

/// Export step descriptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportStep {
    /// Preparing export.
    Preparing,
    /// Applying variable mappings.
    ApplyingMappings,
    /// Running transformations.
    RunningTransforms,
    /// Validating data.
    Validating,
    /// Writing file.
    WritingFile,
}

impl ExportStep {
    /// Returns a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Preparing => "Preparing...",
            Self::ApplyingMappings => "Applying mappings...",
            Self::RunningTransforms => "Running transforms...",
            Self::Validating => "Validating data...",
            Self::WritingFile => "Writing file...",
        }
    }
}
