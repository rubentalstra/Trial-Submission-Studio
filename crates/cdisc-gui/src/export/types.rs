//! Export types - configuration, progress, errors, and UI state.

use crate::settings::ExportFormat;
use cdisc_validate::rules::Category;
use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// ============================================================================
// Export Configuration
// ============================================================================

/// Complete export configuration (resolved from settings + UI).
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Output directory (will create `datasets/` subfolder).
    pub output_dir: PathBuf,
    /// Data format: XPT or Dataset-XML (mutually exclusive).
    pub format: ExportFormat,
    /// Domain codes to export.
    pub selected_domains: BTreeSet<String>,
    /// Active bypasses for this export (for logging/debugging).
    #[allow(dead_code)]
    pub bypasses: ExportBypasses,
}

/// Active bypasses for export (derived from DeveloperSettings).
#[derive(Debug, Clone, Default)]
pub struct ExportBypasses {
    /// Whether developer mode is enabled.
    pub developer_mode: bool,
    /// Allow export despite validation errors.
    pub allow_errors: bool,
    /// Allow export with incomplete required mappings.
    pub allow_incomplete_mappings: bool,
    /// Bypass entire validation categories.
    pub bypassed_categories: HashSet<Category>,
    /// Bypass specific rule IDs (e.g., "SD0056").
    pub bypassed_rule_ids: HashSet<String>,
}

// ============================================================================
// Export Progress (Channel Messages)
// ============================================================================

/// Messages sent from background export thread to UI.
#[derive(Debug, Clone)]
pub enum ExportUpdate {
    /// Progress update for a domain.
    Progress { domain: String, step: ExportStep },
    /// File successfully written.
    FileWritten { path: PathBuf },
    /// Export completed successfully.
    Complete { result: ExportResult },
    /// Export failed.
    Error { error: ExportError },
    /// Export was cancelled by user.
    Cancelled,
}

/// Steps within the export process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportStep {
    #[default]
    Preparing,
    ApplyingMappings,
    WritingFile,
    GeneratingSUPP,
    WritingDefineXml,
}

impl ExportStep {
    /// Get human-readable label for UI.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Preparing => "Preparing...",
            Self::ApplyingMappings => "Applying mappings...",
            Self::WritingFile => "Writing file...",
            Self::GeneratingSUPP => "Generating SUPP...",
            Self::WritingDefineXml => "Writing Define-XML...",
        }
    }
}

// ============================================================================
// Export Result
// ============================================================================

/// Successful export result.
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// Output directory.
    pub output_dir: PathBuf,
    /// All files that were written.
    pub written_files: Vec<PathBuf>,
    /// Total elapsed time in milliseconds.
    pub elapsed_ms: u64,
}

/// Export error.
#[derive(Debug, Clone)]
pub struct ExportError {
    /// Domain where error occurred (None if general error).
    pub domain: Option<String>,
    /// Step where error occurred (for debugging/logging).
    #[allow(dead_code)]
    pub step: ExportStep,
    /// Error message.
    pub message: String,
    /// Additional details.
    pub details: Option<String>,
}

impl ExportError {
    /// Create a new export error.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            domain: None,
            step: ExportStep::Preparing,
            message: message.into(),
            details: None,
        }
    }

    /// Create an error for a specific domain.
    pub fn for_domain(
        domain: impl Into<String>,
        step: ExportStep,
        message: impl Into<String>,
    ) -> Self {
        Self {
            domain: Some(domain.into()),
            step,
            message: message.into(),
            details: None,
        }
    }

    /// Add details to the error.
    #[allow(dead_code)]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

// ============================================================================
// Export Handle (Cancellation)
// ============================================================================

/// Handle to cancel an in-progress export.
#[derive(Clone)]
pub struct ExportHandle {
    cancel_flag: Arc<AtomicBool>,
    written_files: Arc<Mutex<Vec<PathBuf>>>,
}

impl ExportHandle {
    /// Create a new export handle.
    pub fn new() -> Self {
        Self {
            cancel_flag: Arc::new(AtomicBool::new(false)),
            written_files: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Request cancellation.
    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }

    /// Check if cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::SeqCst)
    }

