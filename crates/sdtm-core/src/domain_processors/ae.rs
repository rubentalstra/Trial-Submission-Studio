use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

use super::common::*;

pub(super) fn process_ae(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    if let Some(aedur) = col(domain, "AEDUR") {
        if has_column(df, &aedur) {
            let values = string_column(df, &aedur, Trim::Both)?;
            set_string_column(df, &aedur, values)?;
        }
    }
    for visit_col in ["VISIT", "VISITNUM"] {
        if let Some(name) = col(domain, visit_col) {
            if has_column(df, &name) {
                let values = string_column(df, &name, Trim::Both)?;
                set_string_column(df, &name, values)?;
            }
        }
    }
    if let Some(start) = col(domain, "AESTDTC") {
        ensure_date_pair_order(df, &start, col(domain, "AEENDTC").as_deref())?;
        if let Some(end) = col(domain, "AEENDTC") {
            if has_column(df, &end) {
                let end_vals = string_column(df, &end, Trim::Both)?;
                set_string_column(df, &end, end_vals)?;
            }
        }
        if let Some(aestdy) = col(domain, "AESTDY") {
            compute_study_day(domain, df, &start, &aestdy, ctx, "RFSTDTC")?;
        }
        if let Some(aeend) = col(domain, "AEENDTC") {
            if let Some(aeendy) = col(domain, "AEENDY") {
                compute_study_day(domain, df, &aeend, &aeendy, ctx, "RFSTDTC")?;
            }
        }
    }
    if let Some(teae) = col(domain, "TEAE") {
        if has_column(df, &teae) {
            df.drop_in_place(&teae)?;
        }
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

    if let (Some(seq), Some(usubjid)) = (col(domain, "AESEQ"), col(domain, "USUBJID")) {
        assign_sequence(df, &seq, &usubjid)?;
    }

    if let Some(aesintv) = col(domain, "AESINTV") {
        if has_column(df, &aesintv) {
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
    }

    for visit_col in ["VISIT", "VISITNUM"] {
        if let Some(name) = col(domain, visit_col) {
            if has_column(df, &name) {
                df.drop_in_place(&name)?;
            }
        }
    }
    Ok(())
}
