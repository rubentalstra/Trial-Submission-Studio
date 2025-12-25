use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use polars::prelude::{BooleanChunked, DataFrame, NewChunkedArray};

use sdtm_core::DomainFrame;
use sdtm_model::Domain;

use crate::data_utils::column_value_string;

fn identifier_columns(domain: &Domain) -> Vec<String> {
    let mut columns: Vec<String> = domain
        .variables
        .iter()
        .filter_map(|var| {
            let role = var.role.as_deref()?.trim();
            if role.eq_ignore_ascii_case("Identifier") || role.to_uppercase().contains("IDENTIFIER")
            {
                Some(var.name.clone())
            } else {
                None
            }
        })
        .collect();
    columns.sort_by(|a, b| a.to_uppercase().cmp(&b.to_uppercase()));
    columns
}

pub fn dedupe_frames_by_identifiers(
    frames: &mut [DomainFrame],
    standards_map: &BTreeMap<String, &Domain>,
    suppqual_domain: &Domain,
) -> Result<()> {
    for frame in frames.iter_mut() {
        let code = frame.domain_code.to_uppercase();
        let domain = if let Some(domain) = standards_map.get(&code) {
            *domain
        } else if code.starts_with("SUPP") {
            suppqual_domain
        } else {
            continue;
        };
        let keys = identifier_columns(domain);
        if keys.is_empty() {
            continue;
        }
        dedupe_frame_by_keys(&mut frame.data, &keys)?;
    }
    Ok(())
}

fn dedupe_frame_by_keys(df: &mut DataFrame, keys: &[String]) -> Result<()> {
    if df.height() == 0 {
        return Ok(());
    }
    let mut key_columns = Vec::new();
    for key in keys {
        if df.column(key).is_ok() {
            key_columns.push(key.clone());
        }
    }
    if key_columns.is_empty() {
        return Ok(());
    }
    let mut seen = BTreeSet::new();
    let mut keep = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let mut composite = String::new();
        for (pos, name) in key_columns.iter().enumerate() {
            if pos > 0 {
                composite.push('|');
            }
            composite.push_str(column_value_string(df, name, idx).trim());
        }
        if composite.trim().is_empty() {
            keep.push(true);
            continue;
        }
        keep.push(seen.insert(composite));
    }
    let mask = BooleanChunked::from_slice("dedupe".into(), &keep);
    *df = df.filter(&mask)?;
    Ok(())
}
