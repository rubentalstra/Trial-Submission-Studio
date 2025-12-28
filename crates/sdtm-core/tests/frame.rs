//! Tests for DomainFrame and DomainFrameMeta.

use polars::prelude::DataFrame;
use sdtm_core::frame::{DomainFrame, DomainFrameMeta};
use std::path::PathBuf;

#[test]
fn creates_frame_with_metadata() {
    let data = DataFrame::default();
    let meta = DomainFrameMeta::new()
        .with_dataset_name("FACM")
        .with_base_domain_code("FA")
        .with_split_variant("CM");

    let frame = DomainFrame::with_meta("FA", data, meta);

    assert_eq!(frame.dataset_name(), "FACM");
    assert_eq!(frame.base_domain_code(), "FA");
    assert!(frame.is_split_domain());
}

#[test]
fn defaults_dataset_name_to_domain_code() {
    let data = DataFrame::default();
    let frame = DomainFrame::new("AE", data);

    assert_eq!(frame.dataset_name(), "AE");
    assert_eq!(frame.base_domain_code(), "AE");
    assert!(!frame.is_split_domain());
}

#[test]
fn tracks_source_files() {
    let data = DataFrame::default();
    let mut frame = DomainFrame::new("DM", data);

    frame.add_source_file(PathBuf::from("study/dm.csv"));
    frame.add_source_file(PathBuf::from("study/dm2.csv"));

    assert_eq!(frame.source_files().len(), 2);
}
