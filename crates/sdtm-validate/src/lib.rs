//! SDTM validation and conformance checking.
//!
//! This crate provides comprehensive validation logic for SDTM datasets:
//!
//! - **Controlled Terminology (CT)**: Validates values against CT codelists
//! - **Required Variables**: Checks presence and population of Req variables
//! - **Expected Variables**: Warns about missing Exp variables
//! - **Data Type Validation**: Ensures Num columns contain numeric data
//! - **ISO 8601 Date Validation**: Validates date/datetime format compliance
//! - **Sequence Uniqueness**: Checks for duplicate --SEQ per subject
//! - **Text Length**: Validates character field lengths
//! - **Identifier Nulls**: Checks that ID variables have no nulls
//!
//! # SDTMIG Reference
//!
//! - Chapter 4: Variable Core (Req/Exp/Perm)
//! - Chapter 7: ISO 8601 date/time formats
//! - Chapter 10: Controlled Terminology
//! - Appendix C: Validation Rules

use polars::prelude::{AnyValue, DataFrame, DataType as PolarsDataType};
use regex::Regex;
use sdtm_ingest::any_to_string;
use sdtm_model::ct::{Codelist, ResolvedCodelist, TerminologyRegistry};
use sdtm_model::p21::rule_ids;
use sdtm_model::{
    CaseInsensitiveSet, CheckType, Domain, Severity, ValidationIssue, ValidationReport, Variable,
    VariableType,
};
use std::collections::{BTreeSet, HashSet};
use std::sync::LazyLock;

/// ISO 8601 date patterns per SDTMIG Chapter 7.
/// Supports: YYYY, YYYY-MM, YYYY-MM-DD, YYYY-MM-DDTHH:MM:SS
static ISO8601_DATE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(\d{4})(-((0[1-9]|1[0-2]))(-((0[1-9]|[12]\d|3[01]))(T(([01]\d|2[0-3]):([0-5]\d)(:([0-5]\d)(\.(\d+))?)?))?)?)?$",
    )
    .expect("Invalid ISO 8601 regex")
});

/// Known date/time variable name suffixes that require ISO 8601 validation.
const DATE_SUFFIXES: &[&str] = &["DTC", "DTM", "DT", "TM", "STDTC", "ENDTC", "STDT", "ENDT"];

/// Validate a single domain against SDTM conformance rules.
///
/// Runs all validation checks:
/// - Controlled terminology values
/// - Required variable presence and population
/// - Expected variable presence (warnings)
/// - Data type conformance
/// - ISO 8601 date format validation
/// - Unique sequence numbers per subject
/// - Text length limits
/// - Identifier null checks
pub fn validate_domain(
    domain: &Domain,
    df: &DataFrame,
    ct_registry: Option<&TerminologyRegistry>,
) -> ValidationReport {
    let column_lookup = build_column_lookup(df);
    let mut issues = Vec::new();

    // 1. Required variable checks (presence + population)
    issues.extend(check_required_variables(domain, df, &column_lookup));

    // 2. Expected variable checks (presence only, warnings)
    issues.extend(check_expected_variables(domain, &column_lookup));

    // 3. Data type validation (Num columns must be numeric)
    issues.extend(check_data_types(domain, df, &column_lookup));

    // 4. ISO 8601 date format validation
    issues.extend(check_date_formats(domain, df, &column_lookup));

    // 5. Sequence uniqueness (--SEQ must be unique per USUBJID)
    issues.extend(check_sequence_uniqueness(domain, df, &column_lookup));

    // 6. Text length validation
    issues.extend(check_text_lengths(domain, df, &column_lookup));

    // 7. Identifier null checks
    issues.extend(check_identifier_nulls(domain, df, &column_lookup));

    // 8. Controlled terminology validation
    if let Some(ct_registry) = ct_registry {
        issues.extend(check_controlled_terminology(
            domain,
            df,
            &column_lookup,
            ct_registry,
        ));
    }

    ValidationReport {
        domain_code: domain.code.clone(),
        issues,
    }
}

// =============================================================================
// Required Variable Checks (SDTMIG 4.1)
// =============================================================================

