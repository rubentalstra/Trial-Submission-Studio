use anyhow::Result;
use polars::prelude::{AnyValue, DataFrame, NamedFrom, Series};

fn any_to_string(value: AnyValue) -> String {
    match value {
        AnyValue::String(value) => value.to_string(),
        AnyValue::Null => String::new(),
        _ => value.to_string(),
    }
}

pub fn apply_base_rules(df: &mut DataFrame, study_id: &str) -> Result<()> {
    let usubjid_series = match df.column("USUBJID") {
        Ok(series) => series.clone(),
        Err(_) => return Ok(()),
    };
    let study_series = df.column("STUDYID").ok().cloned();
    let mut updated = Vec::with_capacity(df.height());

    for idx in 0..df.height() {
        let mut usubjid = any_to_string(usubjid_series.get(idx).unwrap_or(AnyValue::Null));
        let study_value = study_series
            .as_ref()
            .map(|series| any_to_string(series.get(idx).unwrap_or(AnyValue::Null)))
            .unwrap_or_else(|| study_id.to_string());
        if !study_value.is_empty() && !usubjid.is_empty() {
            let prefix = format!("{study_value}-");
            if !usubjid.starts_with(&prefix) {
                usubjid = format!("{prefix}{usubjid}");
            }
        }
        updated.push(usubjid);
    }

    let new_series = Series::new("USUBJID".into(), updated);
    df.with_column(new_series)?;
    Ok(())
}
