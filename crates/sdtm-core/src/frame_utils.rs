use std::collections::BTreeMap;

use anyhow::{Context, Result};
use polars::prelude::{AnyValue, DataFrame, NamedFrom, Series};

use sdtm_model::Domain;

use crate::{DomainFrame, infer_seq_column, standard_columns};
use sdtm_ingest::any_to_string;

pub fn insert_frame(map: &mut BTreeMap<String, DomainFrame>, frame: DomainFrame) -> Result<()> {
    let key = frame.domain_code.to_uppercase();
    if let Some(existing) = map.get_mut(&key) {
        existing
            .data
            .vstack_mut(&frame.data)
            .with_context(|| format!("merge {key} frames"))?;
        // Merge source files from the incoming frame
        if let Some(ref meta) = frame.meta {
            for source in &meta.source_files {
                existing.add_source_file(source.clone());
            }
        }
    } else {
        map.insert(
            key.clone(),
            DomainFrame {
                domain_code: key.clone(),
                data: frame.data,
                meta: frame.meta,
            },
        );
    }
    Ok(())
}

pub fn apply_sequence_offsets(
    domain: &Domain,
    df: &mut DataFrame,
    tracker: &mut BTreeMap<String, i64>,
) -> Result<()> {
    let Some(seq_col) = infer_seq_column(domain) else {
        return Ok(());
    };
    let columns = standard_columns(domain);
    let Some(usubjid_col) = columns.usubjid else {
        return Ok(());
    };
    let usubjid_series = match df.column(&usubjid_col) {
        Ok(series) => series.clone(),
        Err(_) => return Ok(()),
    };
    let mut values: Vec<Option<f64>> = Vec::with_capacity(df.height());
    for idx in 0..df.height() {
        let usubjid = any_to_string(usubjid_series.get(idx).unwrap_or(AnyValue::Null));
        let key = usubjid.trim();
        if key.is_empty() {
            values.push(None);
            continue;
        }
        let entry = tracker.entry(key.to_string()).or_insert(0);
        *entry += 1;
        values.push(Some(*entry as f64));
    }
    let series = Series::new(seq_col.into(), values);
    df.with_column(series)?;
    Ok(())
}