/// Check that all Required (Req) variables are present and populated.
fn check_required_variables(
    domain: &Domain,
    df: &DataFrame,
    column_lookup: &CaseInsensitiveSet,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    for variable in &domain.variables {
        let is_required = variable
            .core
            .as_ref()
            .is_some_and(|c| c.eq_ignore_ascii_case("Req"));

        if !is_required {
            continue;
        }

        // Check presence
        let Some(column) = column_lookup.get(&variable.name) else {
            issues.push(ValidationIssue {
                check_type: Some(CheckType::RequiredVariableMissing),
                code: rule_ids::SD0056.to_string(),
                message: format!(
                    "Required variable {} is not present in domain {}.",
                    variable.name, domain.code
                ),
                severity: Severity::Error,
                variable: Some(variable.name.clone()),
                count: None,
                ct_source: None,
                observed_values: None,
                allowed_values: None,
                allowed_count: None,
                ct_examples: None,
            });
            continue;
        };

        // Check population (no nulls allowed for Req)
        let null_count = count_null_values(df, column);
        if null_count > 0 {
            issues.push(ValidationIssue {
                check_type: Some(CheckType::RequiredVariableEmpty),
                code: rule_ids::SD0002.to_string(),
                message: format!(
                    "Required variable {} has {} null value(s). Required variables must be fully populated.",
                    variable.name, null_count
                ),
                severity: Severity::Error,
                variable: Some(variable.name.clone()),
                count: Some(null_count),
                ct_source: None,
                observed_values: None,
                allowed_values: None,
                allowed_count: None,
                ct_examples: None,
            });
        }
    }

    issues
}

// =============================================================================
// Expected Variable Checks (SDTMIG 4.1)
// =============================================================================

/// Check that Expected (Exp) variables are present (warnings only).
fn check_expected_variables(
    domain: &Domain,
    column_lookup: &CaseInsensitiveSet,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    for variable in &domain.variables {
        let is_expected = variable
            .core
            .as_ref()
            .is_some_and(|c| c.eq_ignore_ascii_case("Exp"));

        if !is_expected {
            continue;
        }

        if column_lookup.get(&variable.name).is_none() {
            issues.push(ValidationIssue {
                check_type: Some(CheckType::ExpectedVariableMissing),
                code: rule_ids::SD0057.to_string(),
                message: format!(
                    "Expected variable {} is not present in domain {}. Consider adding if applicable.",
                    variable.name, domain.code
                ),
                severity: Severity::Warning,
                variable: Some(variable.name.clone()),
                count: None,
                ct_source: None,
                observed_values: None,
                allowed_values: None,
                allowed_count: None,
                ct_examples: None,
            });
        }
    }

    issues
}

// =============================================================================
// Data Type Validation (SDTMIG 2.4)
// =============================================================================

/// Check that Num variables contain only numeric values.
fn check_data_types(
    domain: &Domain,
    df: &DataFrame,
    column_lookup: &CaseInsensitiveSet,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    for variable in &domain.variables {
        if variable.data_type != VariableType::Num {
            continue;
        }

        let Some(column) = column_lookup.get(&variable.name) else {
            continue;
        };

        let Ok(series) = df.column(column) else {
            continue;
        };

        // Check if the Polars column is numeric
        let dtype = series.dtype();
        let is_numeric = matches!(
            dtype,
            PolarsDataType::Int8
                | PolarsDataType::Int16
                | PolarsDataType::Int32
                | PolarsDataType::Int64
                | PolarsDataType::UInt8
                | PolarsDataType::UInt16
                | PolarsDataType::UInt32
                | PolarsDataType::UInt64
                | PolarsDataType::Float32
                | PolarsDataType::Float64
        );

        if is_numeric {
            continue;
        }

        // String column - check if values can be parsed as numbers
        let (non_numeric_count, samples) = collect_non_numeric_values(df, column);
        if non_numeric_count > 0 {
            issues.push(ValidationIssue {
                check_type: Some(CheckType::DataTypeMismatch),
                code: rule_ids::SD0055.to_string(),
                message: format!(
                    "Numeric variable {} contains {} non-numeric value(s).",
                    variable.name, non_numeric_count
                ),
                severity: Severity::Error,
                variable: Some(variable.name.clone()),
                count: Some(non_numeric_count),
                ct_source: None,
                observed_values: if samples.is_empty() {
                    None
                } else {
                    Some(samples)
                },
                allowed_values: None,
                allowed_count: None,
                ct_examples: None,
            });
        }
    }

    issues
}

