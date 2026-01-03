//! ISO 8601 date/time parsing and formatting.
//!
//! Per SDTMIG 4.4.4, date/time values must use ISO 8601 extended format.
//! This module handles parsing various input formats and formatting to ISO 8601,
//! while preserving partial precision (e.g., 2003-12 stays 2003-12).

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

/// Result of parsing a date/time string.
#[derive(Debug, Clone, PartialEq)]
pub enum DateTimePrecision {
    /// Full datetime with time: YYYY-MM-DDTHH:MM:SS
    DateTime(NaiveDateTime),
    /// Date only: YYYY-MM-DD
    Date(NaiveDate),
    /// Year and month only: YYYY-MM
    YearMonth { year: i32, month: u32 },
    /// Year only: YYYY
    Year(i32),
    /// Already valid ISO 8601 (preserve as-is)
    Iso8601(String),
    /// Unknown/unparseable format (preserve original)
    Unknown(String),
}

impl DateTimePrecision {
    /// Format to ISO 8601 string, preserving precision.
    pub fn to_iso8601(&self) -> String {
        match self {
            DateTimePrecision::DateTime(dt) => dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
            DateTimePrecision::Date(d) => d.format("%Y-%m-%d").to_string(),
            DateTimePrecision::YearMonth { year, month } => format!("{year:04}-{month:02}"),
            DateTimePrecision::Year(year) => format!("{year:04}"),
            DateTimePrecision::Iso8601(s) | DateTimePrecision::Unknown(s) => s.clone(),
        }
    }

    /// Get the date component if available.
    pub fn date(&self) -> Option<NaiveDate> {
        match self {
            DateTimePrecision::DateTime(dt) => Some(dt.date()),
            DateTimePrecision::Date(d) => Some(*d),
            _ => None,
        }
    }
}

/// Parse a date/time string to detect its precision level.
///
/// Handles various input formats while preserving partial precision.
/// Returns the appropriate precision level for ISO 8601 formatting.
pub fn parse_date(value: &str) -> Option<NaiveDateTime> {
    let precision = parse_date_precision(value);
    match precision {
        DateTimePrecision::DateTime(dt) => Some(dt),
        DateTimePrecision::Date(d) => Some(d.and_time(NaiveTime::MIN)),
        DateTimePrecision::Iso8601(s) => {
            // Try to parse ISO 8601 string into NaiveDateTime
            try_parse_datetime(&s)
                .or_else(|| try_parse_date(&s).map(|d| d.and_time(NaiveTime::MIN)))
        }
        _ => None,
    }
}

/// Parse a date/time string with full precision information.
///
/// This is the main parsing function that preserves partial precision.
pub fn parse_date_precision(value: &str) -> DateTimePrecision {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return DateTimePrecision::Unknown(String::new());
    }

    // Check if already valid ISO 8601
    if is_valid_iso8601(trimmed) {
        return DateTimePrecision::Iso8601(trimmed.to_string());
    }

    // Try full datetime formats
    if let Some(dt) = try_parse_datetime(trimmed) {
        return DateTimePrecision::DateTime(dt);
    }

    // Try date-only formats
    if let Some(d) = try_parse_date(trimmed) {
        return DateTimePrecision::Date(d);
    }

    // Try partial formats (year-month, year)
    if let Some(ym) = try_parse_year_month(trimmed) {
        return ym;
    }

    // Preserve original if unparseable
    DateTimePrecision::Unknown(trimmed.to_string())
}

/// Check if a string is already valid ISO 8601 format.
fn is_valid_iso8601(value: &str) -> bool {
    // Valid ISO 8601 formats for SDTM:
    // YYYY
    // YYYY-MM
    // YYYY-MM-DD
    // YYYY-MM-DDTHH:MM
    // YYYY-MM-DDTHH:MM:SS
    // YYYY-MM-DDTHH:MM:SS.nnn

    let chars: Vec<char> = value.chars().collect();
    let len = chars.len();

    // Minimum: YYYY (4 chars)
    if len < 4 {
        return false;
    }

    // Check year digits
    if !chars[0..4].iter().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // YYYY
    if len == 4 {
        return true;
    }

    // Must have hyphen after year
    if len < 5 || chars[4] != '-' {
        return false;
    }

    // YYYY-MM
    if len == 7 && chars[5].is_ascii_digit() && chars[6].is_ascii_digit() {
        return true;
    }

    // Need more for date
    if len < 10 {
        return false;
    }

    // Check MM-DD
    if chars[5].is_ascii_digit()
        && chars[6].is_ascii_digit()
        && chars[7] == '-'
        && chars[8].is_ascii_digit()
        && chars[9].is_ascii_digit()
    {
        // YYYY-MM-DD
        if len == 10 {
            return true;
        }

        // Check for time part
        if len >= 16 && chars[10] == 'T' {
            // Check HH:MM
            if chars[11].is_ascii_digit()
                && chars[12].is_ascii_digit()
                && chars[13] == ':'
                && chars[14].is_ascii_digit()
                && chars[15].is_ascii_digit()
            {
                // YYYY-MM-DDTHH:MM
                if len == 16 {
                    return true;
                }

                // Check :SS
                if len >= 19
                    && chars[16] == ':'
                    && chars[17].is_ascii_digit()
                    && chars[18].is_ascii_digit()
                {
                    // YYYY-MM-DDTHH:MM:SS or with fractional seconds
                    return true;
                }
            }
        }
    }

    false
}

