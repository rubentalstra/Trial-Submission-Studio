//! Pinnacle 21 validation rules loader.
//!
//! Loads P21 rules from the `standards/pinnacle21/Rules.csv` file
//! using type-safe serde deserialization.

use crate::{P21Category, P21Rule, P21RuleRegistry, P21Severity};
use std::path::{Path, PathBuf};

/// Environment variable for overriding the standards directory.
pub const STANDARDS_ENV_VAR: &str = "CDISC_STANDARDS_DIR";

/// Get the default standards root directory.
///
/// Checks `CDISC_STANDARDS_DIR` environment variable first,
/// then falls back to the `standards/` directory relative to workspace root.
fn standards_root() -> PathBuf {
    if let Ok(root) = std::env::var(STANDARDS_ENV_VAR) {
        return PathBuf::from(root);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../standards")
}

/// CSV row structure for P21 Rules.csv.
#[derive(Debug, serde::Deserialize)]
struct P21CsvRow {
    #[serde(rename = "Pinnacle 21 ID")]
    id: String,
    #[serde(rename = "Publisher ID")]
    publisher_id: String,
    #[serde(rename = "Message")]
    message: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Category")]
    category: String,
    #[serde(rename = "Severity")]
    severity: String,
}

/// Load P21 rules from a CSV file.
///
/// # Arguments
///
/// * `csv_path` - Path to the P21 Rules.csv file
///
/// # Returns
///
/// A registry of P21 rules indexed by ID.
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn load_p21_rules(csv_path: &Path) -> Result<P21RuleRegistry, P21LoadError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(csv_path)
        .map_err(|e| P21LoadError::CsvRead {
            path: csv_path.to_path_buf(),
            source: e,
        })?;

    let mut registry = P21RuleRegistry::new();

    for result in reader.deserialize::<P21CsvRow>() {
        let row = result.map_err(|e| P21LoadError::CsvRead {
            path: csv_path.to_path_buf(),
            source: e,
        })?;

        // Skip empty IDs
        if row.id.trim().is_empty() {
            continue;
        }

        let publisher_ids: Vec<String> = row
            .publisher_id
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let category = P21Category::from_str(&row.category).unwrap_or(P21Category::Consistency);
        let severity = P21Severity::from_str(&row.severity);

        let rule = P21Rule {
            id: row.id.trim().to_string(),
            publisher_ids,
            message: row.message,
            description: row.description,
            category,
            severity,
        };

        registry.insert(rule);
    }

    Ok(registry)
}

/// Load P21 rules from the default location.
///
/// Looks for `standards/pinnacle21/Rules.csv` relative to the standards root.
pub fn load_default_p21_rules() -> Result<P21RuleRegistry, P21LoadError> {
    let path = standards_root().join("pinnacle21").join("Rules.csv");
    if !path.exists() {
        return Err(P21LoadError::FileNotFound { path });
    }
    load_p21_rules(&path)
}

/// Error type for P21 loading operations.
#[derive(Debug)]
pub enum P21LoadError {
    /// CSV file not found.
    FileNotFound { path: PathBuf },
    /// Failed to read or parse CSV.
    CsvRead { path: PathBuf, source: csv::Error },
}

impl std::fmt::Display for P21LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound { path } => {
                write!(f, "P21 rules file not found: {}", path.display())
            }
            Self::CsvRead { path, source } => {
                write!(
                    f,
                    "Failed to read P21 rules from {}: {}",
                    path.display(),
                    source
                )
            }
        }
    }
}

impl std::error::Error for P21LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CsvRead { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_p21_rules() {
        let registry = load_default_p21_rules().expect("load P21 rules");
        assert!(!registry.is_empty(), "P21 registry should not be empty");

        // Check for known rules
        let ct2001 = registry.get("CT2001");
        assert!(ct2001.is_some(), "CT2001 should be present");

        let rule = ct2001.unwrap();
        assert_eq!(rule.category, P21Category::Terminology);
        assert!(!rule.message.is_empty());
    }

    #[test]
    fn test_p21_rule_counts() {
        let registry = load_default_p21_rules().expect("load P21 rules");
        // We expect 500+ rules based on the Rules.csv file
        assert!(
            registry.len() >= 500,
            "Expected at least 500 rules, got {}",
            registry.len()
        );
    }
}