// =============================================================================
// ISO 8601 Date Format Validation (SDTMIG Chapter 7)
// =============================================================================

/// Check that date/datetime variables conform to ISO 8601 format.
fn check_date_formats(
    domain: &Domain,
    df: &DataFrame,
    column_lookup: &CaseInsensitiveSet,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    for variable in &domain.variables {
        // Only check variables that appear to be date/time fields
        if !is_date_variable(&variable.name) {
            continue;
        }

        let Some(column) = column_lookup.get(&variable.name) else {
            continue;
        };

        let (invalid_count, samples) = collect_invalid_dates(df, column);
        if invalid_count > 0 {
            issues.push(ValidationIssue {
                check_type: Some(CheckType::InvalidDateFormat),
                code: rule_ids::SD0003.to_string(),
                message: format!(
                    "Date variable {} has {} value(s) not in ISO 8601 format (YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS).",
                    variable.name, invalid_count
                ),
                severity: Severity::Error,
                variable: Some(variable.name.clone()),
                count: Some(invalid_count),
                ct_source: None,
                observed_values: if samples.is_empty() {
                    None
                } else {
                    Some(samples)
                },
                allowed_values: Some(vec![
                    "YYYY".to_string(),
                    "YYYY-MM".to_string(),
                    "YYYY-MM-DD".to_string(),
                    "YYYY-MM-DDTHH:MM".to_string(),
                    "YYYY-MM-DDTHH:MM:SS".to_string(),
                ]),
                allowed_count: None,
                ct_examples: None,
            });
        }
    }

    issues
}

/// Check if a variable name indicates it's a date/time field.
fn is_date_variable(name: &str) -> bool {
    let upper = name.to_uppercase();
    DATE_SUFFIXES.iter().any(|suffix| upper.ends_with(suffix))
}

// =============================================================================
// Sequence Uniqueness (SDTMIG 4.1.5)
// =============================================================================

/// Check that --SEQ values are unique per USUBJID.
fn check_sequence_uniqueness(
    domain: &Domain,
    df: &DataFrame,
    column_lookup: &CaseInsensitiveSet,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // Find the --SEQ variable for this domain
    let seq_var_name = format!("{}SEQ", domain.code.to_uppercase());
    let seq_column = column_lookup.get(&seq_var_name);

    // USUBJID should always be present
    let usubjid_column = column_lookup.get("USUBJID");

    // If no SEQ column or no USUBJID, skip this check
    let (Some(seq_col), Some(subj_col)) = (seq_column, usubjid_column) else {
        return issues;
    };

    let duplicate_count = count_duplicate_sequences(df, subj_col, seq_col);
    if duplicate_count > 0 {
        issues.push(ValidationIssue {
            check_type: Some(CheckType::DuplicateSequence),
            code: rule_ids::SD0005.to_string(),
            message: format!(
                "{} has {} duplicate sequence number(s) within subject. --SEQ must be unique per USUBJID.",
                seq_var_name, duplicate_count
            ),
            severity: Severity::Error,
            variable: Some(seq_var_name),
            count: Some(duplicate_count),
            ct_source: None,
            observed_values: None,
            allowed_values: None,
            allowed_count: None,
            ct_examples: None,
        });
    }

    issues
}

// =============================================================================
// Text Length Validation (SDTMIG 2.4)
// =============================================================================

/// Check that character variables don't exceed their defined length.
fn check_text_lengths(
    domain: &Domain,
    df: &DataFrame,
    column_lookup: &CaseInsensitiveSet,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    for variable in &domain.variables {
        // Only check Char variables with a defined length
        if variable.data_type != VariableType::Char {
            continue;
        }

        let Some(max_length) = variable.length else {
            continue;
        };

        let Some(column) = column_lookup.get(&variable.name) else {
            continue;
        };

        let (exceeded_count, max_found, samples) =
            collect_length_violations(df, column, max_length);
        if exceeded_count > 0 {
            issues.push(ValidationIssue {
                check_type: Some(CheckType::TextLengthExceeded),
                code: rule_ids::SD0017.to_string(),
                message: format!(
                    "Variable {} has {} value(s) exceeding max length {} (max found: {}).",
                    variable.name, exceeded_count, max_length, max_found
                ),
                severity: Severity::Warning,
                variable: Some(variable.name.clone()),
                count: Some(exceeded_count),
                ct_source: None,
                observed_values: if samples.is_empty() {
                    None
                } else {
                    Some(samples)
                },
                allowed_values: Some(vec![format!("Max {} characters", max_length)]),
                allowed_count: None,
                ct_examples: None,
            });
        }
    }

    issues
}

