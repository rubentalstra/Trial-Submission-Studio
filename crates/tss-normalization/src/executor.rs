//! DataFrame normalization execution.
//!
//! Executes normalization pipelines on source DataFrames to produce
//! SDTM-compliant output DataFrames.

use polars::prelude::*;
use std::collections::BTreeMap;
use tss_common::any_to_string;

use crate::error::NormalizationError;
use crate::normalization::{
    calculate_study_day_from_strings, format_iso8601_duration, normalize_ct_value,
    normalize_without_codelist, parse_numeric, transform_to_iso8601,
};
use crate::types::{NormalizationPipeline, NormalizationContext, NormalizationRule, NormalizationType};

/// Execute transformation pipeline on source DataFrame.
///
/// Returns a new DataFrame with only SDTM-compliant columns.
/// The output DataFrame contains columns in the order defined by the pipeline rules.
/// Variables marked as omitted in the context are excluded from output.
pub fn execute_normalization(
    source_df: &DataFrame,
    pipeline: &NormalizationPipeline,
    context: &NormalizationContext,
) -> Result<DataFrame, NormalizationError> {
    let mut columns: Vec<Column> = Vec::with_capacity(pipeline.rules.len());
    let row_count = source_df.height();

    for rule in pipeline.rules_ordered() {
        // Skip omitted variables
        if context.is_omitted(&rule.target_variable) {
            tracing::debug!(
                target = %rule.target_variable,
                "Skipping omitted variable"
            );
            continue;
        }

        let series = execute_rule(source_df, rule, context, row_count)?;
        columns.push(series.into_column());
    }

    DataFrame::new(columns).map_err(NormalizationError::PolarsError)
}

/// Execute a single transformation rule.
fn execute_rule(
    source_df: &DataFrame,
    rule: &NormalizationRule,
    context: &NormalizationContext,
    row_count: usize,
) -> Result<Series, NormalizationError> {
    let target_name = &rule.target_variable;

    // Get source column from rule or context mappings
    let source_col = rule
        .source_column
        .as_deref()
        .or_else(|| context.get_source_column(target_name));

    match &rule.transform_type {
        NormalizationType::Constant => execute_constant(target_name, context, row_count),
        NormalizationType::UsubjidPrefix => execute_usubjid(source_df, target_name, context, row_count),
        NormalizationType::SequenceNumber => {
            execute_sequence(source_df, target_name, context, row_count)
        }
        NormalizationType::Iso8601DateTime => {
            execute_datetime(source_df, target_name, source_col, row_count)
        }
        NormalizationType::Iso8601Date => execute_date(source_df, target_name, source_col, row_count),
        NormalizationType::Iso8601Duration => {
            execute_duration(source_df, target_name, source_col, row_count)
        }
        NormalizationType::StudyDay { reference_dtc } => {
            execute_study_day(source_df, target_name, reference_dtc, context, row_count)
        }
        NormalizationType::CtNormalization { codelist_code } => execute_ct_normalization(
            source_df,
            target_name,
            codelist_code,
            source_col,
            context,
            row_count,
        ),
        NormalizationType::NumericConversion => {
            execute_numeric(source_df, target_name, source_col, row_count)
        }
        NormalizationType::CopyDirect => execute_copy(source_df, target_name, source_col, row_count),
    }
}

/// Execute constant transformation (STUDYID, DOMAIN).
fn execute_constant(
    target_name: &str,
    context: &NormalizationContext,
    row_count: usize,
) -> Result<Series, NormalizationError> {
    let value = match target_name {
        "STUDYID" => &context.study_id,
        "DOMAIN" => &context.domain_code,
        _ => "",
    };

    Ok(Series::new(target_name.into(), vec![value; row_count]))
}

