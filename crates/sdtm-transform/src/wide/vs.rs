//! VS (Vital Signs) domain wide format processing.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use polars::prelude::DataFrame;

use sdtm_ingest::CsvTable;
use sdtm_model::{Domain, MappingConfig};

use super::types::{VsWideGroup, VsWideShared};
use super::utils::{
    base_row_values, build_wide_base_mapping, build_wide_data, mapping_used_sources, push_row,
};
use crate::data_utils::sanitize_test_code;
use crate::frame::DomainFrame;

/// Build VS domain frame from wide format data.
pub fn build_vs_wide_frame(
    table: &CsvTable,
    domain: &Domain,
    study_id: &str,
) -> Result<Option<(MappingConfig, DomainFrame, BTreeSet<String>)>> {
    let (groups, shared, wide_columns) =
        detect_vs_wide_groups(&table.headers, table.labels.as_deref());
    if groups.is_empty() {
        return Ok(None);
    }
    let (mapping_config, base_frame) =
        build_wide_base_mapping(table, domain, study_id, &wide_columns)?;
    let date_idx = find_vs_date_column(&table.headers);
    let time_idx = find_vs_time_column(&table.headers);
    let (expanded, used_wide) = expand_vs_wide(
        table,
        &base_frame.data,
        domain,
        &groups,
        &shared,
        date_idx,
        time_idx,
    )?;
    let mut used = mapping_used_sources(&mapping_config);
    used.extend(used_wide);
    Ok(Some((
        mapping_config,
        DomainFrame::new(domain.code.clone(), expanded),
        used,
    )))
}

/// Detect VS wide format column groups.
fn detect_vs_wide_groups(
    headers: &[String],
    labels: Option<&[String]>,
) -> (
    BTreeMap<String, VsWideGroup>,
    VsWideShared,
    BTreeSet<String>,
) {
    let mut groups: BTreeMap<String, VsWideGroup> = BTreeMap::new();
    let mut wide_columns = BTreeSet::new();
    let mut shared = VsWideShared::default();

    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();

        if let Some(rest) = upper.strip_prefix("ORRES_") {
            let key = rest.to_string();
            let entry = groups.entry(key.clone()).or_insert_with(|| VsWideGroup {
                key,
                ..VsWideGroup::default()
            });
            entry.orres_col = Some(idx);
            if entry.label.is_none()
                && let Some(labels) = labels
                && let Some(label) = labels.get(idx)
            {
                let trimmed = label.trim();
                if !trimmed.is_empty() {
                    entry.label = Some(trimmed.to_string());
                }
            }
            wide_columns.insert(upper);
            continue;
        }

        if let Some(rest) = upper.strip_prefix("ORRESU_") {
            let mut key = rest.to_string();
            if key.len() > 2 && key.ends_with("CD") {
                key.truncate(key.len() - 2);
                let entry = groups.entry(key.clone()).or_insert_with(|| VsWideGroup {
                    key,
                    ..VsWideGroup::default()
                });
                entry.extra_cols.push(idx);
                wide_columns.insert(upper);
                continue;
            }
            if key == "BP" {
                shared.orresu_bp = Some(idx);
            } else {
                let entry = groups.entry(key.clone()).or_insert_with(|| VsWideGroup {
                    key,
                    ..VsWideGroup::default()
                });
                entry.orresu_col = Some(idx);
            }
            wide_columns.insert(upper);
            continue;
        }

        if let Some(rest) = upper.strip_prefix("POS_") {
            let mut key = rest.to_string();
            if key.len() > 2 && key.ends_with("CD") {
                key.truncate(key.len() - 2);
                let entry = groups.entry(key.clone()).or_insert_with(|| VsWideGroup {
                    key,
                    ..VsWideGroup::default()
                });
                entry.extra_cols.push(idx);
                wide_columns.insert(upper);
                continue;
            }
            if key == "BP" {
                shared.pos_bp = Some(idx);
            } else {
                let entry = groups.entry(key.clone()).or_insert_with(|| VsWideGroup {
                    key,
                    ..VsWideGroup::default()
                });
                entry.pos_col = Some(idx);
            }
            wide_columns.insert(upper);
            continue;
        }
    }

    (groups, shared, wide_columns)
}

/// Find VS date column in headers.
fn find_vs_date_column(headers: &[String]) -> Option<usize> {
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if (upper.ends_with("DAT") || upper.ends_with("DATE"))
            && upper.contains("VS")
            && !upper.contains("EVENT")
        {
            return Some(idx);
        }
    }
    find_generic_date_column(headers)
}

/// Find VS time column in headers.
fn find_vs_time_column(headers: &[String]) -> Option<usize> {
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if (upper.ends_with("TIM") || upper.ends_with("TIME"))
            && upper.contains("VS")
            && !upper.contains("EVENT")
        {
            return Some(idx);
        }
    }
    find_generic_time_column(headers)
}

/// Find generic date column (fallback).
fn find_generic_date_column(headers: &[String]) -> Option<usize> {
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if (upper.ends_with("DAT") || upper.ends_with("DATE")) && !upper.contains("EVENT") {
            return Some(idx);
        }
    }
    None
}

