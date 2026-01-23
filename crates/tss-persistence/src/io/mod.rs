//! File I/O operations for project persistence.
//!
//! This module handles:
//! - Saving projects with atomic writes
//! - Loading projects with format validation
//! - Source file hashing for change detection

mod hash;
mod load;
mod save;

pub use hash::{compute_file_hash, verify_file_hash};
pub use load::{load_project, load_project_async};
pub use save::{save_project, save_project_async};
