use std::collections::{HashMap, HashSet};

use anyhow::Result;
use chrono::NaiveDate;
use polars::prelude::{AnyValue, BooleanChunked, DataFrame, NamedFrom, NewChunkedArray, Series};
use tracing::warn;

use sdtm_model::Domain;

// Import and re-export Polars utilities for domain processors
use crate::datetime::DatePairOrder;
use crate::datetime::parse_date;
use crate::datetime::validate_date_pair;
use crate::domain_utils::column_name;
use crate::pipeline_context::PipelineContext;
pub(super) use sdtm_ingest::{any_to_f64, any_to_i64, any_to_string, parse_f64};

// Re-export shared functions for domain processors
pub(super) use crate::ct_utils::normalize_ct_value;
pub(super) use crate::ct_utils::preferred_term_for;
pub(super) use crate::ct_utils::resolve_ct_value;
pub(super) use crate::datetime::normalize_iso8601;

pub(super) fn col(domain: &Domain, name: &str) -> Option<String> {
    column_name(domain, name)
}
pub(super) fn has_column(df: &DataFrame, name: &str) -> bool {
    df.column(name).is_ok()
}

pub(super) fn string_column(df: &DataFrame, name: &str) -> Result<Vec<String>> {
    let series = df.column(name)?;
    let mut values = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
        values.push(value.trim().to_string());
    }
    Ok(values)
}

fn strip_quotes(value: &str) -> String {
    let trimmed = value.trim();
    if !trimmed.contains('"') {
        return trimmed.to_string();
    }
    trimmed.chars().filter(|ch| *ch != '"').collect()
}

pub(super) fn numeric_column_f64(df: &DataFrame, name: &str) -> Result<Vec<Option<f64>>> {
    let series = df.column(name)?;
    let mut values = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        values.push(any_to_f64(value));
    }
    Ok(values)
}

pub(super) fn numeric_column_i64(df: &DataFrame, name: &str) -> Result<Vec<Option<i64>>> {
    let series = df.column(name)?;
    let mut values = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let value = series.get(idx).unwrap_or(AnyValue::Null);
        values.push(any_to_i64(value));
    }
    Ok(values)
}

pub(super) fn set_string_column(df: &mut DataFrame, name: &str, values: Vec<String>) -> Result<()> {
    let series = Series::new(name.into(), values);
    df.with_column(series)?;
    Ok(())
}

pub(super) fn set_f64_column(
    df: &mut DataFrame,
    name: &str,
    values: Vec<Option<f64>>,
) -> Result<()> {
    let series = Series::new(name.into(), values);
    df.with_column(series)?;
    Ok(())
}

pub(super) fn set_i64_column(
    df: &mut DataFrame,
    name: &str,
    values: Vec<Option<i64>>,
) -> Result<()> {
    let series = Series::new(name.into(), values);
    df.with_column(series)?;
    Ok(())
}

pub(super) fn filter_rows(df: &mut DataFrame, keep: &[bool]) -> Result<()> {
    let mask = BooleanChunked::from_slice("keep".into(), keep);
    *df = df.filter(&mask)?;
    Ok(())
}

pub(super) fn deduplicate<S: AsRef<str>>(df: &mut DataFrame, keys: &[S]) -> Result<()> {
    if keys.is_empty() || df.height() == 0 {
        return Ok(());
    }
    let mut key_columns = Vec::with_capacity(keys.len());
    for key in keys {
        let key = key.as_ref();
        if !has_column(df, key) {
            continue;
        }
        key_columns.push(string_column(df, key)?);
    }
    if key_columns.is_empty() {
        return Ok(());
    }
    let mut seen: HashSet<String> = HashSet::new();
    let mut keep = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let mut key = String::new();
        for col_vals in &key_columns {
            key.push_str(&col_vals[idx]);
            key.push('|');
        }
        if seen.insert(key) {
            keep.push(true);
        } else {
            keep.push(false);
        }
    }
    filter_rows(df, &keep)?;
    Ok(())
}

