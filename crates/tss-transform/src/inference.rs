//! Transformation type inference from SDTM Variable metadata.
//!
//! All transformation logic is derived from Variable fields - no hardcoded domain rules.
//! Priority order for inference (highest to lowest):
//! 1. Name-based patterns (STUDYID, DOMAIN, USUBJID, *SEQ, *DY, *DTC, *DT, *DUR)
//! 2. Described value domain ("ISO 8601 datetime", "duration")
//! 3. Codelist code (CT normalization)
//! 4. Data type (Num -> NumericConversion)
//! 5. Default (CopyDirect)

use tss_model::{Domain, Variable, VariableType};

use crate::types::{DomainPipeline, TransformRule, TransformType};

/// Build transformation pipeline from domain metadata.
///
/// This is the main entry point for pipeline creation. Each variable
/// in the domain is analyzed to determine its transformation type.
pub fn build_pipeline_from_domain(domain: &Domain) -> DomainPipeline {
    let mut pipeline = DomainPipeline::new(&domain.name);

    for variable in &domain.variables {
        let transform_type = infer_transform_type(variable, &domain.name);
        let description = generate_description(&variable.name, &transform_type);
        let order = variable.order.unwrap_or(999);

        pipeline.add_rule(TransformRule {
            target_variable: variable.name.clone(),
            source_column: None, // Set at execution time via mappings
            transform_type,
            description,
            order,
        });
    }

    pipeline
}

/// Infer transformation type from Variable metadata.
///
/// Uses a priority-based algorithm to determine the appropriate
/// transformation based on variable name, described value domain,
/// codelist code, and data type.
fn infer_transform_type(variable: &Variable, domain_code: &str) -> TransformType {
    let name = &variable.name;
    let dvd = variable
        .described_value_domain
        .as_deref()
        .unwrap_or("")
        .to_lowercase();

    // 1. Name-based patterns (highest priority)

    // Constants: STUDYID, DOMAIN
    if name == "STUDYID" || name == "DOMAIN" {
        return TransformType::Constant;
    }

    // USUBJID derivation
    if name == "USUBJID" {
        return TransformType::UsubjidPrefix;
    }

    // Sequence number: domain-prefixed SEQ (e.g., AESEQ, DMSEQ)
    if name.ends_with("SEQ") && name.starts_with(domain_code) && name.len() > 3 {
        return TransformType::SequenceNumber;
    }

    // Study day: *DY suffix (e.g., AESTDY, AEENDY)
    // Derive reference DTC from DY name
    if name.ends_with("DY") && name.len() > 2 {
        let prefix = &name[..name.len() - 2];
        let reference_dtc = format!("{prefix}DTC");
        return TransformType::StudyDay { reference_dtc };
    }

    // ISO 8601 duration: *DUR suffix or described value domain
    if name.ends_with("DUR") || dvd.contains("duration") {
        return TransformType::Iso8601Duration;
    }

    // ISO 8601 datetime: *DTC or *DTM suffix
    if name.ends_with("DTC") || name.ends_with("DTM") {
        return TransformType::Iso8601DateTime;
    }

    // ISO 8601 date: *DT suffix (but not *DTM or *DTC)
    if name.ends_with("DT") && !name.ends_with("DTM") && !name.ends_with("DTC") {
        return TransformType::Iso8601Date;
    }

    // 2. Described Value Domain patterns

    // ISO 8601 datetime from described value domain
    if dvd.contains("iso 8601") && dvd.contains("datetime") {
        return TransformType::Iso8601DateTime;
    }

    // ISO 8601 date/interval from described value domain
    if dvd.contains("iso 8601") && !dvd.contains("duration") {
        return TransformType::Iso8601Date;
    }

    // 3. Codelist Code -> CT Normalization
    if let Some(codelist_code) = &variable.codelist_code {
        let code = codelist_code.trim();
        if !code.is_empty() {
            // Take first codelist if multiple (separated by ; or ,)
            let first_code = code
                .split([';', ','])
                .next()
                .unwrap_or("")
                .trim()
                .to_string();

            if !first_code.is_empty() {
                return TransformType::CtNormalization {
                    codelist_code: first_code,
                };
            }
        }
    }

    // 4. Data Type -> Numeric Conversion
    if variable.data_type == VariableType::Num {
        return TransformType::NumericConversion;
    }

    // 5. Default: Copy directly
    TransformType::CopyDirect
}

