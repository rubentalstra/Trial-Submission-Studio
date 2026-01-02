//! ADaM (Analysis Data Model) types per ADaMIG v1.3.
//!
//! This module provides ADaM-specific types for representing analysis datasets
//! derived from SDTM for statistical analysis.

pub mod dataset;
pub mod enums;

pub use dataset::{AdamDataset, AdamVariable};
pub use enums::{AdamDatasetType, AdamVariableSource};
