use polars::prelude::{Column, DataFrame};

use sdtm_core::apply_base_rules;

#[test]
fn prefixes_usubjid_with_studyid() {
    let mut df = DataFrame::new(vec![
        Column::new("STUDYID".into(), ["STUDY", "STUDY"]),
        Column::new("USUBJID".into(), ["01", "STUDY-02"]),
    ])
    .expect("df");

    apply_base_rules(&mut df, "STUDY").expect("base rules");

    let usubjid = df
        .column("USUBJID")
        .expect("usubjid")
        .as_series()
        .expect("series")
        .str()
        .expect("str");
    assert_eq!(usubjid.get(0).unwrap_or(""), "STUDY-01");
    assert_eq!(usubjid.get(1).unwrap_or(""), "STUDY-02");
}
