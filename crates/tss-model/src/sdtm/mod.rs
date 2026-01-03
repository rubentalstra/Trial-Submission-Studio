//! SDTM (Study Data Tabulation Model) types per SDTMIG v3.4.
//!
//! This module provides SDTM-specific types for representing domains and variables
//! in clinical trial tabulation datasets.

pub mod domain;
pub mod enums;

pub use domain::{DatasetClass, Domain, Variable, VariableType};
pub use enums::VariableRole;
