use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

use super::common::*;

pub(super) fn process_ie(domain: &Domain, df: &mut DataFrame, ctx: &ProcessingContext) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    for col_name in [
        "IEORRES",
        "IESTRESC",
        "IETESTCD",
        "IETEST",
        "IECAT",
        "IESCAT",
        "EPOCH",
    ] {
        if let Some(name) = col(domain, col_name) {
            if has_column(df, &name) {
                let values = string_column(df, &name, Trim::Both)?;
                set_string_column(df, &name, values)?;
            }
        }
    }
    if let Some(ieorres) = col(domain, "IEORRES") {
        let yn_map = map_values([
            ("YES", "Y"),
            ("Y", "Y"),
            ("1", "Y"),
            ("TRUE", "Y"),
            ("NO", "N"),
            ("N", "N"),
            ("0", "N"),
            ("FALSE", "N"),
            ("", ""),
        ]);
        apply_map_upper(df, Some(&ieorres), &yn_map)?;
    }
    if let (Some(ieorres), Some(iestresc)) = (col(domain, "IEORRES"), col(domain, "IESTRESC")) {
        if has_column(df, &ieorres) && has_column(df, &iestresc) {
            let orres = string_column(df, &ieorres, Trim::Both)?;
            let mut stresc = string_column(df, &iestresc, Trim::Both)?;
            for idx in 0..df.height() {
                if stresc[idx].is_empty() && !orres[idx].is_empty() {
                    stresc[idx] = orres[idx].clone();
                }
            }
            set_string_column(df, &iestresc, stresc)?;
        }
    }
    if let (Some(iedtc), Some(iedy)) = (col(domain, "IEDTC"), col(domain, "IEDY")) {
        if has_column(df, &iedtc) {
            let values = string_column(df, &iedtc, Trim::Both)?;
            set_string_column(df, &iedtc, values)?;
            compute_study_day(domain, df, &iedtc, &iedy, ctx, "RFSTDTC")?;
            let numeric = numeric_column_f64(df, &iedy)?;
            set_f64_column(df, &iedy, numeric)?;
        }
    }
    if let (Some(ieseq), Some(usubjid)) = (col(domain, "IESEQ"), col(domain, "USUBJID")) {
        assign_sequence(df, &ieseq, &usubjid)?;
    }
    Ok(())
}
