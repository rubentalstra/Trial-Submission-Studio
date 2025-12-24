use std::collections::BTreeMap;

use anyhow::Result;
use polars::prelude::{AnyValue, Column, DataFrame, NamedFrom, Series};

use sdtm_model::{Domain, VariableType};

use crate::domain_utils::{StandardColumns, infer_seq_column, refid_candidates, standard_columns};
use crate::frame::DomainFrame;

#[derive(Debug, Clone)]
struct EligibleDomain<'a> {
    code: String,
    data: &'a DataFrame,
    idvar: String,
    usubjid_col: String,
}

pub fn build_relrec(
    domain_frames: &[DomainFrame],
    domains: &[Domain],
    relrec_domain: &Domain,
    study_id: &str,
) -> Result<Option<DomainFrame>> {
    let domain_map = build_domain_map(domains);
    let mut eligible: Vec<EligibleDomain<'_>> = Vec::new();
    for frame in domain_frames {
        if frame.data.height() == 0 {
            continue;
        }
        let domain_def = match domain_map.get(&frame.domain_code.to_uppercase()) {
            Some(domain) => domain,
            None => {
                return Err(anyhow::anyhow!(
                    "missing standards metadata for domain {}",
                    frame.domain_code
                ));
            }
        };
        let domain_columns = standard_columns(domain_def);
        let usubjid_col = match domain_columns.usubjid.as_ref() {
            Some(name) => name.clone(),
            None => continue,
        };
        if frame.data.column(&usubjid_col).is_err() {
            continue;
        }
        if let Some(idvar) = infer_idvar(domain_def, &frame.data) {
            eligible.push(EligibleDomain {
                code: frame.domain_code.to_uppercase(),
                data: &frame.data,
                idvar,
                usubjid_col,
            });
        }
    }

    if eligible.is_empty() {
        return Ok(None);
    }

    let reference_code = pick_reference_domain(&eligible);
    let reference = eligible
        .iter()
        .find(|entry| entry.code == reference_code)
        .expect("reference domain");
    let ref_seq_map = build_seq_map(reference.data, &reference.usubjid_col, &reference.idvar);

    let mut records: Vec<BTreeMap<String, String>> = Vec::new();
    for entry in &eligible {
        if entry.code == reference_code {
            continue;
        }
        for idx in 0..entry.data.height() {
            let usubjid = column_value(entry.data, &entry.usubjid_col, idx)
                .trim()
                .to_string();
            if usubjid.is_empty() {
                continue;
            }
            let idvarval = match entry.data.column(&entry.idvar) {
                Ok(series) => {
                    stringify_idvarval(series.get(idx).unwrap_or(AnyValue::Null), idx + 1)
                }
                Err(_) => stringify_idvarval(AnyValue::Null, idx + 1),
            };
            let relid = format!("{}_{}_{}_{}", entry.code, reference_code, usubjid, idvarval);
            records.push(relrec_record(
                relrec_domain,
                study_id,
                &entry.code,
                &usubjid,
                &entry.idvar,
                &idvarval,
                &relid,
            ));
            if let Some(seq) = ref_seq_map.get(&usubjid) {
                records.push(relrec_record(
                    relrec_domain,
                    study_id,
                    &reference.code,
                    &usubjid,
                    &reference.idvar,
                    seq,
                    &relid,
                ));
            }
        }
    }

    if records.is_empty() {
        for (usubjid, seq) in ref_seq_map {
            let relid = format!("{}_ONLY_{}", reference_code, usubjid);
            records.push(relrec_record(
                relrec_domain,
                study_id,
                &reference.code,
                &usubjid,
                &reference.idvar,
                &seq,
                &relid,
            ));
        }
    }

    if records.is_empty() {
        return Ok(None);
    }

    let data = build_domain_frame(relrec_domain, &records)?;
    Ok(Some(DomainFrame {
        domain_code: relrec_domain.code.clone(),
        data,
    }))
}

