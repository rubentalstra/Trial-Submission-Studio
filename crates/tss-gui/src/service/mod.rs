//! Services for background tasks.
//!
//! These services provide async functions for use with Iced's `Task::perform` pattern.
//! All services follow the pattern of taking an input struct and returning a result.

// Allow unused imports - these are public API re-exports
#![allow(unused_imports)]

pub mod export;
pub mod preview;
pub mod update_checker;
pub mod validation;

pub use export::{DomainExportData, ExportError, ExportInput, execute_export};
pub use preview::{PreviewError, PreviewInput, compute_preview};
pub use update_checker::check_for_updates;
pub use validation::{ValidationInput, compute_validation};
