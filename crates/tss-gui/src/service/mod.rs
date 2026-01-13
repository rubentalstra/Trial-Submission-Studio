//! Services for background tasks.
//!
//! These services provide async functions for use with Iced's `Task::perform` pattern.

pub mod preview;

pub use preview::{PreviewError, PreviewInput, compute_preview};
