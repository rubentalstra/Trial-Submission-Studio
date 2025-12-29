//! LB (Laboratory) domain wide format detection and expansion.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use polars::prelude::DataFrame;

use sdtm_ingest::CsvTable;
use sdtm_model::{Domain, MappingConfig};

use crate::data_utils::{sanitize_test_code, table_label};
use crate::frame::DomainFrame;

use super::types::{LbSuffixKind, LbWideGroup};
use super::utils::{
    base_row_values, build_wide_base_mapping, build_wide_data, mapping_used_sources,
    normalize_numeric, push_row,
};

/// Build LB wide format frame from CSV table.
pub fn build_lb_wide_frame(
    table: &CsvTable,
    domain: &Domain,
    study_id: &str,
) -> Result<Option<(MappingConfig, DomainFrame, BTreeSet<String>)>> {
    let (groups, wide_columns) = detect_lb_wide_groups(&table.headers);
    if groups.is_empty() {
        return Ok(None);
    }

    let (mapping_config, base_frame) =
        build_wide_base_mapping(table, domain, study_id, &wide_columns)?;
    let date_idx = find_lb_date_column(&table.headers);
    let time_idx = find_lb_time_column(&table.headers);
    let (expanded, used_wide) =
        expand_lb_wide(table, &base_frame.data, domain, &groups, date_idx, time_idx)?;

    let mut used = mapping_used_sources(&mapping_config);
    used.extend(used_wide);

    Ok(Some((
        mapping_config,
        DomainFrame::new(domain.code.clone(), expanded),
        used,
    )))
}

