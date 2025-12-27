//! Tests for the datetime module.
//!
//! Validates ISO 8601 parsing, duration handling, and study day calculations
//! per SDTMIG v3.4 Chapter 4, Section 4.4.

use chrono::NaiveDate;
use sdtm_core::datetime::{
    DateTimeError, DateTimePrecision, DateTimeValidation, DurationError, DurationValidation,
    calculate_study_day, compare_iso8601, parse_date, parse_iso8601_datetime,
    parse_iso8601_duration,
};

// =========================================================================
// Date/Time Parsing Tests
// =========================================================================

#[test]
fn test_complete_datetime() {
    let result = parse_iso8601_datetime("2003-12-15T13:14:17");
    assert!(result.is_valid());

    if let DateTimeValidation::Valid(dt) = result {
        assert_eq!(dt.year, Some(2003));
        assert_eq!(dt.month, Some(12));
        assert_eq!(dt.day, Some(15));
        assert_eq!(dt.hour, Some(13));
        assert_eq!(dt.minute, Some(14));
        assert_eq!(dt.second, Some(17));
        assert_eq!(dt.precision(), Some(DateTimePrecision::Second));
    }
}

#[test]
fn test_datetime_with_fractional_seconds() {
    let result = parse_iso8601_datetime("2003-12-15T13:14:17.123");
    assert!(result.is_valid());

    if let DateTimeValidation::Valid(dt) = result {
        assert_eq!(dt.nanosecond, Some(123_000_000));
        assert_eq!(dt.precision(), Some(DateTimePrecision::FractionalSecond));
    }
}

#[test]
fn test_date_only() {
    let result = parse_iso8601_datetime("2003-12-15");
    assert!(result.is_valid());

    if let DateTimeValidation::Valid(dt) = result {
        assert_eq!(dt.year, Some(2003));
        assert_eq!(dt.month, Some(12));
        assert_eq!(dt.day, Some(15));
        assert!(dt.hour.is_none());
        assert_eq!(dt.precision(), Some(DateTimePrecision::Day));
    }
}

#[test]
fn test_year_month_only() {
    let result = parse_iso8601_datetime("2003-12");
    assert!(result.is_valid());

    if let DateTimeValidation::Valid(dt) = result {
        assert_eq!(dt.year, Some(2003));
        assert_eq!(dt.month, Some(12));
        assert!(dt.day.is_none());
        assert_eq!(dt.precision(), Some(DateTimePrecision::Month));
    }
}

#[test]
fn test_year_only() {
    let result = parse_iso8601_datetime("2003");
    assert!(result.is_valid());

    if let DateTimeValidation::Valid(dt) = result {
        assert_eq!(dt.year, Some(2003));
        assert!(dt.month.is_none());
        assert_eq!(dt.precision(), Some(DateTimePrecision::Year));
    }
}

#[test]
fn test_datetime_with_utc() {
    let result = parse_iso8601_datetime("2003-12-15T13:14:17Z");
    assert!(result.is_valid());

    if let DateTimeValidation::Valid(dt) = result {
        assert!(dt.is_utc);
        assert!(dt.tz_offset_minutes.is_none());
    }
}

#[test]
fn test_datetime_with_timezone() {
    let result = parse_iso8601_datetime("2003-12-15T13:14:17+05:30");
    assert!(result.is_valid());

    if let DateTimeValidation::Valid(dt) = result {
        assert!(!dt.is_utc);
        assert_eq!(dt.tz_offset_minutes, Some(330)); // 5*60 + 30
    }
}

#[test]
fn test_datetime_negative_timezone() {
    let result = parse_iso8601_datetime("2003-12-15T13:14:17-08:00");
    assert!(result.is_valid());

    if let DateTimeValidation::Valid(dt) = result {
        assert_eq!(dt.tz_offset_minutes, Some(-480)); // -8*60
    }
}