pub fn build_relspec(
    domain_frames: &[DomainFrame],
    domains: &[Domain],
    relspec_domain: &Domain,
    study_id: &str,
) -> Result<Option<DomainFrame>> {
    let domain_map = build_domain_map(domains);
    let mut records: BTreeMap<(String, String), RelspecRecord> = BTreeMap::new();
    for frame in domain_frames {
        let domain_def = match domain_map.get(&frame.domain_code.to_uppercase()) {
            Some(domain) => domain,
            None => {
                return Err(anyhow::anyhow!(
                    "missing standards metadata for domain {}",
                    frame.domain_code
                ));
            }
        };
        let domain_columns = standard_columns(domain_def);
        let usubjid_col = match domain_columns.usubjid.as_ref() {
            Some(name) => name.clone(),
            None => continue,
        };
        let refid_cols = find_refid_columns(domain_def, &frame.data);
        if refid_cols.is_empty() {
            continue;
        }
        let spec_col = domain_columns.spec.clone();
        let parent_col = domain_columns.parent.clone();
        collect_relspec_records(
            &mut records,
            &frame.data,
            study_id,
            relspec_domain,
            &usubjid_col,
            &refid_cols,
            spec_col.as_deref(),
            parent_col.as_deref(),
        );
    }
    if records.is_empty() {
        return Ok(None);
    }
    let relspec_columns = standard_columns(relspec_domain);
    let records: Vec<BTreeMap<String, String>> = records
        .into_values()
        .map(|record| record.into_map(&relspec_columns))
        .collect();
    let data = build_domain_frame(relspec_domain, &records)?;
    Ok(Some(DomainFrame {
        domain_code: relspec_domain.code.clone(),
        data,
    }))
}

pub fn build_relsub(relsub_domain: &Domain) -> Result<DomainFrame> {
    let data = build_domain_frame(relsub_domain, &[])?;
    Ok(DomainFrame {
        domain_code: relsub_domain.code.clone(),
        data,
    })
}

fn infer_idvar(domain: &Domain, df: &DataFrame) -> Option<String> {
    if let Some(seq_col) = infer_seq_column(domain) {
        if df.column(&seq_col).is_ok() {
            return Some(seq_col);
        }
    }
    let mut candidates: Vec<String> = domain
        .variables
        .iter()
        .map(|var| var.name.clone())
        .filter(|name| {
            let upper = name.to_uppercase();
            upper.ends_with("SEQ") && upper != "SEQ"
        })
        .filter(|name| df.column(name).is_ok())
        .collect();
    candidates.sort_by(|a, b| a.to_uppercase().cmp(&b.to_uppercase()));
    if let Some(name) = candidates.first() {
        return Some(name.clone());
    }
    let mut grp_candidates: Vec<String> = domain
        .variables
        .iter()
        .map(|var| var.name.clone())
        .filter(|name| {
            let upper = name.to_uppercase();
            upper.ends_with("GRPID") && upper != "GRPID"
        })
        .filter(|name| df.column(name).is_ok())
        .collect();
    grp_candidates.sort_by(|a, b| a.to_uppercase().cmp(&b.to_uppercase()));
    grp_candidates.first().cloned()
}

