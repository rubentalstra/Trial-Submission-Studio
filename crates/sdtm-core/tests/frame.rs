//! Tests for DomainFrame and DomainFrameMeta.

use polars::prelude::DataFrame;
use sdtm_core::frame::{DomainFrame, DomainFrameMeta};
use std::path::PathBuf;

#[test]
fn creates_frame_with_metadata() {
    let data = DataFrame::default();
    let meta = DomainFrameMeta {
        dataset_name: Some("FACM".to_string()),
        source_files: Vec::new(),
        base_domain_code: Some("FA".to_string()),
    };

    let frame = DomainFrame {
        domain_code: "FA".to_string(),
        data,
        meta: Some(meta),
    };

    assert_eq!(frame.dataset_name(), "FACM");
    assert_eq!(frame.base_domain_code(), "FA");
}

#[test]
fn defaults_dataset_name_to_domain_code() {
    let data = DataFrame::default();
    let frame = DomainFrame::new("AE", data);

    assert_eq!(frame.dataset_name(), "AE");
    assert_eq!(frame.base_domain_code(), "AE");
}

#[test]
fn tracks_source_files() {
    let data = DataFrame::default();
    let mut frame = DomainFrame::new("DM", data);

    frame.add_source_file(PathBuf::from("study/dm.csv"));
    frame.add_source_file(PathBuf::from("study/dm2.csv"));

    assert_eq!(frame.source_files().len(), 2);
}
