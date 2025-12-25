use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use polars::prelude::{DataFrame, NamedFrom, Series};

use crate::{DomainFrame, build_domain_frame_with_mapping};
use sdtm_ingest::{CsvTable, build_column_hints};
use sdtm_map::MappingEngine;
use sdtm_model::{Domain, MappingConfig, VariableType};

use crate::data_utils::{
    column_value_string, mapping_source_for_target, sanitize_test_code, table_label,
};

#[derive(Debug, Default, Clone)]
struct LbWideGroup {
    base_key: String,
    test_col: Option<usize>,
    testcd_col: Option<usize>,
    orres_col: Option<usize>,
    orresu_col: Option<usize>,
    orresu_alt_col: Option<usize>,
    ornr_range_col: Option<usize>,
    ornr_lower_col: Option<usize>,
    ornr_upper_col: Option<usize>,
    range_col: Option<usize>,
    clsig_col: Option<usize>,
    date_col: Option<usize>,
    time_col: Option<usize>,
    extra_cols: Vec<usize>,
}

#[derive(Debug, Default, Clone)]
struct VsWideGroup {
    key: String,
    label: Option<String>,
    orres_col: Option<usize>,
    orresu_col: Option<usize>,
    pos_col: Option<usize>,
    extra_cols: Vec<usize>,
}

#[derive(Debug, Default, Clone)]
struct VsWideShared {
    orresu_bp: Option<usize>,
    pos_bp: Option<usize>,
}

#[derive(Debug, Default, Clone)]
struct IeWideGroup {
    category: String,
    test_col: Option<usize>,
    testcd_col: Option<usize>,
}

pub(crate) fn build_lb_wide_frame(
    table: &CsvTable,
    domain: &Domain,
    study_id: &str,
) -> Result<Option<(MappingConfig, DomainFrame, BTreeSet<String>)>> {
    let (groups, wide_columns) = detect_lb_wide_groups(&table.headers);
    if groups.is_empty() {
        return Ok(None);
    }
    let base_table = filter_table_columns(table, &wide_columns, false);
    let hints = build_column_hints(&base_table);
    let engine = MappingEngine::new((*domain).clone(), 0.5, hints);
    let result = engine.suggest(&base_table.headers);
    let mapping_config = engine.to_config(study_id, result);
    let base_frame = build_domain_frame_with_mapping(&base_table, domain, Some(&mapping_config))?;
    let date_idx = find_lb_date_column(&table.headers);
    let time_idx = find_lb_time_column(&table.headers);
    let (expanded, used_wide) =
        expand_lb_wide(table, &base_frame.data, domain, &groups, date_idx, time_idx)?;
    let mut used: BTreeSet<String> = mapping_config
        .mappings
        .iter()
        .map(|mapping| mapping.source_column.clone())
        .collect();
    used.extend(used_wide);
    Ok(Some((
        mapping_config,
        DomainFrame {
            domain_code: domain.code.clone(),
            data: expanded,
        },
        used,
    )))
}

pub(crate) fn build_vs_wide_frame(
    table: &CsvTable,
    domain: &Domain,
    study_id: &str,
) -> Result<Option<(MappingConfig, DomainFrame, BTreeSet<String>)>> {
    let (groups, shared, wide_columns) =
        detect_vs_wide_groups(&table.headers, table.labels.as_deref());
    if groups.is_empty() {
        return Ok(None);
    }
    let base_table = filter_table_columns(table, &wide_columns, false);
    let hints = build_column_hints(&base_table);
    let engine = MappingEngine::new((*domain).clone(), 0.5, hints);
    let result = engine.suggest(&base_table.headers);
    let mapping_config = engine.to_config(study_id, result);
    let base_frame = build_domain_frame_with_mapping(&base_table, domain, Some(&mapping_config))?;
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
    let mut used: BTreeSet<String> = mapping_config
        .mappings
        .iter()
        .map(|mapping| mapping.source_column.clone())
        .collect();
    used.extend(used_wide);
    Ok(Some((
        mapping_config,
        DomainFrame {
            domain_code: domain.code.clone(),
            data: expanded,
        },
        used,
    )))
}

