//! Demographics (DM) domain processor.
//!
//! Processes DM domain data per SDTMIG v3.4 Section 6.3.1.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    col, compute_study_day, has_column, normalize_ct_columns, numeric_column_f64, set_f64_column,
    set_string_column, string_column,
};

pub(super) fn process_dm(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    if let Some(age) = col(domain, "AGE")
        && has_column(df, age)
    {
        let values = numeric_column_f64(df, age)?;
        set_f64_column(df, age, values)?;
    }

    // Age unit normalization via CT
    normalize_ct_columns(domain, df, context, "AGEU", &["AGEU"])?;

    if let Some(country) = col(domain, "COUNTRY")
        && has_column(df, country)
    {
        let values = string_column(df, country)?;
        set_string_column(df, country, values)?;
    }
    // Ethnicity normalization via CT (Codelist C66790)
    normalize_ct_columns(domain, df, context, "ETHNIC", &["ETHNIC"])?;
    // Race normalization via CT (Codelist C74457)
    normalize_ct_columns(domain, df, context, "RACE", &["RACE"])?;
    // Sex normalization via CT (Codelist C66731)
    normalize_ct_columns(domain, df, context, "SEX", &["SEX"])?;

    for date_col in [
        "RFICDTC", "RFSTDTC", "RFENDTC", "RFXSTDTC", "RFXENDTC", "DMDTC",
    ] {
        if let Some(name) = col(domain, date_col)
            && has_column(df, name)
        {
            let values = string_column(df, name)?;
            set_string_column(df, name, values)?;
        }
    }
    if let (Some(dmdtc), Some(dmdy), Some(rfstdtc)) = (
        col(domain, "DMDTC"),
        col(domain, "DMDY"),
        col(domain, "RFSTDTC"),
    ) && has_column(df, dmdtc)
        && has_column(df, rfstdtc)
    {
        compute_study_day(domain, df, dmdtc, dmdy, context, "RFSTDTC")?;
    }
    Ok(())
}
