//! Dynamic Rule Generator
//!
//! Generates validation rules dynamically from:
//! - Variables.csv (Core designations, CT codelist codes)
//! - CT files (Valid values, extensibility)
//! - P21 Rules.csv (Rule templates with IDs and severity)
//!
//! This eliminates manual rule coding and ensures rules stay in sync with standards.

use std::collections::{BTreeMap, HashMap};

use sdtm_model::{CtRegistry, Domain, Variable};

use crate::P21Rule;

/// A generated validation rule based on metadata.
#[derive(Debug, Clone)]
pub struct GeneratedRule {
    /// P21 rule ID (e.g., "SD0002", "CT2001")
    pub rule_id: String,
    /// Domain code this rule applies to
    pub domain: String,
    /// Variable name this rule applies to
    pub variable: String,
    /// Rule category from P21
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
    /// Parse severity from a string (e.g., from P21 rules).
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
    /// Required variable must be present (SD0056) - column existence check
    RequiredPresence,
    /// Required variable must not be null (SD0002) - null value check
    RequiredVariable,
    /// Expected variable should be present (SD0057)
    ExpectedVariable,
    /// CT validation with valid values (CT2001/CT2002)
    ControlledTerminology {
        codelist_code: String,
        codelist_name: String,
        extensible: bool,
        valid_values: Vec<String>,
    },
    /// ISO 8601 date format (SD0003)
    DateTimeFormat,
    /// Sequence uniqueness (SD0005)
    SequenceUniqueness,
    /// Custom/other rule
    Other(String),
}

/// Rule generator that creates validation rules from metadata.
#[derive(Debug, Default)]
pub struct RuleGenerator {
    /// P21 rules indexed by ID
    p21_rules: HashMap<String, P21Rule>,
}

