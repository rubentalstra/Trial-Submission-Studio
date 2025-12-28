use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::pipeline_context::PipelineContext;

use super::common::*;

pub(super) fn process_ae(
    domain: &Domain,
    df: &mut DataFrame,
    context: &PipelineContext,
) -> Result<()> {
    if let Some(aedur) = col(domain, "AEDUR")
        && has_column(df, &aedur)
    {
        let values = string_column(df, &aedur)?;
        set_string_column(df, &aedur, values)?;
    }
    for visit_col in ["VISIT", "VISITNUM"] {
        if let Some(name) = col(domain, visit_col)
            && has_column(df, &name)
        {
            let values = string_column(df, &name)?;
            set_string_column(df, &name, values)?;
        }
    }
    if let Some(start) = col(domain, "AESTDTC") {
        ensure_date_pair_order(df, &start, col(domain, "AEENDTC").as_deref())?;
        if let Some(end) = col(domain, "AEENDTC")
            && has_column(df, &end)
        {
            let end_vals = string_column(df, &end)?;
            set_string_column(df, &end, end_vals)?;
        }
        if let Some(aestdy) = col(domain, "AESTDY") {
            compute_study_day(domain, df, &start, &aestdy, context, "RFSTDTC")?;
        }
        if let Some(aeend) = col(domain, "AEENDTC")
            && let Some(aeendy) = col(domain, "AEENDY")
        {
            compute_study_day(domain, df, &aeend, &aeendy, context, "RFSTDTC")?;
        }
    }
    if let Some(teae) = col(domain, "TEAE")
        && has_column(df, &teae)
    {
        df.drop_in_place(&teae)?;
    }
    if let (Some(aedecod), Some(aeterm)) = (col(domain, "AEDECOD"), col(domain, "AETERM"))
        && has_column(df, &aedecod)
        && has_column(df, &aeterm)
    {
        let mut decod_vals = string_column(df, &aedecod)?;
        let term_vals = string_column(df, &aeterm)?;
        for idx in 0..df.height() {
            if decod_vals[idx].is_empty() && !term_vals[idx].is_empty() {
                decod_vals[idx] = term_vals[idx].clone();
            }
        }
        set_string_column(df, &aedecod, decod_vals)?;
    }
    apply_map_upper(
        df,
        col(domain, "AEACN").as_deref(),
        &map_values([
            ("NONE", "DOSE NOT CHANGED"),
            ("NO ACTION", "DOSE NOT CHANGED"),
            ("UNK", "UNKNOWN"),
            ("NA", "NOT APPLICABLE"),
            ("N/A", "NOT APPLICABLE"),
        ]),
    )?;
    apply_map_upper(
        df,
        col(domain, "AESER").as_deref(),
        &map_values([
            ("YES", "Y"),
            ("NO", "N"),
            ("1", "Y"),
            ("0", "N"),
            ("TRUE", "Y"),
            ("FALSE", "N"),
        ]),
    )?;
    apply_map_upper(
        df,
        col(domain, "AEREL").as_deref(),
        &map_values([
            ("NO", "NOT RELATED"),
            ("N", "NOT RELATED"),
            ("NOT SUSPECTED", "NOT RELATED"),
            ("UNLIKELY RELATED", "NOT RELATED"),
            ("YES", "RELATED"),
            ("Y", "RELATED"),
            ("POSSIBLY RELATED", "RELATED"),
            ("PROBABLY RELATED", "RELATED"),
            ("SUSPECTED", "RELATED"),
            ("UNK", "UNKNOWN"),
            ("NOT ASSESSED", "UNKNOWN"),
        ]),
    )?;
    apply_map_upper(
        df,
        col(domain, "AEOUT").as_deref(),
        &map_values([
            ("RECOVERED", "RECOVERED/RESOLVED"),
            ("RESOLVED", "RECOVERED/RESOLVED"),
            ("RECOVERED OR RESOLVED", "RECOVERED/RESOLVED"),
            ("RECOVERING", "RECOVERING/RESOLVING"),
            ("RESOLVING", "RECOVERING/RESOLVING"),
            ("NOT RECOVERED", "NOT RECOVERED/NOT RESOLVED"),
            ("NOT RESOLVED", "NOT RECOVERED/NOT RESOLVED"),
            ("UNRESOLVED", "NOT RECOVERED/NOT RESOLVED"),
            (
                "RECOVERED WITH SEQUELAE",
                "RECOVERED/RESOLVED WITH SEQUELAE",
            ),
            ("RESOLVED WITH SEQUELAE", "RECOVERED/RESOLVED WITH SEQUELAE"),
            ("DEATH", "FATAL"),
            ("5", "FATAL"),
            ("GRADE 5", "FATAL"),
            ("UNK", "UNKNOWN"),
            ("U", "UNKNOWN"),
        ]),
    )?;
    apply_map_upper(
        df,
        col(domain, "AESEV").as_deref(),
        &map_values([
            ("1", "MILD"),
            ("GRADE 1", "MILD"),
            ("2", "MODERATE"),
            ("GRADE 2", "MODERATE"),
            ("3", "SEVERE"),
            ("GRADE 3", "SEVERE"),
        ]),
    )?;

    for code in [
        "AEPTCD", "AEHLGTCD", "AEHLTCD", "AELLTCD", "AESOCCD", "AEBDSYCD",
    ] {
        if let Some(name) = col(domain, code) {
            let values = numeric_column_i64(df, &name)?;
            set_i64_column(df, &name, values)?;
        }
    }

    if let Some(aesintv) = col(domain, "AESINTV")
        && has_column(df, &aesintv)
    {
        let yn_map = map_values([
            ("Y", "Y"),
            ("YES", "Y"),
            ("1", "Y"),
            ("TRUE", "Y"),
            ("N", "N"),
            ("NO", "N"),
            ("0", "N"),
            ("FALSE", "N"),
            ("", ""),
            ("NAN", ""),
            ("<NA>", ""),
        ]);
        apply_map_upper(df, Some(&aesintv), &yn_map)?;
    }

    if let Some(aeacndev) = col(domain, "AEACNDEV")
        && has_column(df, &aeacndev)
    {
        let ct_dev = context.resolve_ct(domain, "AEACNDEV");
        let ct_acn = context.resolve_ct(domain, "AEACN");
        let aeacn_col = col(domain, "AEACN").filter(|name| has_column(df, name));
        let aeacnoth_col = col(domain, "AEACNOTH").filter(|name| has_column(df, name));
        if ct_dev.is_some() {
            let mut dev_vals = string_column(df, &aeacndev)?;
            let mut acn_vals = aeacn_col
                .as_ref()
                .map(|name| string_column(df, name))
                .transpose()?
                .unwrap_or_else(|| vec![String::new(); df.height()]);
            let mut oth_vals = aeacnoth_col
                .as_ref()
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
            set_string_column(df, &aeacndev, dev_vals)?;
            if let Some(name) = aeacn_col {
                set_string_column(df, &name, acn_vals)?;
            }
            if let Some(name) = aeacnoth_col {
                set_string_column(df, &name, oth_vals)?;
            }
        }
    }

    for visit_col in ["VISIT", "VISITNUM"] {
        if let Some(name) = col(domain, visit_col)
            && has_column(df, &name)
        {
            df.drop_in_place(&name)?;
        }
    }
    Ok(())
}
