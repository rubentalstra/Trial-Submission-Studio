use std::collections::BTreeSet;

use polars::prelude::{Column, DataFrame};

use sdtm_core::{build_suppqual, suppqual_domain_code};
use sdtm_standards::load_sdtm_ig_domains;

#[test]
fn builds_suppqual_for_any_domain() {
    let df = DataFrame::new(vec![
        Column::new("STUDYID".into(), ["STUDY1", "STUDY1"]),
        Column::new("USUBJID".into(), ["SUBJ1", "SUBJ2"]),
        Column::new("LBTEST".into(), ["HGB", "WBC"]),
        Column::new("EXTRA".into(), ["X", ""]),
    ])
    .expect("df");

    let used = BTreeSet::new();
    let mut core = BTreeSet::new();
    core.insert("STUDYID".to_string());
    core.insert("USUBJID".to_string());
    core.insert("LBTEST".to_string());

    let standards = load_sdtm_ig_domains(std::path::Path::new("../../standards/sdtmig/v3_4"))
        .expect("load standards");
    let suppqual = standards
        .iter()
        .find(|domain| domain.code == "SUPPQUAL")
        .expect("suppqual");
    let ordered: Vec<String> = suppqual.variables.iter().map(|var| var.name.clone()).collect();

    let result = build_suppqual("LB", &df, None, &ordered, &used, &core, "STUDY1")
        .expect("suppqual")
        .expect("suppqual rows");

    assert_eq!(result.domain_code, suppqual_domain_code("LB"));
    assert_eq!(result.data.height(), 1);
    let qnam = result
        .data
        .column("QNAM")
        .expect("qnam")
        .as_series()
        .expect("series")
        .str()
        .expect("str")
        .get(0)
        .unwrap_or("");
    assert_eq!(qnam, "EXTRA");
}
