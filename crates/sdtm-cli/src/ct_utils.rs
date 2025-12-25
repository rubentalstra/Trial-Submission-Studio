use std::collections::BTreeSet;

use sdtm_core::ProcessingContext;
use sdtm_model::{ControlledTerminology, Domain};

use crate::data_utils::{table_column_values, table_label};

pub fn compact_key(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_uppercase())
        .collect()
}

pub fn is_yes_no_token(value: &str) -> bool {
    matches!(
        value.trim().to_uppercase().as_str(),
        "Y" | "N" | "YES" | "NO" | "TRUE" | "FALSE" | "1" | "0"
    )
}

pub fn resolve_ct_submission_value(ct: &ControlledTerminology, raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let key = trimmed.to_uppercase();
    if let Some(mapped) = ct.synonyms.get(&key) {
        return Some(mapped.clone());
    }
    if ct.submission_values.iter().any(|val| val == trimmed) {
        return Some(trimmed.to_string());
    }
    for submission in &ct.submission_values {
        if compact_key(submission) == compact_key(trimmed) {
            return Some(submission.clone());
        }
    }
    for (submission, preferred) in &ct.preferred_terms {
        if compact_key(preferred) == compact_key(trimmed) {
            return Some(submission.clone());
        }
    }
    None
}

pub fn resolve_ct_value_from_hint(ct: &ControlledTerminology, hint: &str) -> Option<String> {
    if let Some(value) = resolve_ct_submission_value(ct, hint) {
        return Some(value);
    }
    let hint_compact = compact_key(hint);
    if hint_compact.len() < 3 {
        return None;
    }
    let mut matches: Vec<String> = Vec::new();
    for submission in &ct.submission_values {
        let compact = compact_key(submission);
        if compact.len() >= 3
            && (hint_compact.contains(&compact) || compact.contains(&hint_compact))
        {
            matches.push(submission.clone());
        }
    }
    for (submission, preferred) in &ct.preferred_terms {
        let compact = compact_key(preferred);
        if compact.len() >= 3
            && (hint_compact.contains(&compact) || compact.contains(&hint_compact))
        {
            matches.push(submission.clone());
        }
    }
    for (synonym, submission) in &ct.synonyms {
        let compact = compact_key(synonym);
        if compact.len() >= 3
            && (hint_compact.contains(&compact) || compact.contains(&hint_compact))
        {
            matches.push(submission.clone());
        }
    }
    matches.sort();
    matches.dedup();
    if matches.len() == 1 {
        Some(matches.remove(0))
    } else {
        let mut best_dist = usize::MAX;
        let mut best_val: Option<String> = None;
        let mut best_count = 0usize;
        for submission in &ct.submission_values {
            let dist = edit_distance(&hint_compact, &compact_key(submission));
            if dist < best_dist {
                best_dist = dist;
                best_val = Some(submission.clone());
                best_count = 1;
            } else if dist == best_dist {
                best_count += 1;
            }
        }
        if best_dist <= 1 && best_count == 1 {
            best_val
        } else {
            None
        }
    }
}

fn edit_distance(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }
    let a_len = a.len();
    let b_len = b.len();
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }
    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0usize; b_len + 1];
    for (i, a_ch) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, b_ch) in b.chars().enumerate() {
            let cost = if a_ch == b_ch { 0 } else { 1 };
            let insert = curr[j] + 1;
            let delete = prev[j + 1] + 1;
            let replace = prev[j] + cost;
            curr[j + 1] = insert.min(delete).min(replace);
        }
        prev.clone_from_slice(&curr);
    }
    prev[b_len]
}

pub fn ct_column_match(
    table: &sdtm_ingest::CsvTable,
    domain: &Domain,
    ct: &ControlledTerminology,
) -> Option<(String, Vec<Option<String>>, Vec<String>)> {
    let mut standard_vars = BTreeSet::new();
    for variable in &domain.variables {
        standard_vars.insert(variable.name.to_uppercase());
    }
    let mut best: Option<(String, Vec<Option<String>>, Vec<String>, f64, usize)> = None;
    for header in &table.headers {
        if standard_vars.contains(&header.to_uppercase()) {
            continue;
        }
        let Some(values) = table_column_values(table, header) else {
            continue;
        };
        let mut mapped = Vec::with_capacity(values.len());
        let mut matches = 0usize;
        let mut non_empty = 0usize;
        for value in &values {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                mapped.push(None);
                continue;
            }
            non_empty += 1;
            if let Some(ct_value) = resolve_ct_value_from_hint(ct, trimmed) {
                matches += 1;
                mapped.push(Some(ct_value));
            } else {
                mapped.push(None);
            }
        }
        if non_empty == 0 || matches == 0 {
            continue;
        }
        let ratio = matches as f64 / non_empty as f64;
        if ratio < 0.6 {
            continue;
        }
        let replace = match &best {
            Some((_, _, _, best_ratio, best_matches)) => {
                ratio > *best_ratio || (ratio == *best_ratio && matches > *best_matches)
            }
            None => true,
        };
        if replace {
            best = Some((header.clone(), mapped, values, ratio, matches));
        }
    }
    best.map(|(header, mapped, values, _ratio, _matches)| (header, mapped, values))
}

fn is_yes_no(value: &str) -> bool {
    matches!(
        value.trim().to_uppercase().as_str(),
        "Y" | "YES" | "N" | "NO"
    )
}

pub fn completion_column(
    table: &sdtm_ingest::CsvTable,
    domain: &Domain,
) -> Option<(Vec<String>, String)> {
    let mut standard_vars = BTreeSet::new();
    for variable in &domain.variables {
        standard_vars.insert(variable.name.to_uppercase());
    }
    for header in &table.headers {
        if standard_vars.contains(&header.to_uppercase()) {
            continue;
        }
        let label = table_label(table, header).unwrap_or_else(|| header.clone());
        let label_upper = label.to_uppercase();
        if !label_upper.contains("COMPLETE") && !label_upper.contains("COMPLETION") {
            continue;
        }
        let Some(values) = table_column_values(table, header) else {
            continue;
        };
        let mut non_empty = 0usize;
        let mut yes_no = 0usize;
        for value in &values {
            if value.trim().is_empty() {
                continue;
            }
            non_empty += 1;
            if is_yes_no(value) {
                yes_no += 1;
            }
        }
        if non_empty > 0 && (yes_no as f64 / non_empty as f64) >= 0.6 {
            return Some((values, label));
        }
    }
    None
}

pub fn resolve_ct_for_variable(
    ctx: &ProcessingContext,
    domain: &Domain,
    variable: &str,
    hint: &str,
    allow_raw: bool,
) -> Option<String> {
    let ct = ctx.resolve_ct(domain, variable)?;
    if let Some(value) = resolve_ct_value_from_hint(ct, hint) {
        return Some(value);
    }
    if allow_raw && ct.extensible {
        let trimmed = hint.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}