/// Detect LB wide format column groups from headers.
pub fn detect_lb_wide_groups(
    headers: &[String],
) -> (BTreeMap<String, LbWideGroup>, BTreeSet<String>) {
    let base_candidates = lb_base_candidates(headers);
    let mut groups: BTreeMap<String, LbWideGroup> = BTreeMap::new();
    let mut wide_columns = BTreeSet::new();

    // First pass: detect PREFIX_KEY patterns
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        let mut matched = false;

        for prefix in [
            "TEST", "ORRES", "ORRESU", "ORRESUO", "ORNR", "RANGE", "CLSIG",
        ] {
            let prefix_tag = format!("{prefix}_");
            if !upper.starts_with(&prefix_tag) {
                continue;
            }

            matched = true;
            let rest = &upper[prefix_tag.len()..];
            let (mut key, attr) = if prefix == "ORNR" {
                if let Some(stripped) = rest.strip_suffix("_LOWER") {
                    (stripped.to_string(), Some("LOWER"))
                } else if let Some(stripped) = rest.strip_suffix("_UPPER") {
                    (stripped.to_string(), Some("UPPER"))
                } else {
                    (rest.to_string(), Some("RANGE"))
                }
            } else {
                (rest.to_string(), None)
            };

            if matches!(prefix, "ORNR" | "RANGE") {
                key = normalize_lb_key(&key);
            }

            let mut is_code = false;
            if key.len() > 2 && key.ends_with("CD") {
                key.truncate(key.len() - 2);
                is_code = true;
            }

            let base_key = lb_base_key(&key, &base_candidates);
            let entry = groups.entry(key.clone()).or_insert_with(|| LbWideGroup {
                base_key,
                ..LbWideGroup::default()
            });

            if is_code {
                entry.extra_cols.push(idx);
                break;
            }

            match prefix {
                "TEST" => entry.test_col = Some(idx),
                "ORRES" => entry.orres_col = Some(idx),
                "ORRESU" => entry.orresu_col = Some(idx),
                "ORRESUO" => entry.orresu_alt_col = Some(idx),
                "ORNR" => match attr {
                    Some("RANGE") => entry.ornr_range_col = Some(idx),
                    Some("LOWER") => entry.ornr_lower_col = Some(idx),
                    Some("UPPER") => entry.ornr_upper_col = Some(idx),
                    _ => {}
                },
                "RANGE" => entry.range_col = Some(idx),
                "CLSIG" => entry.clsig_col = Some(idx),
                _ => {}
            }
            break;
        }

        if matched {
            wide_columns.insert(upper);
        }
    }

    // Second pass: detect KEYSUFFIX patterns (e.g., GLUCOSEORRES)
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if wide_columns.contains(&upper) || upper.contains('_') {
            continue;
        }

        if let Some(stripped) = upper.strip_suffix("CD")
            && let Some((key, kind)) = parse_lb_suffix(stripped)
        {
            let base_key = lb_base_key(&key, &base_candidates);
            let entry = groups.entry(key.clone()).or_insert_with(|| LbWideGroup {
                base_key,
                ..LbWideGroup::default()
            });

            match kind {
                LbSuffixKind::TestCd
                | LbSuffixKind::Test
                | LbSuffixKind::Orres
                | LbSuffixKind::Orresu
                | LbSuffixKind::OrresuAlt
                | LbSuffixKind::OrnrRange
                | LbSuffixKind::OrnrLower
                | LbSuffixKind::OrnrUpper
                | LbSuffixKind::Range
                | LbSuffixKind::Clsig => {
                    entry.extra_cols.push(idx);
                    wide_columns.insert(upper);
                }
            }
            continue;
        }

        if let Some((key, kind)) = parse_lb_suffix(&upper) {
            let base_key = lb_base_key(&key, &base_candidates);
            let entry = groups.entry(key.clone()).or_insert_with(|| LbWideGroup {
                base_key,
                ..LbWideGroup::default()
            });

            match kind {
                LbSuffixKind::TestCd => entry.testcd_col = Some(idx),
                LbSuffixKind::Test => entry.test_col = Some(idx),
                LbSuffixKind::Orres => entry.orres_col = Some(idx),
                LbSuffixKind::Orresu => entry.orresu_col = Some(idx),
                LbSuffixKind::OrresuAlt => entry.orresu_alt_col = Some(idx),
                LbSuffixKind::OrnrRange => entry.ornr_range_col = Some(idx),
                LbSuffixKind::OrnrLower => entry.ornr_lower_col = Some(idx),
                LbSuffixKind::OrnrUpper => entry.ornr_upper_col = Some(idx),
                LbSuffixKind::Range => entry.range_col = Some(idx),
                LbSuffixKind::Clsig => entry.clsig_col = Some(idx),
            }
            wide_columns.insert(upper);
        }
    }

    // Third pass: detect date/time columns
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if wide_columns.contains(&upper) || upper.contains('_') {
            continue;
        }

        if let Some((key, is_time)) = parse_lb_time_suffix(&upper)
            && let Some(entry) = groups.get_mut(&key)
        {
            if is_time {
                entry.time_col = Some(idx);
            } else {
                entry.date_col = Some(idx);
            }
            wide_columns.insert(upper);
        }
    }

    (groups, wide_columns)
}

/// Parse LB column suffix to determine field type.
pub fn parse_lb_suffix(value: &str) -> Option<(String, LbSuffixKind)> {
    let patterns = [
        ("TESTCD", LbSuffixKind::TestCd),
        ("TEST", LbSuffixKind::Test),
        ("ORRESUO", LbSuffixKind::OrresuAlt),
        ("ORRESU", LbSuffixKind::Orresu),
        ("ORRES", LbSuffixKind::Orres),
        ("ORNRLOWER", LbSuffixKind::OrnrLower),
        ("ORNRUPPER", LbSuffixKind::OrnrUpper),
        ("ORNRLO", LbSuffixKind::OrnrLower),
        ("ORNRHI", LbSuffixKind::OrnrUpper),
        ("ORNR", LbSuffixKind::OrnrRange),
        ("CLSIG", LbSuffixKind::Clsig),
        ("RANGE", LbSuffixKind::Range),
    ];

    for (suffix, kind) in patterns {
        if value.len() > suffix.len() && value.ends_with(suffix) {
            let key = value[..value.len() - suffix.len()]
                .trim_end_matches('_')
                .to_string();
            if !key.is_empty() {
                return Some((key, kind));
            }
        }
    }
    None
}

