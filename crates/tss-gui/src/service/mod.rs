//! Services for background tasks.
//!
//! These services provide async functions for use with Iced's `Task::perform` pattern.
//! All services follow the pattern of taking an input struct and returning a result.

pub mod export;
pub mod generated_domains;
pub mod preview;
pub mod study;
pub mod validation;
