//! Application state management.
//!
//! This module contains all runtime state types for the GUI application.
//! The architecture separates concerns into:
//!
//! - **AppState**: Root state with DM-enforced domain access
//! - **StudyState**: Study-level state with DM dependency tracking
//! - **DomainState**: Domain data with version tracking
//! - **DerivedState**: Cached computed state with version-based invalidation
//! - **UiState**: All UI state (separated from domain data)
//!
//! # DM Dependency Enforcement
//!
//! The DM domain must have a valid preview before other domains can be accessed.
//! This is enforced at the `AppState` level through the `domain()` method.
//!
//! # Version-Based Cache Invalidation
//!
//! Each `DomainState` has a `version` counter that increments on mutation.
//! Derived state (`DerivedState`) stores the version it was computed from,
//! enabling automatic staleness detection.

mod app_state;
mod derived_state;
mod domain_state;
mod study_state;
mod ui_state;
mod versioned;

// App state
pub use app_state::{AppState, EditorTab, View};

// Study state
pub use study_state::StudyState;

// Domain state
pub use domain_state::{DomainSource, DomainState};

// Derived state
pub use derived_state::{
    DerivedState, SuppAction, SuppColumnConfig, SuppConfig, suggest_qnam, validate_qnam,
};

// UI state
pub use ui_state::UiState;

// Versioned wrapper
pub use versioned::Versioned;
