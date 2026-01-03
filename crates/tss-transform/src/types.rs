//! Core types for the SDTM transformation system.
//!
//! All transformation logic is derived from Variable metadata - no hardcoded domain rules.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;
use tss_model::ct::TerminologyRegistry;

/// Transformation type inferred from Variable metadata.
///
/// Each variant represents a specific SDTM transformation. The type is
/// automatically determined from Variable fields like `name`, `codelist_code`,
/// `described_value_domain`, and `data_type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TransformType {
    /// Direct copy from source column (passthrough).
    /// Default when no specific transformation is needed.
    CopyDirect,

    /// Constant value (STUDYID, DOMAIN).
    /// Value comes from TransformContext, not source data.
    Constant,

    /// USUBJID derivation: STUDYID-SUBJID pattern.
    /// Per SDTMIG 4.1.2, USUBJID must be unique across submission.
    UsubjidPrefix,

    /// Sequence number generation: unique per USUBJID within domain.
    /// Per SDTMIG 4.1.5, --SEQ is a surrogate key for natural keys.
    SequenceNumber,

    /// ISO 8601 datetime formatting (YYYY-MM-DDTHH:MM:SS).
    /// Preserves partial precision (e.g., 2003-12 stays 2003-12).
    Iso8601DateTime,

    /// ISO 8601 date formatting (YYYY-MM-DD).
    /// Preserves partial precision.
    Iso8601Date,

    /// ISO 8601 duration formatting (PnYnMnDTnHnMnS or PnW).
    /// Per SDTMIG 4.4.4.
    Iso8601Duration,

    /// Study day calculation from reference date.
    /// Per SDTMIG 4.4.4: --DY = (event_date - RFSTDTC) + 1 if >= reference.
    StudyDay {
        /// Reference datetime variable (derived from --DY name, e.g., AESTDY -> AESTDTC)
        reference_dtc: String,
    },

    /// Controlled terminology normalization.
    /// Behavior differs for extensible vs non-extensible codelists.
    CtNormalization {
        /// NCI codelist code (e.g., "C66731" for SEX)
        codelist_code: String,
    },

    /// Numeric type conversion (String -> Float64).
    /// Per SDTMIG, Num variables are 8-byte floating point.
    NumericConversion,
}

impl TransformType {
    /// Returns true if this transform requires source data.
    /// Constants don't need source columns.
    pub fn requires_source(&self) -> bool {
        !matches!(self, TransformType::Constant)
    }

    /// Returns true if this is a generated/derived transform (not CT or copy).
    pub fn is_generated(&self) -> bool {
        !matches!(
            self,
            TransformType::CtNormalization { .. } | TransformType::CopyDirect
        )
    }
}

/// A single transformation rule for a variable.
///
/// Each rule specifies how to transform source data into an SDTM-compliant variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRule {
    /// Target SDTM variable name (e.g., "AESTDTC", "SEX").
    pub target_variable: String,

    /// Source column name from input data (if mapped).
    /// None for constants or unmapped variables.
    pub source_column: Option<String>,

    /// Type of transformation to apply.
    pub transform_type: TransformType,

    /// Human-readable description of the transformation.
    pub description: String,

    /// Variable order (for output column ordering per SDTMIG).
    pub order: u32,
}

/// Complete transformation pipeline for a domain.
///
/// Contains all rules needed to transform source data into SDTM format.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DomainPipeline {
    /// Domain code (e.g., "AE", "DM").
    pub domain_code: String,

    /// Study identifier (set at execution time).
    pub study_id: String,

    /// Ordered list of transformation rules.
    pub rules: Vec<TransformRule>,
}

impl DomainPipeline {
    /// Create a new empty pipeline for a domain.
    pub fn new(domain_code: impl Into<String>) -> Self {
        Self {
            domain_code: domain_code.into(),
            study_id: String::new(),
            rules: Vec::new(),
        }
    }

    /// Add a rule to the pipeline.
    pub fn add_rule(&mut self, rule: TransformRule) {
        self.rules.push(rule);
    }

    /// Get rules sorted by variable order.
    pub fn rules_ordered(&self) -> Vec<&TransformRule> {
        let mut rules: Vec<&TransformRule> = self.rules.iter().collect();
        rules.sort_by_key(|r| r.order);
        rules
    }
}

/// Context for transformation execution.
///
/// Contains runtime data needed during transformation, including
/// study configuration, reference dates, and CT registry.
#[derive(Debug, Clone)]
pub struct TransformContext {
    /// Study identifier (e.g., "CDISC01").
    pub study_id: String,

    /// Domain code (e.g., "AE").
    pub domain_code: String,

    /// Reference date for study day calculation (RFSTDTC from DM).
    /// If None, study day columns will be empty.
    pub reference_date: Option<NaiveDate>,

    /// CT registry for codelist normalization.
    /// If None, CT normalization will preserve original values.
    pub ct_registry: Option<TerminologyRegistry>,

    /// Column mappings: target_variable -> source_column.
    pub mappings: BTreeMap<String, String>,

    /// Variables to omit from output (Permissible only).
    /// These variables will be completely excluded from the output DataFrame.
    pub omitted: BTreeSet<String>,
}

impl TransformContext {
    /// Create a new transform context.
    pub fn new(study_id: impl Into<String>, domain_code: impl Into<String>) -> Self {
        Self {
            study_id: study_id.into(),
            domain_code: domain_code.into(),
            reference_date: None,
            ct_registry: None,
            mappings: BTreeMap::new(),
            omitted: BTreeSet::new(),
        }
    }

    /// Set the reference date for study day calculations.
    pub fn with_reference_date(mut self, date: Option<NaiveDate>) -> Self {
        self.reference_date = date;
        self
    }

    /// Set the CT registry for normalization.
    pub fn with_ct_registry(mut self, registry: Option<TerminologyRegistry>) -> Self {
        self.ct_registry = registry;
        self
    }

    /// Set the column mappings.
    pub fn with_mappings(mut self, mappings: BTreeMap<String, String>) -> Self {
        self.mappings = mappings;
        self
    }

    /// Set the omitted variables.
    pub fn with_omitted(mut self, omitted: BTreeSet<String>) -> Self {
        self.omitted = omitted;
        self
    }

    /// Get the source column for a target variable.
    pub fn get_source_column(&self, target: &str) -> Option<&str> {
        self.mappings.get(target).map(String::as_str)
    }

    /// Check if a variable is marked as omitted.
    pub fn is_omitted(&self, target: &str) -> bool {
        self.omitted.contains(target)
    }
}
