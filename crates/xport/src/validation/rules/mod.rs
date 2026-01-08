//! Validation rules for XPT datasets.
//!
//! This module contains individual validation rules organized by category:
//! - Name validation (dataset and variable names)
//! - Label validation
//! - Format validation
//! - Dataset structure validation
//! - FDA-specific rules

mod dataset;
mod fda;
mod format;
mod label;
mod name;

pub use dataset::{DuplicateVariableRule, VariableLengthRule};
pub use fda::{FdaAsciiRule, FdaVersionRule};
pub use format::FormatNameRule;
pub use label::{DatasetLabelRule, VariableLabelRule};
pub use name::{DatasetNameRule, VariableNameRule};

use crate::header::normalize_name;

/// Check if a name contains only valid SAS characters (A-Z, 0-9, _).
fn is_valid_sas_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let normalized = normalize_name(name);
    let chars: Vec<char> = normalized.chars().collect();

    // Must start with a letter
    if !chars.first().is_some_and(char::is_ascii_alphabetic) {
        return false;
    }

    // Rest must be alphanumeric or underscore
    chars.iter().all(|c| c.is_ascii_alphanumeric() || *c == '_')
}

/// Check if a string contains only ASCII printable characters.
fn is_ascii_printable(s: &str) -> bool {
    s.bytes().all(|b| (0x20..=0x7E).contains(&b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_sas_names() {
        assert!(is_valid_sas_name("DM"));
        assert!(is_valid_sas_name("USUBJID"));
        assert!(is_valid_sas_name("VAR_1"));
        assert!(is_valid_sas_name("A"));
    }

    #[test]
    fn test_invalid_sas_names() {
        assert!(!is_valid_sas_name(""));
        assert!(!is_valid_sas_name("1VAR")); // starts with number
        assert!(!is_valid_sas_name("_VAR")); // starts with underscore
        assert!(!is_valid_sas_name("VAR-1")); // contains hyphen
        assert!(!is_valid_sas_name("VAR 1")); // contains space
    }

    #[test]
    fn test_ascii_printable() {
        assert!(is_ascii_printable("Hello World!"));
        assert!(is_ascii_printable("ABC 123 @#$"));
        assert!(!is_ascii_printable("Hello\tWorld")); // tab
        assert!(!is_ascii_printable("Héllo")); // non-ASCII
    }
}