fn pick_reference_domain(eligible: &[EligibleDomain<'_>]) -> String {
    let mut scores: Vec<(usize, String)> = Vec::new();
    for entry in eligible {
        let subject_map = build_seq_map(entry.data, &entry.usubjid_col, &entry.idvar);
        scores.push((subject_map.len(), entry.code.clone()));
    }
    scores.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    scores
        .first()
        .map(|(_, code)| code.clone())
        .unwrap_or_else(|| "RELREC".to_string())
}

fn build_seq_map(df: &DataFrame, usubjid_col: &str, seq_col: &str) -> BTreeMap<String, String> {
    if df.column(seq_col).is_err() || df.column(usubjid_col).is_err() {
        return BTreeMap::new();
    }
    let mut map: BTreeMap<String, f64> = BTreeMap::new();
    for idx in 0..df.height() {
        let usubjid = column_value(df, usubjid_col, idx).trim().to_string();
        if usubjid.is_empty() {
            continue;
        }
        let raw = match df.column(seq_col) {
            Ok(series) => series.get(idx).unwrap_or(AnyValue::Null),
            Err(_) => AnyValue::Null,
        };
        let value = match any_to_f64(raw) {
            Some(value) => value,
            None => continue,
        };
        let entry = map.entry(usubjid).or_insert(value);
        if value < *entry {
            *entry = value;
        }
    }
    map.into_iter()
        .map(|(key, value)| (key, format_numeric(value)))
        .collect()
}

fn relrec_record(
    relrec_domain: &Domain,
    study_id: &str,
    rdomain: &str,
    usubjid: &str,
    idvar: &str,
    idvarval: &str,
    relid: &str,
) -> BTreeMap<String, String> {
    let mut record = BTreeMap::new();
    let columns = standard_columns(relrec_domain);
    if let Some(name) = columns.study_id {
        record.insert(name, study_id.to_string());
    }
    if let Some(name) = columns.rdomain {
        record.insert(name, rdomain.to_string());
    }
    if let Some(name) = columns.usubjid {
        record.insert(name, usubjid.to_string());
    }
    if let Some(name) = columns.idvar {
        record.insert(name, idvar.to_string());
    }
    if let Some(name) = columns.idvarval {
        record.insert(name, idvarval.to_string());
    }
    if let Some(name) = columns.reltype {
        record.insert(name, String::new());
    }
    if let Some(name) = columns.relid {
        record.insert(name, relid.to_string());
    }
    record
}

fn stringify_idvarval(value: AnyValue, fallback_index: usize) -> String {
    if let Some(num) = any_to_f64(value.clone()) {
        return format_numeric(num);
    }
    match value {
        AnyValue::Null => fallback_index.to_string(),
        AnyValue::String(value) => value.to_string(),
        AnyValue::StringOwned(value) => value.to_string(),
        _ => value.to_string(),
    }
}

fn any_to_f64(value: AnyValue) -> Option<f64> {
    match value {
        AnyValue::Null => None,
        AnyValue::Float32(value) => Some(value as f64),
        AnyValue::Float64(value) => Some(value),
        AnyValue::Int8(value) => Some(value as f64),
        AnyValue::Int16(value) => Some(value as f64),
        AnyValue::Int32(value) => Some(value as f64),
        AnyValue::Int64(value) => Some(value as f64),
        AnyValue::UInt8(value) => Some(value as f64),
        AnyValue::UInt16(value) => Some(value as f64),
        AnyValue::UInt32(value) => Some(value as f64),
        AnyValue::UInt64(value) => Some(value as f64),
        AnyValue::String(value) => value.trim().parse::<f64>().ok(),
        AnyValue::StringOwned(value) => value.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn format_numeric(value: f64) -> String {
    if value.is_nan() {
        return String::new();
    }
    if value.fract() == 0.0 {
        return format!("{}", value as i64);
    }
    value.to_string()
}

fn any_to_string(value: AnyValue) -> String {
    match value {
        AnyValue::String(value) => value.to_string(),
        AnyValue::StringOwned(value) => value.to_string(),
        AnyValue::Null => String::new(),
        _ => value.to_string(),
    }
}

fn column_value(df: &DataFrame, name: &str, idx: usize) -> String {
    match df.column(name) {
        Ok(series) => any_to_string(series.get(idx).unwrap_or(AnyValue::Null)),
        Err(_) => String::new(),
    }
}

fn build_domain_frame(domain: &Domain, records: &[BTreeMap<String, String>]) -> Result<DataFrame> {
    let mut columns: Vec<Column> = Vec::with_capacity(domain.variables.len());
    for variable in &domain.variables {
        match variable.data_type {
            VariableType::Num => {
                let mut values: Vec<Option<f64>> = Vec::with_capacity(records.len());
                for record in records {
                    let raw = record.get(&variable.name).map(|v| v.trim()).unwrap_or("");
                    if raw.is_empty() {
                        values.push(None);
                    } else {
                        values.push(raw.parse::<f64>().ok());
                    }
                }
                columns.push(Series::new(variable.name.as_str().into(), values).into());
            }
            VariableType::Char => {
                let mut values: Vec<String> = Vec::with_capacity(records.len());
                for record in records {
                    values.push(record.get(&variable.name).cloned().unwrap_or_default());
                }
                columns.push(Series::new(variable.name.as_str().into(), values).into());
            }
        }
    }
    let data = DataFrame::new(columns)?;
    Ok(data)
}

#[derive(Debug, Clone)]
struct RelspecRecord {
    study_id: String,
    usubjid: String,
    refid: String,
    spec: String,
    parent: String,
    level: String,
    columns: BTreeMap<String, String>,
}

impl RelspecRecord {
    fn new(study_id: &str, usubjid: &str, refid: &str, relspec_domain: &Domain) -> Self {
        let mut columns = BTreeMap::new();
        for variable in &relspec_domain.variables {
            columns.insert(variable.name.clone(), String::new());
        }
        Self {
            study_id: study_id.to_string(),
            usubjid: usubjid.to_string(),
            refid: refid.to_string(),
            spec: String::new(),
            parent: String::new(),
            level: "1".to_string(),
            columns,
        }
    }

    fn into_map(mut self, columns: &StandardColumns) -> BTreeMap<String, String> {
        if let Some(name) = columns.study_id.as_ref() {
            self.columns.insert(name.clone(), self.study_id.clone());
        }
        if let Some(name) = columns.usubjid.as_ref() {
            self.columns.insert(name.clone(), self.usubjid.clone());
        }
        if let Some(name) = columns.refid.as_ref() {
            self.columns.insert(name.clone(), self.refid.clone());
        }
        if let Some(name) = columns.spec.as_ref() {
            self.columns.insert(name.clone(), self.spec.clone());
        }
        if let Some(name) = columns.parent.as_ref() {
            self.columns.insert(name.clone(), self.parent.clone());
        }
        if let Some(name) = columns.level.as_ref() {
            self.columns.insert(name.clone(), self.level.clone());
        }
        self.columns
    }
}

fn collect_relspec_records(
    records: &mut BTreeMap<(String, String), RelspecRecord>,
    df: &DataFrame,
    study_id: &str,
    relspec_domain: &Domain,
    usubjid_col: &str,
    refid_cols: &[String],
    spec_col: Option<&str>,
    parent_col: Option<&str>,
) {
    if df.height() == 0 {
        return;
    }
    if df.column(usubjid_col).is_err() {
        return;
    }
    for refid_col in refid_cols {
        for idx in 0..df.height() {
            let usubjid = column_value(df, usubjid_col, idx).trim().to_string();
            let refid = column_value(df, &refid_col, idx).trim().to_string();
            if usubjid.is_empty() || refid.is_empty() {
                continue;
            }
            let key = (usubjid.clone(), refid.clone());
            let entry = records
                .entry(key)
                .or_insert_with(|| RelspecRecord::new(study_id, &usubjid, &refid, relspec_domain));
            if entry.spec.is_empty() {
                if let Some(spec_col) = spec_col {
                    let spec = column_value(df, spec_col, idx).trim().to_string();
                    if !spec.is_empty() {
                        entry.spec = spec;
                    }
                }
            }
            if entry.parent.is_empty() {
                if let Some(parent_col) = parent_col {
                    let parent = column_value(df, parent_col, idx).trim().to_string();
                    if !parent.is_empty() {
                        entry.parent = parent;
                    }
                }
            }
        }
    }
}

fn find_refid_columns(domain: &Domain, df: &DataFrame) -> Vec<String> {
    refid_candidates(domain)
        .into_iter()
        .filter(|name| df.column(name.as_str()).is_ok())
        .collect()
}

fn build_domain_map(domains: &[Domain]) -> BTreeMap<String, &Domain> {
    let mut map = BTreeMap::new();
    for domain in domains {
        map.insert(domain.code.to_uppercase(), domain);
    }
    map
}
