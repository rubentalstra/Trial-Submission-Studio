//! Application state management
//!
//! Contains all runtime state types for the GUI application.

mod app_state;
mod study_state;
mod transform_state;

pub use app_state::{AppState, EditorTab, View};
pub use study_state::{DomainState, DomainStatus, StudyState};
pub use transform_state::{TransformRule, TransformState};
