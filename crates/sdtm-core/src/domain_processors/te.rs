use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{col, has_column, set_string_column, string_column};

pub(super) fn process_te(
    domain: &Domain,
    df: &mut DataFrame,
    _context: &PipelineContext,
) -> Result<()> {
    for col_name in ["STUDYID", "DOMAIN", "ETCD", "ELEMENT", "TESTRL", "TEENRL"] {
        if let Some(name) = col(domain, col_name)
            && has_column(df, name)
        {
            let values = string_column(df, name)?;
            set_string_column(df, name, values)?;
        }
    }
    Ok(())
}
