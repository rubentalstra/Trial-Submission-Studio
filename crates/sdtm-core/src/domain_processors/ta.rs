use anyhow::Result;
use polars::prelude::{DataFrame, UInt32Chunked};
use sdtm_model::Domain;

use crate::data_utils::column_value_string;
use crate::pipeline_context::PipelineContext;

use super::common::*;

pub(super) fn process_ta(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    drop_placeholder_rows(domain, df, context)?;
    let key_cols = ["EPOCH", "ARMCD", "ARM", "ETCD"]
        .into_iter()
        .filter_map(|name| col(domain, name))
        .filter(|name| has_column(df, name))
        .collect::<Vec<_>>();
    if !key_cols.is_empty() && df.height() > 0 {
        let mut keep = vec![true; df.height()];
        for (idx, keep_value) in keep.iter_mut().enumerate().take(df.height()) {
            let mut blank = true;
            for col_name in &key_cols {
                let value = column_value_string(df, col_name, idx);
                if !value.trim().is_empty() {
                    blank = false;
                    break;
                }
            }
            if blank {
                *keep_value = false;
            }
        }
        filter_rows(df, &keep)?;
    }
    for col_name in ["EPOCH", "ARMCD", "ARM", "ETCD", "STUDYID", "DOMAIN"] {
        if let Some(name) = col(domain, col_name)
            && has_column(df, &name)
        {
            let values = string_column(df, &name)?;
            set_string_column(df, &name, values)?;
        }
    }
    if let Some(taetord) = col(domain, "TAETORD")
        && has_column(df, &taetord)
    {
        let values = numeric_column_f64(df, &taetord)?;
        set_f64_column(df, &taetord, values)?;
        sort_by_numeric(df, &taetord)?;
    }
    Ok(())
}

fn sort_by_numeric(df: &mut DataFrame, column: &str) -> Result<()> {
    let values = numeric_column_f64(df, column)?;
    let mut indices: Vec<u32> = (0..df.height()).map(|idx| idx as u32).collect();
    indices.sort_by(|a, b| {
        let left = values[*a as usize];
        let right = values[*b as usize];
        left.partial_cmp(&right)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let idx = UInt32Chunked::from_vec("idx".into(), indices);
    *df = df.take(&idx)?;
    Ok(())
}
