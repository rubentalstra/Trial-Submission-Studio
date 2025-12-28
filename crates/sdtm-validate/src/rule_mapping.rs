//! Rule resolver for P21 validation rules.
//!
//! All rule metadata (IDs, messages, descriptions, categories, severities)
//! is loaded dynamically from `standards/p21/Rules.csv`. There are no
//! hardcoded rule definitions in this module.
//!
//! # Usage
//!
//! ```ignore
//! let p21_rules = load_default_p21_rules()?;
//! let resolver = RuleResolver::new(&p21_rules);
//!
//! // Look up rule metadata by ID
//! if let Some(rule) = resolver.get_rule("SD1079") {
//!     println!("Message: {}", rule.message);
//!     println!("Category: {:?}", rule.category);
//! }
//! ```

use std::collections::BTreeMap;

use sdtm_standards::loaders::P21Rule;

/// Resolves rule metadata from P21 rules loaded from CSV.
///
/// All rule metadata comes from the loaded `standards/p21/Rules.csv` file.
#[derive(Debug, Clone)]
pub struct RuleResolver {
    lookup: BTreeMap<String, P21Rule>,
}

impl RuleResolver {
    /// Create a new rule resolver from P21 rules loaded from CSV.
    pub fn new(p21_rules: &[P21Rule]) -> Self {
        let mut lookup = BTreeMap::new();
        for rule in p21_rules {
            lookup.insert(rule.rule_id.to_uppercase(), rule.clone());
        }
        Self { lookup }
    }

    /// Get rule by ID.
    pub fn get_rule(&self, rule_id: &str) -> Option<&P21Rule> {
        self.lookup.get(&rule_id.to_uppercase())
    }

    /// Check if a rule ID exists.
    pub fn has_rule(&self, rule_id: &str) -> bool {
        self.lookup.contains_key(&rule_id.to_uppercase())
    }

    /// Get the message for a rule.
    pub fn get_message(&self, rule_id: &str) -> Option<String> {
        self.get_rule(rule_id).map(|r| {
            if r.message.is_empty() {
                r.description.clone()
            } else {
                r.message.clone()
            }
        })
    }

    /// Get the description for a rule.
    pub fn get_description(&self, rule_id: &str) -> Option<String> {
        self.get_rule(rule_id)
            .filter(|r| !r.description.is_empty())
            .map(|r| r.description.clone())
    }

    /// Get the category for a rule.
    pub fn get_category(&self, rule_id: &str) -> Option<String> {
        self.get_rule(rule_id).and_then(|r| r.category.clone())
    }

    /// Get the severity for a rule.
    pub fn get_severity(&self, rule_id: &str) -> Option<String> {
        self.get_rule(rule_id).and_then(|r| r.severity.clone())
    }
}

impl Default for RuleResolver {
    fn default() -> Self {
        Self::new(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolver_with_empty_rules() {
        let resolver = RuleResolver::default();
        assert!(!resolver.has_rule("SD0002"));
        assert!(resolver.get_message("SD0002").is_none());
    }

    #[test]
    fn test_resolver_with_rules() {
        let rules = vec![P21Rule {
            rule_id: "SD0002".to_string(),
            publisher_id: None,
            message: "Test message".to_string(),
            description: "Test description".to_string(),
            category: Some("Presence".to_string()),
            severity: Some("Error".to_string()),
        }];
        let resolver = RuleResolver::new(&rules);
        assert!(resolver.has_rule("SD0002"));
        assert_eq!(
            resolver.get_message("SD0002"),
            Some("Test message".to_string())
        );
        assert_eq!(
            resolver.get_category("SD0002"),
            Some("Presence".to_string())
        );
    }

    #[test]
    fn test_resolver_case_insensitive() {
        let rules = vec![P21Rule {
            rule_id: "SD1079".to_string(),
            publisher_id: None,
            message: "Variable is in wrong order".to_string(),
            description: "Variables should be ordered correctly".to_string(),
            category: Some("Metadata".to_string()),
            severity: None,
        }];
        let resolver = RuleResolver::new(&rules);
        assert!(resolver.has_rule("sd1079"));
        assert!(resolver.has_rule("SD1079"));
    }
}
