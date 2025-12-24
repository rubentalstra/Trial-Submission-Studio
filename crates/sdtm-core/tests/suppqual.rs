use polars::prelude::{AnyValue, Column, DataFrame};
use std::collections::BTreeSet;

use sdtm_core::{build_suppqual, column_name, suppqual_domain_code};
use sdtm_standards::load_default_sdtm_ig_domains;

#[test]
fn builds_suppqual_for_any_domain() {
    let standards = load_default_sdtm_ig_domains().expect("load standards");
    let suppqual = standards
        .iter()
        .find(|domain| domain.code == "SUPPQUAL")
        .expect("suppqual");
    let parent = standards
        .iter()
        .find(|domain| domain.code == "LB")
        .expect("LB domain");
    let study_col = column_name(parent, "STUDYID").expect("STUDYID");
    let usubjid_col = column_name(parent, "USUBJID").expect("USUBJID");
    let lbtest_col = column_name(parent, "LBTEST").expect("LBTEST");

    let df = DataFrame::new(vec![
        Column::new(study_col.clone().into(), ["STUDY1", "STUDY1"]),
        Column::new(usubjid_col.clone().into(), ["SUBJ1", "SUBJ2"]),
        Column::new(lbtest_col.clone().into(), ["HGB", "WBC"]),
        Column::new("EXTRA".into(), ["X", ""]),
    ])
    .expect("df");

    let used = BTreeSet::new();

    let result = build_suppqual(parent, suppqual, &df, None, &used, "STUDY1")
        .expect("suppqual")
        .expect("suppqual rows");

    assert_eq!(result.domain_code, suppqual_domain_code("LB"));
    assert_eq!(result.data.height(), 1);
    let qnam_col = column_name(suppqual, "QNAM").expect("QNAM");
    let qnam = result
        .data
        .column(&qnam_col)
        .expect("qnam")
        .get(0)
        .unwrap_or(AnyValue::Null);
    let qnam = match qnam {
        AnyValue::String(value) => value.to_string(),
        AnyValue::StringOwned(value) => value.to_string(),
        AnyValue::Null => String::new(),
        _ => qnam.to_string(),
    };
    assert_eq!(qnam, "EXTRA");
}
