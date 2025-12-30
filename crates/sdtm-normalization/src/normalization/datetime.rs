//! SDTM Date/Time utilities conforming to ISO 8601 requirements.
//!
//! This module implements date/time parsing and validation
//! per SDTMIG v3.4 Chapter 4, Section 4.4 "Timing Variable Assumptions".
//!
//! # SDTMIG v3.4 Chapter 4 Reference
//!
//! - Section 4.4.1: Formats for Date/Time Variables
//! - Section 4.4.2: Date/Time Precision
//! - Section 4.4.4: Use of the Study Day Variables
//! - Section 4.4.8: Date and Time Reported in a Domain Based on Findings
//!
//! # Key Requirements
//!
//! - SDTM requires the ISO 8601 **extended format** (with delimiters):
//!   - Dates: `YYYY-MM-DD` (hyphens required)
//!   - Times: `hh:mm:ss` (colons required)
//!   - The ISO 8601 **basic format** (without delimiters) is NOT allowed
//! - Partial/incomplete dates are represented by right truncation or hyphens

use chrono::NaiveDate;
use std::fmt;

/// Represents a parsed ISO 8601 date/time value with precision tracking.
///
/// This struct preserves the original precision of the parsed value,
/// which is critical for SDTM compliance where partial dates are common.
#[derive(Debug, Clone, PartialEq)]
struct Iso8601DateTime {
    /// Year component (always present for valid dates)
    year: Option<i32>,
    /// Month component (1-12)
    month: Option<u32>,
    /// Day component (1-31)
    day: Option<u32>,
    /// Hour component (0-23)
    hour: Option<u32>,
    /// Minute component (0-59)
    minute: Option<u32>,
    /// Second component (0-59)
    second: Option<u32>,
    /// Fractional seconds (nanoseconds)
    nanosecond: Option<u32>,
    /// Timezone offset in minutes (e.g., +05:30 = 330, -08:00 = -480)
    tz_offset_minutes: Option<i32>,
    /// Whether the time is in UTC (indicated by 'Z')
    is_utc: bool,
    /// The original string representation
    original: String,
}

impl Iso8601DateTime {
    /// Returns whether this represents a complete date (year, month, day all present).
    fn has_complete_date(&self) -> bool {
        self.year.is_some() && self.month.is_some() && self.day.is_some()
    }

    /// Attempts to extract a NaiveDate if the date is complete.
    fn to_naive_date(&self) -> Option<NaiveDate> {
        if let (Some(year), Some(month), Some(day)) = (self.year, self.month, self.day) {
            NaiveDate::from_ymd_opt(year, month, day)
        } else {
            None
        }
    }
}

/// Result of validating an ISO 8601 date/time value.
#[derive(Debug, Clone)]
enum DateTimeValidation {
    /// Valid ISO 8601 extended format date/time
    Valid(Iso8601DateTime),
    /// Empty or null value
    Empty,
    /// Invalid format with error description
    Invalid(DateTimeError),
}

/// Errors that can occur when parsing/validating date/time values.
#[derive(Debug, Clone, PartialEq)]
enum DateTimeError {
    /// Basic format (no delimiters) detected - SDTM requires extended format
    BasicFormatNotAllowed,
    /// Invalid year component
    InvalidYear,
    /// Invalid month component (not 01-12)
    InvalidMonth,
    /// Invalid day component (not 01-31 or invalid for month)
    InvalidDay,
    /// Invalid hour component (not 00-23)
    InvalidHour,
    /// Invalid minute component (not 00-59)
    InvalidMinute,
    /// Invalid second component (not 00-59)
    InvalidSecond,
    /// Invalid timezone format
    InvalidTimezone,
    /// General parse error with description
    ParseError(String),
    /// Spaces are not allowed in ISO 8601 representations
    SpacesNotAllowed,
}

impl fmt::Display for DateTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BasicFormatNotAllowed => write!(
                f,
                "ISO 8601 basic format is not allowed in SDTM; use extended format with delimiters"
            ),
            Self::InvalidYear => write!(f, "Invalid year component"),
            Self::InvalidMonth => write!(f, "Invalid month component (must be 01-12)"),
            Self::InvalidDay => write!(f, "Invalid day component"),
            Self::InvalidHour => write!(f, "Invalid hour component (must be 00-23)"),
            Self::InvalidMinute => write!(f, "Invalid minute component (must be 00-59)"),
            Self::InvalidSecond => write!(f, "Invalid second component (must be 00-59)"),
            Self::InvalidTimezone => write!(f, "Invalid timezone format"),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::SpacesNotAllowed => {
                write!(f, "Spaces are not allowed in ISO 8601 representations")
            }
        }
    }
}

