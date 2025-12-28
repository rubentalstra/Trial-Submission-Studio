use anyhow::Result;
use polars::prelude::DataFrame;
use sdtm_model::Domain;

use crate::processing_context::ProcessingContext;

use super::common::*;

pub(super) fn process_cm(
    domain: &Domain,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    drop_placeholder_rows(domain, df, ctx)?;
    if let Some(cmdosu) = col(domain, "CMDOSU")
        && has_column(df, &cmdosu)
    {
        let values = string_column(df, &cmdosu)?
            .into_iter()
            .map(|v| v.to_lowercase())
            .collect();
        set_string_column(df, &cmdosu, values)?;
    }
    if let Some(cmdur) = col(domain, "CMDUR")
        && has_column(df, &cmdur)
    {
        let values = string_column(df, &cmdur)?;
        set_string_column(df, &cmdur, values)?;
    }
    let key_cols = ["USUBJID", "CMTRT", "CMSTDTC", "CMENDTC"]
        .into_iter()
        .filter_map(|name| col(domain, name))
        .filter(|name| has_column(df, name))
        .collect::<Vec<_>>();
    if !key_cols.is_empty() {
        deduplicate(df, &key_cols)?;
    } else {
        deduplicate(df, &df.get_column_names_owned())?;
    }
    if let Some(cmdostxt) = col(domain, "CMDOSTXT")
        && has_column(df, &cmdostxt)
    {
        let values = string_column(df, &cmdostxt)?
            .into_iter()
            .map(|value| {
                let trimmed = value.trim();
                if is_numeric_string(trimmed) {
                    format!("DOSE {}", trimmed)
                } else {
                    trimmed.to_string()
                }
            })
            .collect();
        set_string_column(df, &cmdostxt, values)?;
    }
    if let Some(cmstat) = col(domain, "CMSTAT") {
        let stat_map = map_values([
            ("NOT DONE", "NOT DONE"),
            ("ND", "NOT DONE"),
            ("", ""),
            ("NAN", ""),
        ]);
        apply_map_upper(df, Some(&cmstat), &stat_map)?;
    }
    if let Some(cmdosfrq) = col(domain, "CMDOSFRQ")
        && has_column(df, &cmdosfrq)
    {
        let freq_map = map_values([
            ("ONCE", "ONCE"),
            ("QD", "QD"),
            ("BID", "BID"),
            ("TID", "TID"),
            ("QID", "QID"),
            ("QOD", "QOD"),
            ("QW", "QW"),
            ("QM", "QM"),
            ("PRN", "PRN"),
            ("DAILY", "QD"),
            ("TWICE DAILY", "BID"),
            ("TWICE PER DAY", "BID"),
            ("THREE TIMES DAILY", "TID"),
            ("ONCE DAILY", "QD"),
            ("AS NEEDED", "PRN"),
            ("", ""),
            ("NAN", ""),
        ]);
        apply_map_upper(df, Some(&cmdosfrq), &freq_map)?;
    }
    if let Some(cmroute) = col(domain, "CMROUTE")
        && has_column(df, &cmroute)
    {
        let route_map = map_values([
            ("ORAL", "ORAL"),
            ("PO", "ORAL"),
            ("INTRAVENOUS", "INTRAVENOUS"),
            ("IV", "INTRAVENOUS"),
            ("INTRAMUSCULAR", "INTRAMUSCULAR"),
            ("IM", "INTRAMUSCULAR"),
            ("SUBCUTANEOUS", "SUBCUTANEOUS"),
            ("SC", "SUBCUTANEOUS"),
            ("SUBQ", "SUBCUTANEOUS"),
            ("TOPICAL", "TOPICAL"),
            ("TRANSDERMAL", "TRANSDERMAL"),
            ("INHALATION", "INHALATION"),
            ("INHALED", "INHALATION"),
            ("RECTAL", "RECTAL"),
            ("VAGINAL", "VAGINAL"),
            ("OPHTHALMIC", "OPHTHALMIC"),
            ("OTIC", "OTIC"),
            ("NASAL", "NASAL"),
            ("", ""),
            ("NAN", ""),
        ]);
        apply_map_upper(df, Some(&cmroute), &route_map)?;
    }
    if let Some(cmdosu) = col(domain, "CMDOSU")
        && has_column(df, &cmdosu)
    {
        let values = string_column(df, &cmdosu)?
            .into_iter()
            .map(|value| replace_unknown(&value, ""))
            .collect();
        set_string_column(df, &cmdosu, values)?;
    }
    if let Some(cmstdtc) = col(domain, "CMSTDTC")
        && let Some(cmstdy) = col(domain, "CMSTDY")
    {
        compute_study_day(domain, df, &cmstdtc, &cmstdy, ctx, "RFSTDTC")?;
    }
    if let Some(cmendtc) = col(domain, "CMENDTC")
        && let Some(cmendy) = col(domain, "CMENDY")
    {
        compute_study_day(domain, df, &cmendtc, &cmendy, ctx, "RFSTDTC")?;
    }
    deduplicate(df, &df.get_column_names_owned())?;
    Ok(())
}
