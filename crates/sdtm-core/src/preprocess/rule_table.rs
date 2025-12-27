//! Rule table infrastructure for domain preprocessing.
//!
//! This module provides a rule-driven approach to domain preprocessing,
//! replacing hard-coded logic with configurable rules that can be
//! enabled/disabled and extended.
//!
//! # Architecture
//!
//! The rule table system consists of:
//! - `PreprocessRule` - Trait for individual preprocessing rules
//! - `RuleMetadata` - Metadata about a rule (ID, category, description)
//! - `RuleExecutor` - Executes a set of rules in order
//! - `DomainPreprocessor` - Contains rules for a specific domain
//! - `PreprocessRegistry` - Registry of all domain preprocessors

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use polars::prelude::DataFrame;

use super::common::PreprocessContext;
use super::{da, ds, ex, ie, pe, qs};

/// Categories of preprocessing rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleCategory {
    /// Test field inference (--TEST, --TESTCD, --CAT).
    TestField,
    /// Original result inference (--ORRES).
    OriginalResult,
    /// Unit inference (--ORRESU, --STRESU).
    Unit,
    /// Decode/term inference (--DECOD, --TERM).
    DecodeTerm,
    /// Treatment inference (EXTRT).
    Treatment,
    /// Category inference (--CAT, --SCAT).
    Category,
    /// Other domain-specific inference.
    Other,
}

/// Metadata about a preprocessing rule.
#[derive(Debug, Clone)]
pub struct RuleMetadata {
    /// Unique rule identifier.
    pub id: String,
    /// Rule category.
    pub category: RuleCategory,
    /// Human-readable description.
    pub description: String,
    /// Target variables this rule populates.
    pub target_variables: Vec<String>,
    /// Whether this rule is enabled by default.
    pub enabled_by_default: bool,
}

impl RuleMetadata {
    /// Create new rule metadata.
    pub fn new(
        id: impl Into<String>,
        category: RuleCategory,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            description: description.into(),
            target_variables: Vec::new(),
            enabled_by_default: true,
        }
    }

    /// Set target variables.
    pub fn with_targets(mut self, targets: &[&str]) -> Self {
        self.target_variables = targets.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set whether enabled by default.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled_by_default = enabled;
        self
    }
}

/// Trait for individual preprocessing rules.
///
/// Each rule implements a specific transformation that can be applied
/// to a DataFrame during preprocessing.
pub trait PreprocessRule: Send + Sync {
    /// Get the rule metadata.
    fn metadata(&self) -> &RuleMetadata;

    /// Check if this rule should be applied.
    ///
    /// Override to add conditions based on context or DataFrame state.
    fn should_apply(&self, _ctx: &PreprocessContext, _df: &DataFrame) -> bool {
        true
    }

    /// Apply the rule to the DataFrame.
    fn apply(&self, ctx: &PreprocessContext, df: &mut DataFrame) -> Result<()>;
}

/// Executes a set of rules in order.
pub struct RuleExecutor {
    rules: Vec<Arc<dyn PreprocessRule>>,
    disabled_rules: HashMap<String, bool>,
}

impl Default for RuleExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleExecutor {
    /// Create a new empty rule executor.
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            disabled_rules: HashMap::new(),
        }
    }

    /// Add a rule to the executor.
    pub fn add_rule(&mut self, rule: Arc<dyn PreprocessRule>) {
        self.rules.push(rule);
    }

    /// Disable a rule by ID.
    pub fn disable_rule(&mut self, rule_id: &str) {
        self.disabled_rules.insert(rule_id.to_string(), true);
    }

    /// Enable a previously disabled rule.
    pub fn enable_rule(&mut self, rule_id: &str) {
        self.disabled_rules.remove(rule_id);
    }

    /// Check if a rule is disabled.
    pub fn is_rule_disabled(&self, rule_id: &str) -> bool {
        self.disabled_rules.get(rule_id).copied().unwrap_or(false)
    }

    /// Execute all enabled rules.
    pub fn execute(&self, ctx: &PreprocessContext, df: &mut DataFrame) -> Result<()> {
        for rule in &self.rules {
            let meta = rule.metadata();

            // Skip disabled rules
            if self.is_rule_disabled(&meta.id) {
                continue;
            }

            // Skip rules that shouldn't apply
            if !rule.should_apply(ctx, df) {
                continue;
            }

            // Apply the rule
            rule.apply(ctx, df)?;
        }
        Ok(())
    }

    /// Get all rule metadata.
    pub fn rule_metadata(&self) -> Vec<&RuleMetadata> {
        self.rules.iter().map(|r| r.metadata()).collect()
    }
}