impl RuleGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load P21 rules as templates.
    pub fn with_p21_rules(mut self, rules: Vec<P21Rule>) -> Self {
        for rule in rules {
            self.p21_rules.insert(rule.rule_id.clone(), rule);
        }
        self
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
                // SD0056: Required variable must be present (column existence)
                let p21_presence = self.p21_rules.get("SD0056");
                rules.push(GeneratedRule {
                    rule_id: "SD0056".to_string(),
                    domain: domain.code.clone(),
                    variable: variable.name.clone(),
                    category: p21_presence
                        .and_then(|r| r.category.clone())
                        .unwrap_or_else(|| "Presence".to_string()),
                    severity: RuleSeverity::Error,
                    message: p21_presence
                        .map(|r| {
                            if !r.message.is_empty() {
                                format!("{}: {}", r.message, variable.name)
                            } else {
                                format!(
                                    "Required variable {} not found in {}",
                                    variable.name, domain.code
                                )
                            }
                        })
                        .unwrap_or_else(|| {
                            format!(
                                "Required variable {} not found in {}",
                                variable.name, domain.code
                            )
                        }),
                    description: p21_presence
                        .map(|r| r.description.clone())
                        .unwrap_or_else(|| {
                            "SDTM Required variable not found in dataset.".to_string()
                        }),
                    context: RuleContext::RequiredPresence,
                });

                // SD0002: Required variable cannot be null (value check)
                let p21_null = self.p21_rules.get("SD0002");
                rules.push(GeneratedRule {
                    rule_id: "SD0002".to_string(),
                    domain: domain.code.clone(),
                    variable: variable.name.clone(),
                    category: p21_null
                        .and_then(|r| r.category.clone())
                        .unwrap_or_else(|| "Presence".to_string()),
                    severity: RuleSeverity::Error,
                    message: p21_null
                        .map(|r| {
                            if !r.message.is_empty() {
                                format!("{}: {}", r.message, variable.name)
                            } else {
                                format!(
                                    "Required variable {} in {} cannot be null",
                                    variable.name, domain.code
                                )
                            }
                        })
                        .unwrap_or_else(|| {
                            format!(
                                "Required variable {} in {} cannot be null",
                                variable.name, domain.code
                            )
                        }),
                    description: p21_null.map(|r| r.description.clone()).unwrap_or_else(|| {
                        "Required variables (Core='Req') cannot be null for any records."
                            .to_string()
                    }),
                    context: RuleContext::RequiredVariable,
                });
            }
            "EXP" => {
                // SD0057: Expected variable should be present
                let p21 = self.p21_rules.get("SD0057");
                rules.push(GeneratedRule {
                    rule_id: "SD0057".to_string(),
                    domain: domain.code.clone(),
                    variable: variable.name.clone(),
                    category: p21
                        .and_then(|r| r.category.clone())
                        .unwrap_or_else(|| "Metadata".to_string()),
                    severity: RuleSeverity::Warning,
                    message: p21
                        .map(|r| {
                            if !r.message.is_empty() {
                                format!("{}: {}", r.message, variable.name)
                            } else {
                                format!(
                                    "Expected variable {} should be present in {}",
                                    variable.name, domain.code
                                )
                            }
                        })
                        .unwrap_or_else(|| {
                            format!(
                                "Expected variable {} should be present in {}",
                                variable.name, domain.code
                            )
                        }),
                    description: p21.map(|r| r.description.clone()).unwrap_or_else(|| {
                        "Variables described in SDTM IG as Expected should be included.".to_string()
                    }),
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
            if let Some(resolved) = ct_registry.resolve_by_code(&code, None) {
                let ct = resolved.ct;

                // Determine rule ID based on extensibility
                let (rule_id, severity) = if ct.extensible {
                    ("CT2002", RuleSeverity::Warning) // Extensible = warning
                } else {
                    ("CT2001", RuleSeverity::Error) // Non-extensible = error
                };

                let p21 = self.p21_rules.get(rule_id);

                rules.push(GeneratedRule {
                    rule_id: rule_id.to_string(),
                    domain: domain.code.clone(),
                    variable: variable.name.clone(),
                    category: p21
                        .and_then(|r| r.category.clone())
                        .unwrap_or_else(|| "Terminology".to_string()),
                    severity,
                    message: format!(
                        "Value in {}.{} must be from {} codelist ({})",
                        domain.code, variable.name, ct.codelist_name, code
                    ),
                    description: p21.map(|r| r.description.clone()).unwrap_or_else(|| {
                        if ct.extensible {
                            "Variable should be populated with terms from its CDISC CT codelist."
                                .to_string()
                        } else {
                            "Variable must be populated with terms from its CDISC CT codelist."
                                .to_string()
                        }
                    }),
                    context: RuleContext::ControlledTerminology {
                        codelist_code: code.clone(),
                        codelist_name: ct.codelist_name.clone(),
                        extensible: ct.extensible,
                        valid_values: ct.submission_values.clone(),
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

        let p21 = self.p21_rules.get("SD0003");

        rules.push(GeneratedRule {
            rule_id: "SD0003".to_string(),
            domain: domain.code.clone(),
            variable: variable.name.clone(),
            category: p21
                .and_then(|r| r.category.clone())
                .unwrap_or_else(|| "Format".to_string()),
            severity: RuleSeverity::Error,
            message: format!(
                "{}.{} must be in ISO 8601 format",
                domain.code, variable.name
            ),
            description: p21.map(|r| r.description.clone()).unwrap_or_else(|| {
                "Date/Time variables (*DTC) must conform to ISO 8601 international standard."
                    .to_string()
            }),
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

        let p21 = self.p21_rules.get("SD0005");

        rules.push(GeneratedRule {
            rule_id: "SD0005".to_string(),
            domain: domain.code.clone(),
            variable: variable.name.clone(),
            category: p21
                .and_then(|r| r.category.clone())
                .unwrap_or_else(|| "Consistency".to_string()),
            severity: RuleSeverity::Error,
            message: format!(
                "{} must be unique within USUBJID in {}",
                variable.name, domain.code
            ),
            description: p21.map(|r| r.description.clone()).unwrap_or_else(|| {
                "Sequence Number (--SEQ) must be unique for each record within a domain and USUBJID."
                    .to_string()
            }),
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
                *summary.by_rule_id.entry(rule.rule_id.clone()).or_insert(0) += 1;
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
    pub by_rule_id: BTreeMap<String, usize>,
    pub by_domain: BTreeMap<String, usize>,
    pub by_category: BTreeMap<String, usize>,
}

/// Split codelist codes that may be separated by semicolons or commas.
fn split_codelist_codes(raw: &str) -> Vec<String> {
    raw.split([';', ','])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
