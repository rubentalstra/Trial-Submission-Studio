//! Application state management.
//!
//! This module contains all runtime state types for the GUI application.
//! The architecture separates concerns into:
//!
//! - **AppState**: Root state with domain access
//! - **StudyState**: Study-level state
//! - **DomainState**: Domain data with mapping state
//! - **DerivedState**: Cached computed state (preview, validation, SUPP)
//! - **UiState**: All UI state (separated from domain data)

mod app_state;
mod derived_state;
mod domain_state;
mod study_state;
mod ui_state;

// App state
pub use app_state::{AppState, EditorTab, View, WorkflowMode};

// Study state
pub use study_state::StudyState;

// Domain state
pub use domain_state::{DomainSource, DomainState};

// Derived state
pub use derived_state::{
    DerivedState, QualifierOrigin, SuppAction, SuppColumnConfig, SuppConfig, suggest_qnam,
    validate_qnam,
};

// UI state
pub use ui_state::{AboutUiState, ThirdPartyUiState, UiState, UpdateDialogState};