#[test]
fn test_empty_value() {
    let result = parse_iso8601_datetime("");
    assert!(result.is_empty());

    let result = parse_iso8601_datetime("   ");
    assert!(result.is_empty());
}

#[test]
fn test_basic_format_rejected() {
    let result = parse_iso8601_datetime("20031215");
    assert!(matches!(
        result,
        DateTimeValidation::Invalid(DateTimeError::BasicFormatNotAllowed)
    ));
}

#[test]
fn test_spaces_rejected() {
    let result = parse_iso8601_datetime("2003-12-15 13:14:17");
    assert!(matches!(
        result,
        DateTimeValidation::Invalid(DateTimeError::SpacesNotAllowed)
    ));
}

#[test]
fn test_invalid_month() {
    let result = parse_iso8601_datetime("2003-13-15");
    assert!(matches!(
        result,
        DateTimeValidation::Invalid(DateTimeError::InvalidMonth)
    ));
}

#[test]
fn test_invalid_day() {
    let result = parse_iso8601_datetime("2003-02-30");
    assert!(matches!(
        result,
        DateTimeValidation::Invalid(DateTimeError::InvalidDay)
    ));
}

#[test]
fn test_leap_year_feb_29() {
    // 2000 is a leap year
    let result = parse_iso8601_datetime("2000-02-29");
    assert!(result.is_valid());

    // 2001 is not a leap year
    let result = parse_iso8601_datetime("2001-02-29");
    assert!(matches!(
        result,
        DateTimeValidation::Invalid(DateTimeError::InvalidDay)
    ));
}

#[test]
fn test_missing_month_with_day() {
    // SDTMIG allows --12-15 for missing year with known month/day
    let result = parse_iso8601_datetime("--12-15");
    assert!(result.is_valid());

    if let DateTimeValidation::Valid(dt) = result {
        assert!(dt.year.is_none());
        assert_eq!(dt.month, Some(12));
        assert_eq!(dt.day, Some(15));
    }
}

// =========================================================================
// Duration Parsing Tests
// =========================================================================

#[test]
fn test_duration_years() {
    let result = parse_iso8601_duration("P2Y");
    assert!(result.is_valid());

    if let DurationValidation::Valid(dur) = result {
        assert_eq!(dur.years, Some(2.0));
    }
}

#[test]
fn test_duration_weeks() {
    let result = parse_iso8601_duration("P10W");
    assert!(result.is_valid());

    if let DurationValidation::Valid(dur) = result {
        assert_eq!(dur.weeks, Some(10.0));
        assert!(dur.is_week_duration());
    }
}

#[test]
fn test_duration_complex() {
    let result = parse_iso8601_duration("P3M14D");
    assert!(result.is_valid());

    if let DurationValidation::Valid(dur) = result {
        assert_eq!(dur.months, Some(3.0));
        assert_eq!(dur.days, Some(14.0));
    }
}

#[test]
fn test_duration_with_time() {
    let result = parse_iso8601_duration("P6M17DT3H");
    assert!(result.is_valid());

    if let DurationValidation::Valid(dur) = result {
        assert_eq!(dur.months, Some(6.0));
        assert_eq!(dur.days, Some(17.0));
        assert_eq!(dur.hours, Some(3.0));
    }
}

#[test]
fn test_duration_time_only() {
    let result = parse_iso8601_duration("PT42M18S");
    assert!(result.is_valid());

    if let DurationValidation::Valid(dur) = result {
        assert_eq!(dur.minutes, Some(42.0));
        assert_eq!(dur.seconds, Some(18.0));
    }
}

#[test]
fn test_duration_decimal() {
    let result = parse_iso8601_duration("PT0.5H");
    assert!(result.is_valid());

    if let DurationValidation::Valid(dur) = result {
        assert_eq!(dur.hours, Some(0.5));
    }
}

