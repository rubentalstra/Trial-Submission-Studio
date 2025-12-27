use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

use super::common::*;

pub(super) fn process_dm(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    if let Some(age) = col(domain, "AGE")
        && has_column(df, &age)
    {
        let values = numeric_column_f64(df, &age)?;
        set_f64_column(df, &age, values)?;
    }
    if let Some(ageu) = col(domain, "AGEU")
        && has_column(df, &ageu)
    {
        let values = string_column(df, &ageu, Trim::Both)?
            .into_iter()
            .map(|value| {
                let upper = value.to_uppercase();
                match upper.as_str() {
                    "YEAR" | "YRS" | "Y" => "YEARS".to_string(),
                    _ => upper,
                }
            })
            .collect();
        set_string_column(df, &ageu, values)?;
    }
    if let Some(country) = col(domain, "COUNTRY")
        && has_column(df, &country)
    {
        let values = string_column(df, &country, Trim::Both)?;
        set_string_column(df, &country, values)?;
    }
    if let Some(ethnic) = col(domain, "ETHNIC")
        && has_column(df, &ethnic)
    {
        let values = string_column(df, &ethnic, Trim::Both)?
            .into_iter()
            .map(|value| {
                let upper = value.to_uppercase();
                if upper == "UNK" {
                    "UNKNOWN".to_string()
                } else {
                    upper
                }
            })
            .collect();
        set_string_column(df, &ethnic, values)?;
    }
    if let Some(race) = col(domain, "RACE")
        && has_column(df, &race)
    {
        let values = string_column(df, &race, Trim::Both)?
            .into_iter()
            .map(|value| {
                let upper = value.to_uppercase();
                match upper.as_str() {
                    "WHITE, CAUCASIAN, OR ARABIC" | "CAUCASIAN" => "WHITE".to_string(),
                    "BLACK" | "AFRICAN AMERICAN" => "BLACK OR AFRICAN AMERICAN".to_string(),
                    "UNK" => "UNKNOWN".to_string(),
                    _ => upper,
                }
            })
            .collect();
        set_string_column(df, &race, values)?;
    }
    if let Some(sex) = col(domain, "SEX")
        && has_column(df, &sex)
    {
        let values = string_column(df, &sex, Trim::Both)?
            .into_iter()
            .map(|value| {
                let upper = value.to_uppercase();
                match upper.as_str() {
                    "FEMALE" => "F".to_string(),
                    "MALE" => "M".to_string(),
                    "UNKNOWN" | "UNK" => "U".to_string(),
                    _ => upper,
                }
            })
            .collect();
        set_string_column(df, &sex, values)?;
    }
    for date_col in [
        "RFICDTC", "RFSTDTC", "RFENDTC", "RFXSTDTC", "RFXENDTC", "DMDTC",
    ] {
        if let Some(name) = col(domain, date_col)
            && has_column(df, &name)
        {
            let values = string_column(df, &name, Trim::Both)?;
            set_string_column(df, &name, values)?;
        }
    }
    if let (Some(dmdtc), Some(dmdy), Some(rfstdtc)) = (
        col(domain, "DMDTC"),
        col(domain, "DMDY"),
        col(domain, "RFSTDTC"),
    ) && has_column(df, &dmdtc)
        && has_column(df, &rfstdtc)
    {
        compute_study_day(domain, df, &dmdtc, &dmdy, ctx, "RFSTDTC")?;
    }
    Ok(())
}
