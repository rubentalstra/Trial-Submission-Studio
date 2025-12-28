use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::*;

pub(super) fn process_mh(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    if let Some(mhseq) = col(domain, "MHSEQ")
        && has_column(df, mhseq)
    {
        let values = numeric_column_f64(df, mhseq)?;
        set_f64_column(df, mhseq, values)?;
    }
    if let Some(mhterm) = col(domain, "MHTERM")
        && has_column(df, mhterm)
    {
        let mut terms = string_column(df, mhterm)?;
        if let Some(mhdecod) = col(domain, "MHDECOD")
            && has_column(df, mhdecod)
        {
            let decods = string_column(df, mhdecod)?;
            for idx in 0..df.height() {
                if terms[idx].is_empty() && !decods[idx].is_empty() {
                    terms[idx] = decods[idx].clone();
                }
            }
        }
        set_string_column(df, mhterm, terms)?;
    }
    for col_name in ["MHSTDTC", "MHENDTC", "MHDTC"] {
        if let Some(name) = col(domain, col_name)
            && has_column(df, name)
        {
            let values = string_column(df, name)?
                .into_iter()
                .map(|value| normalize_iso8601(&value))
                .collect();
            set_string_column(df, name, values)?;
        }
    }
    if let Some(mhenrf) = col(domain, "MHENRF")
        && has_column(df, mhenrf)
    {
        let values = string_column(df, mhenrf)?
            .into_iter()
            .map(|value| {
                let upper = value.to_uppercase();
                match upper.as_str() {
                    "Y" | "YES" | "TRUE" | "1" => "ONGOING".to_string(),
                    "N" | "NO" | "FALSE" | "0" => "".to_string(),
                    "PRIOR" => "BEFORE".to_string(),
                    "POST" => "AFTER".to_string(),
                    "CONCURRENT" => "COINCIDENT".to_string(),
                    "UNK" | "U" => "UNKNOWN".to_string(),
                    _ => upper,
                }
            })
            .collect();
        set_string_column(df, mhenrf, values)?;
    }
    if let (Some(mhdtc), Some(mhdy)) = (col(domain, "MHDTC"), col(domain, "MHDY"))
        && has_column(df, mhdtc)
    {
        compute_study_day(domain, df, mhdtc, mhdy, context, "RFSTDTC")?;
        let values = numeric_column_f64(df, mhdy)?;
        set_f64_column(df, mhdy, values)?;
    }

    // Normalize CT columns
    // MHCAT: Category
    normalize_ct_columns(domain, df, context, "MHCAT", &["MHCAT"])?;
    // MHSCAT: Subcategory
    normalize_ct_columns(domain, df, context, "MHSCAT", &["MHSCAT"])?;
    // MHENRF: End Relative to Reference Period (Codelist C66728)
    normalize_ct_columns(domain, df, context, "MHENRF", &["MHENRF"])?;
    // MHSTRF: Start Relative to Reference Period (Codelist C66728)
    normalize_ct_columns(domain, df, context, "MHSTRF", &["MHSTRF"])?;
    // MHPRESP: Pre-specified (Codelist C66742)
    normalize_ct_columns(domain, df, context, "MHPRESP", &["MHPRESP"])?;
    // MHOCCUR: Occurrence (Codelist C66742)
    normalize_ct_columns(domain, df, context, "MHOCCUR", &["MHOCCUR"])?;

    Ok(())
}
