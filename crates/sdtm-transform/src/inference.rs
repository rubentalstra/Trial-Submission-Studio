//! Transform type inference from variable metadata.
//!
//! This module provides the logic to automatically derive transformation types
//! from SDTM variable metadata without any hardcoded domain-specific rules.
//!
//! # Key Principle
//!
//! All transformation types are inferred from:
//! - Variable name patterns (`STUDYID`, `DOMAIN`, `*SEQ`, `*DY`, `*DTC`)
//! - `described_value_domain` field (ISO 8601 formats)
//! - `codelist_code` field (CT normalization)
//! - `data_type` + `role` combination
//!
//! # Example
//!
//! ```ignore
//! use sdtm_transform::inference::infer_transform_type;
//! use sdtm_model::Variable;
//!
//! let transform = infer_transform_type(&variable);
//! ```

use crate::pipeline::{DomainPipeline, TransformRule, TransformType};
use sdtm_model::{Domain, Variable, VariableType};

/// Infer the transformation type for a variable based on its metadata.
///
/// This function uses ONLY the variable's metadata (name, role, codelist_code,
/// described_value_domain, data_type) to determine the appropriate transformation.
/// There are NO hardcoded domain-specific rules.
///
/// # Algorithm
///
/// The inference follows this priority order:
///
/// 1. **Name patterns** (highest priority):
///    - `STUDYID` → Constant
///    - `DOMAIN` → Constant
///    - `USUBJID` → UsubjidPrefix
///    - `*SEQ` (suffix) → SequenceNumber
///    - `*DY` (suffix) → StudyDay
///    - `*DTC` (suffix) → Iso8601DateTime
///
/// 2. **Described value domain**:
///    - Contains "ISO 8601" → Iso8601DateTime or Iso8601Date
///    - Contains "duration" → Iso8601Duration
///
/// 3. **Codelist code**:
///    - Present → CtNormalization
///
/// 4. **Data type + Role**:
///    - Num + Result Qualifier → NumericConversion
///
/// 5. **Default**:
///    - CopyDirect (passthrough)
pub fn infer_transform_type(variable: &Variable) -> TransformType {
    let name = variable.name.to_uppercase();

    // 1. Check name patterns (highest priority)
    if name == "STUDYID" || name == "DOMAIN" {
        return TransformType::Constant;
    }

    if name == "USUBJID" {
        return TransformType::UsubjidPrefix;
    }

    // Sequence numbers: *SEQ suffix (e.g., AESEQ, DMSEQ, LBSEQ)
    if name.len() >= 3 && name.ends_with("SEQ") {
        return TransformType::SequenceNumber;
    }

    // Study day: *DY suffix (e.g., AEDY, EXSTDY, LBDY)
    if name.len() >= 2 && name.ends_with("DY") {
        // Extract the corresponding DTC variable name
        // e.g., AEDY → AEDTC, EXSTDY → EXSTDTC
        let reference_dtc = derive_dtc_from_dy(&name);
        return TransformType::StudyDay { reference_dtc };
    }

    // Datetime columns: *DTC suffix (e.g., AESTDTC, AEENDTC, EXSTDTC)
    if name.len() >= 3 && name.ends_with("DTC") {
        return TransformType::Iso8601DateTime;
    }

    // 2. Check described_value_domain for ISO 8601 hints
    if let Some(ref dvd) = variable.described_value_domain {
        let dvd_upper = dvd.to_uppercase();
        if dvd_upper.contains("DURATION") || dvd_upper.contains("ISO 8601 DURATION") {
            return TransformType::Iso8601Duration;
        }
        if dvd_upper.contains("ISO 8601") {
            // Determine if it's date-only or datetime
            if dvd_upper.contains("DATE") && !dvd_upper.contains("TIME") {
                return TransformType::Iso8601Date;
            }
            return TransformType::Iso8601DateTime;
        }
    }

    // 3. Check for CT normalization
    if let Some(ref codelist) = variable.codelist_code {
        // Handle multiple codelists separated by semicolon
        let code = codelist.split(';').next().unwrap_or("").trim();
        if !code.is_empty() {
            return TransformType::CtNormalization {
                codelist_code: code.to_string(),
            };
        }
    }

    // 4. Check data type + role for numeric conversion
    if variable.data_type == VariableType::Num {
        // Result qualifier variables with numeric type
        if let Some(ref role) = variable.role {
            if role.eq_ignore_ascii_case("Result Qualifier") {
                return TransformType::NumericConversion;
            }
        }
    }

    // 5. Default to direct copy
    TransformType::CopyDirect
}

/// Derive the corresponding DTC variable name from a DY variable name.
///
/// Examples:
/// - `AEDY` → `AEDTC` (standard case)
/// - `AESTDY` → `AESTDTC` (start date)
/// - `AEENDY` → `AEENDTC` (end date)
fn derive_dtc_from_dy(dy_name: &str) -> String {
    // Remove the DY suffix and add DTC
    if let Some(base) = dy_name.strip_suffix("DY") {
        format!("{base}DTC")
    } else {
        // Fallback - shouldn't happen if caller checks for DY suffix
        dy_name.to_string()
    }
}

