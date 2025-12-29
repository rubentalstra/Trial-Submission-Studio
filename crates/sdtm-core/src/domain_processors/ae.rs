//! Adverse Events (AE) domain processor.
//!
//! Processes AE domain data per SDTMIG v3.4 Section 6.3.2.

use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::{
    apply_map_upper, backward_fill_var, col, compute_study_days_batch, ensure_date_pair_order,
    has_column, normalize_ct_batch, normalize_numeric_i64, resolve_ct_value, set_string_column,
    string_column, trim_columns, yn_mapping,
};

pub(super) fn process_ae(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    // Trim string columns
    trim_columns(domain, df, &["AEDUR", "VISIT", "VISITNUM"])?;

    // Date pair validation and study day computation
    if let Some(start) = col(domain, "AESTDTC") {
        ensure_date_pair_order(df, start, col(domain, "AEENDTC"))?;
        trim_columns(domain, df, &["AEENDTC"])?;
    }
    compute_study_days_batch(
        domain,
        df,
        context,
        &[("AESTDTC", "AESTDY"), ("AEENDTC", "AEENDY")],
    )?;

    // Drop TEAE column if present
    if let Some(teae) = col(domain, "TEAE")
        && has_column(df, teae)
    {
        df.drop_in_place(teae)?;
    }

    // Backward fill: AETERM → AEDECOD
    backward_fill_var(domain, df, "AETERM", "AEDECOD")?;

    // Batch CT normalization
    normalize_ct_batch(
        domain,
        df,
        context,
        &["AEACN", "AESER", "AEREL", "AEOUT", "AESEV"],
    )?;

    // Normalize MedDRA numeric codes
    normalize_numeric_i64(
        domain,
        df,
        &[
            "AEPTCD", "AEHLGTCD", "AEHLTCD", "AELLTCD", "AESOCCD", "AEBDSYCD",
        ],
    )?;

    // Apply Y/N mapping to AESINTV
    if let Some(aesintv) = col(domain, "AESINTV") {
        apply_map_upper(df, Some(aesintv), &yn_mapping())?;
    }

    // Process AEACNDEV values (relocate invalid values to AEACN or AEACNOTH)
    process_aeacndev(domain, df, context)?;

    // Drop visit columns
    for visit_col in ["VISIT", "VISITNUM"] {
        if let Some(name) = col(domain, visit_col)
            && has_column(df, name)
        {
            df.drop_in_place(name)?;
        }
    }

    Ok(())
}

/// Process AEACNDEV values per SDTMIG v3.4 Section 6.3.2.
///
/// Invalid AEACNDEV values are relocated:
/// - If valid in AEACN codelist → move to AEACN (if empty)
/// - Otherwise → move to AEACNOTH (if empty)
fn process_aeacndev(domain: &Domain, df: &mut DataFrame, context: &PipelineContext) -> Result<()> {
    let aeacndev = match col(domain, "AEACNDEV") {
        Some(name) if has_column(df, name) => name,
        _ => return Ok(()),
    };

    let ct_dev = match context.resolve_ct(domain, "AEACNDEV") {
        Some(ct) => ct,
        None => return Ok(()),
    };

    let ct_acn = context.resolve_ct(domain, "AEACN");
    let aeacn_col = col(domain, "AEACN").filter(|&name| has_column(df, name));
    let aeacnoth_col = col(domain, "AEACNOTH").filter(|&name| has_column(df, name));

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

        // Check if valid in AEACNDEV codelist
        if resolve_ct_value(ct_dev, &dev_vals[idx], context.options.ct_matching).is_some() {
            continue;
        }

        // Try to move to AEACN if valid there
        let moved = ct_acn
            .and_then(|ct| resolve_ct_value(ct, &dev_vals[idx], context.options.ct_matching))
            .map(|_| {
                if acn_vals[idx].trim().is_empty() {
                    acn_vals[idx] = dev_vals[idx].clone();
                }
            })
            .is_some();

        // Otherwise move to AEACNOTH if empty
        if !moved && oth_vals[idx].trim().is_empty() {
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

    Ok(())
}
