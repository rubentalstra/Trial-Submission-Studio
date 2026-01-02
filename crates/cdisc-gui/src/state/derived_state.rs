//! Derived state - cached computations.
//!
//! This module contains `DerivedState` which holds cached derived data
//! (preview DataFrame, validation report, SUPP config).

use cdisc_validate::ValidationReport;
use polars::prelude::DataFrame;
use std::collections::BTreeMap;

// ============================================================================
// Derived State Container
// ============================================================================

/// Cached derived state for a domain.
///
/// Fields are rebuilt immediately when mappings change.
#[derive(Default)]
pub struct DerivedState {
    /// Validation report (issues found in mapping/data)
    pub validation: Option<ValidationReport>,
    /// Preview DataFrame (transformed output)
    pub preview: Option<DataFrame>,
    /// SUPP configuration (for unmapped columns)
    pub supp: Option<SuppConfig>,
}

impl DerivedState {
    /// Get mutable SUPP config.
    pub fn supp_mut(&mut self) -> Option<&mut SuppConfig> {
        self.supp.as_mut()
    }
}

impl std::fmt::Debug for DerivedState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DerivedState")
            .field("has_validation", &self.validation.is_some())
            .field("has_preview", &self.preview.is_some())
            .field("has_supp", &self.supp.is_some())
            .finish()
    }
}

// ============================================================================
// SUPP Configuration (Domain Data)
// ============================================================================

/// SUPP configuration for unmapped source columns.
///
/// This is domain data (not UI state) that determines which
/// unmapped columns go into the SUPP-- dataset.
#[derive(Debug, Clone, Default)]
pub struct SuppConfig {
    /// Configuration for each unmapped source column
    pub columns: BTreeMap<String, SuppColumnConfig>,
}

impl SuppConfig {
    /// Create new SUPP config from unmapped columns.
    pub fn from_unmapped(unmapped_columns: &[String], domain_code: &str) -> Self {
        let columns = unmapped_columns
            .iter()
            .map(|col| (col.clone(), SuppColumnConfig::new(col.clone(), domain_code)))
            .collect();
        Self { columns }
    }

    /// Count columns by action: (pending, added, skipped)
    pub fn count_by_action(&self) -> (usize, usize, usize) {
        let mut pending = 0;
        let mut added = 0;
        let mut skipped = 0;
        for config in self.columns.values() {
            match config.action {
                SuppAction::Pending => pending += 1,
                SuppAction::AddToSupp => added += 1,
                SuppAction::Skip => skipped += 1,
            }
        }
        (pending, added, skipped)
    }

    /// Get column names in sorted order.
    pub fn column_names(&self) -> Vec<&str> {
        self.columns.keys().map(|s| s.as_str()).collect()
    }

    /// Get config for a specific column.
    pub fn get(&self, column: &str) -> Option<&SuppColumnConfig> {
        self.columns.get(column)
    }

    /// Get mutable config for a specific column.
    pub fn get_mut(&mut self, column: &str) -> Option<&mut SuppColumnConfig> {
        self.columns.get_mut(column)
    }
}

/// Configuration for a single source column in SUPP.
#[derive(Debug, Clone)]
pub struct SuppColumnConfig {
    /// Action: Add to SUPP, Skip, or Pending
    pub action: SuppAction,
    /// QNAM value (max 8 chars, uppercase, no leading numbers)
    pub qnam: String,
    /// QLABEL value (max 40 chars)
    pub qlabel: String,
}

impl SuppColumnConfig {
    /// Create a new config with auto-suggested QNAM.
    pub fn new(source_column: String, domain_code: &str) -> Self {
        let suggested = suggest_qnam(&source_column, domain_code);
        Self {
            action: SuppAction::Pending,
            qnam: suggested,
            qlabel: String::new(),
        }
    }

    /// Validate QNAM according to SDTMIG rules.
    pub fn validate_qnam(&self) -> Result<(), String> {
        validate_qnam(&self.qnam)
    }
}

/// Action for a SUPP column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SuppAction {
    /// Not yet decided
    #[default]
    Pending,
    /// Include in SUPP-- dataset
    AddToSupp,
    /// Exclude from export
    Skip,
}

// ============================================================================
// QNAM Validation Helpers
// ============================================================================

/// Suggest a QNAM from a source column name.
///
/// Rules per SDTMIG:
/// - Uppercase only
/// - Max 8 characters
/// - No leading numbers
/// - Prefix with domain code
pub fn suggest_qnam(column_name: &str, domain_code: &str) -> String {
    let domain_upper = domain_code.to_uppercase();

    // Clean up the column name
    let clean = column_name
        .to_uppercase()
        .replace('_', "")
        .replace('-', "")
        .replace(' ', "");

    // Strip common prefixes
    let base = clean
        .strip_prefix("EXTRA")
        .or_else(|| clean.strip_prefix("ADDITIONAL"))
        .or_else(|| clean.strip_prefix("OTHER"))
        .or_else(|| clean.strip_prefix("CUSTOM"))
        .unwrap_or(&clean);

    // If column already starts with domain code, don't add it again
    // e.g., "AEACNIMPNA" for domain "AE" should stay "AEACNIMP" not become "AEAEACNI"
    let already_has_prefix = base.starts_with(&domain_upper);

    // If base is empty or starts with a number, use column name chars
    let base = if base.is_empty()
        || base
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
    {
        &clean
    } else {
        base
    };

    if already_has_prefix {
        // Column already has domain prefix, just truncate to 8 chars
        base.chars().take(8).collect()
    } else {
        // Add domain prefix
        let max_base_len = 8usize.saturating_sub(domain_upper.len());
        let truncated_base: String = base.chars().take(max_base_len).collect();
        let suggested = format!("{}{}", domain_upper, truncated_base);
        suggested.chars().take(8).collect()
    }
}

/// Validate a QNAM according to SDTMIG rules.
pub fn validate_qnam(qnam: &str) -> Result<(), String> {
    if qnam.is_empty() {
        return Err("QNAM cannot be empty".to_string());
    }
    if qnam.len() > 8 {
        return Err("QNAM must be 8 characters or less".to_string());
    }
    if qnam
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        return Err("QNAM cannot start with a number".to_string());
    }
    if !qnam.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("QNAM can only contain letters, numbers, and underscores".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_qnam() {
        assert_eq!(suggest_qnam("extra_data", "AE"), "AEDATA");
        assert_eq!(suggest_qnam("custom_field", "DM"), "DMFIELD");
        assert_eq!(suggest_qnam("SUBJECT_ID", "AE"), "AESUBJEC");

        // Don't add domain prefix if column already starts with it
        assert_eq!(suggest_qnam("AEACNIMPNA", "AE"), "AEACNIMP");
        assert_eq!(suggest_qnam("AETERM", "AE"), "AETERM");
        assert_eq!(suggest_qnam("DMRACE", "DM"), "DMRACE");
    }

    #[test]
    fn test_validate_qnam() {
        assert!(validate_qnam("AEDATA").is_ok());
        assert!(validate_qnam("").is_err());
        assert!(validate_qnam("123TEST").is_err());
        assert!(validate_qnam("TOOLONGNAME").is_err());
        assert!(validate_qnam("TEST@VAL").is_err());
    }
}