/// Build a transformation pipeline from a domain definition.
///
/// Creates transformation rules for all variables in the domain based on
/// their metadata. No hardcoded domain-specific rules are used.
pub fn build_pipeline_from_domain(domain: &Domain) -> DomainPipeline {
    let mut pipeline = DomainPipeline::new(&domain.code);

    for (order, variable) in domain.variables.iter().enumerate() {
        let transform_type = infer_transform_type(variable);
        let rule = TransformRule::derived(&variable.name, transform_type, order as u32);
        pipeline.add_rule(rule);
    }

    pipeline
}

/// Categorize variables by their inferred transform types.
///
/// Returns a summary of how many variables fall into each transform category.
/// Useful for debugging and reporting.
#[derive(Debug, Default)]
pub struct TransformSummary {
    pub constants: usize,
    pub identifiers: usize,
    pub sequences: usize,
    pub ct_normalized: usize,
    pub datetime: usize,
    pub study_day: usize,
    pub numeric: usize,
    pub copy_direct: usize,
}

impl TransformSummary {
    /// Summarize transforms for a domain.
    pub fn from_domain(domain: &Domain) -> Self {
        let mut summary = Self::default();

        for variable in &domain.variables {
            let transform = infer_transform_type(variable);
            match transform {
                TransformType::Constant => summary.constants += 1,
                TransformType::UsubjidPrefix => summary.identifiers += 1,
                TransformType::SequenceNumber => summary.sequences += 1,
                TransformType::CtNormalization { .. } => summary.ct_normalized += 1,
                TransformType::Iso8601DateTime
                | TransformType::Iso8601Date
                | TransformType::Iso8601Duration => {
                    summary.datetime += 1;
                }
                TransformType::StudyDay { .. } => summary.study_day += 1,
                TransformType::NumericConversion => summary.numeric += 1,
                TransformType::CopyDirect => summary.copy_direct += 1,
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_variable(name: &str) -> Variable {
        Variable {
            name: name.to_string(),
            label: None,
            role: None,
            codelist_code: None,
            described_value_domain: None,
            data_type: VariableType::Char,
            order: None,
            core: None,
            length: None,
        }
    }

    #[test]
    fn test_studyid_is_constant() {
        let var = make_variable("STUDYID");
        assert_eq!(infer_transform_type(&var), TransformType::Constant);
    }

    #[test]
    fn test_domain_is_constant() {
        let var = make_variable("DOMAIN");
        assert_eq!(infer_transform_type(&var), TransformType::Constant);
    }

    #[test]
    fn test_usubjid_prefix() {
        let var = make_variable("USUBJID");
        assert_eq!(infer_transform_type(&var), TransformType::UsubjidPrefix);
    }

    #[test]
    fn test_sequence_pattern() {
        let var = make_variable("AESEQ");
        assert_eq!(infer_transform_type(&var), TransformType::SequenceNumber);

        let var2 = make_variable("LBSEQ");
        assert_eq!(infer_transform_type(&var2), TransformType::SequenceNumber);
    }

    #[test]
    fn test_datetime_pattern() {
        let var = make_variable("AESTDTC");
        assert_eq!(infer_transform_type(&var), TransformType::Iso8601DateTime);

        let var2 = make_variable("AEENDTC");
        assert_eq!(infer_transform_type(&var2), TransformType::Iso8601DateTime);
    }

    #[test]
    fn test_study_day_pattern() {
        let var = make_variable("AEDY");
        assert_eq!(
            infer_transform_type(&var),
            TransformType::StudyDay {
                reference_dtc: "AEDTC".to_string()
            }
        );

        let var2 = make_variable("AESTDY");
        assert_eq!(
            infer_transform_type(&var2),
            TransformType::StudyDay {
                reference_dtc: "AESTDTC".to_string()
            }
        );
    }

    #[test]
    fn test_codelist_normalization() {
        let mut var = make_variable("SEX");
        var.codelist_code = Some("C66731".to_string());
        assert_eq!(
            infer_transform_type(&var),
            TransformType::CtNormalization {
                codelist_code: "C66731".to_string()
            }
        );
    }

    #[test]
    fn test_multiple_codelists_uses_first() {
        let mut var = make_variable("AEREL");
        var.codelist_code = Some("C66769; C66770".to_string());
        assert_eq!(
            infer_transform_type(&var),
            TransformType::CtNormalization {
                codelist_code: "C66769".to_string()
            }
        );
    }

    #[test]
    fn test_copy_direct_default() {
        let var = make_variable("AETERM");
        assert_eq!(infer_transform_type(&var), TransformType::CopyDirect);
    }

    #[test]
    fn test_derive_dtc_from_dy() {
        assert_eq!(derive_dtc_from_dy("AEDY"), "AEDTC");
        assert_eq!(derive_dtc_from_dy("AESTDY"), "AESTDTC");
        assert_eq!(derive_dtc_from_dy("AEENDY"), "AEENDTC");
    }
}
