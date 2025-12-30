//! Tests for datetime normalization.

use chrono::NaiveDate;
use sdtm_transform::normalization::{parse_date, validate_date_pair, DatePairOrder};

#[test]
fn parse_date_complete() {
    assert!(parse_date("2023-12-25").is_some());
    assert_eq!(
        parse_date("2023-12-25"),
        Some(NaiveDate::from_ymd_opt(2023, 12, 25).unwrap())
    );
}

#[test]
fn parse_date_partial_returns_none() {
    assert!(parse_date("2023-12").is_none()); // Year-month only
    assert!(parse_date("2023").is_none()); // Year only
}

#[test]
fn parse_date_invalid_returns_none() {
    assert!(parse_date("").is_none());
    assert!(parse_date("invalid").is_none());
    assert!(parse_date("2023-13-01").is_none()); // Invalid month
}

#[test]
fn validate_date_pair_valid() {
    assert_eq!(
        validate_date_pair("2023-01-01", "2023-12-31"),
        DatePairOrder::Valid
    );
    assert_eq!(
        validate_date_pair("2023-06-15", "2023-06-15"),
        DatePairOrder::Valid
    );
}

#[test]
fn validate_date_pair_end_before_start() {
    assert_eq!(
        validate_date_pair("2023-12-31", "2023-01-01"),
        DatePairOrder::EndBeforeStart
    );
}

#[test]
fn validate_date_pair_missing() {
    assert_eq!(
        validate_date_pair("", "2023-01-01"),
        DatePairOrder::StartMissing
    );
    assert_eq!(
        validate_date_pair("2023-01-01", ""),
        DatePairOrder::EndMissing
    );
    assert_eq!(validate_date_pair("", ""), DatePairOrder::BothMissing);
}
