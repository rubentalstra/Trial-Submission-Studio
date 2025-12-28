use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::data_utils::column_value_string;
use crate::frame::DomainFrame;
use crate::frame_builder::build_domain_frame_from_records;

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

fn insert_if_some(record: &mut BTreeMap<String, String>, column: Option<&str>, value: String) {
    if let Some(name) = column {
        record.insert(name.to_string(), value);
    }
}

pub fn build_suppqual(input: SuppqualInput<'_>) -> Result<Option<DomainFrame>> {
    let parent_domain_code = input.parent_domain.code.to_uppercase();
    if input.suppqual_domain.variables.is_empty() {
        return Ok(None);
    }
    let core_variables: BTreeSet<String> = input
        .parent_domain
        .variables
        .iter()
        .map(|variable| variable.name.to_uppercase())
        .collect();
    let parent_study_col = input.parent_domain.column_name("STUDYID");
    let parent_usubjid_col = input.parent_domain.column_name("USUBJID");
    let supp_study_col = input.suppqual_domain.column_name("STUDYID");
    let supp_rdomain_col = input.suppqual_domain.column_name("RDOMAIN");
    let supp_usubjid_col = input.suppqual_domain.column_name("USUBJID");
    let supp_idvar_col = input.suppqual_domain.column_name("IDVAR");
    let supp_idvarval_col = input.suppqual_domain.column_name("IDVARVAL");
    let supp_qnam_col = input.suppqual_domain.column_name("QNAM");
    let supp_qlabel_col = input.suppqual_domain.column_name("QLABEL");
    let supp_qval_col = input.suppqual_domain.column_name("QVAL");
    let supp_qorig_col = input.suppqual_domain.column_name("QORIG");
    let supp_qeval_col = input.suppqual_domain.column_name("QEVAL");
    let mut extra_cols: Vec<String> = Vec::new();
    for series in input.source_df.get_columns() {
        let name = series.name().to_string();
        if input.used_source_columns.contains(&name) {
            continue;
        }
        if core_variables.contains(&name.to_uppercase()) {
            continue;
        }
        if let Some(exclusions) = input.exclusion_columns
            && exclusions.contains(&name.to_uppercase())
        {
            continue;
        }
        extra_cols.push(name);
    }

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
        input.parent_domain.infer_seq_column().and_then(|seq_var| {
            if input
                .mapped_df
                .and_then(|df| df.column(seq_var).ok())
                .is_some()
            {
                Some(seq_var)
            } else {
                None
            }
        })
    };

    let mut records: Vec<BTreeMap<String, String>> = Vec::new();
    let mut qnam_used: BTreeMap<String, String> = BTreeMap::new();
    let mut qnam_map: BTreeMap<String, String> = BTreeMap::new();
    for col in &extra_cols {
        let qnam = unique_qnam(col, &mut qnam_used);
        qnam_map.insert(col.clone(), qnam);
    }

    let study_values: Vec<String> = parent_study_col
        .map(|name| {
            (0..row_count)
                .map(|idx| strip_wrapping_quotes(&column_value_string(input.source_df, name, idx)))
                .collect()
        })
        .unwrap_or_else(|| vec![String::new(); row_count]);
    let usubjid_values: Vec<String> = parent_usubjid_col
        .map(|name| {
            (0..row_count)
                .map(|idx| strip_wrapping_quotes(&column_value_string(input.source_df, name, idx)))
                .collect()
        })
        .unwrap_or_else(|| vec![String::new(); row_count]);
    let mapped_usubjid_values: Vec<String> = input
        .mapped_df
        .and_then(|df| {
            parent_usubjid_col.map(|name| {
                (0..row_count)
                    .map(|idx| strip_wrapping_quotes(&column_value_string(df, name, idx)))
                    .collect()
            })
        })
        .unwrap_or_else(|| vec![String::new(); row_count]);
    let idvar_values: Vec<String> =
        if let (Some(mapped), Some(idvar_name)) = (input.mapped_df, &idvar) {
            (0..row_count)
                .map(|idx| column_value_string(mapped, idvar_name, idx))
                .collect()
        } else {
            vec![String::new(); row_count]
        };

    let idvar_value = idvar.map(str::to_string).unwrap_or_default();
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
            let study_value = study_values[idx].clone();
            let usubjid_value = usubjid_values[idx].clone();
            let mapped_usubjid = mapped_usubjid_values[idx].clone();
            let final_usubjid = if !usubjid_value.is_empty() {
                usubjid_value
            } else {
                mapped_usubjid
            };
            let idvarval = idvar_values[idx].clone();

            let mut record = BTreeMap::new();
            insert_if_some(
                &mut record,
                supp_study_col,
                if !study_value.is_empty() {
                    study_value
                } else {
                    input.study_id.to_string()
                },
            );
            insert_if_some(&mut record, supp_rdomain_col, parent_domain_code.clone());
            insert_if_some(&mut record, supp_usubjid_col, final_usubjid);
            insert_if_some(&mut record, supp_idvar_col, idvar_value.clone());
            insert_if_some(&mut record, supp_idvarval_col, idvarval);
            insert_if_some(&mut record, supp_qnam_col, qnam.clone());
            insert_if_some(&mut record, supp_qlabel_col, qlabel.clone());
            insert_if_some(&mut record, supp_qval_col, raw_val);
            insert_if_some(&mut record, supp_qorig_col, qorig.clone());
            insert_if_some(&mut record, supp_qeval_col, String::new());
            records.push(record);
        }
    }

    if records.is_empty() {
        return Ok(None);
    }

    let data = build_domain_frame_from_records(input.suppqual_domain, &records)?;

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
