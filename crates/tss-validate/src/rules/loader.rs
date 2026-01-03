//! CSV loader for P21 rules.

use std::collections::HashMap;
use std::path::Path;

use super::category::Category;
use super::registry::{Rule, RuleRegistry};
use crate::issue::Severity;

/// Error loading rules from CSV.
#[derive(Debug)]
pub enum LoadError {
    Io(std::io::Error),
    Csv(csv::Error),
    MissingColumn(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Csv(e) => write!(f, "CSV error: {}", e),
            Self::MissingColumn(col) => write!(f, "Missing column: {}", col),
        }
    }
}

impl std::error::Error for LoadError {}

impl From<std::io::Error> for LoadError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<csv::Error> for LoadError {
    fn from(e: csv::Error) -> Self {
        Self::Csv(e)
    }
}

/// Load rules from a CSV file.
pub fn load_rules(path: &Path) -> Result<RuleRegistry, LoadError> {
    let mut reader = csv::Reader::from_path(path)?;
    let headers = reader.headers()?.clone();

    // Find column indices
    let id_idx = find_column(&headers, "Pinnacle 21 ID")?;
    let publisher_idx = find_column(&headers, "Publisher ID")?;
    let message_idx = find_column(&headers, "Message")?;
    let description_idx = find_column(&headers, "Description")?;
    let category_idx = find_column(&headers, "Category")?;
    let severity_idx = find_column(&headers, "Severity")?;

    let mut rules = HashMap::new();

    for result in reader.records() {
        let record = result?;

        let id = record.get(id_idx).unwrap_or("").trim().to_string();
        if id.is_empty() {
            continue;
        }

        let publisher_ids: Vec<String> = record
            .get(publisher_idx)
            .unwrap_or("")
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let message = record.get(message_idx).unwrap_or("").trim().to_string();
        let description = record.get(description_idx).unwrap_or("").trim().to_string();
        let category = Category::parse(record.get(category_idx).unwrap_or(""));
        let severity = parse_severity(record.get(severity_idx).unwrap_or(""));

        let rule = Rule {
            id: id.clone(),
            publisher_ids,
            message,
            description,
            category,
            severity,
        };

        rules.insert(id, rule);
    }

    Ok(RuleRegistry::from_rules(rules))
}

/// Load rules from the default location.
pub fn load_default_rules() -> Result<RuleRegistry, LoadError> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("standards")
        .join("validation")
        .join("sdtm")
        .join("Rules.csv");

    load_rules(&path)
}

fn find_column(headers: &csv::StringRecord, name: &str) -> Result<usize, LoadError> {
    headers
        .iter()
        .position(|h| h == name)
        .ok_or_else(|| LoadError::MissingColumn(name.to_string()))
}

fn parse_severity(s: &str) -> Option<Severity> {
    match s.trim().to_lowercase().as_str() {
        "reject" => Some(Severity::Reject),
        "error" => Some(Severity::Error),
        "warning" => Some(Severity::Warning),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_rules() {
        let registry = load_default_rules().expect("Failed to load rules");
        assert!(!registry.is_empty(), "Should have loaded some rules");

        // Check a known rule exists
        let ct2001 = registry.get("CT2001");
        assert!(ct2001.is_some(), "CT2001 should exist");

        let rule = ct2001.unwrap();
        assert_eq!(rule.category, Category::Terminology);
        assert!(!rule.message.is_empty());
    }
}
