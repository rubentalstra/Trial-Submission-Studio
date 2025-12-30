//! Data-driven transformation pipeline for SDTM domains.
//!
//! This module provides a **metadata-driven** transformation system where
//! transformation rules are automatically derived from variable metadata
//! (role, codelist_code, described_value_domain) rather than hardcoded per domain.
//!
//! # Key Principle
//!
//! **No hardcoded domain-specific rules.** All transformation types are inferred from:
//! - Variable name patterns (`STUDYID`, `DOMAIN`, `*SEQ`, `*DY`, `*DTC`)
//! - `described_value_domain` field (ISO 8601 formats)
//! - `codelist_code` field (CT normalization)
//! - `data_type` + `role` combination
//!
//! # Example
//!
//! ```ignore
//! use sdtm_transform::pipeline::{DomainPipeline, TransformContext};
//!
//! // Build pipeline automatically from domain metadata
//! let pipeline = DomainPipeline::from_domain(&domain);
//!
//! // Execute all transformations
//! let result = pipeline.execute(&mut df, &ctx)?;
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transformation type derived from variable metadata.
///
/// Each variant corresponds to a specific transformation operation.
/// The type is **inferred** from variable metadata, not hardcoded per domain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformType {
    /// Direct copy from source column (no transformation).
    CopyDirect,

    /// Set a constant value.
    /// For STUDYID/DOMAIN: value is set at execution time from context.
    Constant,

    /// Construct USUBJID with STUDYID prefix.
    /// Per SDTMIG 4.1.2: `STUDYID-SUBJID` format.
    UsubjidPrefix,

    /// Generate sequence numbers per subject.
    /// Per SDTMIG 4.1.5: unique within domain per subject.
    SequenceNumber,

    /// Normalize to controlled terminology.
    /// Codelist code is stored in the rule.
    CtNormalization {
        /// NCI codelist code (e.g., "C66731" for SEX).
        codelist_code: String,
    },

    /// Parse and format as ISO 8601 datetime.
    /// Inferred from `described_value_domain` containing "ISO 8601".
    Iso8601DateTime,

    /// Parse and format as ISO 8601 date only.
    Iso8601Date,

    /// Parse and format as ISO 8601 duration.
    /// Inferred from `described_value_domain` containing "duration".
    Iso8601Duration,

    /// Calculate study day from a reference date.
    /// Per SDTMIG 4.4.4: `--DY = --DTC - RFSTDTC + 1` (if after) or `--DTC - RFSTDTC` (if before).
    StudyDay {
        /// The corresponding DTC variable (e.g., "AEDTC" for "AEDY").
        reference_dtc: String,
    },

    /// Convert to numeric value.
    /// For result qualifier variables with Num data type.
    NumericConversion,
}

impl TransformType {
    /// Returns a human-readable display name for the transform type.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::CopyDirect => "Copy",
            Self::Constant => "Constant",
            Self::UsubjidPrefix => "USUBJID Prefix",
            Self::SequenceNumber => "Sequence",
            Self::CtNormalization { .. } => "CT Normalize",
            Self::Iso8601DateTime => "ISO 8601 DateTime",
            Self::Iso8601Date => "ISO 8601 Date",
            Self::Iso8601Duration => "ISO 8601 Duration",
            Self::StudyDay { .. } => "Study Day",
            Self::NumericConversion => "Numeric",
        }
    }

    /// Returns the egui_phosphor icon name for this transform type.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::CopyDirect => "COPY",
            Self::Constant => "LOCK",
            Self::UsubjidPrefix => "IDENTIFICATION_CARD",
            Self::SequenceNumber => "HASH",
            Self::CtNormalization { .. } => "BOOK_OPEN",
            Self::Iso8601DateTime | Self::Iso8601Date => "CALENDAR",
            Self::Iso8601Duration => "TIMER",
            Self::StudyDay { .. } => "CALENDAR_CHECK",
            Self::NumericConversion => "FUNCTION",
        }
    }
}

