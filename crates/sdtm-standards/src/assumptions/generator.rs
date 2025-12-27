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
    /// Date pair ordering (end date must not precede start date) per SDTMIG 4.4
    DatePairOrder {
        /// Start date variable (e.g., --STDTC)
        start_variable: String,
        /// End date variable (e.g., --ENDTC)
        end_variable: String,
    },
    /// Study day requires complete dates per SDTMIG 4.4.4
    StudyDayCompleteness {
        /// Date/time variable (e.g., --DTC, --STDTC, --ENDTC)
        dtc_variable: String,
        /// Study day variable (e.g., --DY, --STDY, --ENDY)
        dy_variable: String,
    },
    /// Findings class date/time usage per SDTMIG 4.4.8
    /// --DTC is required, --STDTC is disallowed in Findings class domains
    FindingsTimingVariable {
        /// The variable in question (e.g., --DTC, --STDTC)
        variable_suffix: String,
        /// Whether this variable is allowed (true) or disallowed (false)
        is_allowed: bool,
    },
    /// Relative timing variables per SDTMIG 4.4.7
    /// --STRF/--ENRF with allowed values and required anchor variables
    RelativeTimingVariable {
        /// The relative timing variable (--STRF or --ENRF)
        variable: String,
        /// Allowed values for this variable (BEFORE, AFTER, DURING, etc.)
        allowed_values: Vec<String>,
        /// The required anchor variable (--STTPT or --ENTPT)
        anchor_variable: String,
    },
    /// Duration variable usage per SDTMIG 4.4.3
    /// --DUR should only be used when start/end dates are not collected
    DurationUsage {
        /// The duration variable (--DUR)
        dur_variable: String,
        /// Start date variable (--STDTC)
        stdtc_variable: String,
        /// End date variable (--ENDTC)
        endtc_variable: String,
    },
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

        // Generate domain-level timing rules per SDTMIG 4.4.8
        rules.extend(self.generate_findings_timing_rules(domain));

        // Generate date pair ordering rules per SDTMIG 4.4
        rules.extend(self.generate_date_pair_order_rules(domain));

        // Generate study day completeness rules per SDTMIG 4.4.4
        rules.extend(self.generate_study_day_rules(domain));

        // Generate relative timing rules per SDTMIG 4.4.7
        rules.extend(self.generate_relative_timing_rules(domain));

        // Generate duration usage rules per SDTMIG 4.4.3
        rules.extend(self.generate_duration_usage_rules(domain));

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

    /// Generate Findings class timing rules per SDTMIG v3.4 Section 4.4.8.
    ///
    /// Per SDTMIG 4.4.8 "Date and Time Reported in a Domain Based on Findings":
    /// - In Findings class domains, --DTC is used for collection date/time
    /// - For single-point collections: only --DTC
    /// - For interval collections: --DTC for start and --ENDTC for end
    /// - **--STDTC should NOT be used in Findings class domains**
    ///
    /// The table from SDTMIG 4.4.8:
    /// | Collection Type         | --DTC | --STDTC | --ENDTC |
    /// | ----------------------- | ----- | ------- | ------- |
    /// | Single-point Collection | X     |         |         |
    /// | Interval Collection     | X     |         | X       |
    fn generate_findings_timing_rules(&self, domain: &Domain) -> Vec<GeneratedRule> {
        let mut rules = Vec::new();

        // Only apply to Findings and Findings About class domains
        let class = domain.dataset_class.as_ref();
        let is_findings_class = matches!(
            class,
            Some(sdtm_model::DatasetClass::Findings | sdtm_model::DatasetClass::FindingsAbout)
        );

        if !is_findings_class {
            return rules;
        }

        // Build the domain-specific variable names
        let prefix = if domain.code.len() >= 2 {
            &domain.code[..2]
        } else {
            &domain.code
        };

        let stdtc_var = format!("{}STDTC", prefix);
        let dtc_var = format!("{}DTC", prefix);

        // Check if --STDTC is present in domain variables (it shouldn't be for Findings)
        let has_stdtc = domain
            .variables
            .iter()
            .any(|v| v.name.to_uppercase().ends_with("STDTC"));

        // Check if --DTC is present (it should be for Findings)
        let has_dtc = domain
            .variables
            .iter()
            .any(|v| v.name.to_uppercase().ends_with("DTC") && !v.name.to_uppercase().ends_with("STDTC") && !v.name.to_uppercase().ends_with("ENDTC"));

        // Rule: --STDTC should not be used in Findings class domains
        if has_stdtc {
            rules.push(GeneratedRule {
                rule_id: "SD4408".to_string(), // Custom rule ID for SDTMIG 4.4.8
                domain: domain.code.clone(),
                variable: stdtc_var.clone(),
                category: "Timing".to_string(),
                severity: RuleSeverity::Error, // Error - this is a conformance issue
                message: format!(
                    "{}STDTC should not be used in Findings class domain {} per SDTMIG 4.4.8",
                    prefix, domain.code
                ),
                description: "Per SDTMIG v3.4 Section 4.4.8, --STDTC should not be used in \
                              Findings class domains. Use --DTC for single-point collections \
                              and --DTC/--ENDTC for interval collections."
                    .to_string(),
                context: RuleContext::FindingsTimingVariable {
                    variable_suffix: "STDTC".to_string(),
                    is_allowed: false,
                },
            });
        }

        // Rule: --DTC should be present in Findings class domains for collection timing
        if !has_dtc {
            rules.push(GeneratedRule {
                rule_id: "SD4408B".to_string(), // Supplementary rule for SDTMIG 4.4.8
                domain: domain.code.clone(),
                variable: dtc_var.clone(),
                category: "Timing".to_string(),
                severity: RuleSeverity::Warning, // Warning - some Findings domains may have different timing
                message: format!(
                    "{}DTC should be present in Findings class domain {} per SDTMIG 4.4.8",
                    prefix, domain.code
                ),
                description: "Per SDTMIG v3.4 Section 4.4.8, --DTC should be used for \
                              collection date/time in Findings class domains. For specimen-based \
                              findings, --DTC represents the time of specimen collection."
                    .to_string(),
                context: RuleContext::FindingsTimingVariable {
                    variable_suffix: "DTC".to_string(),
                    is_allowed: true,
                },
            });
        }

        rules
    }

    /// Generate date pair ordering rules per SDTMIG v3.4 Section 4.4.
    ///
    /// Per SDTMIG 4.4: End date (--ENDTC) must not precede start date (--STDTC).
    /// This is a hard error that should block submission.
    fn generate_date_pair_order_rules(&self, domain: &Domain) -> Vec<GeneratedRule> {
        let mut rules = Vec::new();

        // Find start and end date variables
        let has_stdtc = domain
            .variables
            .iter()
            .any(|v| v.name.to_uppercase().ends_with("STDTC"));
        let has_endtc = domain
            .variables
            .iter()
            .any(|v| v.name.to_uppercase().ends_with("ENDTC"));

        if !has_stdtc || !has_endtc {
            return rules;
        }

        // Build the domain-specific variable names
        let prefix = if domain.code.len() >= 2 {
            &domain.code[..2]
        } else {
            &domain.code
        };

        let stdtc_var = format!("{}STDTC", prefix);
        let endtc_var = format!("{}ENDTC", prefix);

        // Generate rule: End date must not precede start date
        // Per SDTMIG 4.4: This is a conformance error, not a warning
        rules.push(GeneratedRule {
            rule_id: "SD0009".to_string(), // P21 rule for date ordering
            domain: domain.code.clone(),
            variable: endtc_var.clone(),
            category: "Timing".to_string(),
            severity: RuleSeverity::Error, // Hard error per SDTMIG
            message: format!(
                "{} must not precede {} in {} per SDTMIG 4.4",
                endtc_var, stdtc_var, domain.code
            ),
            description: "Per SDTMIG v3.4 Section 4.4, the end date (--ENDTC) must not \
                          precede the start date (--STDTC). This is a data quality error \
                          that indicates invalid temporal sequencing."
                .to_string(),
            context: RuleContext::DatePairOrder {
                start_variable: stdtc_var,
                end_variable: endtc_var,
            },
        });

        rules
    }

    /// Generate study day completeness rules per SDTMIG v3.4 Section 4.4.4.
    ///
    /// Per SDTMIG 4.4.4: Study day (--DY) can only be computed when:
    /// 1. Observation date is complete (year, month, day)
    /// 2. Reference date (RFSTDTC) is complete
    ///
    /// This rule warns when study day is expected but cannot be computed due to
    /// partial/incomplete dates.
    fn generate_study_day_rules(&self, domain: &Domain) -> Vec<GeneratedRule> {
        let mut rules = Vec::new();

        // Build the domain-specific variable prefix
        let prefix = if domain.code.len() >= 2 {
            &domain.code[..2]
        } else {
            &domain.code
        };

        // Check for DTC/DY pairs, STDTC/STDY pairs, ENDTC/ENDY pairs
        let pairs = [
            (format!("{}DTC", prefix), format!("{}DY", prefix)),
            (format!("{}STDTC", prefix), format!("{}STDY", prefix)),
            (format!("{}ENDTC", prefix), format!("{}ENDY", prefix)),
        ];

        for (dtc_var, dy_var) in pairs {
            let has_dtc = domain
                .variables
                .iter()
                .any(|v| v.name.to_uppercase() == dtc_var.to_uppercase());
            let has_dy = domain
                .variables
                .iter()
                .any(|v| v.name.to_uppercase() == dy_var.to_uppercase());

            if has_dtc && has_dy {
                rules.push(GeneratedRule {
                    rule_id: "SD0010".to_string(), // Custom rule ID for study day completeness
                    domain: domain.code.clone(),
                    variable: dy_var.clone(),
                    category: "Timing".to_string(),
                    severity: RuleSeverity::Warning, // Warning, not error - partial dates are valid
                    message: format!(
                        "{} requires complete date in {} per SDTMIG 4.4.4",
                        dy_var, dtc_var
                    ),
                    description: format!(
                        "Per SDTMIG v3.4 Section 4.4.4, study day ({}) can only be computed \
                         when both the observation date ({}) and reference date (RFSTDTC) \
                         are complete (year, month, day). Partial dates cannot be used for \
                         study day calculation.",
                        dy_var, dtc_var
                    ),
                    context: RuleContext::StudyDayCompleteness {
                        dtc_variable: dtc_var,
                        dy_variable: dy_var,
                    },
                });
            }
        }

        rules
    }

    /// Generate relative timing variable rules per SDTMIG v3.4 Section 4.4.7.
    ///
    /// Per SDTMIG 4.4.7 "Use of Relative Timing Variables --STRF and --ENRF":
    /// - --STRF/--ENRF represent timing relative to the sponsor-defined study reference period
    /// - Allowed values: BEFORE, ONGOING, AFTER (or similar per CT)
    /// - --STRF requires --STTPT anchor; --ENRF requires --ENTPT anchor
    /// - Should not derive --STRF/--ENRF when actual dates are collected
    fn generate_relative_timing_rules(&self, domain: &Domain) -> Vec<GeneratedRule> {
        let mut rules = Vec::new();

        // Build the domain-specific variable prefix
        let prefix = if domain.code.len() >= 2 {
            &domain.code[..2]
        } else {
            &domain.code
        };

        // Check for --STRF and --ENRF variables
        let has_strf = domain
            .variables
            .iter()
            .any(|v| v.name.to_uppercase().ends_with("STRF"));
        let has_enrf = domain
            .variables
            .iter()
            .any(|v| v.name.to_uppercase().ends_with("ENRF"));
        let has_sttpt = domain
            .variables
            .iter()
            .any(|v| v.name.to_uppercase().ends_with("STTPT"));
        let has_entpt = domain
            .variables
            .iter()
            .any(|v| v.name.to_uppercase().ends_with("ENTPT"));
        let has_stdtc = domain
            .variables
            .iter()
            .any(|v| v.name.to_uppercase().ends_with("STDTC"));
        let has_endtc = domain
            .variables
            .iter()
            .any(|v| v.name.to_uppercase().ends_with("ENDTC"));

        // Allowed values per SDTMIG 4.4.7 and CT (C66728 STENRF)
        let allowed_values = vec![
            "BEFORE".to_string(),
            "DURING".to_string(),
            "AFTER".to_string(),
            "ONGOING".to_string(),
            "CONTINUING".to_string(),
            "U".to_string(), // Unknown
        ];

        // Rule: --STRF should have --STTPT anchor
        if has_strf && !has_sttpt {
            let strf_var = format!("{}STRF", prefix);
            let sttpt_var = format!("{}STTPT", prefix);
            rules.push(GeneratedRule {
                rule_id: "SD4407A".to_string(),
                domain: domain.code.clone(),
                variable: strf_var.clone(),
                category: "Timing".to_string(),
                severity: RuleSeverity::Warning,
                message: format!(
                    "{} requires {} anchor variable per SDTMIG 4.4.7",
                    strf_var, sttpt_var
                ),
                description: "Per SDTMIG v3.4 Section 4.4.7, when --STRF is used to represent \
                              relative start timing, the --STTPT anchor variable should be \
                              present to provide the text description of the reference point."
                    .to_string(),
                context: RuleContext::RelativeTimingVariable {
                    variable: strf_var,
                    allowed_values: allowed_values.clone(),
                    anchor_variable: sttpt_var,
                },
            });
        }

        // Rule: --ENRF should have --ENTPT anchor
        if has_enrf && !has_entpt {
            let enrf_var = format!("{}ENRF", prefix);
            let entpt_var = format!("{}ENTPT", prefix);
            rules.push(GeneratedRule {
                rule_id: "SD4407B".to_string(),
                domain: domain.code.clone(),
                variable: enrf_var.clone(),
                category: "Timing".to_string(),
                severity: RuleSeverity::Warning,
                message: format!(
                    "{} requires {} anchor variable per SDTMIG 4.4.7",
                    enrf_var, entpt_var
                ),
                description: "Per SDTMIG v3.4 Section 4.4.7, when --ENRF is used to represent \
                              relative end timing, the --ENTPT anchor variable should be \
                              present to provide the text description of the reference point."
                    .to_string(),
                context: RuleContext::RelativeTimingVariable {
                    variable: enrf_var,
                    allowed_values: allowed_values.clone(),
                    anchor_variable: entpt_var,
                },
            });
        }

        // Rule: Avoid derived --STRF when --STDTC is collected
        if has_strf && has_stdtc {
            let strf_var = format!("{}STRF", prefix);
            let stdtc_var = format!("{}STDTC", prefix);
            rules.push(GeneratedRule {
                rule_id: "SD4407C".to_string(),
                domain: domain.code.clone(),
                variable: strf_var.clone(),
                category: "Timing".to_string(),
                severity: RuleSeverity::Warning,
                message: format!(
                    "{} should not be derived when {} is collected per SDTMIG 4.4.7",
                    strf_var, stdtc_var
                ),
                description: "Per SDTMIG v3.4 Section 4.4.7, --STRF represents timing relative \
                              to the study reference period and should not be derived when \
                              actual dates (--STDTC) are collected. Use --STRF only when \
                              relative timing was collected in lieu of an actual date."
                    .to_string(),
                context: RuleContext::RelativeTimingVariable {
                    variable: strf_var,
                    allowed_values: allowed_values.clone(),
                    anchor_variable: stdtc_var,
                },
            });
        }

        // Rule: Avoid derived --ENRF when --ENDTC is collected
        if has_enrf && has_endtc {
            let enrf_var = format!("{}ENRF", prefix);
            let endtc_var = format!("{}ENDTC", prefix);
            rules.push(GeneratedRule {
                rule_id: "SD4407D".to_string(),
                domain: domain.code.clone(),
                variable: enrf_var.clone(),
                category: "Timing".to_string(),
                severity: RuleSeverity::Warning,
                message: format!(
                    "{} should not be derived when {} is collected per SDTMIG 4.4.7",
                    enrf_var, endtc_var
                ),
                description: "Per SDTMIG v3.4 Section 4.4.7, --ENRF represents timing relative \
                              to the study reference period and should not be derived when \
                              actual dates (--ENDTC) are collected. Use --ENRF only when \
                              relative timing was collected in lieu of an actual date."
                    .to_string(),
                context: RuleContext::RelativeTimingVariable {
                    variable: enrf_var,
                    allowed_values,
                    anchor_variable: endtc_var,
                },
            });
        }

        rules
    }

    /// Generate duration usage rules per SDTMIG v3.4 Section 4.4.3.
    ///
    /// Per SDTMIG 4.4.3 "Intervals of Time and Use of Duration for --DUR Variables":
    /// - --DUR should generally be used if collected in lieu of --STDTC and --ENDTC
    /// - If both --STDTC and --ENDTC are collected, duration can be calculated and
    ///   need not be in the submission dataset
    /// - --DUR should not be populated when both start and end dates are collected
    fn generate_duration_usage_rules(&self, domain: &Domain) -> Vec<GeneratedRule> {
        let mut rules = Vec::new();

        // Build the domain-specific variable prefix
        let prefix = if domain.code.len() >= 2 {
            &domain.code[..2]
        } else {
            &domain.code
        };

        // Check for timing variables
        let has_dur = domain
            .variables
            .iter()
            .any(|v| v.name.to_uppercase().ends_with("DUR"));
        let has_stdtc = domain
            .variables
            .iter()
            .any(|v| v.name.to_uppercase().ends_with("STDTC"));
        let has_endtc = domain
            .variables
            .iter()
            .any(|v| v.name.to_uppercase().ends_with("ENDTC"));

        // Rule: --DUR should not be used when both --STDTC and --ENDTC are collected
        if has_dur && has_stdtc && has_endtc {
            let dur_var = format!("{}DUR", prefix);
            let stdtc_var = format!("{}STDTC", prefix);
            let endtc_var = format!("{}ENDTC", prefix);

            rules.push(GeneratedRule {
                rule_id: "SD4403".to_string(),
                domain: domain.code.clone(),
                variable: dur_var.clone(),
                category: "Timing".to_string(),
                severity: RuleSeverity::Warning,
                message: format!(
                    "{} should not be populated when both {} and {} are collected per SDTMIG 4.4.3",
                    dur_var, stdtc_var, endtc_var
                ),
                description: "Per SDTMIG v3.4 Section 4.4.3, if both --STDTC and --ENDTC are \
                              collected, durations can be calculated by the difference in these \
                              2 values, and need not be in the submission dataset. --DUR should \
                              generally be used if collected in lieu of a start date/time and \
                              end date/time."
                    .to_string(),
                context: RuleContext::DurationUsage {
                    dur_variable: dur_var,
                    stdtc_variable: stdtc_var,
                    endtc_variable: endtc_var,
                },
            });
        }

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