pub(crate) fn build_ie_wide_frame(
    table: &CsvTable,
    domain: &Domain,
    study_id: &str,
) -> Result<Option<(MappingConfig, DomainFrame, BTreeSet<String>)>> {
    let (groups, wide_columns) = detect_ie_wide_groups(&table.headers);
    if groups.is_empty() {
        return Ok(None);
    }
    let base_table = filter_table_columns(table, &wide_columns, false);
    let hints = build_column_hints(&base_table);
    let engine = MappingEngine::new((*domain).clone(), 0.5, hints);
    let result = engine.suggest(&base_table.headers);
    let mapping_config = engine.to_config(study_id, result);
    let base_frame = build_domain_frame_with_mapping(&base_table, domain, Some(&mapping_config))?;
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
    let mut used: BTreeSet<String> = mapping_config
        .mappings
        .iter()
        .map(|mapping| mapping.source_column.clone())
        .collect();
    used.extend(used_wide);
    Ok(Some((
        mapping_config,
        DomainFrame {
            domain_code: domain.code.clone(),
            data: expanded,
        },
        used,
    )))
}

fn detect_lb_wide_groups(headers: &[String]) -> (BTreeMap<String, LbWideGroup>, BTreeSet<String>) {
    let base_candidates = lb_base_candidates(headers);
    let mut groups: BTreeMap<String, LbWideGroup> = BTreeMap::new();
    let mut wide_columns = BTreeSet::new();
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

    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if wide_columns.contains(&upper) || upper.contains('_') {
            continue;
        }
        if let Some(stripped) = upper.strip_suffix("CD") {
            if let Some((key, kind)) = parse_lb_suffix(stripped) {
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

    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if wide_columns.contains(&upper) || upper.contains('_') {
            continue;
        }
        if let Some((key, is_time)) = parse_lb_time_suffix(&upper) {
            if let Some(entry) = groups.get_mut(&key) {
                if is_time {
                    entry.time_col = Some(idx);
                } else {
                    entry.date_col = Some(idx);
                }
                wide_columns.insert(upper);
            }
        }
    }
    (groups, wide_columns)
}

#[derive(Debug, Clone, Copy)]
enum LbSuffixKind {
    TestCd,
    Test,
    Orres,
    Orresu,
    OrresuAlt,
    OrnrRange,
    OrnrLower,
    OrnrUpper,
    Range,
    Clsig,
}

fn parse_lb_suffix(value: &str) -> Option<(String, LbSuffixKind)> {
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
            if entry.label.is_none() {
                if let Some(labels) = labels {
                    if let Some(label) = labels.get(idx) {
                        let trimmed = label.trim();
                        if !trimmed.is_empty() {
                            entry.label = Some(trimmed.to_string());
                        }
                    }
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

fn source_is_ie_test(source: &Option<String>) -> bool {
    let Some(source) = source else {
        return false;
    };
    let upper = source.to_uppercase();
    upper.contains("IETEST") || upper.contains("IEINTEST") || upper.contains("IEEXTEST")
}

fn source_is_ie_cat(source: &Option<String>) -> bool {
    let Some(source) = source else {
        return false;
    };
    source.to_uppercase().contains("IECAT")
}

fn filter_table_columns(table: &CsvTable, columns: &BTreeSet<String>, include: bool) -> CsvTable {
    let mut indices = Vec::new();
    let mut headers = Vec::new();
    let mut labels = table.labels.as_ref().map(|_| Vec::new());
    for (idx, header) in table.headers.iter().enumerate() {
        let has = columns.contains(&header.to_uppercase());
        if has == include {
            indices.push(idx);
            headers.push(header.clone());
            if let Some(label_vec) = table.labels.as_ref() {
                if let Some(labels_mut) = labels.as_mut() {
                    labels_mut.push(label_vec.get(idx).cloned().unwrap_or_default());
                }
            }
        }
    }
    let mut rows = Vec::with_capacity(table.rows.len());
    for row in &table.rows {
        let mut next = Vec::with_capacity(indices.len());
        for &idx in &indices {
            next.push(row.get(idx).cloned().unwrap_or_default());
        }
        rows.push(next);
    }
    CsvTable {
        headers,
        rows,
        labels,
    }
}

fn expand_ie_wide(
    table: &CsvTable,
    base_df: &DataFrame,
    domain: &Domain,
    groups: &BTreeMap<String, IeWideGroup>,
    allow_base_test: bool,
    allow_base_cat: bool,
) -> Result<(DataFrame, BTreeSet<String>)> {
    let mut values: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for variable in &domain.variables {
        values.insert(variable.name.clone(), Vec::new());
    }
    let mut used = BTreeSet::new();
    for group in groups.values() {
        for idx in [group.test_col, group.testcd_col] {
            if let Some(idx) = idx {
                if let Some(name) = table.headers.get(idx) {
                    used.insert(name.clone());
                }
            }
        }
    }
    let test_col = crate::domain_utils::column_name(domain, "IETEST");
    let testcd_col = crate::domain_utils::column_name(domain, "IETESTCD");
    let cat_col = crate::domain_utils::column_name(domain, "IECAT");
    let mut total_rows = 0usize;
    for row_idx in 0..table.rows.len() {
        let base_test = if allow_base_test {
            test_col
                .as_deref()
                .map(|name| column_value_string(base_df, name, row_idx))
                .unwrap_or_default()
        } else {
            String::new()
        };
        let base_testcd = if allow_base_test {
            testcd_col
                .as_deref()
                .map(|name| column_value_string(base_df, name, row_idx))
                .unwrap_or_default()
        } else {
            String::new()
        };
        let base_cat = if allow_base_cat {
            cat_col
                .as_deref()
                .map(|name| column_value_string(base_df, name, row_idx))
                .unwrap_or_default()
        } else {
            String::new()
        };
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

            let mut base_values: BTreeMap<String, String> = BTreeMap::new();
            for variable in &domain.variables {
                let val = column_value_string(base_df, &variable.name, row_idx);
                base_values.insert(variable.name.clone(), val);
            }
            if let Some(name) = testcd_col.as_ref() {
                if !test_code.trim().is_empty() {
                    base_values.insert(name.clone(), test_code);
                }
            }
            if let Some(name) = test_col.as_ref() {
                if !test_label.trim().is_empty() {
                    base_values.insert(name.clone(), test_label);
                }
            }
            if let Some(name) = cat_col.as_ref() {
                let current = base_values.get(name).cloned().unwrap_or_default();
                if current.trim().is_empty() {
                    base_values.insert(name.clone(), group.category.clone());
                }
            }
            for (name, list) in values.iter_mut() {
                let value = base_values.get(name).cloned().unwrap_or_default();
                list.push(value);
            }
            total_rows += 1;
            added = true;
        }
        if !added {
            let base_has = !base_test.trim().is_empty()
                || !base_testcd.trim().is_empty()
                || !base_cat.trim().is_empty();
            if base_has {
                let mut base_values: BTreeMap<String, String> = BTreeMap::new();
                for variable in &domain.variables {
                    let val = column_value_string(base_df, &variable.name, row_idx);
                    base_values.insert(variable.name.clone(), val);
                }
                for (name, list) in values.iter_mut() {
                    let value = base_values.get(name).cloned().unwrap_or_default();
                    list.push(value);
                }
                total_rows += 1;
            }
        }
    }
    if total_rows == 0 {
        return Ok((base_df.clone(), used));
    }
    let mut columns = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        let vals = values.remove(&variable.name).unwrap_or_default();
        let column = match variable.data_type {
            VariableType::Num => {
                let numeric: Vec<Option<f64>> = vals
                    .iter()
                    .map(|value| value.trim().parse::<f64>().ok())
                    .collect();
                Series::new(variable.name.as_str().into(), numeric).into()
            }
            VariableType::Char => Series::new(variable.name.as_str().into(), vals).into(),
        };
        columns.push(column);
    }
    let data = DataFrame::new(columns)?;
    Ok((data, used))
}

fn normalize_numeric(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.parse::<f64>().is_ok() {
        trimmed.to_string()
    } else {
        String::new()
    }
}

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
    find_lb_date_column(headers)
}

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
    find_lb_time_column(headers)
}

