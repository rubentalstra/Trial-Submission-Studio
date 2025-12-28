use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use polars::prelude::{AnyValue, Column, DataFrame, NamedFrom, Series};
use sdtm_model::Domain;

use crate::data_utils::column_value_string;
use crate::domain_utils::{infer_seq_column, standard_columns};
use crate::frame::DomainFrame;
use sdtm_ingest::any_to_string;

pub struct SuppqualInput<'a> {
    pub parent_domain: &'a Domain,
    pub suppqual_domain: &'a Domain,
    pub source_df: &'a DataFrame,
    pub mapped_df: Option<&'a DataFrame>,
    pub used_source_columns: &'a BTreeSet<String>,
    pub study_id: &'a str,
    pub exclusion_columns: Option<&'a BTreeSet<String>>,
    pub source_labels: Option<&'a BTreeMap<String, String>>,
    pub derived_columns: Option<&'a BTreeSet<String>>,
}

/// Generate a SUPPQUAL dataset code from a parent domain code.
///
/// Per SDTMIG v3.4 Section 4.1.7 and Section 8.4.2:
/// - There should be ONE SUPP dataset per base domain, regardless of splits.
/// - For split domains (e.g., LBCH, LBHE, LBUR), all supplemental qualifiers
///   go into a single SUPPLB dataset with RDOMAIN="LB".
/// - The SUPP dataset is named after the base domain code, not split dataset names.
/// - If SUPP{domain} exceeds 8 characters, fall back to SQ{domain}.
/// - If still > 8 characters, truncate to 8.
///
/// # Arguments
/// * `parent_domain` - The base domain code (e.g., "LB", "AE", "FA")
///
/// # Returns
/// A SUPP dataset code (e.g., "SUPPLB", "SUPPAE", "SUPPFA")
///
/// # Examples
/// ```
/// use sdtm_core::suppqual::suppqual_dataset_code;
///
/// // Simple domain
/// assert_eq!(suppqual_dataset_code("LB"), "SUPPLB");
/// assert_eq!(suppqual_dataset_code("AE"), "SUPPAE");
///
/// // Long domain code falls back to SQ prefix
/// assert_eq!(suppqual_dataset_code("FAMRSK"), "SQFAMRSK");
/// ```
pub fn suppqual_dataset_code(parent_domain: &str) -> String {
    let domain = parent_domain.to_uppercase();
    let candidate = format!("SUPP{domain}");
    if candidate.len() <= 8 {
        return candidate;
    }
    let short = format!("SQ{domain}");
    if short.len() <= 8 {
        return short;
    }
    short.chars().take(8).collect()
}

fn strip_wrapping_quotes(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        return trimmed[1..trimmed.len() - 1].to_string();
    }
    trimmed.to_string()
}

fn sanitize_qnam(name: &str) -> String {
    let mut safe = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            safe.push(ch.to_ascii_uppercase());
        } else {
            safe.push('_');
        }
    }
    while safe.contains("__") {
        safe = safe.replace("__", "_");
    }
    safe = safe.trim_matches('_').to_string();
    if safe.is_empty() {
        safe = "QVAL".to_string();
    }
    if safe
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        safe = format!("Q{safe}");
    }
    safe.chars().take(8).collect()
}

fn unique_qnam(name: &str, used: &mut BTreeMap<String, String>) -> String {
    let base = sanitize_qnam(name);
    if let Some(existing) = used.get(&base) {
        if existing.eq_ignore_ascii_case(name) {
            return base;
        }
    } else {
        used.insert(base.clone(), name.to_string());
        return base;
    }
    for idx in 1..=99 {
        let suffix = format!("{idx:02}");
        let prefix_len = 8usize.saturating_sub(suffix.len()).max(1);
        let mut prefix: String = base.chars().take(prefix_len).collect();
        if prefix.is_empty() {
            prefix = "Q".to_string();
        }
        let candidate = format!("{prefix}{suffix}");
        if !used.contains_key(&candidate) {
            used.insert(candidate.clone(), name.to_string());
            return candidate;
        }
    }
    base
}

fn populated_columns(df: &DataFrame) -> BTreeSet<String> {
    let mut populated = BTreeSet::new();
    for series in df.get_columns() {
        let mut has_value = false;
        for idx in 0..df.height() {
            let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
            if !value.trim().is_empty() {
                has_value = true;
                break;
            }
        }
        if has_value {
            populated.insert(series.name().to_uppercase());
        }
    }
    populated
}

