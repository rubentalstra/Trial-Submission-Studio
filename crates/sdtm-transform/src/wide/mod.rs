//! Wide format processing for SDTM domains.
//!
//! This module handles the conversion of wide format CSV data to long format
//! SDTM domains. Wide format data has multiple test results in separate columns
//! (e.g., `ORRES_HR`, `ORRES_SYSBP`), while long format has one row per test.
//!
//! ## Supported Domains
//!
//! - **LB (Laboratory)**: Lab test results with values, units, and ranges
//! - **VS (Vital Signs)**: Vital sign measurements with positions
//! - **IE (Inclusion/Exclusion)**: Eligibility criteria with categories

mod ie;
mod lb;
mod types;
mod utils;
mod vs;

// Re-export public functions
pub use ie::build_ie_wide_frame;
pub use lb::build_lb_wide_frame;
pub use vs::build_vs_wide_frame;