// =============================================================================
// Identifier Null Checks (SDTMIG 4.1.2)
// =============================================================================

/// Check that Identifier role variables have no null values.
fn check_identifier_nulls(
    domain: &Domain,
    df: &DataFrame,
    column_lookup: &CaseInsensitiveSet,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    for variable in &domain.variables {
        let is_identifier = variable
            .role
            .as_ref()
            .is_some_and(|r| r.eq_ignore_ascii_case("Identifier"));

        if !is_identifier {
            continue;
        }

        let Some(column) = column_lookup.get(&variable.name) else {
            continue;
        };

        let null_count = count_null_values(df, column);
        if null_count > 0 {
            issues.push(ValidationIssue {
                check_type: Some(CheckType::IdentifierNull),
                code: rule_ids::SD0002.to_string(),
                message: format!(
                    "Identifier variable {} has {} null value(s). Identifiers must not be null.",
                    variable.name, null_count
                ),
                severity: Severity::Error,
                variable: Some(variable.name.clone()),
                count: Some(null_count),
                ct_source: None,
                observed_values: None,
                allowed_values: None,
                allowed_count: None,
                ct_examples: None,
            });
        }
    }

    issues
}

// =============================================================================
// Controlled Terminology Validation (SDTMIG Chapter 10)
// =============================================================================

/// Check that values conform to controlled terminology.
fn check_controlled_terminology(
    domain: &Domain,
    df: &DataFrame,
    column_lookup: &CaseInsensitiveSet,
    ct_registry: &TerminologyRegistry,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    for variable in &domain.variables {
        let Some(column) = column_lookup.get(&variable.name) else {
            continue;
        };
        if let Some(resolved) = resolve_ct(ct_registry, variable)
            && let Some(issue) = ct_issue(variable, df, column, &resolved)
        {
            issues.push(issue);
        }
    }

    issues
}

fn ct_issue(
    variable: &Variable,
    df: &DataFrame,
    column: &str,
    resolved: &ResolvedCodelist,
) -> Option<ValidationIssue> {
    let ct = resolved.codelist;
    let invalid = collect_invalid_ct_values(df, column, ct);
    if invalid.is_empty() {
        return None;
    }
    const MAX_ALLOWED_VALUES_IN_MESSAGE: usize = 12;
    const MAX_INVALID_EXAMPLES: usize = 5;
    const MAX_CT_EXAMPLES: usize = 5;

    // P21 rules: CT2001 for non-extensible (Error), CT2002 for extensible (Warning)
    let (p21_rule, severity) = if ct.extensible {
        (rule_ids::CT2002, Severity::Warning)
    } else {
        (rule_ids::CT2001, Severity::Error)
    };

    let observed_values: Vec<String> = invalid.iter().take(MAX_INVALID_EXAMPLES).cloned().collect();
    let mut message = format!(
        "{}: {} has {} value(s) not in {} ({}) from {}.",
        p21_rule,
        variable.name,
        invalid.len(),
        ct.name,
        ct.code,
        resolved.source()
    );
    if ct.extensible {
        message.push_str(" Codelist is extensible; invalid values are warnings.");
    }
    let allowed_values = ct.submission_values();
    let allowed_values_list = if allowed_values.len() <= MAX_ALLOWED_VALUES_IN_MESSAGE {
        Some(
            allowed_values
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        )
    } else {
        None
    };
    let allowed_count = if allowed_values.len() > MAX_ALLOWED_VALUES_IN_MESSAGE {
        Some(allowed_values.len() as u64)
    } else {
        None
    };
    let ct_examples = if allowed_values.len() > MAX_ALLOWED_VALUES_IN_MESSAGE {
        let mut examples = allowed_values.to_vec();
        examples.sort_unstable();
        examples.truncate(MAX_CT_EXAMPLES);
        Some(examples.into_iter().map(String::from).collect())
    } else {
        None
    };
    Some(ValidationIssue {
        check_type: Some(CheckType::ControlledTerminology),
        code: p21_rule.to_string(),
        message,
        severity,
        variable: Some(variable.name.clone()),
        count: Some(invalid.len() as u64),
        ct_source: Some(resolved.source().to_string()),
        observed_values: if observed_values.is_empty() {
            None
        } else {
            Some(observed_values)
        },
        allowed_values: allowed_values_list,
        allowed_count,
        ct_examples,
    })
}

