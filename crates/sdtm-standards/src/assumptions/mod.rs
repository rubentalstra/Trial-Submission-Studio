//! SDTMIG Validation Rule Generation
//!
//! This module generates validation rules **dynamically from standards metadata**:
//!
//! - `Variables.csv`: Core designations (Req/Exp/Perm), CT codelist codes (C66742, etc.)
//! - `CT files`: Valid values per codelist, extensibility flag (Yes/No)
//! - `P21 Rules.csv`: Rule templates with IDs (SD0002, CT2001, etc.) and severity
//!
//! ## Why Dynamic Generation?
//!
//! Rules are NOT manually coded. Instead, they are derived from the same metadata
//! files that define SDTM standards. This ensures:
//!
//! 1. Rules stay in sync with standards (no drift)
//! 2. CT validation uses actual valid values from CT files
//! 3. Core designation rules (Req/Exp/Perm) match Variables.csv exactly
//! 4. P21 rule IDs and messages are consistent with industry tooling
//!
//! ## Example Usage
//!
//! ```ignore
//! use sdtm_standards::{
//!     load_default_sdtm_ig_domains, load_default_ct_registry, load_default_p21_rules,
//!     RuleGenerator,
//! };
//!
//! let domains = load_default_sdtm_ig_domains()?;
//! let ct_registry = load_default_ct_registry()?;
//! let p21_rules = load_default_p21_rules()?;
//!
//! let generator = RuleGenerator::new().with_p21_rules(p21_rules);
//!
//! for domain in &domains {
//!     let rules = generator.generate_rules_for_domain(domain, &ct_registry);
//!     for rule in rules {
//!         // rule.rule_id: "SD0002", "CT2001", "CT2002", "SD0003", etc.
//!         // All rule IDs are official P21 rules from the CSV
//!     }
//! }
//! ```
//!
//! ## Rule Types Generated
//!
//! | Source | Rule ID | Description |
//! |--------|---------|-------------|
//! | Core="Req" in Variables.csv | SD0002 | Required variable cannot be null |
//! | Core="Req" in Variables.csv | SD0056 | Required variable must be present |
//! | Core="Exp" in Variables.csv | SD0057 | Expected variable should be present |
//! | codelist_code + extensible=No | CT2001 | Value must be from non-extensible codelist |
//! | codelist_code + extensible=Yes | CT2002 | Value should be from extensible codelist |
//! | Variable ends with DTC | SD0003 | Must be valid ISO 8601 format |
//! | Variable ends with SEQ | SD0005 | Must be unique within USUBJID |

mod generator;

pub use generator::{
    GeneratedRule, RuleContext, RuleGenerationSummary, RuleGenerator, RuleSeverity,
};

// Re-export CoreDesignation for parsing Core column values
mod core;
pub use core::CoreDesignation;
