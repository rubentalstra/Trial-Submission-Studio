//! SUPP (Supplemental Qualifiers) domain generation.
//!
//! Per SDTM IG 3.4 Section 8.4:
//! - IDVAR = --SEQ (e.g., AESEQ) to link to parent record
//! - Exception: For SUPPDM, IDVAR and IDVARVAL are null (USUBJID alone is key)

use crate::state::{DomainState, SuppAction};
use cdisc_output::types::DomainFrame;
use polars::prelude::*;

/// Check if domain has any columns configured for SUPP.
pub fn has_supp_columns(domain: &DomainState) -> bool {
    domain
        .derived
        .supp
        .as_ref()
        .map(|supp| {
            supp.columns
                .values()
                .any(|cfg| cfg.action == SuppAction::AddToSupp)
        })
        .unwrap_or(false)
}

/// Count SUPP columns for a domain.
pub fn count_supp_columns(domain: &DomainState) -> usize {
    domain
        .derived
        .supp
        .as_ref()
        .map(|supp| {
            supp.columns
                .values()
                .filter(|cfg| cfg.action == SuppAction::AddToSupp)
                .count()
        })
        .unwrap_or(0)
}

/// Estimate the number of SUPP rows for display purposes.
/// Returns the count of non-null values across all SUPP columns.
///
/// Note: SUPP columns are unmapped SOURCE columns, so we look in source.data,
/// not the preview DataFrame (which contains transformed SDTM columns).
pub fn estimate_supp_row_count(domain: &DomainState) -> usize {
    let Some(supp_config) = &domain.derived.supp else {
        return 0;
    };

    // Use SOURCE data, not preview - SUPP columns are unmapped source columns
    let source_data = &domain.source.data;

    let mut count = 0;
    for (col_name, cfg) in &supp_config.columns {
        if cfg.action != SuppAction::AddToSupp {
            continue;
        }

        if let Ok(series) = source_data.column(col_name) {
            // Count non-null values
            count += series.len() - series.null_count();
        }
    }

    count
}