pub(super) fn apply_map_upper(
    df: &mut DataFrame,
    column: Option<&str>,
    mapping: &HashMap<String, String>,
) -> Result<()> {
    let Some(column) = column else {
        return Ok(());
    };
    if !has_column(df, column) {
        return Ok(());
    }
    let values = string_column(df, column)?
        .into_iter()
        .map(|value| {
            let upper = value.to_uppercase();
            mapping.get(&upper).cloned().unwrap_or(upper)
        })
        .collect();
    set_string_column(df, column, values)?;
    Ok(())
}

pub(super) fn map_values<const N: usize>(pairs: [(&str, &str); N]) -> HashMap<String, String> {
    let mut map = HashMap::with_capacity(N);
    for (key, value) in pairs {
        map.insert(key.to_string(), value.to_string());
    }
    map
}

// CT functions are provided by re-exports from ct_utils above

/// Drop placeholder/header rows that have missing or invalid USUBJID values.
///
/// # SDTMIG Reference (Section 4.1.2)
///
/// USUBJID is a required identifier for all General Observation class records.
/// It must be "a unique identifier for each subject in the study" and is
/// "a concatenation of STUDYID and a subject identifier unique within that study."
///
/// This function:
/// 1. First attempts to derive USUBJID from STUDYID + SUBJID if USUBJID is missing
///    (SDTMIG-approved derivation per Section 4.1.2)
/// 2. Drops rows that still have invalid USUBJID values (placeholder/header rows)
/// 3. Logs a warning when rows are dropped so the user is aware
///
/// Placeholder values that are dropped: empty string, "NaN", "<NA>", "NONE", "NULL"
///
/// # Arguments
///
/// * `domain` - Domain metadata
/// * `df` - DataFrame to process (modified in place)
/// * `context` - Processing context
pub(super) fn drop_placeholder_rows(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    let Some(usubjid_col) = col(domain, "USUBJID") else {
        return Ok(());
    };
    if !has_column(df, &usubjid_col) {
        return Ok(());
    }
    let mut usubjid_vals = string_column(df, &usubjid_col)?;
    for value in &mut usubjid_vals {
        *value = strip_quotes(value);
    }
    let mut missing = vec![false; df.height()];
    for idx in 0..df.height() {
        missing[idx] = is_missing_usubjid(&usubjid_vals[idx]);
    }

    if missing.iter().any(|value| *value) {
        if let Some(subjid_col) = col(domain, "SUBJID")
            && has_column(df, &subjid_col)
        {
            let subjid_vals = string_column(df, &subjid_col)?;
            let studyid_vals = col(domain, "STUDYID")
                .filter(|name| has_column(df, name))
                .and_then(|name| string_column(df, &name).ok())
                .unwrap_or_else(|| vec![String::new(); df.height()]);
            for idx in 0..df.height() {
                if !missing[idx] {
                    continue;
                }
                let subjid = strip_quotes(&subjid_vals[idx]);
                let subjid = subjid.trim();
                let placeholder = matches!(
                    subjid.to_uppercase().as_str(),
                    "SUBJID" | "SUBJECTID" | "SUBJECT ID"
                );
                if subjid.is_empty() || placeholder {
                    continue;
                }
                let studyid = strip_quotes(studyid_vals[idx].trim());
                if studyid.is_empty() {
                    usubjid_vals[idx] = subjid.to_string();
                } else {
                    usubjid_vals[idx] = format!("{}-{}", studyid, subjid);
                }
            }
        }
        for idx in 0..df.height() {
            missing[idx] = is_missing_usubjid(&usubjid_vals[idx]);
        }
    }

    if missing.iter().any(|value| *value) {
        let drop_count = missing.iter().filter(|v| **v).count();
        let keep = missing.iter().map(|value| !*value).collect::<Vec<_>>();
        set_string_column(df, &usubjid_col, usubjid_vals)?;

        // Log dropped rows - these are placeholder/header rows with invalid USUBJID
        // SDTMIG 4.1.2: USUBJID is required for all General Observation records
        warn!(
            domain_code = %domain.code,
            dropped_count = drop_count,
            "Dropped rows with missing/invalid USUBJID (placeholder/header rows)"
        );

        filter_rows(df, &keep)?;
    } else {
        set_string_column(df, &usubjid_col, usubjid_vals.clone())?;
    }

    let mut study_id = String::new();
    if let Some(studyid_col) = col(domain, "STUDYID")
        && has_column(df, &studyid_col)
    {
        let study_vals = string_column(df, &studyid_col)?;
        if let Some(found) = study_vals.iter().find(|value| !value.is_empty()) {
            study_id = strip_quotes(found);
        }
    }
    if study_id.is_empty() {
        study_id = context.study_id.to_string();
    }
    if !study_id.is_empty() {
        let prefix = format!("{study_id}-");
        let mut updated = string_column(df, &usubjid_col)?;
        for value in &mut updated {
            if !value.is_empty() && !value.starts_with(&prefix) {
                *value = format!("{prefix}{value}");
            }
        }
        set_string_column(df, &usubjid_col, updated)?;
    }
    Ok(())
}