/// Generate a human-readable description for a transformation.
fn generate_description(var_name: &str, transform_type: &TransformType) -> String {
    match transform_type {
        TransformType::Constant => {
            if var_name == "STUDYID" {
                "Study identifier from configuration".to_string()
            } else if var_name == "DOMAIN" {
                "Domain code constant".to_string()
            } else {
                "Constant value".to_string()
            }
        }
        TransformType::UsubjidPrefix => "Derive as STUDYID-SUBJID".to_string(),
        TransformType::SequenceNumber => "Generate unique sequence per USUBJID".to_string(),
        TransformType::Iso8601DateTime => {
            "Format as ISO 8601 datetime (preserves precision)".to_string()
        }
        TransformType::Iso8601Date => "Format as ISO 8601 date (preserves precision)".to_string(),
        TransformType::Iso8601Duration => {
            "Format as ISO 8601 duration (PnYnMnDTnHnMnS)".to_string()
        }
        TransformType::StudyDay { reference_dtc } => {
            format!("Calculate study day from {reference_dtc} relative to RFSTDTC")
        }
        TransformType::CtNormalization { codelist_code } => {
            format!("Normalize using codelist {codelist_code}")
        }
        TransformType::NumericConversion => "Convert to numeric (Float64)".to_string(),
        TransformType::CopyDirect => "Copy value directly".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_variable(name: &str) -> Variable {
        Variable {
            name: name.to_string(),
            label: None,
            data_type: VariableType::Char,
            length: None,
            role: None,
            core: None,
            codelist_code: None,
            described_value_domain: None,
            order: None,
        }
    }

    #[test]
    fn test_infer_studyid_constant() {
        let var = make_variable("STUDYID");
        assert_eq!(infer_transform_type(&var, "AE"), TransformType::Constant);
    }

    #[test]
    fn test_infer_domain_constant() {
        let var = make_variable("DOMAIN");
        assert_eq!(infer_transform_type(&var, "DM"), TransformType::Constant);
    }

    #[test]
    fn test_infer_usubjid() {
        let var = make_variable("USUBJID");
        assert_eq!(
            infer_transform_type(&var, "AE"),
            TransformType::UsubjidPrefix
        );
    }

    #[test]
    fn test_infer_sequence_number() {
        let var = make_variable("AESEQ");
        assert_eq!(
            infer_transform_type(&var, "AE"),
            TransformType::SequenceNumber
        );
    }

    #[test]
    fn test_infer_datetime_from_suffix() {
        let var = make_variable("AESTDTC");
        assert_eq!(
            infer_transform_type(&var, "AE"),
            TransformType::Iso8601DateTime
        );
    }

    #[test]
    fn test_infer_date_from_suffix() {
        let var = make_variable("BRTHDTDT");
        assert_eq!(infer_transform_type(&var, "DM"), TransformType::Iso8601Date);
    }

    #[test]
    fn test_infer_studyday_from_suffix() {
        let var = make_variable("AESTDY");
        let result = infer_transform_type(&var, "AE");
        assert!(matches!(
            result,
            TransformType::StudyDay { ref reference_dtc } if reference_dtc == "AESTDTC"
        ));
    }

    #[test]
    fn test_infer_duration_from_suffix() {
        let var = make_variable("AEDUR");
        assert_eq!(
            infer_transform_type(&var, "AE"),
            TransformType::Iso8601Duration
        );
    }

    #[test]
    fn test_infer_ct_normalization() {
        let mut var = make_variable("SEX");
        var.codelist_code = Some("C66731".to_string());
        assert!(matches!(
            infer_transform_type(&var, "DM"),
            TransformType::CtNormalization { codelist_code } if codelist_code == "C66731"
        ));
    }

    #[test]
    fn test_infer_numeric_conversion() {
        let mut var = make_variable("AGE");
        var.data_type = VariableType::Num;
        assert_eq!(
            infer_transform_type(&var, "DM"),
            TransformType::NumericConversion
        );
    }

    #[test]
    fn test_infer_copy_default() {
        let var = make_variable("CUSTOMVAR");
        assert_eq!(infer_transform_type(&var, "AE"), TransformType::CopyDirect);
    }

    #[test]
    fn test_infer_datetime_from_described_value_domain() {
        let mut var = make_variable("CUSTOMDTC");
        var.described_value_domain = Some("ISO 8601 datetime or interval".to_string());
        // This should match the suffix first
        assert_eq!(
            infer_transform_type(&var, "XX"),
            TransformType::Iso8601DateTime
        );
    }

    #[test]
    fn test_infer_duration_from_described_value_domain() {
        let mut var = make_variable("EXDURATION");
        var.described_value_domain = Some("ISO 8601 duration".to_string());
        assert_eq!(
            infer_transform_type(&var, "EX"),
            TransformType::Iso8601Duration
        );
    }
}