fn collect_invalid_ct_values(df: &DataFrame, column: &str, ct: &Codelist) -> BTreeSet<String> {
    let mut invalid = BTreeSet::new();
    let series = match df.column(column) {
        Ok(series) => series,
        Err(_) => return invalid,
    };
    let submission_values: BTreeSet<String> = ct
        .submission_values()
        .iter()
        .map(|value| value.to_uppercase())
        .collect();

    for idx in 0..df.height() {
        let raw = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        let normalized = ct.normalize(trimmed);
        if normalized.is_empty() {
            continue;
        }
        let key = normalized.to_uppercase();
        if submission_values.contains(&key) {
            continue;
        }
        invalid.insert(trimmed.to_string());
    }
    invalid
}

fn resolve_ct<'a>(
    registry: &'a TerminologyRegistry,
    variable: &Variable,
) -> Option<ResolvedCodelist<'a>> {
    let codelist_code = variable.codelist_code.as_ref()?;
    let code = codelist_code.split(';').next()?.trim();
    if code.is_empty() {
        return None;
    }
    registry.resolve(code, None)
}

// =============================================================================
// Helper Functions
// =============================================================================

fn build_column_lookup(df: &DataFrame) -> CaseInsensitiveSet {
    CaseInsensitiveSet::new(df.get_column_names_owned())
}

/// Count null/empty values in a column.
fn count_null_values(df: &DataFrame, column: &str) -> u64 {
    let Ok(series) = df.column(column) else {
        return 0;
    };

    let mut count = 0u64;
    for idx in 0..df.height() {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        let str_value = any_to_string(value);
        if str_value.trim().is_empty() {
            count += 1;
        }
    }
    count
}

/// Collect non-numeric values from a column that should be numeric.
fn collect_non_numeric_values(df: &DataFrame, column: &str) -> (u64, Vec<String>) {
    let Ok(series) = df.column(column) else {
        return (0, vec![]);
    };

    let mut count = 0u64;
    let mut samples = Vec::new();
    const MAX_SAMPLES: usize = 5;

    for idx in 0..df.height() {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        let str_value = any_to_string(value);
        let trimmed = str_value.trim();

        if trimmed.is_empty() {
            continue; // Nulls are not type errors
        }

        // Try to parse as a number
        if trimmed.parse::<f64>().is_err() {
            count += 1;
            if samples.len() < MAX_SAMPLES {
                samples.push(trimmed.to_string());
            }
        }
    }

    (count, samples)
}

/// Collect values that don't conform to ISO 8601.
fn collect_invalid_dates(df: &DataFrame, column: &str) -> (u64, Vec<String>) {
    let Ok(series) = df.column(column) else {
        return (0, vec![]);
    };

    let mut count = 0u64;
    let mut samples = Vec::new();
    const MAX_SAMPLES: usize = 5;

    for idx in 0..df.height() {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        let str_value = any_to_string(value);
        let trimmed = str_value.trim();

        if trimmed.is_empty() {
            continue; // Nulls are OK for dates
        }

        if !ISO8601_DATE_REGEX.is_match(trimmed) {
            count += 1;
            if samples.len() < MAX_SAMPLES {
                samples.push(trimmed.to_string());
            }
        }
    }

    (count, samples)
}

/// Count duplicate sequence values per subject.
fn count_duplicate_sequences(df: &DataFrame, subject_col: &str, seq_col: &str) -> u64 {
    let (Ok(subj_series), Ok(seq_series)) = (df.column(subject_col), df.column(seq_col)) else {
        return 0;
    };

    // Build map of (USUBJID, SEQ) pairs and count duplicates
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut duplicate_count = 0u64;

    for idx in 0..df.height() {
        let subj = any_to_string(subj_series.get(idx).unwrap_or(AnyValue::Null));
        let seq = any_to_string(seq_series.get(idx).unwrap_or(AnyValue::Null));

        let key = (subj.trim().to_string(), seq.trim().to_string());
        if key.0.is_empty() || key.1.is_empty() {
            continue;
        }

        if !seen.insert(key) {
            duplicate_count += 1;
        }
    }

    duplicate_count
}