fn is_duplicate_of_mapped(name: &str, populated: &BTreeSet<String>) -> bool {
    if populated.is_empty() {
        return false;
    }
    let upper = name.to_uppercase();
    if upper.ends_with("SEQ") && populated.iter().any(|col| col.ends_with("SEQ")) {
        return true;
    }
    if upper.ends_with("CD") && upper.len() > 2 {
        let base = &upper[..upper.len() - 2];
        if populated.contains(base) {
            return true;
        }
    }
    if let Some(prefix) = upper.strip_suffix("DATE")
        && populated.contains(&format!("{prefix}DTC"))
    {
        return true;
    }
    if let Some(prefix) = upper.strip_suffix("DAT")
        && populated.contains(&format!("{prefix}DTC"))
    {
        return true;
    }
    if let Some(prefix) = upper.strip_suffix("DT")
        && populated.contains(&format!("{prefix}DTC"))
    {
        return true;
    }
    false
}

pub fn build_suppqual(input: SuppqualInput<'_>) -> Result<Option<DomainFrame>> {
    let parent_domain_code = input.parent_domain.code.to_uppercase();
    let ordered_columns: Vec<String> = input
        .suppqual_domain
        .variables
        .iter()
        .map(|variable| variable.name.clone())
        .collect();
    let core_variables: BTreeSet<String> = input
        .parent_domain
        .variables
        .iter()
        .map(|variable| variable.name.to_uppercase())
        .collect();
    let suppqual_cols = standard_columns(input.suppqual_domain);
    let parent_cols = standard_columns(input.parent_domain);
    let populated = input.mapped_df.map(populated_columns).unwrap_or_default();
    if ordered_columns.is_empty() {
        return Ok(None);
    }
    let mut extra_cols: Vec<String> = Vec::new();
    for series in input.source_df.get_columns() {
        let name = series.name().to_string();
        if input.used_source_columns.contains(&name) {
            continue;
        }
        if core_variables.contains(&name.to_uppercase()) {
            continue;
        }
        if is_duplicate_of_mapped(&name, &populated) {
            continue;
        }
        if let Some(exclusions) = input.exclusion_columns
            && exclusions.contains(&name.to_uppercase())
        {
            continue;
        }
        extra_cols.push(name);
    }

    let mut extra_upper: BTreeMap<String, String> = BTreeMap::new();
    let mut non_empty_upper = BTreeSet::new();
    for name in &extra_cols {
        extra_upper.insert(name.to_uppercase(), name.clone());
        let upper = name.to_uppercase();
        if let Ok(series) = input.source_df.column(name) {
            for idx in 0..input.source_df.height() {
                let value = any_to_string(series.get(idx).unwrap_or(AnyValue::Null));
                if !value.trim().is_empty() {
                    non_empty_upper.insert(upper.clone());
                    break;
                }
            }
        }
    }
    extra_cols.retain(|name| {
        let upper = name.to_uppercase();
        if upper.ends_with("CD") && upper.len() > 2 {
            let base = &upper[..upper.len() - 2];
            if extra_upper.contains_key(base) && non_empty_upper.contains(base) {
                return false;
            }
        }
        true
    });

    if extra_cols.is_empty() {
        return Ok(None);
    }

    let mut row_count = input.source_df.height();
    if let Some(mapped) = input.mapped_df {
        row_count = row_count.min(mapped.height());
    }

    // Per SDTMIG 8.4.1: For DM, IDVAR and IDVARVAL should be blank because
    // DM has no --SEQ (the only domain without a sequence number).
    // For all other domains, use the --SEQ column as IDVAR.
    let is_dm_domain = parent_domain_code == "DM";

    let idvar = if is_dm_domain {
        // DM domain: IDVAR should be blank per SDTMIG 8.4.1
        None
    } else {
        // Non-DM domains: try to infer the sequence column
        infer_seq_column(input.parent_domain).and_then(|seq_var| {
            if input
                .mapped_df
                .and_then(|df| df.column(&seq_var).ok())
                .is_some()
            {
                Some(seq_var)
            } else {
                None
            }
        })
    };

    let mut values: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for key in &ordered_columns {
        values.insert(key.to_string(), Vec::new());
    }

    let mut push_value = |key: Option<&str>, value: String| {
        if let Some(name) = key
            && let Some(entry) = values.get_mut(name)
        {
            entry.push(value);
        }
    };

    let mut qnam_used: BTreeMap<String, String> = BTreeMap::new();
    let mut qnam_map: BTreeMap<String, String> = BTreeMap::new();
    for col in &extra_cols {
        let qnam = unique_qnam(col, &mut qnam_used);
        qnam_map.insert(col.clone(), qnam);
    }

    let mut seen_keys: BTreeSet<String> = BTreeSet::new();
    for col in &extra_cols {
        let qnam = qnam_map
            .get(col)
            .cloned()
            .unwrap_or_else(|| sanitize_qnam(col));
        let qlabel = qlabel_for_column(col, input.source_labels);
        let qorig = qorig_for_column(col, input.derived_columns);
        for idx in 0..row_count {
            let raw_val = strip_wrapping_quotes(&column_value_string(input.source_df, col, idx));
            if raw_val.is_empty() {
                continue;
            }
            let study_value = parent_cols
                .study_id
                .as_deref()
                .map(|name| strip_wrapping_quotes(&column_value_string(input.source_df, name, idx)))
                .unwrap_or_default();
            let usubjid_value = parent_cols
                .usubjid
                .as_deref()
                .map(|name| strip_wrapping_quotes(&column_value_string(input.source_df, name, idx)))
                .unwrap_or_default();
            let mapped_usubjid = input
                .mapped_df
                .and_then(|df| {
                    parent_cols
                        .usubjid
                        .as_deref()
                        .map(|name| strip_wrapping_quotes(&column_value_string(df, name, idx)))
                })
                .unwrap_or_default();
            let final_usubjid = if !usubjid_value.is_empty() {
                usubjid_value
            } else {
                mapped_usubjid
            };

            let idvar_value = idvar.clone().unwrap_or_default();
            let idvarval = if let (Some(mapped), Some(idvar_name)) = (input.mapped_df, &idvar) {
                column_value_string(mapped, idvar_name, idx)
            } else {
                String::new()
            };

            let dedupe_key = format!(
                "{}|{}|{}|{}|{}|{}",
                study_value.trim(),
                parent_domain_code,
                final_usubjid.trim(),
                idvar_value.trim(),
                idvarval.trim(),
                qnam
            );
            if !seen_keys.insert(dedupe_key) {
                continue;
            }

            push_value(
                suppqual_cols.study_id.as_deref(),
                if !study_value.is_empty() {
                    study_value
                } else {
                    input.study_id.to_string()
                },
            );
            push_value(suppqual_cols.rdomain.as_deref(), parent_domain_code.clone());
            push_value(suppqual_cols.usubjid.as_deref(), final_usubjid);
            push_value(suppqual_cols.idvar.as_deref(), idvar_value);
            push_value(suppqual_cols.idvarval.as_deref(), idvarval);
            push_value(suppqual_cols.qnam.as_deref(), qnam.clone());
            push_value(suppqual_cols.qlabel.as_deref(), qlabel.clone());
            push_value(suppqual_cols.qval.as_deref(), raw_val);
            push_value(suppqual_cols.qorig.as_deref(), qorig.clone());
            push_value(suppqual_cols.qeval.as_deref(), String::new());
        }
    }

    let total_rows = suppqual_cols
        .qval
        .as_deref()
        .and_then(|name| values.get(name))
        .map(|vals| vals.len())
        .unwrap_or(0);
    if total_rows == 0 {
        return Ok(None);
    }

    let columns: Vec<Column> = ordered_columns
        .iter()
        .map(|name| {
            let mut vals = values.remove(name).unwrap_or_default();
            if vals.len() < total_rows {
                vals.resize(total_rows, String::new());
            }
            Series::new(name.as_str().into(), vals).into()
        })
        .collect();
    let data = DataFrame::new(columns)?;

    // Per SDTMIG 4.1.7/8.4.2: Use base domain code for SUPP naming.
    // All split datasets (e.g., LBCH, LBHE, LBUR) merge into one SUPPLB.
    Ok(Some(DomainFrame {
        domain_code: suppqual_dataset_code(&parent_domain_code),
        data,
        meta: None,
    }))
}

fn qlabel_for_column(name: &str, labels: Option<&BTreeMap<String, String>>) -> String {
    let label = labels
        .and_then(|map| map.get(&name.to_uppercase()))
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .unwrap_or_else(|| name.to_string());
    label.chars().take(40).collect()
}

fn qorig_for_column(name: &str, derived_columns: Option<&BTreeSet<String>>) -> String {
    if derived_columns
        .map(|set| set.contains(&name.to_uppercase()))
        .unwrap_or(false)
    {
        "Derived".to_string()
    } else {
        "CRF".to_string()
    }
}
