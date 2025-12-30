//! Pinnacle 21 validation rules loader.
//!
//! Loads P21 rules from the `standards/pinnacle21/Rules.csv` file.

use crate::csv_utils::{default_standards_root, read_csv_rows};
use anyhow::Result;
use sdtm_model::p21::{P21Category, P21Rule, P21RuleRegistry, P21Severity};
use std::path::Path;

/// Load P21 rules from a CSV file.
///
/// # CSV Format
///
/// Expected columns:
/// - `Pinnacle 21 ID`: Rule identifier (e.g., "CT2001", "SD0002")
/// - `Publisher ID`: FDA/CDISC codes (comma-separated)
/// - `Message`: Short message
/// - `Description`: Detailed description
/// - `Category`: Rule category
/// - `Severity`: Default severity (often empty)
pub fn load_p21_rules(csv_path: &Path) -> Result<P21RuleRegistry> {
    let rows = read_csv_rows(csv_path)?;
    let mut registry = P21RuleRegistry::new();

    for row in rows {
        let id = row.get("Pinnacle 21 ID").map(String::as_str).unwrap_or("");
        if id.is_empty() {
            continue;
        }

        let publisher_ids: Vec<String> = row
            .get("Publisher ID")
            .map(|s| {
                s.split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let message = row
            .get("Message")
            .cloned()
            .unwrap_or_default();

        let description = row
            .get("Description")
            .cloned()
            .unwrap_or_default();

        let category_str = row.get("Category").map(String::as_str).unwrap_or("");
        let category = P21Category::from_str(category_str).unwrap_or(P21Category::Consistency);

        let severity_str = row.get("Severity").map(String::as_str).unwrap_or("");
        let severity = P21Severity::from_str(severity_str);

        let rule = P21Rule {
            id: id.to_string(),
            publisher_ids,
            message,
            description,
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
pub fn load_default_p21_rules() -> Result<P21RuleRegistry> {
    let root = default_standards_root();
    let path = root.join("pinnacle21").join("Rules.csv");
    load_p21_rules(&path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_p21_rules() {
        let registry = load_default_p21_rules().expect("load P21 rules");

        // Should have loaded rules
        assert!(!registry.is_empty(), "Registry should not be empty");

        // Check some known rules exist
        assert!(registry.get("CT2001").is_some(), "CT2001 should exist");
        assert!(registry.get("CT2002").is_some(), "CT2002 should exist");
        assert!(registry.get("SD0002").is_some(), "SD0002 should exist");
        assert!(registry.get("SD0003").is_some(), "SD0003 should exist");
        assert!(registry.get("SD0005").is_some(), "SD0005 should exist");
    }

    #[test]
    fn test_p21_rule_categories() {
        let registry = load_default_p21_rules().expect("load P21 rules");

        // CT rules should be Terminology
        if let Some(ct2001) = registry.get("CT2001") {
            assert_eq!(ct2001.category, P21Category::Terminology);
        }

        // SD0002 should be Presence
        if let Some(sd0002) = registry.get("SD0002") {
            assert_eq!(sd0002.category, P21Category::Presence);
        }

        // SD0003 should be Format
        if let Some(sd0003) = registry.get("SD0003") {
            assert_eq!(sd0003.category, P21Category::Format);
        }
    }

    #[test]
    fn test_p21_rule_messages() {
        let registry = load_default_p21_rules().expect("load P21 rules");

        // Check CT2001 message
        if let Some(ct2001) = registry.get("CT2001") {
            assert!(
                ct2001.message.contains("non-extensible"),
                "CT2001 message should mention non-extensible: {}",
                ct2001.message
            );
        }

        // Check SD0002 message
        if let Some(sd0002) = registry.get("SD0002") {
            assert!(
                sd0002.message.contains("Required") || sd0002.message.contains("Null"),
                "SD0002 message should mention Required or Null: {}",
                sd0002.message
            );
        }
    }
}
