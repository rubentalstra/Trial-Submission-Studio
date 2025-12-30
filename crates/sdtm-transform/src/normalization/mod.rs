//! Normalization functions for SDTM data transformation.
//!
//! This module provides functions for normalizing data to SDTM-compliant formats:
//! - **datetime**: ISO 8601 date/time parsing and formatting
//! - **studyday**: Study day calculation (--DY variables)
//! - **duration**: ISO 8601 duration formatting
//! - **ct**: Controlled terminology normalization
//! - **numeric**: Numeric type conversion

pub mod ct;
pub mod datetime;
pub mod duration;
pub mod numeric;
pub mod studyday;

// Re-export commonly used items
pub use ct::{normalize_ct_value, normalize_without_codelist, CtNormalizationResult};
pub use datetime::{
    format_iso8601_date, format_iso8601_datetime, parse_date, parse_date_precision,
    transform_to_iso8601, DateTimePrecision,
};
pub use duration::format_iso8601_duration;
pub use numeric::{is_numeric, parse_numeric, transform_to_numeric};
pub use studyday::{calculate_study_day, calculate_study_day_from_strings};