impl std::error::Error for DateTimeError {}

/// Parses and validates an ISO 8601 date/time string per SDTMIG v3.4 requirements.
///
/// # SDTMIG Requirements (Chapter 4, Section 4.4.1)
///
/// - Extended format required (YYYY-MM-DD, not YYYYMMDD)
/// - Spaces are not allowed
/// - Time designator 'T' required between date and time
/// - Supports partial/incomplete dates via right truncation
///
fn parse_iso8601_datetime(value: &str) -> DateTimeValidation {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return DateTimeValidation::Empty;
    }

    // Check for spaces (not allowed per SDTMIG)
    if trimmed.contains(' ') {
        return DateTimeValidation::Invalid(DateTimeError::SpacesNotAllowed);
    }

    // Check for basic format (no delimiters) - not allowed
    // Basic format would be like "20031215" instead of "2003-12-15"
    if looks_like_basic_format(trimmed) {
        return DateTimeValidation::Invalid(DateTimeError::BasicFormatNotAllowed);
    }

    // Handle interval format (start/end with solidus)
    if trimmed.contains('/') {
        // For now, validate intervals by checking the first part
        // Full interval support can be added later
        let parts: Vec<&str> = trimmed.split('/').collect();
        if let Some(first) = parts.first()
            && !first.starts_with('P')
        {
            return parse_iso8601_datetime(first);
        }
        return DateTimeValidation::Invalid(DateTimeError::ParseError(
            "Interval format not fully supported".to_string(),
        ));
    }

    // Parse the date/time components
    parse_datetime_extended(trimmed)
}

/// Checks if a string looks like ISO 8601 basic format (no delimiters).
fn looks_like_basic_format(value: &str) -> bool {
    // Basic format examples: "20031215", "20031215T131417"
    // We reject if it starts with 8 consecutive digits (YYYYMMDD)
    let date_part = if let Some(t_pos) = value.find('T') {
        &value[..t_pos]
    } else {
        value
    };

    // If date part is exactly 8 digits, it's basic format
    if date_part.len() == 8 && date_part.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    // Also check for dates starting with "--" (missing year) but without proper delimiters
    false
}