/// Origin of a transformation rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TransformOrigin {
    /// Auto-derived from variable metadata.
    #[default]
    Derived,
    /// User has customized this rule.
    UserDefined,
}

/// A single transformation rule for a variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRule {
    /// Target SDTM variable name (e.g., "AESEQ", "AESTDTC").
    pub target_variable: String,

    /// Source column from mapping (if mapped).
    pub source_column: Option<String>,

    /// The transformation type to apply.
    pub transform_type: TransformType,

    /// Whether this rule was auto-derived or user-customized.
    pub origin: TransformOrigin,

    /// Execution order (based on variable order in domain).
    pub order: u32,
}

impl TransformRule {
    /// Create a new derived rule.
    pub fn derived(target: impl Into<String>, transform_type: TransformType, order: u32) -> Self {
        Self {
            target_variable: target.into(),
            source_column: None,
            transform_type,
            origin: TransformOrigin::Derived,
            order,
        }
    }

    /// Set the source column for this rule.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source_column = Some(source.into());
        self
    }

    /// Check if this rule has a source column mapped.
    pub fn is_mapped(&self) -> bool {
        self.source_column.is_some()
    }

    /// Check if this rule requires a source column to execute.
    pub fn requires_source(&self) -> bool {
        matches!(
            self.transform_type,
            TransformType::CopyDirect
                | TransformType::CtNormalization { .. }
                | TransformType::Iso8601DateTime
                | TransformType::Iso8601Date
                | TransformType::Iso8601Duration
                | TransformType::NumericConversion
        )
    }

    /// Check if this rule can be executed (has source if needed).
    pub fn can_execute(&self) -> bool {
        !self.requires_source() || self.is_mapped()
    }
}

/// Transformation pipeline for a domain.
///
/// Contains all transformation rules derived from the domain's variable metadata.
/// Rules are executed in order to transform source data into SDTM format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainPipeline {
    /// The SDTM domain code (e.g., "AE", "DM", "LB").
    pub domain_code: String,

    /// Ordered list of transformation rules.
    pub rules: Vec<TransformRule>,

    /// Custom CT mappings: codelist_code -> (source_value -> target_value).
    #[serde(default)]
    pub custom_ct_maps: HashMap<String, HashMap<String, String>>,
}

impl DomainPipeline {
    /// Create a new empty pipeline for a domain.
    pub fn new(domain_code: impl Into<String>) -> Self {
        Self {
            domain_code: domain_code.into(),
            rules: Vec::new(),
            custom_ct_maps: HashMap::new(),
        }
    }

    /// Add a rule to the pipeline.
    pub fn add_rule(&mut self, rule: TransformRule) {
        self.rules.push(rule);
    }

    /// Get a rule by target variable name.
    pub fn get_rule(&self, target: &str) -> Option<&TransformRule> {
        self.rules
            .iter()
            .find(|r| r.target_variable.eq_ignore_ascii_case(target))
    }

    /// Get a mutable rule by target variable name.
    pub fn get_rule_mut(&mut self, target: &str) -> Option<&mut TransformRule> {
        self.rules
            .iter_mut()
            .find(|r| r.target_variable.eq_ignore_ascii_case(target))
    }

    /// Set source column for a target variable.
    pub fn set_source(&mut self, target: &str, source: impl Into<String>) {
        if let Some(rule) = self.get_rule_mut(target) {
            rule.source_column = Some(source.into());
        }
    }

    /// Get rules grouped by transform type category.
    pub fn rules_by_category(&self) -> RulesByCategory<'_> {
        let mut categories = RulesByCategory::default();

