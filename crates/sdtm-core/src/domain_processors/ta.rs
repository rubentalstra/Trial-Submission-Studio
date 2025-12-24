use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

use super::common::*;

pub(super) fn process_ta(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    let key_cols = ["EPOCH", "ARMCD", "ARM", "ETCD"]
        .into_iter()
        .filter_map(|name| col(domain, name))
        .filter(|name| has_column(df, name))
        .collect::<Vec<_>>();
    if !key_cols.is_empty() && df.height() > 0 {
        let mut keep = vec![true; df.height()];
        for idx in 0..df.height() {
            let mut blank = true;
            for col_name in &key_cols {
                let value = string_value(df, col_name, idx);
                if !value.trim().is_empty() {
                    blank = false;
                    break;
                }
            }
            if blank {
                keep[idx] = false;
            }
        }
        filter_rows(df, &keep)?;
    }
    for col_name in ["EPOCH", "ARMCD", "ARM", "ETCD", "STUDYID", "DOMAIN"] {
        if let Some(name) = col(domain, col_name) {
            if has_column(df, &name) {
                let values = string_column(df, &name, Trim::Both)?;
                set_string_column(df, &name, values)?;
            }
        }
    }
    if let Some(taetord) = col(domain, "TAETORD") {
        if has_column(df, &taetord) {
            let values = numeric_column_f64(df, &taetord)?;
            set_f64_column(df, &taetord, values)?;
            sort_by_numeric(df, &taetord)?;
        }
    }
    Ok(())
}
