use polars::prelude::{AnyValue, Column, DataFrame};

use sdtm_core::{apply_base_rules, column_name};
use sdtm_standards::load_default_sdtm_ig_domains;

#[test]
fn prefixes_usubjid_with_studyid() {
    let standards = load_default_sdtm_ig_domains().expect("standards");
    let domain = standards
        .iter()
        .find(|domain| domain.code == "DM")
        .expect("DM domain");
    let study_col = column_name(domain, "STUDYID").expect("STUDYID");
    let usubjid_col = column_name(domain, "USUBJID").expect("USUBJID");
    let mut df = DataFrame::new(vec![
        Column::new(study_col.clone().into(), ["STUDY", "STUDY"]),
        Column::new(usubjid_col.clone().into(), ["01", "STUDY-02"]),
    ])
    .expect("df");

    apply_base_rules(domain, &mut df, "STUDY").expect("base rules");

    let usubjid = df.column(&usubjid_col).expect("usubjid");
    let first = usubjid.get(0).unwrap_or(AnyValue::Null);
    let second = usubjid.get(1).unwrap_or(AnyValue::Null);
    let first = match first {
        AnyValue::String(value) => value.to_string(),
        AnyValue::StringOwned(value) => value.to_string(),
        _ => first.to_string(),
    };
    let second = match second {
        AnyValue::String(value) => value.to_string(),
        AnyValue::StringOwned(value) => value.to_string(),
        _ => second.to_string(),
    };
    assert_eq!(first, "STUDY-01");
    assert_eq!(second, "STUDY-02");
}
