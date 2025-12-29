//! Adverse Events (AE) domain processor.
//!
//! Processes AE domain data per SDTMIG v3.4 Section 6.3.2.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    apply_map_upper, backward_fill_var, col, compute_study_day, ensure_date_pair_order, has_column,
    normalize_ct_columns, numeric_column_i64, resolve_ct_value, set_i64_column, set_string_column,
    string_column, yn_mapping,
};

pub(super) fn process_ae(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    if let Some(aedur) = col(domain, "AEDUR")
        && has_column(df, aedur)
    {
        let values = string_column(df, aedur)?;
        set_string_column(df, aedur, values)?;
    }
    for visit_col in ["VISIT", "VISITNUM"] {
        if let Some(name) = col(domain, visit_col)
            && has_column(df, name)
        {
            let values = string_column(df, name)?;
            set_string_column(df, name, values)?;
        }
    }
    if let Some(start) = col(domain, "AESTDTC") {
        ensure_date_pair_order(df, start, col(domain, "AEENDTC"))?;
        if let Some(end) = col(domain, "AEENDTC")
            && has_column(df, end)
        {
            let end_vals = string_column(df, end)?;
            set_string_column(df, end, end_vals)?;
        }
        if let Some(aestdy) = col(domain, "AESTDY") {
            compute_study_day(domain, df, start, aestdy, context, "RFSTDTC")?;
        }
        if let Some(aeend) = col(domain, "AEENDTC")
            && let Some(aeendy) = col(domain, "AEENDY")
        {
            compute_study_day(domain, df, aeend, aeendy, context, "RFSTDTC")?;
        }
    }
    if let Some(teae) = col(domain, "TEAE")
        && has_column(df, teae)
    {
        df.drop_in_place(teae)?;
    }
    // Backward fill: AETERM → AEDECOD
    backward_fill_var(domain, df, "AETERM", "AEDECOD")?;

    // Normalize via CT (dynamic, using synonyms from CT files)
    // AEACN: Codelist C66767 (Action Taken with Study Treatment)
    normalize_ct_columns(domain, df, context, "AEACN", &["AEACN"])?;
    // AESER: Codelist C66742 (No Yes Response)
    normalize_ct_columns(domain, df, context, "AESER", &["AESER"])?;
    // AEREL: Codelist C66766 (Causality)
    normalize_ct_columns(domain, df, context, "AEREL", &["AEREL"])?;
    // AEOUT: Codelist C66768 (Outcome of Adverse Event)
    normalize_ct_columns(domain, df, context, "AEOUT", &["AEOUT"])?;
    // AESEV: Codelist C66769 (Severity/Intensity Scale for Adverse Events)
    normalize_ct_columns(domain, df, context, "AESEV", &["AESEV"])?;

    for code in [
        "AEPTCD", "AEHLGTCD", "AEHLTCD", "AELLTCD", "AESOCCD", "AEBDSYCD",
    ] {
        if let Some(name) = col(domain, code) {
            let values = numeric_column_i64(df, name)?;
            set_i64_column(df, name, values)?;
        }
    }

    // Apply Y/N mapping to AESINTV
    if let Some(aesintv) = col(domain, "AESINTV") {
        apply_map_upper(df, Some(aesintv), &yn_mapping())?;
    }

    if let Some(aeacndev) = col(domain, "AEACNDEV")
        && has_column(df, aeacndev)
    {
        let ct_dev = context.resolve_ct(domain, "AEACNDEV");
        let ct_acn = context.resolve_ct(domain, "AEACN");
        let aeacn_col = col(domain, "AEACN").filter(|&name| has_column(df, name));
        let aeacnoth_col = col(domain, "AEACNOTH").filter(|&name| has_column(df, name));
        if ct_dev.is_some() {
            let mut dev_vals = string_column(df, aeacndev)?;
            let mut acn_vals = aeacn_col
                .map(|name| string_column(df, name))
                .transpose()?
                .unwrap_or_else(|| vec![String::new(); df.height()]);
            let mut oth_vals = aeacnoth_col
                .map(|name| string_column(df, name))
                .transpose()?
                .unwrap_or_else(|| vec![String::new(); df.height()]);
            for idx in 0..df.height() {
                if dev_vals[idx].trim().is_empty() {
                    continue;
                }
                let dev_valid = ct_dev
                    .and_then(|ct| {
                        resolve_ct_value(ct, &dev_vals[idx], context.options.ct_matching)
                    })
                    .is_some();
                if dev_valid {
                    continue;
                }
                let moved_to_acn = ct_acn
                    .and_then(|ct| {
                        resolve_ct_value(ct, &dev_vals[idx], context.options.ct_matching)
                    })
                    .map(|_| {
                        if acn_vals[idx].trim().is_empty() {
                            acn_vals[idx] = dev_vals[idx].clone();
                        }
                        true
                    })
                    .unwrap_or(false);
                if !moved_to_acn && oth_vals[idx].trim().is_empty() {
                    oth_vals[idx] = dev_vals[idx].clone();
                }
                dev_vals[idx].clear();
            }
            set_string_column(df, aeacndev, dev_vals)?;
            if let Some(name) = aeacn_col {
                set_string_column(df, name, acn_vals)?;
            }
            if let Some(name) = aeacnoth_col {
                set_string_column(df, name, oth_vals)?;
            }
        }
    }

    for visit_col in ["VISIT", "VISITNUM"] {
        if let Some(name) = col(domain, visit_col)
            && has_column(df, name)
        {
            df.drop_in_place(name)?;
        }
    }
    Ok(())
}
