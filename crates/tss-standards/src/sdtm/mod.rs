//! SDTM (Study Data Tabulation Model) types per SDTMIG v3.4.
//!
//! This module provides SDTM-specific types for representing domains and variables
//! in clinical trial tabulation datasets.

pub mod domain;
pub mod enums;
pub mod reciprocal;

pub use domain::{SdtmDomain, SdtmVariable};
pub use enums::{SdtmDatasetClass, VariableRole};
pub use reciprocal::{get_parent_srel_for_child, get_reciprocal_srel, is_symmetric_srel};