/// Execute USUBJID derivation (STUDYID-SUBJID pattern).
fn execute_usubjid(
    df: &DataFrame,
    target_name: &str,
    context: &NormalizationContext,
    row_count: usize,
) -> Result<Series, NormalizationError> {
    // Try to find SUBJID or USUBJID mapping
    let source_col = context
        .get_source_column("SUBJID")
        .or_else(|| context.get_source_column("USUBJID"));

    let Some(source_col) = source_col else {
        // No mapping - return empty strings
        tracing::warn!(
            target = %target_name,
            "No SUBJID mapping found for USUBJID derivation"
        );
        return Ok(Series::new(target_name.into(), vec![""; row_count]));
    };

    let source_series = df
        .column(source_col)
        .map_err(|_| NormalizationError::ColumnNotFound(source_col.to_string()))?;

    let mut values = Vec::with_capacity(row_count);

    for idx in 0..row_count {
        let subjid = any_to_string(source_series.get(idx)?);
        if subjid.trim().is_empty() {
            values.push(String::new());
        } else {
            values.push(format!("{}-{}", context.study_id, subjid.trim()));
        }
    }

    Ok(Series::new(target_name.into(), values))
}

/// Execute sequence number generation (unique per USUBJID).
fn execute_sequence(
    df: &DataFrame,
    target_name: &str,
    context: &NormalizationContext,
    row_count: usize,
) -> Result<Series, NormalizationError> {
    // Get USUBJID column for grouping
    let usubjid_col = context
        .get_source_column("USUBJID")
        .or_else(|| context.get_source_column("SUBJID"));

    let Some(source_col) = usubjid_col else {
        // No grouping column - generate simple 1..N sequence
        tracing::warn!(
            target = %target_name,
            "No USUBJID mapping found for sequence grouping, using simple 1..N"
        );
        let seq: Vec<i64> = (1..=row_count as i64).collect();
        return Ok(Series::new(target_name.into(), seq));
    };

    let source_series = df
        .column(source_col)
        .map_err(|_| NormalizationError::ColumnNotFound(source_col.to_string()))?;

    let mut counters: BTreeMap<String, i64> = BTreeMap::new();
    let mut values = Vec::with_capacity(row_count);

    for idx in 0..row_count {
        let usubjid = any_to_string(source_series.get(idx)?);
        let key = usubjid.trim().to_string();
        let count = counters.entry(key).or_insert(0);
        *count += 1;
        values.push(*count);
    }

    Ok(Series::new(target_name.into(), values))
}

/// Execute ISO 8601 datetime transformation.
fn execute_datetime(
    df: &DataFrame,
    target_name: &str,
    source_col: Option<&str>,
    row_count: usize,
) -> Result<Series, NormalizationError> {
    let Some(source_col) = source_col else {
        return Ok(Series::new(target_name.into(), vec![""; row_count]));
    };

    let source_series = df
        .column(source_col)
        .map_err(|_| NormalizationError::ColumnNotFound(source_col.to_string()))?;

    let mut values = Vec::with_capacity(row_count);

    for idx in 0..row_count {
        let raw = any_to_string(source_series.get(idx)?);
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            values.push(String::new());
        } else {
            // transform_to_iso8601 preserves partial precision and original on failure
            values.push(transform_to_iso8601(trimmed));
        }
    }

    Ok(Series::new(target_name.into(), values))
}

/// Execute ISO 8601 date transformation.
fn execute_date(
    df: &DataFrame,
    target_name: &str,
    source_col: Option<&str>,
    row_count: usize,
) -> Result<Series, NormalizationError> {
    // Same as datetime but we might want to truncate time if present
    execute_datetime(df, target_name, source_col, row_count)
}

/// Execute ISO 8601 duration transformation.
fn execute_duration(
    df: &DataFrame,
    target_name: &str,
    source_col: Option<&str>,
    row_count: usize,
) -> Result<Series, NormalizationError> {
    let Some(source_col) = source_col else {
        return Ok(Series::new(target_name.into(), vec![""; row_count]));
    };

    let source_series = df
        .column(source_col)
        .map_err(|_| NormalizationError::ColumnNotFound(source_col.to_string()))?;

    let mut values = Vec::with_capacity(row_count);

    for idx in 0..row_count {
        let raw = any_to_string(source_series.get(idx)?);
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            values.push(String::new());
        } else {
            // Try to format as ISO 8601 duration, preserve original on failure
            let formatted = format_iso8601_duration(trimmed).unwrap_or_else(|| {
                tracing::warn!(
                    target = %target_name,
                    value = %trimmed,
                    "Failed to parse duration, preserving original"
                );
                trimmed.to_string()
            });
            values.push(formatted);
        }
    }

    Ok(Series::new(target_name.into(), values))
}

