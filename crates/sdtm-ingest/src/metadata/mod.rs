//! Study metadata handling.
//!
//! This module provides functionality for loading and applying study metadata
//! from Items.csv and CodeLists.csv files commonly exported from EDC systems.

mod application;
mod detection;
mod loader;
mod types;

pub use application::{AppliedStudyMetadata, apply_study_metadata};
pub use loader::load_study_metadata;
pub use types::{SourceColumn, StudyCodelist, StudyMetadata};
