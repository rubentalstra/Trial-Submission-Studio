//! Re-export of data utilities from sdtm-transform.
//!
//! This module provides backward compatibility for internal imports.
//! All data utility functions are now implemented in the `sdtm-transform` crate.

pub(crate) use sdtm_transform::data_utils::{
    column_trimmed_values, column_value_string, mapping_source_for_target, sanitize_qnam,
    sanitize_test_code, strip_all_quotes, strip_quotes, table_label,
};