/// Execute study day calculation.
fn execute_study_day(
    df: &DataFrame,
    target_name: &str,
    reference_dtc: &str,
    context: &NormalizationContext,
    row_count: usize,
) -> Result<Series, NormalizationError> {
    // Get reference date from context (RFSTDTC from DM)
    let Some(ref_date) = context.reference_date else {
        tracing::warn!(
            target = %target_name,
            "No reference date (RFSTDTC) available for study day calculation"
        );
        let nulls: Vec<Option<i32>> = vec![None; row_count];
        return Ok(Series::new(target_name.into(), nulls));
    };

    // Get the source DTC column from mappings
    let source_col = context.get_source_column(reference_dtc);

    let Some(source_col) = source_col else {
        tracing::warn!(
            target = %target_name,
            reference_dtc = %reference_dtc,
            "No mapping found for reference DTC column"
        );
        let nulls: Vec<Option<i32>> = vec![None; row_count];
        return Ok(Series::new(target_name.into(), nulls));
    };

    let source_series = df
        .column(source_col)
        .map_err(|_| NormalizationError::ColumnNotFound(source_col.to_string()))?;

    let ref_date_str = ref_date.format("%Y-%m-%d").to_string();
    let mut values: Vec<Option<i32>> = Vec::with_capacity(row_count);

    for idx in 0..row_count {
        let event_date_str = any_to_string(source_series.get(idx)?);
        let trimmed = event_date_str.trim();

        if trimmed.is_empty() {
            values.push(None);
        } else {
            // Calculate study day
            let study_day = calculate_study_day_from_strings(trimmed, &ref_date_str);
            values.push(study_day);
        }
    }

    Ok(Series::new(target_name.into(), values))
}

/// Execute CT normalization.
fn execute_ct_normalization(
    df: &DataFrame,
    target_name: &str,
    codelist_code: &str,
    source_col: Option<&str>,
    context: &NormalizationContext,
    row_count: usize,
) -> Result<Series, NormalizationError> {
    let Some(source_col) = source_col else {
        return Ok(Series::new(target_name.into(), vec![""; row_count]));
    };

    let source_series = df
        .column(source_col)
        .map_err(|_| NormalizationError::ColumnNotFound(source_col.to_string()))?;

    // Try to get the codelist from registry
    let codelist = context
        .ct_registry
        .as_ref()
        .and_then(|registry| registry.resolve(codelist_code, None))
        .map(|resolved| resolved.codelist.clone());

    let mut values = Vec::with_capacity(row_count);

    for idx in 0..row_count {
        let raw = any_to_string(source_series.get(idx)?);
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            values.push(String::new());
        } else if let Some(ref cl) = codelist {
            let result = normalize_ct_value(trimmed, cl);
            values.push(result.value);
        } else {
            // No codelist available - use normalize_without_codelist
            let result = normalize_without_codelist(trimmed, codelist_code);
            values.push(result.value);
        }
    }

    Ok(Series::new(target_name.into(), values))
}

/// Execute numeric conversion.
fn execute_numeric(
    df: &DataFrame,
    target_name: &str,
    source_col: Option<&str>,
    row_count: usize,
) -> Result<Series, NormalizationError> {
    let Some(source_col) = source_col else {
        let nulls: Vec<Option<f64>> = vec![None; row_count];
        return Ok(Series::new(target_name.into(), nulls));
    };

    let source_series = df
        .column(source_col)
        .map_err(|_| NormalizationError::ColumnNotFound(source_col.to_string()))?;

    let mut values: Vec<Option<f64>> = Vec::with_capacity(row_count);

    for idx in 0..row_count {
        let raw = any_to_string(source_series.get(idx)?);
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            values.push(None);
        } else {
            match parse_numeric(trimmed) {
                Some(num) => values.push(Some(num)),
                None => {
                    tracing::warn!(
                        target = %target_name,
                        value = %trimmed,
                        "Failed to parse numeric, setting to null"
                    );
                    values.push(None);
                }
            }
        }
    }

    Ok(Series::new(target_name.into(), values))
}

