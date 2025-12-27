use std::collections::BTreeSet;

use anyhow::Result;
use polars::prelude::{AnyValue, DataFrame, NamedFrom, Series};

use sdtm_ingest::CsvTable;
use sdtm_ingest::any_to_string;
use sdtm_model::{CaseInsensitiveLookup, Domain, MappingConfig};

use crate::ProcessingContext;
use crate::ct_utils::{
    completion_column, ct_column_match, is_yes_no_token, resolve_ct_for_variable,
    resolve_ct_value_from_hint,
};
use crate::data_utils::{
    column_hint_for_domain, fill_string_column, mapping_source_for_target, sanitize_test_code,
    table_column_values, table_label,
};

type DaOrresCandidate = (Option<String>, Option<String>, Option<String>, Vec<String>);

/// Fill missing test fields based on source data and column hints.
///
/// This function performs heuristic inference to populate missing SDTM variables
/// from source column headers and labels. The inference is gated by the
/// `allow_heuristic_inference` option in `ProcessingOptions`.
///
/// # SDTMIG References
///
/// While this function helps populate common test-related variables, the
/// derivations are based on heuristics rather than explicit SDTMIG rules.
/// Sponsors should validate the inferred values against their study metadata.
///
/// # Domains Handled
///
/// - **QS**: QSTEST, QSTESTCD, QSCAT from ORRES column hints
/// - **PE**: PETEST, PETESTCD from ORRES column hints
/// - **DS**: DSDECOD, DSTERM from CT matches and completion columns
/// - **EX**: EXTRT from treatment-related column hints
/// - **DA**: DAORRES, DATEST, DATESTCD, DAORRESU from column patterns
/// - **LB**: LBORRES, LBTEST, LBTESTCD, LBORRESU from column patterns
/// - **VS**: VSORRES, VSTEST, VSTESTCD, VSORRESU from column patterns
///
/// # Arguments
///
/// * `domain` - The domain metadata
/// * `mapping` - The mapping configuration used for this domain
/// * `table` - The source CSV table
/// * `df` - The DataFrame to update
/// * `ctx` - The processing context (contains options)
pub fn fill_missing_test_fields(
    domain: &Domain,
    mapping: &MappingConfig,
    table: &CsvTable,
    df: &mut DataFrame,
    ctx: &ProcessingContext,
) -> Result<()> {
    // Gate heuristic inference behind the option
    // When disabled, only explicit mappings from the mapping config are used
    if !ctx.options.allow_heuristic_inference {
        return Ok(());
    }

    let code = domain.code.to_uppercase();
    let column_lookup = CaseInsensitiveLookup::new(df.get_column_names_owned());
    let column_name = |name: &str| {
        column_lookup
            .get(name)
            .map(|value| value.to_string())
            .unwrap_or_else(|| name.to_string())
    };
    if code == "QS" {
        let orres_source = mapping_source_for_target(mapping, "QSORRES")
            .or_else(|| mapping_source_for_target(mapping, "QSSTRESC"));
        let label_hint = orres_source
            .as_deref()
            .and_then(|col| column_hint_for_domain(table, domain, col))
            .or_else(|| column_hint_for_domain(table, domain, "QSPGARS"))
            .or_else(|| column_hint_for_domain(table, domain, "QSPGARSCD"));
        if let Some((label, allow_raw)) = label_hint {
            let test_code = sanitize_test_code(&label);
            fill_string_column(df, "QSTEST", &label)?;
            fill_string_column(df, "QSTESTCD", &test_code)?;
            if let Some(qscat) = resolve_ct_for_variable(ctx, domain, "QSCAT", &label, allow_raw) {
                fill_string_column(df, "QSCAT", &qscat)?;
            }
        }
    } else if code == "PE" {
        let orres_source = mapping_source_for_target(mapping, "PEORRES")
            .or_else(|| mapping_source_for_target(mapping, "PEORRESSP"));
        let label_hint = orres_source
            .as_deref()
            .and_then(|col| column_hint_for_domain(table, domain, col))
            .or_else(|| column_hint_for_domain(table, domain, "PEORRES"))
            .or_else(|| column_hint_for_domain(table, domain, "PEORRESSP"));
        if let Some((label, _allow_raw)) = label_hint {
            let test_code = sanitize_test_code(&label);
            fill_string_column(df, "PETEST", &label)?;
            fill_string_column(df, "PETESTCD", &test_code)?;
        }
    } else if code == "DS" {
        let decod_col = column_name("DSDECOD");
        let term_col = column_name("DSTERM");
        let mut decod_vals = if let Ok(series) = df.column(&decod_col) {
            (0..df.height())
                .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                .collect::<Vec<_>>()
        } else {
            vec![String::new(); df.height()]
        };
        let mut term_vals = if let Ok(series) = df.column(&term_col) {
            (0..df.height())
                .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                .collect::<Vec<_>>()
        } else {
            vec![String::new(); df.height()]
        };
        if let Some(ct) = ctx.resolve_ct(domain, "DSDECOD")
            && let Some((_header, mapped, raw)) = ct_column_match(table, domain, ct)
        {
            for idx in 0..df.height().min(mapped.len()).min(raw.len()) {
                if decod_vals[idx].trim().is_empty()
                    && let Some(ct_value) = &mapped[idx]
                {
                    decod_vals[idx] = ct_value.clone();
                }
                if term_vals[idx].trim().is_empty() && !raw[idx].trim().is_empty() {
                    term_vals[idx] = raw[idx].trim().to_string();
                }
            }
        }
        if let Some((values, label)) = completion_column(table, domain) {
            for idx in 0..df.height().min(values.len()) {
                if decod_vals[idx].trim().is_empty() && !values[idx].trim().is_empty() {
                    decod_vals[idx] = values[idx].trim().to_string();
                }
                if term_vals[idx].trim().is_empty() && !label.trim().is_empty() {
                    term_vals[idx] = label.clone();
                }
            }
        }
        df.with_column(Series::new(decod_col.as_str().into(), decod_vals))?;
        df.with_column(Series::new(term_col.as_str().into(), term_vals))?;
    } else if code == "EX" {
        let extrt_col = column_name("EXTRT");
        let mut extrt_vals = if let Ok(series) = df.column(&extrt_col) {
            (0..df.height())
                .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                .collect::<Vec<_>>()
        } else {
            vec![String::new(); df.height()]
        };
        let mut standard_vars = BTreeSet::new();
        for variable in &domain.variables {
            standard_vars.insert(variable.name.to_uppercase());
        }
        let mut candidate_headers: Vec<String> = Vec::new();
        if let Some(preferred) = mapping_source_for_target(mapping, "EXTRT") {
            candidate_headers.push(preferred);
        }
        let keywords = ["TREAT", "DRUG", "THERAP", "INTERVENT"];
        for header in &table.headers {
            if standard_vars.contains(&header.to_uppercase()) {
                continue;
            }
            let label = table_label(table, header).unwrap_or_default();
            let mut hay = header.to_uppercase();
            if !label.is_empty() {
                hay.push(' ');
                hay.push_str(&label.to_uppercase());
            }
            if keywords.iter().any(|kw| hay.contains(kw)) {
                candidate_headers.push(header.clone());
            }
        }
        for fallback in ["EventName", "ActivityName"] {
            if table
                .headers
                .iter()
                .any(|header| header.eq_ignore_ascii_case(fallback))
            {
                candidate_headers.push(fallback.to_string());
            }
        }
        candidate_headers.sort();
        candidate_headers.dedup();
        let mut candidates: Vec<Vec<String>> = Vec::new();
        for header in candidate_headers {
            if let Some(values) = table_column_values(table, &header)
                && values.iter().any(|value| !value.trim().is_empty())
            {
                candidates.push(values);
            }
        }
        if !candidates.is_empty() {
            for (idx, extrt_value) in extrt_vals.iter_mut().enumerate().take(df.height()) {
                if !extrt_value.trim().is_empty() {
                    continue;
                }
                for values in &candidates {
                    let value = values.get(idx).map(|v| v.trim()).unwrap_or("");
                    if !value.is_empty() {
                        *extrt_value = value.to_string();
                        break;
                    }
                }
            }
            df.with_column(Series::new(extrt_col.as_str().into(), extrt_vals))?;
        }
    } else if code == "DA" {
        let ctdatest = ctx.resolve_ct(domain, "DATEST");
        let ctdatestcd = ctx.resolve_ct(domain, "DATESTCD");
        let ct_units = ctx.resolve_ct(domain, "DAORRESU");
        let datest_extensible = ctdatest.map(|ct| ct.extensible).unwrap_or(false);
        let datestcd_extensible = ctdatestcd.map(|ct| ct.extensible).unwrap_or(false);
        let mut candidates: Vec<DaOrresCandidate> = Vec::new();
        let mut candidate_headers: Vec<String> = Vec::new();
        if let Some(preferred) = mapping_source_for_target(mapping, "DAORRES") {
            candidate_headers.push(preferred);
        } else {
            for header in &table.headers {
                if header.to_uppercase().ends_with("_DAORRES") {
                    candidate_headers.push(header.clone());
                }
            }
        }
        let mut standard_vars = BTreeSet::new();
        for variable in &domain.variables {
            standard_vars.insert(variable.name.to_uppercase());
        }
        for header in &table.headers {
            let upper = header.to_uppercase();
            if !upper.starts_with("DA") {
                continue;
            }
            if upper.ends_with("CD") {
                continue;
            }
            if standard_vars.contains(&upper) {
                continue;
            }
            candidate_headers.push(header.clone());
        }
        candidate_headers.sort();
        candidate_headers.dedup();
        for header in candidate_headers {
            let upper = header.to_uppercase();
            let prefix = upper.strip_suffix("_DAORRES").unwrap_or(&upper);
            if let Some(values) = table_column_values(table, &header) {
                let label = table_label(table, &header);
                let hint = label.clone().unwrap_or_else(|| prefix.to_string());
                let mut test_code = ctdatestcd
                    .and_then(|ct| resolve_ct_value_from_hint(ct, prefix))
                    .or_else(|| {
                        label.as_deref().and_then(|text| {
                            ctdatestcd.and_then(|ct| resolve_ct_value_from_hint(ct, text))
                        })
                    })
                    .or_else(|| ctdatestcd.and_then(|ct| resolve_ct_value_from_hint(ct, &hint)));
                let mut test_name = ctdatest
                    .and_then(|ct| resolve_ct_value_from_hint(ct, prefix))
                    .or_else(|| {
                        label.as_deref().and_then(|text| {
                            ctdatest.and_then(|ct| resolve_ct_value_from_hint(ct, text))
                        })
                    })
                    .or_else(|| ctdatest.and_then(|ct| resolve_ct_value_from_hint(ct, &hint)));
                if test_name.is_none()
                    && let (Some(ct), Some(code)) = (ctdatestcd, test_code.as_ref())
                {
                    test_name = ct.preferred_terms.get(code).cloned();
                }
                if test_name.is_none() && datest_extensible {
                    test_name = label.clone().or_else(|| Some(prefix.to_string()));
                }
                if test_code.is_none() && datestcd_extensible {
                    let raw = label.clone().unwrap_or_else(|| prefix.to_string());
                    test_code = Some(sanitize_test_code(&raw));
                }
                let unit = ct_units
                    .and_then(|ct| resolve_ct_value_from_hint(ct, &hint))
                    .or_else(|| {
                        label.as_deref().and_then(|text| {
                            ct_units.and_then(|ct| resolve_ct_value_from_hint(ct, text))
                        })
                    })
                    .or_else(|| ct_units.and_then(|ct| resolve_ct_value_from_hint(ct, prefix)));
                candidates.push((test_name, test_code, unit, values));
            }
        }
        if !candidates.is_empty() {
            let daorres_col = column_name("DAORRES");
            let datest_col = column_name("DATEST");
            let datestcd_col = column_name("DATESTCD");
            let daorresu_col = column_name("DAORRESU");
            let dastresu_col = column_name("DASTRESU");
            let mut daorres_vals = if let Ok(series) = df.column(&daorres_col) {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            let mut datest_vals = if let Ok(series) = df.column(&datest_col) {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            let mut datestcd_vals = if let Ok(series) = df.column(&datestcd_col) {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            let mut daorresu_vals = if let Ok(series) = df.column(&daorresu_col) {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            let mut dastresu_vals = if let Ok(series) = df.column(&dastresu_col) {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            for idx in 0..df.height() {
                let needs_orres = daorres_vals[idx].trim().is_empty();
                let needs_test = datest_vals[idx].trim().is_empty();
                let needs_testcd = datestcd_vals[idx].trim().is_empty();
                let needs_orresu = daorresu_vals[idx].trim().is_empty();
                let needs_stresu = dastresu_vals[idx].trim().is_empty();
                if !needs_orres && !needs_test && !needs_testcd && !needs_orresu && !needs_stresu {
                    continue;
                }
                for (test_name, test_code, unit, values) in &candidates {
                    let value = values.get(idx).map(|v| v.trim()).unwrap_or("");
                    if value.is_empty() {
                        continue;
                    }
                    if needs_test && test_name.is_none() {
                        continue;
                    }
                    if needs_testcd && test_code.is_none() {
                        continue;
                    }
                    if needs_orres {
                        daorres_vals[idx] = value.to_string();
                    }
                    if needs_test && let Some(name) = test_name {
                        datest_vals[idx] = name.clone();
                    }
                    if needs_testcd && let Some(code) = test_code {
                        datestcd_vals[idx] = code.clone();
                    }
                    if needs_orresu && let Some(unit) = unit {
                        daorresu_vals[idx] = unit.clone();
                    }
                    if needs_stresu && let Some(unit) = unit {
                        dastresu_vals[idx] = unit.clone();
                    }
                    break;
                }
            }
            df.with_column(Series::new(daorres_col.as_str().into(), daorres_vals))?;
            df.with_column(Series::new(datest_col.as_str().into(), datest_vals))?;
            df.with_column(Series::new(datestcd_col.as_str().into(), datestcd_vals))?;
            df.with_column(Series::new(daorresu_col.as_str().into(), daorresu_vals))?;
            df.with_column(Series::new(dastresu_col.as_str().into(), dastresu_vals))?;
        }
    } else if code == "IE" {
        let mut candidates: Vec<(String, Vec<String>, String)> = Vec::new();
        let ct_cat = ctx.resolve_ct(domain, "IECAT");
        for header in &table.headers {
            let upper = header.to_uppercase();
            if !upper.starts_with("IE") {
                continue;
            }
            let label = table_label(table, header).unwrap_or_else(|| header.clone());
            let category = ct_cat.and_then(|ct| resolve_ct_value_from_hint(ct, &label));
            if let Some(category) = category
                && let Some(values) = table_column_values(table, header)
            {
                candidates.push((label, values, category));
            }
        }
        if !candidates.is_empty() {
            let ietest_col = column_name("IETEST");
            let ietestcd_col = column_name("IETESTCD");
            let iecat_col = column_name("IECAT");
            let ieorres_col = column_name("IEORRES");
            let mut ietest_vals = if let Ok(series) = df.column(&ietest_col) {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            let mut ietestcd_vals = if let Ok(series) = df.column(&ietestcd_col) {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            let mut iecat_vals = if let Ok(series) = df.column(&iecat_col) {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            let orres_vals = if let Ok(series) = df.column(&ieorres_col) {
                (0..df.height())
                    .map(|idx| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
                    .collect::<Vec<_>>()
            } else {
                vec![String::new(); df.height()]
            };
            for idx in 0..df.height() {
                let testcd_raw = ietestcd_vals[idx].trim();
                let orres_raw = orres_vals.get(idx).map(|val| val.trim()).unwrap_or("");
                let needs_test = ietest_vals[idx].trim().is_empty();
                let needs_testcd = testcd_raw.is_empty()
                    || is_yes_no_token(testcd_raw)
                    || (!orres_raw.is_empty() && testcd_raw.eq_ignore_ascii_case(orres_raw));
                let needs_cat = iecat_vals[idx].trim().is_empty();
                if !needs_test && !needs_cat && !needs_testcd {
                    continue;
                }
                for (label, values, category) in &candidates {
                    let value = values.get(idx).map(|v| v.trim()).unwrap_or("");
                    if value.is_empty() {
                        continue;
                    }
                    if needs_test {
                        ietest_vals[idx] = label.clone();
                    }
                    if needs_testcd {
                        ietestcd_vals[idx] = sanitize_test_code(label);
                    }
                    if needs_cat {
                        iecat_vals[idx] = category.clone();
                    }
                    break;
                }
            }
            df.with_column(Series::new(ietest_col.as_str().into(), ietest_vals))?;
            df.with_column(Series::new(ietestcd_col.as_str().into(), ietestcd_vals))?;
            df.with_column(Series::new(iecat_col.as_str().into(), iecat_vals))?;
        }
    }
    Ok(())
}
