//! Dynamic Rule Generator
//!
//! Generates validation rules dynamically from:
//! - Variables.csv (Core designations, CT codelist codes)
//! - CT files (Valid values, extensibility)
//!
//! This module generates rules based on SDTMIG metadata only.

use sdtm_model::{CtRegistry, Domain, Variable};

/// A generated validation rule based on metadata.
#[derive(Debug, Clone)]
pub struct GeneratedRule {
    /// Domain code this rule applies to
    pub domain: String,
    /// Variable name this rule applies to
    pub variable: String,
    /// Rule category
    pub category: String,
    /// Severity (Error/Warning/Info)
    pub severity: RuleSeverity,
    /// Human-readable message
    pub message: String,
    /// Detailed description
    pub description: String,
    /// Additional context
    pub context: RuleContext,
}

/// Rule severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleSeverity {
    Error,
    Warning,
    Info,
}

impl RuleSeverity {
    /// Parse severity from a string.
    pub fn parse(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "REJECT" | "ERROR" => RuleSeverity::Error,
            "WARNING" | "WARN" => RuleSeverity::Warning,
            _ => RuleSeverity::Info,
        }
    }
}

/// Additional context for rule evaluation.
#[derive(Debug, Clone)]
pub enum RuleContext {
    /// Required variable must be present - column existence check
    RequiredPresence,
    /// Required variable must not be null - null value check
    RequiredVariable,
    /// Expected variable should be present
    ExpectedVariable,
    /// CT validation with valid values
    ControlledTerminology {
        codelist_code: String,
        codelist_name: String,
        extensible: bool,
        valid_values: Vec<String>,
        ct_source: String,
    },
    /// ISO 8601 date format
    DateTimeFormat,
    /// Sequence uniqueness
    SequenceUniqueness,
    /// Custom/other rule
    Other(String),
}

/// Rule generator that creates validation rules from metadata.
#[derive(Debug, Default)]
pub struct RuleGenerator {}