/// Parses a date/time string in ISO 8601 extended format.
fn parse_datetime_extended(value: &str) -> DateTimeValidation {
    let original = value.to_string();
    let mut dt = Iso8601DateTime {
        year: None,
        month: None,
        day: None,
        hour: None,
        minute: None,
        second: None,
        nanosecond: None,
        tz_offset_minutes: None,
        is_utc: false,
        original: original.clone(),
    };

    // Split date and time parts
    let (date_part, time_part) = if let Some(t_pos) = value.find('T') {
        (&value[..t_pos], Some(&value[t_pos + 1..]))
    } else {
        (value, None)
    };

    // Parse date part
    if let Err(e) = parse_date_part(date_part, &mut dt) {
        return DateTimeValidation::Invalid(e);
    }

    // Parse time part if present
    if let Some(time_str) = time_part
        && let Err(e) = parse_time_part(time_str, &mut dt)
    {
        return DateTimeValidation::Invalid(e);
    }

    // Validate the parsed components
    if let Some(month) = dt.month
        && !(1..=12).contains(&month)
    {
        return DateTimeValidation::Invalid(DateTimeError::InvalidMonth);
    }

    if let Some(day) = dt.day {
        let max_day = max_days_in_month(dt.year, dt.month);
        if day < 1 || day > max_day {
            return DateTimeValidation::Invalid(DateTimeError::InvalidDay);
        }
    }

    if let Some(hour) = dt.hour
        && hour > 23
    {
        return DateTimeValidation::Invalid(DateTimeError::InvalidHour);
    }

    if let Some(minute) = dt.minute
        && minute > 59
    {
        return DateTimeValidation::Invalid(DateTimeError::InvalidMinute);
    }

    if let Some(second) = dt.second
        && second > 59
    {
        return DateTimeValidation::Invalid(DateTimeError::InvalidSecond);
    }

    DateTimeValidation::Valid(dt)
}
/// Parses the date portion of an ISO 8601 string.
fn parse_date_part(date_str: &str, dt: &mut Iso8601DateTime) -> Result<(), DateTimeError> {
    if date_str.is_empty() {
        return Ok(());
    }

    // Handle missing year format: --MM-DD or ---DD
    if let Some(rest) = date_str.strip_prefix("--") {
        if let Some(day_str) = rest.strip_prefix('-') {
            // ---DD format (missing year and month)
            if !day_str.is_empty() {
                dt.day = Some(day_str.parse().map_err(|_| DateTimeError::InvalidDay)?);
            }
        } else if rest.contains('-') {
            // --MM-DD format
            let parts: Vec<&str> = rest.split('-').collect();
            if let Some(month_str) = parts.first()
                && !month_str.is_empty()
            {
                dt.month = Some(month_str.parse().map_err(|_| DateTimeError::InvalidMonth)?);
            }
            if let Some(day_str) = parts.get(1)
                && !day_str.is_empty()
            {
                dt.day = Some(day_str.parse().map_err(|_| DateTimeError::InvalidDay)?);
            }
        } else {
            // --MM format (missing year only)
            if !rest.is_empty() {
                dt.month = Some(rest.parse().map_err(|_| DateTimeError::InvalidMonth)?);
            }
        }
        return Ok(());
    }

    // Standard YYYY-MM-DD format (with possible truncation)
    let parts: Vec<&str> = date_str.split('-').collect();

    // Parse year
    if let Some(year_str) = parts.first()
        && !year_str.is_empty()
    {
        dt.year = Some(year_str.parse().map_err(|_| DateTimeError::InvalidYear)?);
    }

    // Parse month
    if let Some(month_str) = parts.get(1)
        && !month_str.is_empty()
        && *month_str != "-"
    {
        dt.month = Some(month_str.parse().map_err(|_| DateTimeError::InvalidMonth)?);
    }

    // Parse day
    if let Some(day_str) = parts.get(2)
        && !day_str.is_empty()
        && *day_str != "-"
    {
        dt.day = Some(day_str.parse().map_err(|_| DateTimeError::InvalidDay)?);
    }

    Ok(())
}

/// Parses the time portion of an ISO 8601 string.
fn parse_time_part(time_str: &str, dt: &mut Iso8601DateTime) -> Result<(), DateTimeError> {
    if time_str.is_empty() {
        return Ok(());
    }

    // Check for timezone at the end
    let (time_without_tz, tz_str) = extract_timezone(time_str);

    // Parse timezone if present
    if let Some(tz) = tz_str {
        if tz == "Z" {
            dt.is_utc = true;
        } else {
            dt.tz_offset_minutes = Some(parse_timezone_offset(tz)?);
        }
    }

    // Handle missing hour format: -:mm:ss or similar
    let time_parts: Vec<&str> = time_without_tz.split(':').collect();

    // Parse hour (missing hour indicator "-" is handled by not setting dt.hour)
    if let Some(hour_str) = time_parts.first()
        && !hour_str.is_empty()
        && *hour_str != "-"
    {
        dt.hour = Some(hour_str.parse().map_err(|_| DateTimeError::InvalidHour)?);
    }

    // Parse minute
    if let Some(minute_str) = time_parts.get(1)
        && !minute_str.is_empty()
        && *minute_str != "-"
    {
        dt.minute = Some(
            minute_str
                .parse()
                .map_err(|_| DateTimeError::InvalidMinute)?,
        );
    }

    // Parse second (may include fractional part)
    if let Some(second_str) = time_parts.get(2)
        && !second_str.is_empty()
        && *second_str != "-"
    {
        if let Some(dot_pos) = second_str.find('.') {
            // Has fractional seconds
            let whole_str = &second_str[..dot_pos];
            let frac_str = &second_str[dot_pos + 1..];

            dt.second = Some(
                whole_str
                    .parse()
                    .map_err(|_| DateTimeError::InvalidSecond)?,
            );

            // Convert fractional string to nanoseconds
            if !frac_str.is_empty() {
                let padded = format!("{:0<9}", frac_str);
                let nanos: u32 = padded[..9]
                    .parse()
                    .map_err(|_| DateTimeError::InvalidSecond)?;
                dt.nanosecond = Some(nanos);
            }
        } else {
            dt.second = Some(
                second_str
                    .parse()
                    .map_err(|_| DateTimeError::InvalidSecond)?,
            );
        }
    }

    Ok(())
}