#[test]
fn test_duration_decimal_weeks() {
    let result = parse_iso8601_duration("P4.5W");
    assert!(result.is_valid());

    if let DurationValidation::Valid(dur) = result {
        assert_eq!(dur.weeks, Some(4.5));
    }
}

#[test]
fn test_duration_missing_p() {
    let result = parse_iso8601_duration("2Y3M");
    assert!(matches!(
        result,
        DurationValidation::Invalid(DurationError::MissingPPrefix)
    ));
}

#[test]
fn test_duration_empty() {
    let result = parse_iso8601_duration("");
    assert!(matches!(result, DurationValidation::Empty));
}

// =========================================================================
// Study Day Calculation Tests
// =========================================================================

#[test]
fn test_study_day_reference_date() {
    // Reference date is Day 1
    assert_eq!(calculate_study_day("2003-12-15", "2003-12-15"), Some(1));
}

#[test]
fn test_study_day_after_reference() {
    assert_eq!(calculate_study_day("2003-12-16", "2003-12-15"), Some(2));
    assert_eq!(calculate_study_day("2003-12-17", "2003-12-15"), Some(3));
}

#[test]
fn test_study_day_before_reference() {
    // No Day 0 in SDTM
    assert_eq!(calculate_study_day("2003-12-14", "2003-12-15"), Some(-1));
    assert_eq!(calculate_study_day("2003-12-13", "2003-12-15"), Some(-2));
}

#[test]
fn test_study_day_partial_date() {
    // Partial dates should return None
    assert_eq!(calculate_study_day("2003-12", "2003-12-15"), None);
    assert_eq!(calculate_study_day("2003-12-15", "2003-12"), None);
}

// =========================================================================
// ISO 8601 String Output Tests
// =========================================================================

#[test]
fn test_to_iso8601_string() {
    let result = parse_iso8601_datetime("2003-12-15T13:14:17");
    if let DateTimeValidation::Valid(dt) = result {
        assert_eq!(dt.to_iso8601_string(), "2003-12-15T13:14:17");
    }
}

#[test]
fn test_to_iso8601_string_partial() {
    let result = parse_iso8601_datetime("2003-12");
    if let DateTimeValidation::Valid(dt) = result {
        assert_eq!(dt.to_iso8601_string(), "2003-12");
    }
}

#[test]
fn test_duration_to_iso8601_string() {
    let result = parse_iso8601_duration("P3M14D");
    if let DurationValidation::Valid(dur) = result {
        assert_eq!(dur.to_iso8601_string(), "P3M14D");
    }
}

// =========================================================================
// Comparison Tests
// =========================================================================

#[test]
fn test_compare_dates() {
    assert_eq!(
        compare_iso8601("2003-12-15", "2003-12-16"),
        Some(std::cmp::Ordering::Less)
    );
    assert_eq!(
        compare_iso8601("2003-12-15", "2003-12-15"),
        Some(std::cmp::Ordering::Equal)
    );
    assert_eq!(
        compare_iso8601("2003-12-16", "2003-12-15"),
        Some(std::cmp::Ordering::Greater)
    );
}

#[test]
fn test_compare_incompatible_precision() {
    // Cannot compare date with year-only
    assert_eq!(compare_iso8601("2003-12-15", "2003"), None);
}

// =========================================================================
// parse_date Convenience Function Tests
// =========================================================================

#[test]
fn test_parse_date() {
    // Complete date
    assert_eq!(
        parse_date("2003-12-15"),
        Some(NaiveDate::from_ymd_opt(2003, 12, 15).unwrap())
    );

    // Date/time extracts just the date
    assert_eq!(
        parse_date("2003-12-15T13:14:17"),
        Some(NaiveDate::from_ymd_opt(2003, 12, 15).unwrap())
    );

    // Empty returns None
    assert_eq!(parse_date(""), None);

    // Invalid returns None
    assert_eq!(parse_date("invalid"), None);

    // Partial date (year-month only) returns None - no complete date
    assert_eq!(parse_date("2003-12"), None);
}
