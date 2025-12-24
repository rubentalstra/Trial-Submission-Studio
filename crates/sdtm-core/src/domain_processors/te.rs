use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

use super::common::*;

pub(super) fn process_te(domain: &Domain, df: &mut DataFrame, ctx: &ProcessingContext) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    for col_name in ["STUDYID", "DOMAIN", "ETCD", "ELEMENT", "TESTRL", "TEENRL"] {
        if let Some(name) = col(domain, col_name) {
            if has_column(df, &name) {
                let values = string_column(df, &name, Trim::Both)?;
                set_string_column(df, &name, values)?;
            }
        }
    }
    Ok(())
}
