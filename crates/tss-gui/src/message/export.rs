//! Export view messages.
//!
//! Messages for the export functionality including domain selection,
//! format selection, and export progress tracking.

use std::path::PathBuf;

/// Messages for the Export view.
#[derive(Debug, Clone)]
pub enum ExportMessage {
    // =========================================================================
    // Domain selection
    // =========================================================================
    /// Toggle a domain for export
    DomainToggled(String),

    /// Select all exportable domains
    SelectAll,

    /// Deselect all domains
    DeselectAll,

    // =========================================================================
    // Export configuration
    // =========================================================================
    /// Change the export format
    FormatChanged(ExportFormat),

    /// User clicked to change output directory
    OutputDirChangeClicked,

    /// User selected an output directory
    OutputDirSelected(PathBuf),

    /// Change XPT version (V5 or V8)
    XptVersionChanged(XptVersion),

    // =========================================================================
    // Export execution
    // =========================================================================
    /// Start the export process
    StartExport,

    /// Cancel the export in progress
    CancelExport,

    /// Export progress update (from background task)
    Progress(ExportProgress),

    /// Export completed (from background task)
    Complete(Result<ExportResult, ExportError>),

    // =========================================================================
    // Post-export actions
    // =========================================================================
    /// Dismiss the completion modal
    DismissCompletion,

    /// Retry the export after an error
    RetryExport,

    /// Open the output folder in file manager
    OpenOutputFolder,
}

/// Export file format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    /// SAS Transport File (XPT)
    #[default]
    Xpt,
    /// Dataset-XML (ODM-based)
    DatasetXml,
    /// Define-XML (metadata)
    DefineXml,
}

impl ExportFormat {
    /// Returns a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Xpt => "XPT (SAS Transport)",
            Self::DatasetXml => "Dataset-XML",
            Self::DefineXml => "Define-XML 2.1",
        }
    }

    /// Returns the file extension.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Xpt => "xpt",
            Self::DatasetXml => "xml",
            Self::DefineXml => "xml",
        }
    }
}

/// XPT file version options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum XptVersion {
    /// Version 5 (legacy, wider compatibility)
    V5,
    /// Version 8 (modern, recommended)
    #[default]
    V8,
}

impl XptVersion {
    /// Returns a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::V5 => "Version 5 (Legacy)",
            Self::V8 => "Version 8 (Modern)",
        }
    }
}

/// Progress update during export.
#[derive(Debug, Clone)]
pub enum ExportProgress {
    /// Starting export for a domain
    StartingDomain(String),

    /// Current step in the export process
    Step(ExportStep),

    /// Domain export completed
    DomainComplete(String),

    /// Overall progress percentage (0.0 - 1.0)
    OverallProgress(f32),
}

/// Export step descriptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportStep {
    /// Applying variable mappings
    ApplyingMappings,
    /// Running transformations
    RunningTransforms,
    /// Validating data
    Validating,
    /// Writing file
    WritingFile,
}

impl ExportStep {
    /// Returns a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::ApplyingMappings => "Applying mappings...",
            Self::RunningTransforms => "Running transforms...",
            Self::Validating => "Validating data...",
            Self::WritingFile => "Writing file...",
        }
    }
}

/// Successful export result.
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// List of files that were written
    pub files_written: Vec<PathBuf>,
    /// Total domains exported
    pub domains_exported: usize,
    /// Any warnings generated during export
    pub warnings: Vec<String>,
}

/// Export error.
#[derive(Debug, Clone)]
pub struct ExportError {
    /// Error message
    pub message: String,
    /// Domain that caused the error (if applicable)
    pub domain: Option<String>,
    /// Whether the export can be retried
    pub retryable: bool,
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(domain) = &self.domain {
            write!(f, "Error exporting {}: {}", domain, self.message)
        } else {
            write!(f, "Export error: {}", self.message)
        }
    }
}