/// Domain-specific preprocessor containing rules for a single domain.
pub struct DomainPreprocessor {
    domain_code: String,
    executor: RuleExecutor,
}

impl DomainPreprocessor {
    /// Create a new domain preprocessor.
    pub fn new(domain_code: impl Into<String>) -> Self {
        Self {
            domain_code: domain_code.into().to_uppercase(),
            executor: RuleExecutor::new(),
        }
    }

    /// Get the domain code.
    pub fn domain_code(&self) -> &str {
        &self.domain_code
    }

    /// Add a rule to this domain's preprocessor.
    pub fn add_rule(&mut self, rule: Arc<dyn PreprocessRule>) {
        self.executor.add_rule(rule);
    }

    /// Get mutable access to the rule executor.
    pub fn executor_mut(&mut self) -> &mut RuleExecutor {
        &mut self.executor
    }

    /// Process the domain DataFrame.
    pub fn process(&self, ctx: &PreprocessContext, df: &mut DataFrame) -> Result<()> {
        self.executor.execute(ctx, df)
    }
}

/// Registry of domain preprocessors.
#[derive(Default)]
pub struct PreprocessRegistry {
    preprocessors: HashMap<String, DomainPreprocessor>,
}

impl PreprocessRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a domain preprocessor.
    pub fn register(&mut self, preprocessor: DomainPreprocessor) {
        let code = preprocessor.domain_code().to_uppercase();
        self.preprocessors.insert(code, preprocessor);
    }

    /// Get a preprocessor for a domain code.
    pub fn get(&self, domain_code: &str) -> Option<&DomainPreprocessor> {
        self.preprocessors.get(&domain_code.to_uppercase())
    }

    /// Get mutable access to a preprocessor.
    pub fn get_mut(&mut self, domain_code: &str) -> Option<&mut DomainPreprocessor> {
        self.preprocessors.get_mut(&domain_code.to_uppercase())
    }

    /// Process a domain using its registered preprocessor.
    ///
    /// Returns Ok if no preprocessor is registered for the domain.
    pub fn process(
        &self,
        domain_code: &str,
        ctx: &PreprocessContext,
        df: &mut DataFrame,
    ) -> Result<()> {
        if let Some(preprocessor) = self.get(domain_code) {
            preprocessor.process(ctx, df)
        } else {
            Ok(())
        }
    }

    /// List all registered domain codes.
    pub fn registered_domains(&self) -> Vec<String> {
        self.preprocessors.keys().cloned().collect()
    }
}

/// Build the default preprocess registry with all standard domain rules.
pub fn build_default_preprocess_registry() -> PreprocessRegistry {
    let mut registry = PreprocessRegistry::new();

    // Register QS preprocessor
    let mut qs_preprocessor = DomainPreprocessor::new("QS");
    qs_preprocessor.add_rule(Arc::new(qs::QsTestFieldRule::new()));
    registry.register(qs_preprocessor);

    // Register PE preprocessor
    let mut pe_preprocessor = DomainPreprocessor::new("PE");
    pe_preprocessor.add_rule(Arc::new(pe::PeTestFieldRule::new()));
    registry.register(pe_preprocessor);

    // Register DS preprocessor
    let mut ds_preprocessor = DomainPreprocessor::new("DS");
    ds_preprocessor.add_rule(Arc::new(ds::DsDecodeTermRule::new()));
    registry.register(ds_preprocessor);

    // Register EX preprocessor
    let mut ex_preprocessor = DomainPreprocessor::new("EX");
    ex_preprocessor.add_rule(Arc::new(ex::ExTreatmentRule::new()));
    registry.register(ex_preprocessor);

    // Register DA preprocessor
    let mut da_preprocessor = DomainPreprocessor::new("DA");
    da_preprocessor.add_rule(Arc::new(da::DaTestFieldRule::new()));
    registry.register(da_preprocessor);

    // Register IE preprocessor
    let mut ie_preprocessor = DomainPreprocessor::new("IE");
    ie_preprocessor.add_rule(Arc::new(ie::IeTestFieldRule::new()));
    registry.register(ie_preprocessor);

    registry
}