        for rule in &self.rules {
            match &rule.transform_type {
                TransformType::Constant | TransformType::UsubjidPrefix => {
                    categories.identifiers.push(rule);
                }
                TransformType::SequenceNumber => {
                    categories.identifiers.push(rule);
                }
                TransformType::CtNormalization { .. } => {
                    categories.terminology.push(rule);
                }
                TransformType::Iso8601DateTime
                | TransformType::Iso8601Date
                | TransformType::Iso8601Duration
                | TransformType::StudyDay { .. } => {
                    categories.timing.push(rule);
                }
                TransformType::NumericConversion => {
                    categories.numeric.push(rule);
                }
                TransformType::CopyDirect => {
                    categories.copy.push(rule);
                }
            }
        }

        categories
    }

    /// Count rules that can be executed (have required sources).
    pub fn executable_count(&self) -> usize {
        self.rules.iter().filter(|r| r.can_execute()).count()
    }

    /// Count rules that are mapped.
    pub fn mapped_count(&self) -> usize {
        self.rules.iter().filter(|r| r.is_mapped()).count()
    }

    /// Get statistics about the pipeline.
    pub fn stats(&self) -> PipelineStats {
        PipelineStats {
            total_rules: self.rules.len(),
            mapped_rules: self.mapped_count(),
            executable_rules: self.executable_count(),
            ct_rules: self
                .rules
                .iter()
                .filter(|r| matches!(r.transform_type, TransformType::CtNormalization { .. }))
                .count(),
            timing_rules: self
                .rules
                .iter()
                .filter(|r| {
                    matches!(
                        r.transform_type,
                        TransformType::Iso8601DateTime
                            | TransformType::Iso8601Date
                            | TransformType::Iso8601Duration
                            | TransformType::StudyDay { .. }
                    )
                })
                .count(),
        }
    }
}

/// Rules grouped by category for display.
#[derive(Debug, Default)]
pub struct RulesByCategory<'a> {
    pub identifiers: Vec<&'a TransformRule>,
    pub terminology: Vec<&'a TransformRule>,
    pub timing: Vec<&'a TransformRule>,
    pub numeric: Vec<&'a TransformRule>,
    pub copy: Vec<&'a TransformRule>,
}

/// Statistics about a pipeline.
#[derive(Debug, Clone, Default)]
pub struct PipelineStats {
    pub total_rules: usize,
    pub mapped_rules: usize,
    pub executable_rules: usize,
    pub ct_rules: usize,
    pub timing_rules: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_type_display() {
        assert_eq!(TransformType::CopyDirect.display_name(), "Copy");
        assert_eq!(TransformType::UsubjidPrefix.display_name(), "USUBJID Prefix");
        assert_eq!(
            TransformType::CtNormalization {
                codelist_code: "C66731".into()
            }
            .display_name(),
            "CT Normalize"
        );
    }

    #[test]
    fn test_transform_rule_requires_source() {
        let copy = TransformRule::derived("TEST", TransformType::CopyDirect, 1);
        assert!(copy.requires_source());
        assert!(!copy.can_execute());

        let copy_mapped = copy.with_source("SOURCE");
        assert!(copy_mapped.can_execute());

        let constant = TransformRule::derived("STUDYID", TransformType::Constant, 1);
        assert!(!constant.requires_source());
        assert!(constant.can_execute());
    }

    #[test]
    fn test_pipeline_stats() {
        let mut pipeline = DomainPipeline::new("AE");
        pipeline.add_rule(TransformRule::derived("STUDYID", TransformType::Constant, 1));
        pipeline.add_rule(TransformRule::derived("DOMAIN", TransformType::Constant, 2));
        pipeline.add_rule(TransformRule::derived("USUBJID", TransformType::UsubjidPrefix, 3));
        pipeline.add_rule(TransformRule::derived(
            "AESEV",
            TransformType::CtNormalization {
                codelist_code: "C66769".into(),
            },
            4,
        ));
        pipeline.add_rule(TransformRule::derived(
            "AESTDTC",
            TransformType::Iso8601DateTime,
            5,
        ));

        let stats = pipeline.stats();
        assert_eq!(stats.total_rules, 5);
        assert_eq!(stats.ct_rules, 1);
        assert_eq!(stats.timing_rules, 1);
    }
}