/// Parse LB time/date suffix.
fn parse_lb_time_suffix(value: &str) -> Option<(String, bool)> {
    let patterns = [
        ("DATE", false),
        ("DAT", false),
        ("TIME", true),
        ("TIM", true),
    ];

    for (suffix, is_time) in patterns {
        if value.len() > suffix.len() && value.ends_with(suffix) {
            let key = value[..value.len() - suffix.len()]
                .trim_end_matches('_')
                .to_string();
            if !key.is_empty() {
                return Some((key, is_time));
            }
        }
    }
    None
}

/// Normalize LB key by stripping range-related suffixes.
fn normalize_lb_key(value: &str) -> String {
    let upper = value.to_uppercase();
    if let Some((base, suffix)) = upper.rsplit_once('_') {
        let suffix = suffix.trim();
        if matches!(
            suffix,
            "LOWER"
                | "UPPER"
                | "LOW"
                | "HIGH"
                | "HI"
                | "LO"
                | "COMPARATOR"
                | "COMP"
                | "CMP"
                | "RANGE"
                | "IND"
                | "FLAG"
        ) {
            return base.to_string();
        }
    }
    value.to_string()
}

/// Strip trailing digits from a key.
fn strip_trailing_digits(value: &str) -> String {
    let mut trimmed = value.to_string();
    while trimmed
        .chars()
        .last()
        .map(|ch| ch.is_ascii_digit())
        .unwrap_or(false)
    {
        trimmed.pop();
    }
    if trimmed.is_empty() {
        value.to_string()
    } else {
        trimmed
    }
}

/// Build set of base key candidates from headers.
fn lb_base_candidates(headers: &[String]) -> BTreeSet<String> {
    let mut bases = BTreeSet::new();

    for header in headers {
        let upper = header.to_uppercase();
        for prefix in [
            "TEST", "ORRES", "ORRESU", "ORRESUO", "ORNR", "RANGE", "CLSIG",
        ] {
            let prefix_tag = format!("{prefix}_");
            if !upper.starts_with(&prefix_tag) {
                continue;
            }

            let rest = &upper[prefix_tag.len()..];
            let mut key = if prefix == "ORNR" {
                if let Some(stripped) = rest.strip_suffix("_LOWER") {
                    stripped.to_string()
                } else if let Some(stripped) = rest.strip_suffix("_UPPER") {
                    stripped.to_string()
                } else {
                    rest.to_string()
                }
            } else {
                rest.to_string()
            };

            if matches!(prefix, "ORNR" | "RANGE") {
                key = normalize_lb_key(&key);
            }
            if key.len() > 2 && key.ends_with("CD") {
                key.truncate(key.len() - 2);
            }

            let base = strip_trailing_digits(&key);
            bases.insert(base);
            break;
        }
    }
    bases
}

/// Determine base key for grouping related columns.
fn lb_base_key(value: &str, bases: &BTreeSet<String>) -> String {
    let mut base = strip_trailing_digits(value);
    if base.ends_with("OT") && base.len() > 2 {
        let without_ot = base[..base.len() - 2].to_string();
        if bases.contains(&without_ot) {
            base = without_ot;
        }
    }
    if base.is_empty() {
        value.to_string()
    } else {
        base
    }
}

/// Find global date column for LB domain.
pub fn find_lb_date_column(headers: &[String]) -> Option<usize> {
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if (upper.ends_with("DAT") || upper.ends_with("DATE")) && !upper.contains("EVENT") {
            return Some(idx);
        }
    }
    None
}

/// Find global time column for LB domain.
pub fn find_lb_time_column(headers: &[String]) -> Option<usize> {
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if (upper.ends_with("TIM") || upper.ends_with("TIME")) && !upper.contains("EVENT") {
            return Some(idx);
        }
    }
    None
}

