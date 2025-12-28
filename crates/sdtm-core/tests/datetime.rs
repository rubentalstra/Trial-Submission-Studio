use chrono::NaiveDate;

use sdtm_core::datetime::{DatePairOrder, parse_date, parse_iso8601_datetime, validate_date_pair};

#[test]
fn parse_iso8601_datetime_accepts_extended_format() {
    let result = parse_iso8601_datetime("2003-12-15T13:14:17");
    assert!(result.is_valid());
}

#[test]
fn parse_iso8601_datetime_rejects_basic_format() {
    let result = parse_iso8601_datetime("20031215");
    assert!(!result.is_valid());
}

#[test]
fn parse_date_requires_complete_dates() {
    let date = parse_date("2003-12-15");
    assert_eq!(date, NaiveDate::from_ymd_opt(2003, 12, 15));
    assert!(parse_date("2003-12").is_none());
}

#[test]
fn validate_date_pair_flags_ordering() {
    assert_eq!(
        validate_date_pair("2003-12-15", "2003-12-01"),
        DatePairOrder::EndBeforeStart
    );
    assert_eq!(
        validate_date_pair("2003-12", "2003-12-15"),
        DatePairOrder::StartIncomplete
    );
}