/// Extracts timezone from the end of a time string.
fn extract_timezone(time_str: &str) -> (&str, Option<&str>) {
    // Check for 'Z'
    if let Some(stripped) = time_str.strip_suffix('Z') {
        return (stripped, Some("Z"));
    }

    // Check for +/-HH:MM offset
    // Look for + or - that's followed by digits
    for (i, c) in time_str.char_indices().rev() {
        if (c == '+' || c == '-') && i > 0 {
            // Check if this looks like a timezone
            let potential_tz = &time_str[i..];
            if potential_tz.len() >= 5 {
                // +HH:MM is 6 chars
                return (&time_str[..i], Some(potential_tz));
            }
        }
    }

    (time_str, None)
}

/// Parses a timezone offset string like "+05:30" or "-08:00".
fn parse_timezone_offset(tz_str: &str) -> Result<i32, DateTimeError> {
    if tz_str.is_empty() {
        return Err(DateTimeError::InvalidTimezone);
    }

    let sign = match tz_str.chars().next() {
        Some('+') => 1,
        Some('-') => -1,
        _ => return Err(DateTimeError::InvalidTimezone),
    };

    let parts: Vec<&str> = tz_str[1..].split(':').collect();
    if parts.len() != 2 {
        return Err(DateTimeError::InvalidTimezone);
    }

    let hours: i32 = parts[0]
        .parse()
        .map_err(|_| DateTimeError::InvalidTimezone)?;
    let minutes: i32 = parts[1]
        .parse()
        .map_err(|_| DateTimeError::InvalidTimezone)?;

    if !(0..=14).contains(&hours) || !(0..=59).contains(&minutes) {
        return Err(DateTimeError::InvalidTimezone);
    }

    Ok(sign * (hours * 60 + minutes))
}

/// Returns the maximum number of days in a month.
fn max_days_in_month(year: Option<i32>, month: Option<u32>) -> u32 {
    match month {
        Some(1) | Some(3) | Some(5) | Some(7) | Some(8) | Some(10) | Some(12) => 31,
        Some(4) | Some(6) | Some(9) | Some(11) => 30,
        Some(2) => {
            if let Some(y) = year {
                if is_leap_year(y) { 29 } else { 28 }
            } else {
                29 // Allow 29 if year is unknown
            }
        }
        None => 31, // Allow max if month unknown
        _ => 31,
    }
}

/// Returns true if the given year is a leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

// =============================================================================
// Public API - Convenience functions for common operations
// =============================================================================

/// Parses a date string and returns a NaiveDate if it represents a complete date.
///
/// This is a convenience function for cases where only the date portion is needed.
/// Returns `None` for partial dates, invalid values, or empty strings.
pub fn parse_date(value: &str) -> Option<NaiveDate> {
    match parse_iso8601_datetime(value) {
        DateTimeValidation::Valid(dt) => dt.to_naive_date(),
        _ => None,
    }
}

/// Normalizes an ISO 8601 date/time string by trimming whitespace.
///
/// This function performs minimal normalization to preserve the original
/// precision and format of the value.
pub fn normalize_iso8601(value: &str) -> String {
    value.trim().to_string()
}