/// Collect values that exceed the specified length.
fn collect_length_violations(
    df: &DataFrame,
    column: &str,
    max_length: u32,
) -> (u64, usize, Vec<String>) {
    let Ok(series) = df.column(column) else {
        return (0, 0, vec![]);
    };

    let mut count = 0u64;
    let mut max_found = 0usize;
    let mut samples = Vec::new();
    const MAX_SAMPLES: usize = 3;

    for idx in 0..df.height() {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        let str_value = any_to_string(value);
        let len = str_value.len();

        if len > max_length as usize {
            count += 1;
            max_found = max_found.max(len);
            if samples.len() < MAX_SAMPLES {
                // Truncate sample for display
                let truncated = if str_value.len() > 50 {
                    format!("{}...", &str_value[..50])
                } else {
                    str_value
                };
                samples.push(format!("{} (len={})", truncated, len));
            }
        }
    }

    (count, max_found, samples)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    fn make_domain(variables: Vec<Variable>) -> Domain {
        Domain {
            code: "AE".to_string(),
            description: None,
            class_name: None,
            dataset_class: None,
            label: None,
            structure: None,
            dataset_name: None,
            variables,
        }
    }

    fn make_variable(name: &str, core: Option<&str>, data_type: VariableType) -> Variable {
        Variable {
            name: name.to_string(),
            label: None,
            data_type,
            length: None,
            role: None,
            core: core.map(String::from),
            codelist_code: None,
            order: None,
        }
    }

    #[test]
    fn test_required_variable_missing() {
        let domain = make_domain(vec![make_variable(
            "USUBJID",
            Some("Req"),
            VariableType::Char,
        )]);

        let df = DataFrame::new(vec![Series::new("OTHER".into(), vec!["A"]).into()]).unwrap();

        let report = validate_domain(&domain, &df, None);
        assert_eq!(report.issues.len(), 1);
        assert_eq!(
            report.issues[0].check_type,
            Some(CheckType::RequiredVariableMissing)
        );
    }

    #[test]
    fn test_required_variable_empty() {
        let domain = make_domain(vec![make_variable(
            "USUBJID",
            Some("Req"),
            VariableType::Char,
        )]);

        let df = DataFrame::new(vec![Series::new("USUBJID".into(), vec!["A", ""]).into()]).unwrap();

        let report = validate_domain(&domain, &df, None);
        assert_eq!(report.issues.len(), 1);
        assert_eq!(
            report.issues[0].check_type,
            Some(CheckType::RequiredVariableEmpty)
        );
    }

    #[test]
    fn test_expected_variable_missing() {
        let domain = make_domain(vec![make_variable(
            "AETERM",
            Some("Exp"),
            VariableType::Char,
        )]);

        let df = DataFrame::new(vec![Series::new("OTHER".into(), vec!["A"]).into()]).unwrap();

        let report = validate_domain(&domain, &df, None);
        assert_eq!(report.issues.len(), 1);
        assert_eq!(
            report.issues[0].check_type,
            Some(CheckType::ExpectedVariableMissing)
        );
        assert_eq!(report.issues[0].severity, Severity::Warning);
    }

    #[test]
    fn test_iso8601_date_validation() {
        // Test valid dates pass
        assert!(ISO8601_DATE_REGEX.is_match("2024"));
        assert!(ISO8601_DATE_REGEX.is_match("2024-01"));
        assert!(ISO8601_DATE_REGEX.is_match("2024-01-15"));
        assert!(ISO8601_DATE_REGEX.is_match("2024-01-15T10:30"));
        assert!(ISO8601_DATE_REGEX.is_match("2024-01-15T10:30:45"));

        // Test invalid dates fail
        assert!(!ISO8601_DATE_REGEX.is_match("01/15/2024"));
        assert!(!ISO8601_DATE_REGEX.is_match("15-01-2024"));
        assert!(!ISO8601_DATE_REGEX.is_match("2024/01/15"));
    }

    #[test]
    fn test_date_variable_detection() {
        assert!(is_date_variable("AESTDTC"));
        assert!(is_date_variable("AEENDTC"));
        assert!(is_date_variable("DMDTC"));
        assert!(!is_date_variable("AETERM"));
        assert!(!is_date_variable("USUBJID"));
    }
}
