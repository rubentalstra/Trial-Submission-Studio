use polars::prelude::{AnyValue, Column, DataFrame};

use sdtm_core::{column_name, process_domain};
use sdtm_standards::load_default_sdtm_ig_domains;

fn any_to_i64(value: AnyValue) -> Option<i64> {
    match value {
        AnyValue::Int64(value) => Some(value),
        AnyValue::Int32(value) => Some(value as i64),
        AnyValue::Int16(value) => Some(value as i64),
        AnyValue::Int8(value) => Some(value as i64),
        AnyValue::UInt64(value) => Some(value as i64),
        AnyValue::UInt32(value) => Some(value as i64),
        AnyValue::UInt16(value) => Some(value as i64),
        AnyValue::UInt8(value) => Some(value as i64),
        AnyValue::Float64(value) => Some(value as i64),
        AnyValue::Float32(value) => Some(value as i64),
        _ => None,
    }
}

#[test]
fn assigns_sequence_by_usubjid_when_available() {
    let standards = load_default_sdtm_ig_domains().expect("standards");
    let domain = standards
        .iter()
        .find(|domain| domain.code == "DS")
        .expect("DS domain");
    let usubjid_col = column_name(domain, "USUBJID").expect("USUBJID");
    let dsterm_col = column_name(domain, "DSTERM").expect("DSTERM");

    let mut df = DataFrame::new(vec![
        Column::new(usubjid_col.clone().into(), ["SUBJ1", "SUBJ1", "SUBJ2"]),
        Column::new(dsterm_col.clone().into(), ["TERM1", "TERM2", "TERM3"]),
    ])
    .expect("df");

    process_domain(domain, &mut df, "STUDY1").expect("process");

    let seq = df.column("DSSEQ").expect("DSSEQ");
    let values: Vec<Option<i64>> = (0..df.height())
        .map(|idx| any_to_i64(seq.get(idx).unwrap_or(AnyValue::Null)))
        .collect();

    assert_eq!(values, vec![Some(1), Some(2), Some(1)]);
}
