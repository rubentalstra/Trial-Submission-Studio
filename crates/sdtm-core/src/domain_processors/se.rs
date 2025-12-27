use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

use super::common::*;

pub(super) fn process_se(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    for col_name in [
        "STUDYID", "DOMAIN", "USUBJID", "ETCD", "ELEMENT", "EPOCH", "SESTDTC", "SEENDTC",
    ] {
        if let Some(name) = col(domain, col_name)
            && has_column(df, &name)
        {
            let values = string_column(df, &name, Trim::Both)?;
            set_string_column(df, &name, values)?;
        }
    }
    if let Some(sestdtc) = col(domain, "SESTDTC") {
        ensure_date_pair_order(df, &sestdtc, col(domain, "SEENDTC").as_deref())?;
        if let Some(sestdy) = col(domain, "SESTDY") {
            compute_study_day(domain, df, &sestdtc, &sestdy, ctx, "RFSTDTC")?;
        }
    }
    if let Some(seendtc) = col(domain, "SEENDTC")
        && let Some(seendy) = col(domain, "SEENDY")
    {
        compute_study_day(domain, df, &seendtc, &seendy, ctx, "RFSTDTC")?;
    }
    Ok(())
}
