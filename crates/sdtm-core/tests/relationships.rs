use std::collections::BTreeSet;

use polars::prelude::{AnyValue, Column, DataFrame};

use sdtm_core::{DomainFrame, build_relrec, build_relspec, build_relsub, column_name};
use sdtm_standards::load_default_sdtm_ig_domains;

fn any_to_string(value: AnyValue) -> String {
    match value {
        AnyValue::String(value) => value.to_string(),
        AnyValue::StringOwned(value) => value.to_string(),
        AnyValue::Null => String::new(),
        _ => value.to_string(),
    }
}

#[test]
fn builds_relrec_from_domain_frames() {
    let standards = load_default_sdtm_ig_domains().expect("standards");
    let relrec = standards
        .iter()
        .find(|domain| domain.code == "RELREC")
        .expect("RELREC domain");
    let ds_domain = standards
        .iter()
        .find(|domain| domain.code == "DS")
        .expect("DS domain");
    let lb_domain = standards
        .iter()
        .find(|domain| domain.code == "LB")
        .expect("LB domain");
    let ds_usubjid = column_name(ds_domain, "USUBJID").expect("DS USUBJID");
    let ds_seq = column_name(ds_domain, "DSSEQ").expect("DSSEQ");
    let lb_usubjid = column_name(lb_domain, "USUBJID").expect("LB USUBJID");
    let lb_seq = column_name(lb_domain, "LBSEQ").expect("LBSEQ");

    let ds = DataFrame::new(vec![
        Column::new(ds_usubjid.clone().into(), ["SUBJ1", "SUBJ1"]),
        Column::new(ds_seq.clone().into(), [1_i64, 2_i64]),
    ])
    .expect("ds");
    let lb = DataFrame::new(vec![
        Column::new(lb_usubjid.clone().into(), ["SUBJ1"]),
        Column::new(lb_seq.clone().into(), [5_i64]),
    ])
    .expect("lb");

    let frames = vec![
        DomainFrame {
            domain_code: "DS".to_string(),
            data: ds,
        },
        DomainFrame {
            domain_code: "LB".to_string(),
            data: lb,
        },
    ];

    let relrec_frame = build_relrec(&frames, &standards, relrec, "STUDY1")
        .expect("relrec")
        .expect("relrec data");

    assert_eq!(relrec_frame.data.height(), 2);
    let rdomain_col = column_name(relrec, "RDOMAIN").expect("RDOMAIN");
    let rdomain = relrec_frame.data.column(&rdomain_col).expect("RDOMAIN");
    let values: BTreeSet<String> = (0..relrec_frame.data.height())
        .map(|idx| any_to_string(rdomain.get(idx).unwrap_or(AnyValue::Null)))
        .collect();
    assert_eq!(values, BTreeSet::from(["DS".to_string(), "LB".to_string()]));
}

#[test]
fn builds_relspec_from_refid_columns() {
    let standards = load_default_sdtm_ig_domains().expect("standards");
    let relspec = standards
        .iter()
        .find(|domain| domain.code == "RELSPEC")
        .expect("RELSPEC domain");
    let (source_domain, usubjid_col, refid_col) = standards
        .iter()
        .filter_map(|domain| {
            let usubjid = column_name(domain, "USUBJID")?;
            let refid = domain
                .variables
                .iter()
                .find(|var| var.name.to_uppercase().ends_with("REFID"))
                .map(|var| var.name.clone())?;
            Some((domain.code.clone(), usubjid, refid))
        })
        .next()
        .expect("domain with REFID");

    let df = DataFrame::new(vec![
        Column::new(usubjid_col.clone().into(), ["SUBJ1"]),
        Column::new(refid_col.clone().into(), ["REF001"]),
    ])
    .expect("df");
    let frames = vec![DomainFrame {
        domain_code: source_domain,
        data: df,
    }];

    let relspec_frame = build_relspec(&frames, &standards, relspec, "STUDY1")
        .expect("relspec")
        .expect("relspec data");

    assert_eq!(relspec_frame.data.height(), 1);
    let refid_col = column_name(relspec, "REFID").expect("REFID");
    let refid = relspec_frame.data.column(&refid_col).expect("REFID");
    let value = any_to_string(refid.get(0).unwrap_or(AnyValue::Null));
    assert_eq!(value, "REF001");
}

#[test]
fn skips_relsub_when_no_inputs_support_it() {
    let standards = load_default_sdtm_ig_domains().expect("standards");
    let relsub = standards
        .iter()
        .find(|domain| domain.code == "RELSUB")
        .expect("RELSUB domain");

    let frame = build_relsub(&[], relsub, "STUDY1").expect("relsub");

    assert!(frame.is_none());
}

#[test]
fn builds_relsub_from_matching_rows() {
    let standards = load_default_sdtm_ig_domains().expect("standards");
    let relsub = standards
        .iter()
        .find(|domain| domain.code == "RELSUB")
        .expect("RELSUB domain");
    let study_col = column_name(relsub, "STUDYID").expect("STUDYID");
    let usubjid_col = column_name(relsub, "USUBJID").expect("USUBJID");
    let rsubjid_col = column_name(relsub, "RSUBJID").expect("RSUBJID");
    let srel_col = column_name(relsub, "SREL").expect("SREL");

    let df = DataFrame::new(vec![
        Column::new(study_col.clone().into(), ["STUDY1"]),
        Column::new(usubjid_col.clone().into(), ["SUBJ1"]),
        Column::new(rsubjid_col.clone().into(), ["SUBJ2"]),
        Column::new(srel_col.clone().into(), ["TWIN"]),
    ])
    .expect("df");

    let frame = build_relsub(
        &[DomainFrame {
            domain_code: "RELSUB".to_string(),
            data: df,
        }],
        relsub,
        "STUDY1",
    )
    .expect("relsub")
    .expect("relsub data");

    assert_eq!(frame.data.height(), 1);
    let columns: Vec<String> = frame
        .data
        .get_columns()
        .iter()
        .map(|col| col.name().to_string())
        .collect();
    let expected: Vec<String> = relsub
        .variables
        .iter()
        .map(|var| var.name.clone())
        .collect();
    assert_eq!(columns, expected);
}