    /// Get the cancel flag for sharing with thread.
    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        self.cancel_flag.clone()
    }

    /// Get the written files tracker for sharing with thread.
    pub fn written_files(&self) -> Arc<Mutex<Vec<PathBuf>>> {
        self.written_files.clone()
    }

    /// Cleanup any partial files that were written.
    #[allow(dead_code)]
    pub fn cleanup_partial_files(&self) {
        if let Ok(files) = self.written_files.lock() {
            for path in files.iter() {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

impl Default for ExportHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ExportHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExportHandle")
            .field("is_cancelled", &self.is_cancelled())
            .finish()
    }
}

// ============================================================================
// Export UI State
// ============================================================================

/// Export phase (controls modal display).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportPhase {
    /// Main view, ready for configuration.
    #[default]
    Idle,
    /// Progress modal visible.
    Exporting,
    /// Completion modal visible.
    Complete,
}

/// UI state for the export view.
#[derive(Debug, Default)]
pub struct ExportUiState {
    /// Current phase (controls modal display).
    pub phase: ExportPhase,
    /// Domains selected for export.
    pub selected_domains: BTreeSet<String>,
    /// Override output directory.
    pub output_dir: Option<PathBuf>,
    /// Override export format.
    pub format: Option<ExportFormat>,
    /// Current domain being processed.
    pub current_domain: Option<String>,
    /// Current step within domain.
    pub current_step: ExportStep,
    /// Files written so far.
    pub written_files: Vec<PathBuf>,
    /// Expected total files for progress calculation.
    pub total_expected_files: usize,
    /// Final result (success or error).
    pub result: Option<Result<ExportResult, ExportError>>,
    /// Handle to cancel export.
    pub cancel_handle: Option<ExportHandle>,
}

impl ExportUiState {
    /// Check if export is in progress.
    #[allow(dead_code)]
    pub fn is_exporting(&self) -> bool {
        self.phase == ExportPhase::Exporting
    }

    /// Toggle domain selection.
    pub fn toggle_domain(&mut self, code: &str) {
        if self.selected_domains.contains(code) {
            self.selected_domains.remove(code);
        } else {
            self.selected_domains.insert(code.to_string());
        }
    }

    /// Select all domains from a list.
    #[allow(dead_code)]
    pub fn select_all(&mut self, codes: impl IntoIterator<Item = impl AsRef<str>>) {
        for code in codes {
            self.selected_domains.insert(code.as_ref().to_string());
        }
    }

    /// Deselect all domains.
    pub fn deselect_all(&mut self) {
        self.selected_domains.clear();
    }

    /// Get the number of selected domains.
    pub fn selection_count(&self) -> usize {
        self.selected_domains.len()
    }

    /// Get effective output directory (with fallback to default).
    #[allow(dead_code)]
    pub fn output_dir_or_default(&self, study_folder: &std::path::Path) -> PathBuf {
        self.output_dir
            .clone()
            .unwrap_or_else(|| study_folder.join("export"))
    }

    /// Request cancellation of in-progress export.
    pub fn request_cancel(&mut self) {
        if let Some(ref handle) = self.cancel_handle {
            handle.cancel();
        }
    }

    /// Reset to idle state.
    pub fn reset(&mut self) {
        self.phase = ExportPhase::Idle;
        self.current_domain = None;
        self.current_step = ExportStep::Preparing;
        self.written_files.clear();
        self.total_expected_files = 0;
        self.result = None;
        self.cancel_handle = None;
    }

    /// Reset all state (e.g., when loading a new study).
    #[allow(dead_code)]
    pub fn reset_all(&mut self) {
        self.selected_domains.clear();
        self.output_dir = None;
        self.format = None;
        self.reset();
    }

    /// Calculate progress fraction (0.0 to 1.0).
    pub fn progress_fraction(&self) -> f32 {
        if self.total_expected_files == 0 {
            return 0.0;
        }
        self.written_files.len() as f32 / self.total_expected_files as f32
    }
}