fn find_lb_date_column(headers: &[String]) -> Option<usize> {
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if (upper.ends_with("DAT") || upper.ends_with("DATE")) && !upper.contains("EVENT") {
            return Some(idx);
        }
    }
    None
}

fn find_lb_time_column(headers: &[String]) -> Option<usize> {
    for (idx, header) in headers.iter().enumerate() {
        let upper = header.to_uppercase();
        if (upper.ends_with("TIM") || upper.ends_with("TIME")) && !upper.contains("EVENT") {
            return Some(idx);
        }
    }
    None
}

fn expand_vs_wide(
    table: &CsvTable,
    base_df: &DataFrame,
    domain: &Domain,
    groups: &BTreeMap<String, VsWideGroup>,
    shared: &VsWideShared,
    date_idx: Option<usize>,
    time_idx: Option<usize>,
) -> Result<(DataFrame, BTreeSet<String>)> {
    let mut values: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for variable in &domain.variables {
        values.insert(variable.name.clone(), Vec::new());
    }
    let mut used = BTreeSet::new();
    for group in groups.values() {
        for idx in [group.orres_col, group.orresu_col, group.pos_col] {
            if let Some(idx) = idx {
                if let Some(name) = table.headers.get(idx) {
                    used.insert(name.clone());
                }
            }
        }
        for idx in &group.extra_cols {
            if let Some(name) = table.headers.get(*idx) {
                used.insert(name.clone());
            }
        }
    }
    for idx in [shared.orresu_bp, shared.pos_bp] {
        if let Some(idx) = idx {
            if let Some(name) = table.headers.get(idx) {
                used.insert(name.clone());
            }
        }
    }
    if let Some(idx) = date_idx {
        if let Some(name) = table.headers.get(idx) {
            used.insert(name.clone());
        }
    }
    if let Some(idx) = time_idx {
        if let Some(name) = table.headers.get(idx) {
            used.insert(name.clone());
        }
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
            let mut base_values: BTreeMap<String, String> = BTreeMap::new();
            for variable in &domain.variables {
                let val = column_value_string(base_df, &variable.name, row_idx);
                base_values.insert(variable.name.clone(), val);
            }
            if let Some(value) = base_values.get_mut("VSTESTCD") {
                *value = test_code.clone();
            }
            if let Some(value) = base_values.get_mut("VSTEST") {
                if !test_label.is_empty() {
                    *value = test_label.clone();
                } else if !test_code.is_empty() {
                    *value = test_code.clone();
                }
            }
            if let Some(value) = base_values.get_mut("VSORRES") {
                *value = orres_value.clone();
            }
            if let Some(value) = base_values.get_mut("VSORRESU") {
                if !orresu_value.trim().is_empty() {
                    *value = orresu_value.clone();
                } else {
                    *value = orresu_fallback.clone();
                }
            }
            if let Some(value) = base_values.get_mut("VSPOS") {
                if !pos_value.trim().is_empty() {
                    *value = pos_value.clone();
                } else {
                    *value = pos_fallback.clone();
                }
            }
            if let Some(value) = base_values.get_mut("VSDTC") {
                let date_value = base_date_value.clone();
                let time_value = base_time_value.clone();
                if !date_value.trim().is_empty() {
                    if !time_value.trim().is_empty() && !date_value.contains('T') {
                        *value = format!("{}T{}", date_value.trim(), time_value.trim());
                    } else {
                        *value = date_value.clone();
                    }
                }
            }

            for (name, list) in values.iter_mut() {
                let value = base_values.get(name).cloned().unwrap_or_default();
                list.push(value);
            }
        }
    }
    if total_rows == 0 {
        return Ok((base_df.clone(), used));
    }
    let mut columns = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        let vals = values.remove(&variable.name).unwrap_or_default();
        let column = match variable.data_type {
            VariableType::Num => {
                let numeric: Vec<Option<f64>> = vals
                    .iter()
                    .map(|value| value.trim().parse::<f64>().ok())
                    .collect();
                Series::new(variable.name.as_str().into(), numeric).into()
            }
            VariableType::Char => Series::new(variable.name.as_str().into(), vals).into(),
        };
        columns.push(column);
    }
    let data = DataFrame::new(columns)?;
    Ok((data, used))
}

