//! State management for Trial Submission Studio (Iced Architecture).
//!
//! # Architecture Overview
//!
//! This module follows Iced's Elm architecture principles:
//!
//! - **State is the single source of truth**
//! - **All state changes happen through messages in `update()`**
//! - **Views are pure functions of state**
//! - **Async operations use `Task`, not channels**
//!
//! ## Module Structure
//!
//! - [`AppState`] - Root state container (flat, no nesting complexity)
//! - [`Study`] - Loaded study with domains (domain data)
//! - [`Domain`] - Single SDTM domain with source data and mapping
//! - [`ViewState`] - Current view and its UI state
//! - [`Settings`] - Persisted user preferences
//!
//! ## Key Design Decisions
//!
//! 1. **No channels/polling** - Iced's `Task` system handles async
//! 2. **No mutable accessors** - State changes only in `update()`
//! 3. **Derived data computed on demand** - Not cached in state
//! 4. **UI state lives with views** - Not mixed with domain data

mod domain;
mod settings;
mod study;
mod view_state;

// Re-exports
pub use domain::{Domain, DomainSource, SuppAction, SuppColumnConfig, SuppOrigin};
pub use settings::{ExportFormat, ExportSettings, GeneralSettings, Settings};
pub use study::Study;
pub use view_state::{
    EditorTab, ExportFormat, ExportPhase, ExportResult, ExportViewState, MappingUiState,
    NormalizationUiState, NotCollectedEdit, PreviewUiState, SeverityFilter, SuppEditDraft,
    SuppFilterMode, SuppUiState, ValidationUiState, ViewState, WorkflowMode, XptVersion,
};

use tss_model::TerminologyRegistry;

// =============================================================================
// ROOT APPLICATION STATE
// =============================================================================

/// Root application state.
///
/// This is intentionally flat and simple. All state changes happen through
/// message handling in `App::update()`.
///
/// # Example
///
/// ```ignore
/// // In App::update()
/// Message::StudyLoaded(result) => {
///     match result {
///         Ok(study) => {
///             self.state.study = Some(study);
///             self.state.view = ViewState::home();
///         }
///         Err(e) => {
///             self.state.error = Some(e);
///         }
///     }
///     Task::none()
/// }
/// ```
pub struct AppState {
    /// Current view and its associated UI state.
    ///
    /// This determines what's rendered and holds view-specific state
    /// (selections, pagination, filters, etc.)
    pub view: ViewState,

    /// Loaded study data.
    ///
    /// `None` when no study is open. Contains all domain data and mappings.
    pub study: Option<Study>,

    /// User settings (persisted to disk).
    pub settings: Settings,

    /// CDISC Controlled Terminology registry.
    ///
    /// Loaded once when first study is opened. Used for validation
    /// and controlled term lookups.
    pub terminology: Option<TerminologyRegistry>,

    /// Current error message to display (transient).
    ///
    /// Set when an operation fails, cleared on user acknowledgment.
    pub error: Option<String>,

    /// Whether a background task is running (for UI feedback).
    pub is_loading: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            view: ViewState::default(),
            study: None,
            settings: Settings::default(),
            terminology: None,
            error: None,
            is_loading: false,
        }
    }
}

impl AppState {
    /// Create app state with loaded settings.
    pub fn with_settings(settings: Settings) -> Self {
        Self {
            settings,
            ..Default::default()
        }
    }

    // =========================================================================
    // READ-ONLY ACCESSORS (no mutation - that happens in update())
    // =========================================================================

    /// Get study reference.
    #[inline]
    pub fn study(&self) -> Option<&Study> {
        self.study.as_ref()
    }

    /// Get domain by code.
    #[inline]
    pub fn domain(&self, code: &str) -> Option<&Domain> {
        self.study.as_ref()?.domain(code)
    }

    /// Check if a study is loaded.
    #[inline]
    pub fn has_study(&self) -> bool {
        self.study.is_some()
    }

    /// Get domain codes (for iteration).
    pub fn domain_codes(&self) -> Vec<&str> {
        self.study
            .as_ref()
            .map(|s| s.domain_codes())
            .unwrap_or_default()
    }

    /// Get current workflow mode from view state.
    #[inline]
    pub fn workflow_mode(&self) -> WorkflowMode {
        self.view.workflow_mode()
    }

    /// Get current domain code if in domain editor.
    #[inline]
    pub fn current_domain_code(&self) -> Option<&str> {
        self.view.current_domain()
    }

    /// Get current editor tab if in domain editor.
    #[inline]
    pub fn current_tab(&self) -> Option<EditorTab> {
        self.view.current_tab()
    }
}