/// Try to parse full datetime formats.
fn try_parse_datetime(value: &str) -> Option<NaiveDateTime> {
    // ISO 8601 with T separator
    let formats = [
        "%Y-%m-%dT%H:%M:%S%.f", // With fractional seconds
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%d %H:%M",
        "%d-%b-%Y %H:%M:%S", // 15-Jan-2024 10:30:00
        "%d-%b-%Y %H:%M",
        "%d/%m/%Y %H:%M:%S", // European
        "%d/%m/%Y %H:%M",
        "%m/%d/%Y %H:%M:%S", // US
        "%m/%d/%Y %H:%M",
    ];

    for fmt in &formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(value, fmt) {
            return Some(dt);
        }
    }

    None
}

/// Try to parse date-only formats.
fn try_parse_date(value: &str) -> Option<NaiveDate> {
    let formats = [
        "%Y-%m-%d",
        "%Y/%m/%d",
        "%d-%b-%Y",  // 15-Jan-2024
        "%d-%B-%Y",  // 15-January-2024
        "%d/%m/%Y",  // European: 15/01/2024
        "%m/%d/%Y",  // US: 01/15/2024
        "%d.%m.%Y",  // German: 15.01.2024
        "%Y%m%d",    // Compact: 20240115
        "%b %d, %Y", // Jan 15, 2024
        "%B %d, %Y", // January 15, 2024
        "%d %b %Y",  // 15 Jan 2024
        "%d %B %Y",  // 15 January 2024
        "%Y-%b-%d",  // 2024-Jan-15
        "%d-%m-%Y",  // 15-01-2024
    ];

    for fmt in &formats {
        if let Ok(d) = NaiveDate::parse_from_str(value, fmt) {
            return Some(d);
        }
    }

    None
}

/// Try to parse partial formats (year-month or year only).
fn try_parse_year_month(value: &str) -> Option<DateTimePrecision> {
    // YYYY-MM format
    if value.len() == 7 && value.chars().nth(4) == Some('-') {
        if let (Ok(year), Ok(month)) = (value[0..4].parse::<i32>(), value[5..7].parse::<u32>()) {
            if (1..=12).contains(&month) {
                return Some(DateTimePrecision::YearMonth { year, month });
            }
        }
    }

    // YYYY format
    if value.len() == 4 {
        if let Ok(year) = value.parse::<i32>() {
            if (1900..=2100).contains(&year) {
                return Some(DateTimePrecision::Year(year));
            }
        }
    }

    // Month-Year formats
    let month_year_formats = [
        ("%b %Y", true),  // Jan 2024
        ("%B %Y", true),  // January 2024
        ("%m/%Y", false), // 01/2024
        ("%Y-%m", false), // 2024-01 (already checked above but for safety)
    ];

    for (fmt, has_month_name) in &month_year_formats {
        if let Ok(d) = NaiveDate::parse_from_str(&format!("{value} 01"), &format!("{fmt} %d")) {
            let year = d.year();
            let month = d.month();
            if *has_month_name || (1..=12).contains(&month) {
                return Some(DateTimePrecision::YearMonth { year, month });
            }
        }
    }

    None
}

use chrono::Datelike;

/// Format a NaiveDateTime to ISO 8601 datetime string.
pub fn format_iso8601_datetime(dt: NaiveDateTime) -> String {
    dt.format("%Y-%m-%dT%H:%M:%S").to_string()
}

/// Format a NaiveDate to ISO 8601 date string.
pub fn format_iso8601_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Transform a value to ISO 8601 format, preserving precision.
///
/// This is the main transformation function for datetime variables.
/// - Returns ISO 8601 formatted string if parseable
/// - Preserves partial precision (YYYY-MM stays YYYY-MM)
/// - Returns original value if already valid ISO 8601
/// - Returns original value if unparseable (for error preservation)
pub fn transform_to_iso8601(value: &str) -> String {
    let precision = parse_date_precision(value);
    precision.to_iso8601()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_already_iso8601() {
        assert!(is_valid_iso8601("2024"));
        assert!(is_valid_iso8601("2024-01"));
        assert!(is_valid_iso8601("2024-01-15"));
        assert!(is_valid_iso8601("2024-01-15T10:30"));
        assert!(is_valid_iso8601("2024-01-15T10:30:45"));
        assert!(is_valid_iso8601("2024-01-15T10:30:45.123"));

        assert!(!is_valid_iso8601("01/15/2024"));
        assert!(!is_valid_iso8601("15-Jan-2024"));
    }

    #[test]
    fn test_preserve_partial_precision() {
        assert_eq!(transform_to_iso8601("2024"), "2024");
        assert_eq!(transform_to_iso8601("2024-01"), "2024-01");
        assert_eq!(transform_to_iso8601("2024-01-15"), "2024-01-15");
    }

    #[test]
    fn test_transform_various_formats() {
        // US format -> ISO 8601
        assert_eq!(transform_to_iso8601("01/15/2024"), "2024-01-15");

        // European format -> ISO 8601
        assert_eq!(transform_to_iso8601("15/01/2024"), "2024-01-15");

        // Text month -> ISO 8601
        assert_eq!(transform_to_iso8601("15-Jan-2024"), "2024-01-15");
    }

    #[test]
    fn test_preserve_unknown() {
        // Unknown formats should be preserved
        assert_eq!(transform_to_iso8601("invalid date"), "invalid date");
        assert_eq!(transform_to_iso8601(""), "");
    }

    #[test]
    fn test_parse_date() {
        let dt = parse_date("2024-01-15T10:30:45").unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_format_iso8601_datetime() {
        let dt = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(10, 30, 45)
            .unwrap();
        assert_eq!(format_iso8601_datetime(dt), "2024-01-15T10:30:45");
    }

    #[test]
    fn test_format_iso8601_date() {
        let d = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(format_iso8601_date(d), "2024-01-15");
    }
}
