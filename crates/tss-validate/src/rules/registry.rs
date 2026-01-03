//! P21 rule registry loaded from CSV.

use std::collections::HashMap;

use super::category::Category;
use crate::issue::Severity;

/// A P21 validation rule loaded from CSV.
#[derive(Debug, Clone)]
pub struct Rule {
    pub id: String,
    pub publisher_ids: Vec<String>,
    pub message: String,
    pub description: String,
    pub category: Category,
    pub severity: Option<Severity>,
}

/// Registry of P21 rules indexed by ID.
#[derive(Debug, Clone, Default)]
pub struct RuleRegistry {
    rules: HashMap<String, Rule>,
}

impl RuleRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    /// Create registry from a map of rules.
    pub fn from_rules(rules: HashMap<String, Rule>) -> Self {
        Self { rules }
    }

    /// Insert a rule into the registry.
    pub fn insert(&mut self, rule: Rule) {
        self.rules.insert(rule.id.clone(), rule);
    }

    /// Get rule by ID.
    pub fn get(&self, id: &str) -> Option<&Rule> {
        self.rules.get(id)
    }

    /// Number of rules in the registry.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Check if registry is empty.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Iterate over all rules.
    pub fn iter(&self) -> impl Iterator<Item = &Rule> {
        self.rules.values()
    }
}