impl RuleGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate all rules for a domain based on its variable metadata and CT registry.
    pub fn generate_rules_for_domain(
        &self,
        domain: &Domain,
        ct_registry: &CtRegistry,
    ) -> Vec<GeneratedRule> {
        let mut rules = Vec::new();

        for variable in &domain.variables {
            // Generate Core designation rules
            rules.extend(self.generate_core_rules(domain, variable));

            // Generate CT validation rules
            rules.extend(self.generate_ct_rules(domain, variable, ct_registry));

            // Generate datetime format rules for *DTC variables
            rules.extend(self.generate_datetime_rules(domain, variable));

            // Generate sequence rules for *SEQ variables
            rules.extend(self.generate_sequence_rules(domain, variable));
        }

        rules
    }

    /// Generate rules from Core designation (Req/Exp/Perm).
    fn generate_core_rules(&self, domain: &Domain, variable: &Variable) -> Vec<GeneratedRule> {
        let mut rules = Vec::new();

        let core = variable.core.as_deref().unwrap_or("").to_uppercase();

        match core.as_str() {
            "REQ" => {
                // Required variable must be present (column existence)
                rules.push(GeneratedRule {
                    domain: domain.code.clone(),
                    variable: variable.name.clone(),
                    category: "Required Variable Missing".to_string(),
                    severity: RuleSeverity::Error,
                    message: format!(
                        "Required variable {} not found in {}",
                        variable.name, domain.code
                    ),
                    description: "SDTM Required variable not found in dataset.".to_string(),
                    context: RuleContext::RequiredPresence,
                });

                // Required variable cannot be null (value check)
                rules.push(GeneratedRule {
                    domain: domain.code.clone(),
                    variable: variable.name.clone(),
                    category: "Required Value Missing".to_string(),
                    severity: RuleSeverity::Error,
                    message: format!(
                        "Required variable {} in {} cannot be null",
                        variable.name, domain.code
                    ),
                    description: "Required variables (Core='Req') cannot be null for any records."
                        .to_string(),
                    context: RuleContext::RequiredVariable,
                });
            }
            "EXP" => {
                // Expected variable should be present
                rules.push(GeneratedRule {
                    domain: domain.code.clone(),
                    variable: variable.name.clone(),
                    category: "Expected Variable Missing".to_string(),
                    severity: RuleSeverity::Warning,
                    message: format!(
                        "Expected variable {} should be present in {}",
                        variable.name, domain.code
                    ),
                    description: "Variables described in SDTMIG as Expected should be included."
                        .to_string(),
                    context: RuleContext::ExpectedVariable,
                });
            }
            _ => {
                // Permissible - no rule needed
            }
        }

        rules
    }

    /// Generate CT validation rules based on codelist_code.
    fn generate_ct_rules(
        &self,
        domain: &Domain,
        variable: &Variable,
        ct_registry: &CtRegistry,
    ) -> Vec<GeneratedRule> {
        let mut rules = Vec::new();

        // Check if variable has a CT codelist code
        let codelist_code = match &variable.codelist_code {
            Some(code) if !code.is_empty() => code,
            _ => return rules,
        };

        // Split multiple codes (some variables have "C66742; C66789")
        for code in split_codelist_codes(codelist_code) {
            // Look up the CT in the registry
            if let Some(resolved) = ct_registry.resolve(&code, None) {
                let codelist = resolved.codelist;

                // Determine severity based on extensibility
                let severity = if codelist.extensible {
                    RuleSeverity::Warning // Extensible = warning
                } else {
                    RuleSeverity::Error // Non-extensible = error
                };

                let description = if codelist.extensible {
                    "Variable should be populated with terms from its CDISC CT codelist."
                        .to_string()
                } else {
                    "Variable must be populated with terms from its CDISC CT codelist.".to_string()
                };

                // Collect valid submission values
                let valid_values: Vec<String> = codelist
                    .submission_values()
                    .iter()
                    .map(|s| s.to_string())
                    .collect();

                rules.push(GeneratedRule {
                    domain: domain.code.clone(),
                    variable: variable.name.clone(),
                    category: code.clone(), // Use CT codelist code as category
                    severity,
                    message: format!(
                        "Value in {}.{} must be from {} codelist ({})",
                        domain.code, variable.name, codelist.name, code
                    ),
                    description,
                    context: RuleContext::ControlledTerminology {
                        codelist_code: code.clone(),
                        codelist_name: codelist.name.clone(),
                        extensible: codelist.extensible,
                        valid_values,
                        ct_source: resolved.source().to_string(),
                    },
                });
            }
        }

        rules
    }

    /// Generate datetime format rules for *DTC variables.
    fn generate_datetime_rules(&self, domain: &Domain, variable: &Variable) -> Vec<GeneratedRule> {
        let mut rules = Vec::new();

        // Check if variable ends with DTC (datetime)
        if !variable.name.to_uppercase().ends_with("DTC") {
            return rules;
        }

        rules.push(GeneratedRule {
            domain: domain.code.clone(),
            variable: variable.name.clone(),
            category: "Invalid ISO 8601 Format".to_string(),
            severity: RuleSeverity::Error,
            message: format!(
                "{}.{} must be in ISO 8601 format",
                domain.code, variable.name
            ),
            description:
                "Date/Time variables (*DTC) must conform to ISO 8601 international standard."
                    .to_string(),
            context: RuleContext::DateTimeFormat,
        });

        rules
    }

    /// Generate sequence uniqueness rules for *SEQ variables.
    fn generate_sequence_rules(&self, domain: &Domain, variable: &Variable) -> Vec<GeneratedRule> {
        let mut rules = Vec::new();

        // Check if variable ends with SEQ
        if !variable.name.to_uppercase().ends_with("SEQ") {
            return rules;
        }

        rules.push(GeneratedRule {
            domain: domain.code.clone(),
            variable: variable.name.clone(),
            category: "Duplicate Sequence Number".to_string(),
            severity: RuleSeverity::Error,
            message: format!(
                "{} must be unique within USUBJID in {}",
                variable.name, domain.code
            ),
            description:
                "Sequence Number (--SEQ) must be unique for each record within a domain and USUBJID."
                    .to_string(),
            context: RuleContext::SequenceUniqueness,
        });

        rules
    }

    /// Get a summary of generated rules.
    pub fn generate_summary(
        &self,
        domains: &[Domain],
        ct_registry: &CtRegistry,
    ) -> RuleGenerationSummary {
        let mut summary = RuleGenerationSummary::default();

        for domain in domains {
            let rules = self.generate_rules_for_domain(domain, ct_registry);

            for rule in &rules {
                summary.total_rules += 1;
                *summary.by_domain.entry(rule.domain.clone()).or_insert(0) += 1;
                *summary
                    .by_category
                    .entry(rule.category.clone())
                    .or_insert(0) += 1;
            }
        }

        summary
    }
}

/// Summary of generated rules.
#[derive(Debug, Default)]
pub struct RuleGenerationSummary {
    pub total_rules: usize,
    pub by_domain: std::collections::BTreeMap<String, usize>,
    pub by_category: std::collections::BTreeMap<String, usize>,
}

/// Split codelist codes that may be separated by semicolons or commas.
fn split_codelist_codes(raw: &str) -> Vec<String> {
    raw.split([';', ','])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
