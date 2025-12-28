//! IE (Inclusion/Exclusion) domain wide format processing.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use polars::prelude::DataFrame;

use sdtm_ingest::CsvTable;
use sdtm_model::{Domain, MappingConfig};

use super::types::IeWideGroup;
use super::utils::{
    base_row_values, build_wide_base_mapping, build_wide_data, mapping_used_sources, push_row,
};
use crate::data_utils::{mapping_source_for_target, sanitize_test_code, table_label};
use crate::frame::DomainFrame;

/// Build IE domain frame from wide format data.
pub fn build_ie_wide_frame(
    table: &CsvTable,
    domain: &Domain,
    study_id: &str,
) -> Result<Option<(MappingConfig, DomainFrame, BTreeSet<String>)>> {
    let (groups, wide_columns) = detect_ie_wide_groups(&table.headers);
    if groups.is_empty() {
        return Ok(None);
    }
    let (mapping_config, base_frame) =
        build_wide_base_mapping(table, domain, study_id, &wide_columns)?;
    let test_source = mapping_source_for_target(&mapping_config, "IETEST");
    let testcd_source = mapping_source_for_target(&mapping_config, "IETESTCD");
    let cat_source = mapping_source_for_target(&mapping_config, "IECAT");
    let allow_base_test = source_is_ie_test(&test_source) || source_is_ie_test(&testcd_source);
    let allow_base_cat = source_is_ie_cat(&cat_source);
    let (expanded, used_wide) = expand_ie_wide(
        table,
        &base_frame.data,
        domain,
        &groups,
        allow_base_test,
        allow_base_cat,
    )?;
    let mut used = mapping_used_sources(&mapping_config);
    used.extend(used_wide);
    Ok(Some((
        mapping_config,
        DomainFrame::new(domain.code.clone(), expanded),
        used,
    )))
}

/// Detect IE wide format column groups.
fn detect_ie_wide_groups(headers: &[String]) -> (BTreeMap<String, IeWideGroup>, BTreeSet<String>) {
    let mut groups: BTreeMap<String, IeWideGroup> = BTreeMap::new();
    let mut wide_columns = BTreeSet::new();

    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        let (category, rest) = if let Some(rest) = upper.strip_prefix("IEINTESTCD") {
            ("INCLUSION", rest)
        } else if let Some(rest) = upper.strip_prefix("IEEXTESTCD") {
            ("EXCLUSION", rest)
        } else {
            continue;
        };

        if rest.is_empty() {
            continue;
        }

        let (number, is_code) = if rest.ends_with("CD") && rest.len() > 2 {
            (&rest[..rest.len() - 2], true)
        } else {
            (rest, false)
        };

        if number.is_empty() {
            continue;
        }

        let key = format!(
            "{}{}",
            if category == "INCLUSION" { "IN" } else { "EX" },
            number
        );
        let entry = groups.entry(key).or_insert_with(|| IeWideGroup {
            category: category.to_string(),
            ..IeWideGroup::default()
        });

        if is_code {
            entry.testcd_col = Some(idx);
        } else {
            entry.test_col = Some(idx);
        }
        wide_columns.insert(upper);
    }

    (groups, wide_columns)
}

/// Check if source column is an IE test column.
fn source_is_ie_test(source: &Option<String>) -> bool {
    let Some(source) = source else {
        return false;
    };
    let upper = source.to_uppercase();
    upper.contains("IETEST") || upper.contains("IEINTEST") || upper.contains("IEEXTEST")
}

/// Check if source column is an IE category column.
fn source_is_ie_cat(source: &Option<String>) -> bool {
    let Some(source) = source else {
        return false;
    };
    source.to_uppercase().contains("IECAT")
}