fn is_missing_usubjid(value: &str) -> bool {
    matches!(
        value.trim().to_uppercase().as_str(),
        "" | "NAN" | "<NA>" | "NONE" | "NULL"
    )
}

pub(super) fn ensure_date_pair_order(
    df: &mut DataFrame,
    start_col: &str,
    end_col: Option<&str>,
) -> Result<()> {
    if !has_column(df, start_col) {
        return Ok(());
    }

    // Normalize start dates (trim whitespace only)
    let start_vals = string_column(df, start_col)?
        .into_iter()
        .map(|value| normalize_iso8601(&value))
        .collect::<Vec<_>>();
    set_string_column(df, start_col, start_vals.clone())?;

    if let Some(end_col) = end_col
        && has_column(df, end_col)
    {
        // Normalize end dates (trim whitespace only)
        let end_vals = string_column(df, end_col)?
            .into_iter()
            .map(|value| normalize_iso8601(&value))
            .collect::<Vec<_>>();
        set_string_column(df, end_col, end_vals.clone())?;

        // Validate date pairs without mutating
        // Per SDTMIG v3.4, we do not auto-correct date order issues
        // Instead, validation errors should be reported upstream
        let mut invalid_count = 0;
        for idx in 0..df.height() {
            let result = validate_date_pair(&start_vals[idx], &end_vals[idx]);
            if matches!(result, DatePairOrder::EndBeforeStart) {
                invalid_count += 1;
                // Log warning for visibility but do not mutate
                tracing::warn!(
                    row = idx,
                    start_col,
                    end_col,
                    start_value = %start_vals[idx],
                    end_value = %end_vals[idx],
                    "Date pair validation: end date precedes start date"
                );
            }
        }

        if invalid_count > 0 {
            tracing::warn!(
                start_col,
                end_col,
                invalid_count,
                "Date pair validation complete: {} records have end date before start date",
                invalid_count
            );
        }
    }
    Ok(())
}