/// Compares two ISO 8601 date/time values.
///
/// Returns:
/// - `Some(Ordering::Less)` if `a < b`
/// - `Some(Ordering::Equal)` if `a == b`
/// - `Some(Ordering::Greater)` if `a > b`
/// - `None` if comparison is not possible (partial dates or invalid values)
///
/// Note: This only compares values that have the same or compatible precision.
fn compare_iso8601(a: &str, b: &str) -> Option<std::cmp::Ordering> {
    let dt_a = match parse_iso8601_datetime(a) {
        DateTimeValidation::Valid(dt) => dt,
        _ => return None,
    };

    let dt_b = match parse_iso8601_datetime(b) {
        DateTimeValidation::Valid(dt) => dt,
        _ => return None,
    };

    // Compare years
    match (dt_a.year, dt_b.year) {
        (Some(ya), Some(yb)) => {
            if ya != yb {
                return Some(ya.cmp(&yb));
            }
        }
        _ => return None,
    }

    // Compare months
    match (dt_a.month, dt_b.month) {
        (Some(ma), Some(mb)) => {
            if ma != mb {
                return Some(ma.cmp(&mb));
            }
        }
        (None, None) => return Some(std::cmp::Ordering::Equal),
        _ => return None, // Incompatible precision
    }

    // Compare days
    match (dt_a.day, dt_b.day) {
        (Some(da), Some(db)) => {
            if da != db {
                return Some(da.cmp(&db));
            }
        }
        (None, None) => return Some(std::cmp::Ordering::Equal),
        _ => return None,
    }

    // Compare time components if both have time
    if dt_a.hour.is_some() && dt_b.hour.is_some() {
        let time_a = (
            dt_a.hour.unwrap_or(0),
            dt_a.minute.unwrap_or(0),
            dt_a.second.unwrap_or(0),
            dt_a.nanosecond.unwrap_or(0),
        );
        let time_b = (
            dt_b.hour.unwrap_or(0),
            dt_b.minute.unwrap_or(0),
            dt_b.second.unwrap_or(0),
            dt_b.nanosecond.unwrap_or(0),
        );
        return Some(time_a.cmp(&time_b));
    }

    Some(std::cmp::Ordering::Equal)
}

// =============================================================================
// Date Pair Validation
// =============================================================================

/// Result of comparing a start/end date pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatePairOrder {
    /// Both dates are present and end >= start (valid)
    Valid,
    /// End date is before start date (invalid)
    EndBeforeStart,
    /// Start date is missing
    StartMissing,
    /// End date is missing
    EndMissing,
    /// Both dates are missing
    BothMissing,
    /// Start date is incomplete (partial)
    StartIncomplete,
    /// End date is incomplete (partial)
    EndIncomplete,
    /// Cannot compare (incompatible precision)
    IncompatiblePrecision,
    /// Start date is invalid
    StartInvalid(String),
    /// End date is invalid
    EndInvalid(String),
}

/// Validates that an end date is not before a start date.
///
/// Per SDTMIG v3.4, this function validates the temporal ordering of date pairs
/// without modifying any values. Returns an error for invalid orderings.
///
/// # Arguments
///
/// * `start_value` - The start date/time value (--STDTC)
/// * `end_value` - The end date/time value (--ENDTC)
///
/// # Returns
///
/// A `DatePairOrder` indicating the validation result.
///
pub fn validate_date_pair(start_value: &str, end_value: &str) -> DatePairOrder {
    let start_trimmed = start_value.trim();
    let end_trimmed = end_value.trim();

    // Handle empty values
    if start_trimmed.is_empty() && end_trimmed.is_empty() {
        return DatePairOrder::BothMissing;
    }
    if start_trimmed.is_empty() {
        return DatePairOrder::StartMissing;
    }
    if end_trimmed.is_empty() {
        return DatePairOrder::EndMissing;
    }

    // Parse both dates
    let start_result = parse_iso8601_datetime(start_trimmed);
    let end_result = parse_iso8601_datetime(end_trimmed);

    // Check for parse errors
    let start_dt = match start_result {
        DateTimeValidation::Valid(dt) => dt,
        DateTimeValidation::Invalid(err) => return DatePairOrder::StartInvalid(err.to_string()),
        DateTimeValidation::Empty => return DatePairOrder::StartMissing,
    };

    let end_dt = match end_result {
        DateTimeValidation::Valid(dt) => dt,
        DateTimeValidation::Invalid(err) => return DatePairOrder::EndInvalid(err.to_string()),
        DateTimeValidation::Empty => return DatePairOrder::EndMissing,
    };

    // Check for incomplete dates
    if !start_dt.has_complete_date() {
        return DatePairOrder::StartIncomplete;
    }
    if !end_dt.has_complete_date() {
        return DatePairOrder::EndIncomplete;
    }

    // Compare the dates
    match compare_iso8601(start_trimmed, end_trimmed) {
        Some(std::cmp::Ordering::Greater) => DatePairOrder::EndBeforeStart,
        Some(_) => DatePairOrder::Valid,
        None => DatePairOrder::IncompatiblePrecision,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn normalize_iso8601_trims_whitespace() {
        assert_eq!(normalize_iso8601("  2023-12-25  "), "2023-12-25");
        assert_eq!(
            normalize_iso8601("2023-12-25T10:30:00"),
            "2023-12-25T10:30:00"
        );
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
}