/// Execute direct copy (passthrough).
fn execute_copy(
    df: &DataFrame,
    target_name: &str,
    source_col: Option<&str>,
    row_count: usize,
) -> Result<Series, NormalizationError> {
    let Some(source_col) = source_col else {
        return Ok(Series::new(target_name.into(), vec![""; row_count]));
    };

    let source_series = df
        .column(source_col)
        .map_err(|_| NormalizationError::ColumnNotFound(source_col.to_string()))?;

    let mut values = Vec::with_capacity(row_count);

    for idx in 0..row_count {
        let raw = any_to_string(source_series.get(idx)?);
        values.push(raw);
    }

    Ok(Series::new(target_name.into(), values))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::infer_normalization_rules;
    use tss_model::{CoreDesignation, Domain, Variable, VariableRole, VariableType};

    fn create_test_domain() -> Domain {
        Domain {
            name: "AE".to_string(),
            label: Some("Adverse Events".to_string()),
            class: Some(tss_model::DatasetClass::Events),
            structure: None,
            dataset_name: None,
            variables: vec![
                Variable {
                    name: "STUDYID".to_string(),
                    label: Some("Study Identifier".to_string()),
                    data_type: VariableType::Char,
                    length: None,
                    role: Some(VariableRole::Identifier),
                    core: Some(CoreDesignation::Required),
                    codelist_code: None,
                    described_value_domain: None,
                    order: Some(1),
                },
                Variable {
                    name: "DOMAIN".to_string(),
                    label: Some("Domain Abbreviation".to_string()),
                    data_type: VariableType::Char,
                    length: None,
                    role: Some(VariableRole::Identifier),
                    core: Some(CoreDesignation::Required),
                    codelist_code: None,
                    described_value_domain: None,
                    order: Some(2),
                },
                Variable {
                    name: "USUBJID".to_string(),
                    label: Some("Unique Subject Identifier".to_string()),
                    data_type: VariableType::Char,
                    length: None,
                    role: Some(VariableRole::Identifier),
                    core: Some(CoreDesignation::Required),
                    codelist_code: None,
                    described_value_domain: None,
                    order: Some(3),
                },
                Variable {
                    name: "AESEQ".to_string(),
                    label: Some("Sequence Number".to_string()),
                    data_type: VariableType::Num,
                    length: None,
                    role: Some(VariableRole::Identifier),
                    core: Some(CoreDesignation::Required),
                    codelist_code: None,
                    described_value_domain: None,
                    order: Some(4),
                },
            ],
        }
    }

    #[test]
    fn test_execute_constant() {
        let context = NormalizationContext::new("CDISC01", "AE");
        let result = execute_constant("STUDYID", &context, 3).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result.get(0).unwrap(), AnyValue::String("CDISC01"));
        assert_eq!(result.get(1).unwrap(), AnyValue::String("CDISC01"));
        assert_eq!(result.get(2).unwrap(), AnyValue::String("CDISC01"));
    }

    #[test]
    fn test_execute_normalization() {
        let domain = create_test_domain();
        let pipeline = infer_normalization_rules(&domain);

        // Create source DataFrame
        let df = df! {
            "SUBJECT" => &["001", "001", "002"],
        }
        .unwrap();

        let mut mappings = BTreeMap::new();
        mappings.insert("SUBJID".to_string(), "SUBJECT".to_string());

        let context = NormalizationContext::new("CDISC01", "AE").with_mappings(mappings);

        let result = execute_normalization(&df, &pipeline, &context).unwrap();

        // Check structure
        assert_eq!(result.width(), 4); // STUDYID, DOMAIN, USUBJID, AESEQ
        assert_eq!(result.height(), 3);

        // Check column names
        let names: Vec<&str> = result
            .get_column_names()
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert!(names.contains(&"STUDYID"));
        assert!(names.contains(&"DOMAIN"));
        assert!(names.contains(&"USUBJID"));
        assert!(names.contains(&"AESEQ"));
    }
}
