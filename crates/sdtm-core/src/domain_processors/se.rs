use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    col, compute_study_day, ensure_date_pair_order, has_column, set_string_column, string_column,
};

pub(super) fn process_se(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    for col_name in [
        "STUDYID", "DOMAIN", "USUBJID", "ETCD", "ELEMENT", "EPOCH", "SESTDTC", "SEENDTC",
    ] {
        if let Some(name) = col(domain, col_name)
            && has_column(df, name)
        {
            let values = string_column(df, name)?;
            set_string_column(df, name, values)?;
        }
    }
    if let Some(sestdtc) = col(domain, "SESTDTC") {
        ensure_date_pair_order(df, sestdtc, col(domain, "SEENDTC"))?;
        if let Some(sestdy) = col(domain, "SESTDY") {
            compute_study_day(domain, df, sestdtc, sestdy, context, "RFSTDTC")?;
        }
    }
    if let Some(seendtc) = col(domain, "SEENDTC")
        && let Some(seendy) = col(domain, "SEENDY")
    {
        compute_study_day(domain, df, seendtc, seendy, context, "RFSTDTC")?;
    }
    Ok(())
}
