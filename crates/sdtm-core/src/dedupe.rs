use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use polars::prelude::{BooleanChunked, NewChunkedArray};

use crate::frame::DomainFrame;
use sdtm_model::Domain;

use crate::data_utils::column_value_string;

fn is_generic_identifier(name: &str) -> bool {
    matches!(
        name.to_uppercase().as_str(),
        "STUDYID" | "DOMAIN" | "RDOMAIN" | "USUBJID"
    )
}

pub fn dedupe_frames_by_identifiers(
    frames: &mut [DomainFrame],
    standards_map: &BTreeMap<String, Domain>,
    suppqual_domain: &Domain,
) -> Result<()> {
    for frame in frames.iter_mut() {
        let code = frame.domain_code.to_uppercase();
        let domain = if let Some(domain) = standards_map.get(&code) {
            domain
        } else if code.starts_with("SUPP") {
            suppqual_domain
        } else {
            continue;
        };
        if frame.data.height() == 0 {
            continue;
        }
        let mut keys: Vec<String> = domain
            .variables
            .iter()
            .filter_map(|var| {
                let role = var.role.as_deref()?.trim();
                let upper = role.to_uppercase();
                if upper.contains("IDENTIFIER") {
                    Some(var.name.clone())
                } else {
                    None
                }
            })
            .collect();
        keys.sort_by_key(|name| name.to_uppercase());
        if keys.is_empty() {
            continue;
        }
        let key_columns: Vec<String> = keys
            .into_iter()
            .filter(|key| frame.data.column(key).is_ok())
            .collect();
        if key_columns.is_empty() {
            continue;
        }
        if key_columns.iter().all(|name| is_generic_identifier(name)) {
            continue;
        }
        let mut seen = BTreeSet::new();
        let row_count = frame.data.height();
        let mut keep = Vec::with_capacity(row_count);
        for idx in 0..row_count {
            let mut composite = String::new();
            for (pos, name) in key_columns.iter().enumerate() {
                if pos > 0 {
                    composite.push('|');
                }
                composite.push_str(column_value_string(&frame.data, name, idx).trim());
            }
            if composite.trim().is_empty() {
                keep.push(true);
                continue;
            }
            keep.push(seen.insert(composite));
        }
        let mask = BooleanChunked::from_slice("dedupe".into(), &keep);
        frame.data = frame.data.filter(&mask)?;
    }
    Ok(())
}