fn expand_lb_wide(
    table: &CsvTable,
    base_df: &DataFrame,
    domain: &Domain,
    groups: &BTreeMap<String, LbWideGroup>,
    date_idx: Option<usize>,
    time_idx: Option<usize>,
) -> Result<(DataFrame, BTreeSet<String>)> {
    let mut values: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for variable in &domain.variables {
        values.insert(variable.name.clone(), Vec::new());
    }
    let mut used = BTreeSet::new();
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
            if let Some(idx) = idx {
                if let Some(name) = table.headers.get(idx) {
                    used.insert(name.clone());
                }
            }
        }
        for idx in &group.extra_cols {
            if let Some(name) = table.headers.get(*idx) {
                used.insert(name.clone());
            }
        }
    }
    if let Some(idx) = date_idx {
        if let Some(name) = table.headers.get(idx) {
            used.insert(name.clone());
        }
    }
    if let Some(idx) = time_idx {
        if let Some(name) = table.headers.get(idx) {
            used.insert(name.clone());
        }
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
            let _ornr_range_value = group
                .ornr_range_col
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
            let _range_value = group
                .range_col
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
            let mut base_values: BTreeMap<String, String> = BTreeMap::new();
            for variable in &domain.variables {
                let val = column_value_string(base_df, &variable.name, row_idx);
                base_values.insert(variable.name.clone(), val);
            }
            if let Some(value) = base_values.get_mut("LBTESTCD") {
                *value = test_code;
            }
            if let Some(value) = base_values.get_mut("LBTEST") {
                *value = test_label;
            }
            if let Some(value) = base_values.get_mut("LBORRES") {
                *value = orres_value.clone();
            }
            if let Some(value) = base_values.get_mut("LBORRESU") {
                if !orresu_value.trim().is_empty() {
                    *value = orresu_value.clone();
                } else {
                    *value = orresu_alt_value.clone();
                }
            }
            if let Some(value) = base_values.get_mut("LBORNRLO") {
                *value = normalize_numeric(&ornr_lower_value);
            }
            if let Some(value) = base_values.get_mut("LBORNRHI") {
                *value = normalize_numeric(&ornr_upper_value);
            }
            if let Some(value) = base_values.get_mut("LBCLSIG") {
                *value = clsig_value.clone();
            }
            if let Some(value) = base_values.get_mut("LBDTC") {
                if !date_value.trim().is_empty() {
                    if !time_value.trim().is_empty() && !date_value.contains('T') {
                        *value = format!("{}T{}", date_value.trim(), time_value.trim());
                    } else {
                        *value = date_value.clone();
                    }
                }
            }

            for (name, list) in values.iter_mut() {
                let value = base_values.get(name).cloned().unwrap_or_default();
                list.push(value);
            }
        }
    }
    if total_rows == 0 {
        return Ok((base_df.clone(), used));
    }
    let mut columns = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        let vals = values.remove(&variable.name).unwrap_or_default();
        let column = match variable.data_type {
            VariableType::Num => {
                let numeric: Vec<Option<f64>> = vals
                    .iter()
                    .map(|value| value.trim().parse::<f64>().ok())
                    .collect();
                Series::new(variable.name.as_str().into(), numeric).into()
            }
            VariableType::Char => Series::new(variable.name.as_str().into(), vals).into(),
        };
        columns.push(column);
    }
    let data = DataFrame::new(columns)?;
    Ok((data, used))
}
