use polars::prelude::{AnyValue, Column, DataFrame};

use sdtm_core::domain_utils::column_name;
use sdtm_core::processing_context::ProcessingContext;
use sdtm_core::processor::process_domain_with_context_and_tracker;
use sdtm_ingest::any_to_i64;
use sdtm_standards::load_default_sdtm_ig_domains;

#[test]
fn assigns_sequence_by_usubjid_when_available() {
    let standards = load_default_sdtm_ig_domains().expect("standards");
    let domain = standards
        .iter()
        .find(|domain| domain.code == "DS")
        .expect("DS domain");
    let usubjid_col = column_name(domain, "USUBJID").expect("USUBJID");
    let dsterm_col = column_name(domain, "DSTERM").expect("DSTERM");

    let mut data = DataFrame::new(vec![
        Column::new(usubjid_col.clone().into(), ["SUBJ1", "SUBJ1", "SUBJ2"]),
        Column::new(dsterm_col.clone().into(), ["TERM1", "TERM2", "TERM3"]),
    ])
    .expect("df");

    let context = ProcessingContext::new("STUDY1");
    process_domain_with_context_and_tracker(domain, &mut data, &context, None).expect("process");

    let seq_col = column_name(domain, "DSSEQ").expect("DSSEQ");
    let seq = data.column(&seq_col).expect("DSSEQ");
    let values: Vec<Option<i64>> = (0..data.height())
        .map(|idx| any_to_i64(seq.get(idx).unwrap_or(AnyValue::Null)))
        .collect();

    assert_eq!(values, vec![Some(1), Some(2), Some(1)]);
}