/// Expand IE wide format to long format.
fn expand_ie_wide(
    table: &CsvTable,
    base_df: &DataFrame,
    domain: &Domain,
    groups: &BTreeMap<String, IeWideGroup>,
    allow_base_test: bool,
    allow_base_cat: bool,
) -> Result<(DataFrame, BTreeSet<String>)> {
    let variable_names: Vec<String> = domain
        .variables
        .iter()
        .map(|var| var.name.clone())
        .collect();
    let mut values: Vec<Vec<String>> = variable_names.iter().map(|_| Vec::new()).collect();

    let test_idx = variable_names.iter().position(|name| name == "IETEST");
    let testcd_idx = variable_names.iter().position(|name| name == "IETESTCD");
    let cat_idx = variable_names.iter().position(|name| name == "IECAT");

    let mut used = BTreeSet::new();

    // Track used columns
    for group in groups.values() {
        for idx in [group.test_col, group.testcd_col] {
            if let Some(idx) = idx
                && let Some(name) = table.headers.get(idx)
            {
                used.insert(name.clone());
            }
        }
    }

    let test_col = domain.column_name("IETEST");
    let testcd_col = domain.column_name("IETESTCD");
    let cat_col = domain.column_name("IECAT");

    let mut total_rows = 0usize;

    for row_idx in 0..table.rows.len() {
        let base_test = if allow_base_test {
            test_col
                .map(|name| crate::data_utils::column_value_string(base_df, name, row_idx))
                .unwrap_or_default()
        } else {
            String::new()
        };
        let base_testcd = if allow_base_test {
            testcd_col
                .map(|name| crate::data_utils::column_value_string(base_df, name, row_idx))
                .unwrap_or_default()
        } else {
            String::new()
        };
        let base_cat = if allow_base_cat {
            cat_col
                .map(|name| crate::data_utils::column_value_string(base_df, name, row_idx))
                .unwrap_or_default()
        } else {
            String::new()
        };

        let base_row = base_row_values(base_df, &variable_names, row_idx);
        let mut added = false;

        for group in groups.values() {
            let test_value = group
                .test_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let testcd_value = group
                .testcd_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();

            if test_value.trim().is_empty() && testcd_value.trim().is_empty() {
                continue;
            }

            let label = group
                .test_col
                .and_then(|idx| table.headers.get(idx))
                .and_then(|name| table_label(table, name))
                .unwrap_or_default();

            let mut test_label = if !test_value.trim().is_empty() {
                test_value.clone()
            } else if !label.is_empty() {
                label.clone()
            } else {
                String::new()
            };

            let mut test_code = if !testcd_value.trim().is_empty() {
                testcd_value.clone()
            } else if !test_value.trim().is_empty() {
                test_value.clone()
            } else if !label.is_empty() {
                label.clone()
            } else {
                String::new()
            };

            if test_code
                .chars()
                .next()
                .map(|ch| ch.is_ascii_digit())
                .unwrap_or(false)
            {
                test_code = format!("IE{}", test_code);
            }
            test_code = sanitize_test_code(&test_code);

            if test_label.trim().is_empty() {
                test_label = test_code.clone();
            }

            let mut row_values = base_row.clone();
            if let Some(idx) = testcd_idx
                && !test_code.trim().is_empty()
            {
                row_values[idx] = test_code;
            }
            if let Some(idx) = test_idx
                && !test_label.trim().is_empty()
            {
                row_values[idx] = test_label;
            }
            if let Some(idx) = cat_idx
                && row_values[idx].trim().is_empty()
            {
                row_values[idx] = group.category.clone();
            }

            push_row(&mut values, row_values);
            total_rows += 1;
            added = true;
        }

        if !added {
            let base_has = !base_test.trim().is_empty()
                || !base_testcd.trim().is_empty()
                || !base_cat.trim().is_empty();
            if base_has {
                push_row(&mut values, base_row);
                total_rows += 1;
            }
        }
    }

    if total_rows == 0 {
        return Ok((base_df.clone(), used));
    }

    let data = build_wide_data(domain, values)?;
    Ok((data, used))
}
