//! Pinnacle 21 validation rule types.
//!
//! This module provides types for representing Pinnacle 21 validation rules
//! loaded from the official P21 Rules.csv file.
//!
//! # Rule Categories
//!
//! P21 rules are organized into categories:
//! - **Terminology**: Controlled terminology validation (CT2xxx)
//! - **Presence**: Required/expected variable checks (SD0001, SD0002, etc.)
//! - **Format**: Data format validation like ISO 8601 (SD0003)
//! - **Consistency**: Cross-variable consistency checks
//! - **Limit**: Value range and boundary checks
//! - **Metadata**: Define.xml and dataset structure checks
//! - **Cross-reference**: Cross-domain reference checks
//! - **Structure**: Dataset structure validation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pinnacle 21 validation rule category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum P21Category {
    /// Controlled terminology validation (CT2xxx rules).
    Terminology,
    /// Variable presence checks (required/expected).
    Presence,
    /// Data format validation (ISO 8601, lengths).
    Format,
    /// Cross-variable consistency checks.
    Consistency,
    /// Value range and boundary checks.
    Limit,
    /// Define.xml and dataset structure.
    Metadata,
    /// Cross-domain reference validation.
    CrossReference,
    /// Dataset structure validation.
    Structure,
}

impl P21Category {
    /// Parse category from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "terminology" => Some(Self::Terminology),
            "presence" => Some(Self::Presence),
            "format" => Some(Self::Format),
            "consistency" => Some(Self::Consistency),
            "limit" => Some(Self::Limit),
            "metadata" => Some(Self::Metadata),
            "cross-reference" | "crossreference" => Some(Self::CrossReference),
            "structure" => Some(Self::Structure),
            _ => None,
        }
    }

    /// Get human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Terminology => "Terminology",
            Self::Presence => "Presence",
            Self::Format => "Format",
            Self::Consistency => "Consistency",
            Self::Limit => "Limit",
            Self::Metadata => "Metadata",
            Self::CrossReference => "Cross-reference",
            Self::Structure => "Structure",
        }
    }
}

/// Pinnacle 21 rule severity.
///
/// Most P21 rules don't have an explicit severity - they derive it from context.
/// For example, CT2001 (non-extensible CT) is an Error, CT2002 (extensible CT) is a Warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub enum P21Severity {
    /// Rule not applicable / informational.
    #[default]
    Notice,
    /// Warning - should be reviewed.
    Warning,
    /// Error - requires correction.
    Error,
    /// Reject - blocks submission.
    Reject,
}

impl P21Severity {
    /// Parse severity from string.
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "reject" => Self::Reject,
            "error" => Self::Error,
            "warning" => Self::Warning,
            _ => Self::Notice,
        }
    }
}

/// A Pinnacle 21 validation rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P21Rule {
    /// Pinnacle 21 rule ID (e.g., "CT2001", "SD0002").
    pub id: String,
    /// Publisher IDs (FDA, CDISC codes).
    pub publisher_ids: Vec<String>,
    /// Short message describing the issue.
    pub message: String,
    /// Detailed description of the rule.
    pub description: String,
    /// Rule category.
    pub category: P21Category,
    /// Default severity (can be overridden by context).
    pub severity: P21Severity,
}

impl P21Rule {
    /// Check if this is a CT (controlled terminology) rule.
    pub fn is_ct_rule(&self) -> bool {
        self.id.starts_with("CT")
    }

    /// Check if this is a non-extensible CT rule (CT2001, CT2004).
    pub fn is_non_extensible_ct(&self) -> bool {
        self.id == "CT2001" || self.id == "CT2004"
    }

    /// Check if this is an extensible CT rule (CT2002, CT2005).
    pub fn is_extensible_ct(&self) -> bool {
        self.id == "CT2002" || self.id == "CT2005"
    }

    /// Get the effective severity for this rule.
    ///
    /// Some rules derive severity from context (e.g., CT rules depend on codelist extensibility).
    pub fn effective_severity(&self, is_extensible: bool) -> P21Severity {
        if self.is_ct_rule() {
            if is_extensible {
                P21Severity::Warning
            } else {
                P21Severity::Error
            }
        } else if self.severity != P21Severity::Notice {
            self.severity
        } else {
            // Default based on category
            match self.category {
                P21Category::Terminology => P21Severity::Error,
                P21Category::Presence => P21Severity::Error,
                P21Category::Format => P21Severity::Error,
                P21Category::Limit => P21Severity::Error,
                P21Category::Consistency => P21Severity::Warning,
                P21Category::Metadata => P21Severity::Warning,
                P21Category::CrossReference => P21Severity::Warning,
                P21Category::Structure => P21Severity::Error,
            }
        }
    }
}

/// Registry of Pinnacle 21 validation rules.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct P21RuleRegistry {
    /// Rules indexed by P21 ID.
    rules: HashMap<String, P21Rule>,
}

impl P21RuleRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    /// Add a rule to the registry.
    pub fn insert(&mut self, rule: P21Rule) {
        self.rules.insert(rule.id.clone(), rule);
    }

    /// Get a rule by ID.
    pub fn get(&self, id: &str) -> Option<&P21Rule> {
        self.rules.get(id)
    }

    /// Get all rules.
    pub fn rules(&self) -> impl Iterator<Item = &P21Rule> {
        self.rules.values()
    }

    /// Get the number of rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Get rules by category.
    pub fn rules_by_category(&self, category: P21Category) -> Vec<&P21Rule> {
        self.rules
            .values()
            .filter(|r| r.category == category)
            .collect()
    }
}

// Well-known P21 rule IDs for compile-time reference
pub mod rule_ids {
    //! Well-known Pinnacle 21 rule IDs.

    // Terminology rules
    /// Non-extensible codelist violation.
    pub const CT2001: &str = "CT2001";
    /// Extensible codelist violation.
    pub const CT2002: &str = "CT2002";
    /// Coded/decoded value mismatch.
    pub const CT2003: &str = "CT2003";

    // Presence rules
    /// No records in dataset.
    pub const SD0001: &str = "SD0001";
    /// Required variable null.
    pub const SD0002: &str = "SD0002";
    /// No baseline flag.
    pub const SD0006: &str = "SD0006";

    // Format rules
    /// Invalid ISO 8601 date.
    pub const SD0003: &str = "SD0003";
    /// Invalid --TEST length (>40 chars).
    pub const SD0017: &str = "SD0017";
    /// Invalid --TESTCD format.
    pub const SD0018: &str = "SD0018";

    // Consistency rules
    /// Inconsistent DOMAIN value.
    pub const SD0004: &str = "SD0004";
    /// Duplicate --SEQ value.
    pub const SD0005: &str = "SD0005";

    // Metadata rules
    /// Data type mismatch.
    pub const SD0055: &str = "SD0055";
    /// Required variable not found.
    pub const SD0056: &str = "SD0056";
    /// Expected variable not found.
    pub const SD0057: &str = "SD0057";
    /// Variable not in SDTM model.
    pub const SD0058: &str = "SD0058";

    // Limit rules
    /// Start date after end date.
    pub const SD0013: &str = "SD0013";
    /// Negative dose value.
    pub const SD0014: &str = "SD0014";
    /// Study day equals 0.
    pub const SD0038: &str = "SD0038";
}