/// Compute study day (--DY) per SDTMIG v3.4 Section 4.4.4.
///
/// # SDTMIG v3.4 Reference
///
/// Per Section 4.4.4 "Use of the Study Day Variables":
///
/// - Study day is computed relative to RFSTDTC (reference start date from DM)
/// - RFSTDTC is designated as study day 1
/// - Days after RFSTDTC are incremented by 1 for each subsequent date
/// - Days before RFSTDTC are decremented by 1 (no day 0)
///
/// ## Formula
///
/// - `--DY = (date portion of --DTC) - (date portion of RFSTDTC) + 1` if --DTC >= RFSTDTC
/// - `--DY = (date portion of --DTC) - (date portion of RFSTDTC)` if --DTC < RFSTDTC
///
/// ## Requirements
///
/// - **Complete dates required**: Both observation date and reference date must
///   have complete date components (year, month, day). Partial dates cannot be
///   used for study day calculation.
/// - **Result type**: All study day values are integers
/// - **Not for calculations**: Study day is not suited for duration calculations
///   due to the absence of day 0. Use raw date values instead.
///
/// # Arguments
///
/// * `domain` - The domain metadata
/// * `df` - The DataFrame to update
/// * `dtc_col` - The date/time column (--DTC, --STDTC, or --ENDTC)
/// * `dy_col` - The study day column to populate (--DY, --STDY, or --ENDY)
/// * `context` - Processing context with reference starts
/// * `reference_col` - Fallback reference column name (typically "RFSTDTC")
pub(super) fn compute_study_day(
    domain: &Domain,
    df: &mut DataFrame,
    dtc_col: &str,
    dy_col: &str,
    context: &PipelineContext,
    reference_col: &str,
) -> Result<()> {
    if !has_column(df, dtc_col) {
        return Ok(());
    }

    let dtc_vals = string_column(df, dtc_col)?;

    // Build baseline (reference) dates from context or column
    let mut baseline_vals: Vec<Option<NaiveDate>> = vec![None; df.height()];
    if !context.reference_starts.is_empty()
        && let Some(usubjid_col) = col(domain, "USUBJID")
        && has_column(df, &usubjid_col)
    {
        let usub_vals = string_column(df, &usubjid_col)?;
        for idx in 0..df.height() {
            if let Some(start) = context.reference_starts.get(&usub_vals[idx]) {
                // parse_date returns None for partial dates
                baseline_vals[idx] = parse_date(start);
            }
        }
    }

    // Fallback to reference column if no context reference starts found
    if baseline_vals.iter().all(|value| value.is_none()) && has_column(df, reference_col) {
        let ref_vals = string_column(df, reference_col)?;
        for idx in 0..df.height() {
            baseline_vals[idx] = parse_date(&ref_vals[idx]);
        }
    }

    // No reference dates available at all
    if baseline_vals.iter().all(|value| value.is_none()) {
        tracing::debug!(
            domain = domain.code.as_str(),
            dtc_col,
            dy_col,
            "Study day derivation skipped: no reference dates available"
        );
        return Ok(());
    }

    // Compute study day for each record
    let mut dy_vals: Vec<Option<f64>> = Vec::with_capacity(df.height());
    let mut derived_count = 0usize;
    let mut partial_date_count = 0usize;
    let mut missing_reference_count = 0usize;

    for idx in 0..df.height() {
        // parse_date returns None for empty or partial dates
        // Per SDTMIG 4.4.4: requires complete dates for study day calculation
        let obs_date = parse_date(&dtc_vals[idx]);
        let baseline = baseline_vals[idx];

        match (obs_date, baseline) {
            (Some(obs), Some(base)) => {
                // Both dates complete - compute study day
                let delta = obs.signed_duration_since(base).num_days();
                // Per SDTMIG: no day 0
                let adjusted = if delta >= 0 { delta + 1 } else { delta };
                dy_vals.push(Some(adjusted as f64));
                derived_count += 1;
            }
            (None, Some(_)) => {
                // Observation date is missing or partial
                dy_vals.push(None);
                if !dtc_vals[idx].trim().is_empty() {
                    partial_date_count += 1;
                }
            }
            (Some(_), None) => {
                // Reference date is missing - cannot compute
                dy_vals.push(None);
                missing_reference_count += 1;
            }
            (None, None) => {
                // Both missing
                dy_vals.push(None);
            }
        }
    }

    set_f64_column(df, dy_col, dy_vals)?;

    // Log derivation summary
    if derived_count > 0 {
        tracing::debug!(
            domain = domain.code.as_str(),
            dtc_col,
            dy_col,
            derived_count,
            "Study day values derived"
        );
    }

    // Warn about partial dates preventing study day calculation
    if partial_date_count > 0 {
        tracing::warn!(
            domain = domain.code.as_str(),
            dtc_col,
            dy_col,
            partial_date_count,
            "Study day not computed for {} records with partial/incomplete dates",
            partial_date_count
        );
    }

    if missing_reference_count > 0 {
        tracing::warn!(
            domain = domain.code.as_str(),
            dtc_col,
            dy_col,
            missing_reference_count,
            "Study day not computed for {} records with missing reference dates",
            missing_reference_count
        );
    }

    Ok(())
}