/// Expand LB wide format to long format.
fn expand_lb_wide(
    table: &CsvTable,
    base_df: &DataFrame,
    domain: &Domain,
    groups: &BTreeMap<String, LbWideGroup>,
    date_idx: Option<usize>,
    time_idx: Option<usize>,
) -> Result<(DataFrame, BTreeSet<String>)> {
    let variable_names: Vec<String> = domain
        .variables
        .iter()
        .map(|var| var.name.clone())
        .collect();

    let mut values: Vec<Vec<String>> = variable_names.iter().map(|_| Vec::new()).collect();

    let lbtestcd_idx = variable_names.iter().position(|name| name == "LBTESTCD");
    let lbtest_idx = variable_names.iter().position(|name| name == "LBTEST");
    let lborres_idx = variable_names.iter().position(|name| name == "LBORRES");
    let lborresu_idx = variable_names.iter().position(|name| name == "LBORRESU");
    let lbornrlo_idx = variable_names.iter().position(|name| name == "LBORNRLO");
    let lbornrhi_idx = variable_names.iter().position(|name| name == "LBORNRHI");
    let lbclsig_idx = variable_names.iter().position(|name| name == "LBCLSIG");
    let lbdtc_idx = variable_names.iter().position(|name| name == "LBDTC");

    let mut used = BTreeSet::new();

    // Collect used column names
    for group in groups.values() {
        for idx in [
            group.test_col,
            group.testcd_col,
            group.orres_col,
            group.orresu_col,
            group.orresu_alt_col,
            group.ornr_range_col,
            group.ornr_lower_col,
            group.ornr_upper_col,
            group.range_col,
            group.clsig_col,
            group.date_col,
            group.time_col,
        ] {
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
            let group_date_value = group
                .date_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let group_time_value = group
                .time_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();

            let date_value = if !group_date_value.trim().is_empty() {
                group_date_value
            } else {
                base_date_value.clone()
            };
            let time_value = if !group_time_value.trim().is_empty() {
                group_time_value
            } else {
                base_time_value.clone()
            };

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
            let orresu_alt_value = group
                .orresu_alt_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let ornr_lower_value = group
                .ornr_lower_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let ornr_upper_value = group
                .ornr_upper_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();
            let clsig_value = group
                .clsig_col
                .and_then(|idx| table.rows[row_idx].get(idx))
                .cloned()
                .unwrap_or_default();

            let has_result = !test_value.trim().is_empty()
                || !testcd_value.trim().is_empty()
                || !orres_value.trim().is_empty()
                || !orresu_value.trim().is_empty()
                || !orresu_alt_value.trim().is_empty();

            if !has_result {
                continue;
            }

            total_rows += 1;

            let label = group
                .test_col
                .and_then(|idx| table.headers.get(idx))
                .and_then(|name| table_label(table, name))
                .or_else(|| {
                    group
                        .orres_col
                        .and_then(|idx| table.headers.get(idx))
                        .and_then(|name| table_label(table, name))
                })
                .unwrap_or_default();

            let test_code = if !testcd_value.trim().is_empty() {
                sanitize_test_code(testcd_value.trim())
            } else if !test_value.trim().is_empty() {
                sanitize_test_code(test_value.trim())
            } else if !label.trim().is_empty() {
                sanitize_test_code(label.trim())
            } else {
                sanitize_test_code(&group.base_key)
            };

            let test_label = if !test_value.trim().is_empty() {
                test_value.clone()
            } else if !label.trim().is_empty() {
                label.clone()
            } else {
                group.base_key.clone()
            };

            let mut row_values = base_row.clone();

            if let Some(idx) = lbtestcd_idx {
                row_values[idx] = test_code;
            }
            if let Some(idx) = lbtest_idx {
                row_values[idx] = test_label;
            }
            if let Some(idx) = lborres_idx {
                row_values[idx] = orres_value.clone();
            }
            if let Some(idx) = lborresu_idx {
                if !orresu_value.trim().is_empty() {
                    row_values[idx] = orresu_value.clone();
                } else {
                    row_values[idx] = orresu_alt_value.clone();
                }
            }
            if let Some(idx) = lbornrlo_idx {
                row_values[idx] = normalize_numeric(&ornr_lower_value);
            }
            if let Some(idx) = lbornrhi_idx {
                row_values[idx] = normalize_numeric(&ornr_upper_value);
            }
            if let Some(idx) = lbclsig_idx {
                row_values[idx] = clsig_value.clone();
            }
            if let Some(idx) = lbdtc_idx
                && !date_value.trim().is_empty()
            {
                if !time_value.trim().is_empty() && !date_value.contains('T') {
                    row_values[idx] = format!("{}T{}", date_value.trim(), time_value.trim());
                } else {
                    row_values[idx] = date_value.clone();
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