/// Build SUPP dataset from configured columns.
///
/// Per SDTM IG 3.4 Section 8.4:
/// - IDVAR = --SEQ (e.g., AESEQ) to link to parent record
/// - Exception: For SUPPDM, IDVAR and IDVARVAL are null (USUBJID alone is key)
///
/// SUPP structure (vertical format):
/// STUDYID, RDOMAIN, USUBJID, IDVAR, IDVARVAL, QNAM, QLABEL, QVAL, QORIG, QEVAL
///
/// Note: QVAL comes from SOURCE data (unmapped columns), while key columns
/// (STUDYID, USUBJID, --SEQ) come from the preview/parent data.
pub fn build_supp_frame(
    domain_code: &str,
    domain: &DomainState,
    parent_data: &DataFrame,
) -> Option<DomainFrame> {
    let supp_config = domain.derived.supp.as_ref()?;

    let supp_columns: Vec<_> = supp_config
        .columns
        .iter()
        .filter(|(_, cfg)| cfg.action == SuppAction::AddToSupp)
        .collect();

    if supp_columns.is_empty() {
        return None;
    }

    // Source data contains the unmapped columns for QVAL
    let source_data = &domain.source.data;

    // Get key columns from parent/preview data (SDTM columns)
    let studyid_series = parent_data.column("STUDYID").ok();
    let usubjid_series = parent_data.column("USUBJID").ok();

    // For DM domain, IDVAR is null; for others, use --SEQ
    let is_dm = domain_code == "DM";
    let seq_var_name = format!("{}SEQ", domain_code);
    let seq_series = if is_dm {
        None
    } else {
        parent_data.column(&seq_var_name).ok()
    };

    // Build SUPP rows
    let mut studyid_col: Vec<String> = Vec::new();
    let mut rdomain_col: Vec<String> = Vec::new();
    let mut usubjid_col: Vec<String> = Vec::new();
    let mut idvar_col: Vec<String> = Vec::new();
    let mut idvarval_col: Vec<String> = Vec::new();
    let mut qnam_col: Vec<String> = Vec::new();
    let mut qlabel_col: Vec<String> = Vec::new();
    let mut qval_col: Vec<String> = Vec::new();
    let mut qorig_col: Vec<String> = Vec::new();
    let mut qeval_col: Vec<String> = Vec::new();

    for (col_name, config) in &supp_columns {
        // QVAL comes from SOURCE data (unmapped columns)
        let Ok(values) = source_data.column(*col_name) else {
            continue;
        };

        for idx in 0..values.len() {
            let value = values.get(idx).ok();

            // Skip null values
            if value.as_ref().map(|v| v.is_null()).unwrap_or(true) {
                continue;
            }

            // STUDYID
            let studyid = studyid_series
                .as_ref()
                .and_then(|s| s.get(idx).ok())
                .map(|v| format_anyvalue(&v))
                .unwrap_or_default();
            studyid_col.push(studyid);

            // RDOMAIN
            rdomain_col.push(domain_code.to_string());

            // USUBJID
            let usubjid = usubjid_series
                .as_ref()
                .and_then(|s| s.get(idx).ok())
                .map(|v| format_anyvalue(&v))
                .unwrap_or_default();
            usubjid_col.push(usubjid);

            // IDVAR/IDVARVAL: null for DM, --SEQ for others
            if is_dm {
                idvar_col.push(String::new());
                idvarval_col.push(String::new());
            } else {
                idvar_col.push(seq_var_name.clone());
                let seq_val = seq_series
                    .as_ref()
                    .and_then(|s| s.get(idx).ok())
                    .map(|v| format_anyvalue(&v))
                    .unwrap_or_default();
                idvarval_col.push(seq_val);
            }

            // QNAM, QLABEL from config
            qnam_col.push(config.qnam.clone());
            qlabel_col.push(config.qlabel.clone());

            // QVAL - the actual value
            qval_col.push(format_anyvalue(&value.unwrap()));

            // QORIG - default to "CRF"
            qorig_col.push("CRF".to_string());

            // QEVAL - typically empty
            qeval_col.push(String::new());
        }
    }

    if qval_col.is_empty() {
        return None;
    }

    // Build DataFrame
    let df = DataFrame::new(vec![
        Column::new("STUDYID".into(), studyid_col),
        Column::new("RDOMAIN".into(), rdomain_col),
        Column::new("USUBJID".into(), usubjid_col),
        Column::new("IDVAR".into(), idvar_col),
        Column::new("IDVARVAL".into(), idvarval_col),
        Column::new("QNAM".into(), qnam_col),
        Column::new("QLABEL".into(), qlabel_col),
        Column::new("QVAL".into(), qval_col),
        Column::new("QORIG".into(), qorig_col),
        Column::new("QEVAL".into(), qeval_col),
    ])
    .ok()?;

    // Create DomainFrame with lowercase filename (per SDTM IG 3.4)
    Some(DomainFrame::with_dataset_name(
        domain_code.to_string(),
        df,
        format!("supp{}", domain_code.to_lowercase()),
    ))
}

/// Format an AnyValue to a string for QVAL.
fn format_anyvalue(value: &AnyValue) -> String {
    match value {
        AnyValue::Null => String::new(),
        AnyValue::String(s) => s.to_string(),
        AnyValue::Int8(n) => n.to_string(),
        AnyValue::Int16(n) => n.to_string(),
        AnyValue::Int32(n) => n.to_string(),
        AnyValue::Int64(n) => n.to_string(),
        AnyValue::UInt8(n) => n.to_string(),
        AnyValue::UInt16(n) => n.to_string(),
        AnyValue::UInt32(n) => n.to_string(),
        AnyValue::UInt64(n) => n.to_string(),
        AnyValue::Float32(n) => n.to_string(),
        AnyValue::Float64(n) => n.to_string(),
        AnyValue::Boolean(b) => if *b { "Y" } else { "N" }.to_string(),
        AnyValue::StringOwned(s) => s.to_string(),
        _ => format!("{}", value),
    }
}