/// Find generic time column (fallback).
fn find_generic_time_column(headers: &[String]) -> Option<usize> {
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if (upper.ends_with("TIM") || upper.ends_with("TIME")) && !upper.contains("EVENT") {
            return Some(idx);
        }
    }
    None
}

/// Expand VS wide format to long format.
fn expand_vs_wide(
    table: &CsvTable,
    base_df: &DataFrame,
    domain: &Domain,
    groups: &BTreeMap<String, VsWideGroup>,
    shared: &VsWideShared,
    date_idx: Option<usize>,
    time_idx: Option<usize>,
) -> Result<(DataFrame, BTreeSet<String>)> {
    let variable_names: Vec<String> = domain
        .variables
        .iter()
        .map(|var| var.name.clone())
        .collect();
    let mut values: Vec<Vec<String>> = variable_names.iter().map(|_| Vec::new()).collect();

    let vstestcd_idx = variable_names.iter().position(|name| name == "VSTESTCD");
    let vstest_idx = variable_names.iter().position(|name| name == "VSTEST");
    let vsorres_idx = variable_names.iter().position(|name| name == "VSORRES");
    let vsorresu_idx = variable_names.iter().position(|name| name == "VSORRESU");
    let vspos_idx = variable_names.iter().position(|name| name == "VSPOS");
    let vsdtc_idx = variable_names.iter().position(|name| name == "VSDTC");

    let mut used = BTreeSet::new();

    // Track used columns
    for group in groups.values() {
        for idx in [group.orres_col, group.orresu_col, group.pos_col] {
            if let Some(idx) = idx
                && let Some(name) = table.headers.get(idx)
            {
                used.insert(name.clone());
            }
        }
        for idx in &group.extra_cols {
            if let Some(name) = table.headers.get(*idx) {
                used.insert(name.clone());
            }
        }
    }

    for idx in [shared.orresu_bp, shared.pos_bp] {
        if let Some(idx) = idx
            && let Some(name) = table.headers.get(idx)
        {
            used.insert(name.clone());
        }
    }

    if let Some(idx) = date_idx
        && let Some(name) = table.headers.get(idx)
    {
        used.insert(name.clone());
    }
    if let Some(idx) = time_idx
        && let Some(name) = table.headers.get(idx)
    {
        used.insert(name.clone());
    }

    let mut total_rows = 0usize;

    for row_idx in 0..table.rows.len() {
        let base_date_value = date_idx
            .and_then(|idx| table.rows[row_idx].get(idx))
            .cloned()
            .unwrap_or_default();
        let base_time_value = time_idx
            .and_then(|idx| table.rows[row_idx].get(idx))
            .cloned()
            .unwrap_or_default();
        let base_row = base_row_values(base_df, &variable_names, row_idx);

        for group in groups.values() {
            let orres_value = group
                .orres_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let orresu_value = group
                .orresu_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let pos_value = group
                .pos_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();

            let orresu_fallback = if group.key.ends_with("BP") || group.key.contains("BP") {
                shared
                    .orresu_bp
                    .and_then(|idx| table.rows[row_idx].get(idx))
                    .cloned()
                    .unwrap_or_default()
            } else {
                String::new()
            };

            let pos_fallback = if group.key.ends_with("BP") || group.key.contains("BP") {
                shared
                    .pos_bp
                    .and_then(|idx| table.rows[row_idx].get(idx))
                    .cloned()
                    .unwrap_or_default()
            } else {
                String::new()
            };

            if orres_value.trim().is_empty()
                && orresu_value.trim().is_empty()
                && pos_value.trim().is_empty()
            {
                continue;
            }

            total_rows += 1;
            let test_code = sanitize_test_code(&group.key);
            let test_label = group.label.clone().unwrap_or_default();
            let mut row_values = base_row.clone();

            if let Some(idx) = vstestcd_idx {
                row_values[idx] = test_code.clone();
            }
            if let Some(idx) = vstest_idx {
                if !test_label.is_empty() {
                    row_values[idx] = test_label.clone();
                } else if !test_code.is_empty() {
                    row_values[idx] = test_code.clone();
                }
            }
            if let Some(idx) = vsorres_idx {
                row_values[idx] = orres_value.clone();
            }
            if let Some(idx) = vsorresu_idx {
                if !orresu_value.trim().is_empty() {
                    row_values[idx] = orresu_value.clone();
                } else {
                    row_values[idx] = orresu_fallback.clone();
                }
            }
            if let Some(idx) = vspos_idx {
                if !pos_value.trim().is_empty() {
                    row_values[idx] = pos_value.clone();
                } else {
                    row_values[idx] = pos_fallback.clone();
                }
            }
            if let Some(idx) = vsdtc_idx
                && !base_date_value.trim().is_empty()
            {
                if !base_time_value.trim().is_empty() && !base_date_value.contains('T') {
                    row_values[idx] =
                        format!("{}T{}", base_date_value.trim(), base_time_value.trim());
                } else {
                    row_values[idx] = base_date_value.clone();
                }
            }

            push_row(&mut values, row_values);
        }
    }

    if total_rows == 0 {
        return Ok((base_df.clone(), used));
    }

    let data = build_wide_data(domain, values)?;
    Ok((data, used))
}
