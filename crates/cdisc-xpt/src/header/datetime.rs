//! XPT datetime parsing and formatting.
//!
//! SAS Transport files use the datetime format: ddMMMyy:hh:mm:ss
//! For example: "15MAR24:14:30:45"

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

/// Parse a SAS datetime string in format ddMMMyy:hh:mm:ss
///
/// # Arguments
/// * `s` - Datetime string like "15MAR24:14:30:45"
///
/// # Returns
/// `Some(NaiveDateTime)` if parsing succeeds, `None` otherwise.
///
/// # Examples
/// ```
/// use cdisc_xpt::header::parse_xpt_datetime;
/// use chrono::Datelike;
///
/// let dt = parse_xpt_datetime("15MAR24:14:30:45").unwrap();
/// assert_eq!(dt.year(), 2024);
/// assert_eq!(dt.month(), 3);
/// assert_eq!(dt.day(), 15);
/// ```
#[must_use]
pub fn parse_xpt_datetime(s: &str) -> Option<NaiveDateTime> {
    let s = s.trim();
    if s.len() < 16 {
        return None;
    }

    // Parse day (2 digits)
    let day: u32 = s.get(0..2)?.parse().ok()?;

    // Parse month (3 letter abbreviation)
    let month_str = s.get(2..5)?;
    let month = parse_month(month_str)?;

    // Parse year (2 digits)
    let year_str = s.get(5..7)?;
    let year = parse_two_digit_year(year_str.parse().ok()?);

    // Check for colon separator
    if s.get(7..8)? != ":" {
        return None;
    }

    // Parse time (hh:mm:ss)
    let time_str = s.get(8..)?;
    let time = parse_time(time_str)?;

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    Some(NaiveDateTime::new(date, time))
}

/// Format a datetime as SAS format: ddMMMyy:hh:mm:ss
///
/// # Arguments
/// * `dt` - The datetime to format
///
/// # Returns
/// Formatted string like "15MAR24:14:30:45"
///
/// # Examples
/// ```
/// use chrono::NaiveDate;
/// use cdisc_xpt::header::format_xpt_datetime;
///
/// let dt = NaiveDate::from_ymd_opt(2024, 3, 15)
///     .unwrap()
///     .and_hms_opt(14, 30, 45)
///     .unwrap();
/// assert_eq!(format_xpt_datetime(dt), "15MAR24:14:30:45");
/// ```
#[must_use]
pub fn format_xpt_datetime(dt: NaiveDateTime) -> String {
    dt.format("%d%b%y:%H:%M:%S").to_string().to_uppercase()
}

/// Parse a month abbreviation to month number (1-12).
fn parse_month(s: &str) -> Option<u32> {
    match s.to_uppercase().as_str() {
        "JAN" => Some(1),
        "FEB" => Some(2),
        "MAR" => Some(3),
        "APR" => Some(4),
        "MAY" => Some(5),
        "JUN" => Some(6),
        "JUL" => Some(7),
        "AUG" => Some(8),
        "SEP" => Some(9),
        "OCT" => Some(10),
        "NOV" => Some(11),
        "DEC" => Some(12),
        _ => None,
    }
}

/// Convert 2-digit year to 4-digit year.
///
/// Uses SAS convention:
/// - 00-29 → 2000-2029
/// - 30-99 → 1930-1999
fn parse_two_digit_year(yy: u32) -> i32 {
    if yy <= 29 {
        2000 + yy as i32
    } else {
        1900 + yy as i32
    }
}

/// Parse time in format hh:mm:ss
fn parse_time(s: &str) -> Option<NaiveTime> {
    if s.len() < 8 {
        return None;
    }

    let hour: u32 = s.get(0..2)?.parse().ok()?;
    if s.get(2..3)? != ":" {
        return None;
    }
    let min: u32 = s.get(3..5)?.parse().ok()?;
    if s.get(5..6)? != ":" {
        return None;
    }
    let sec: u32 = s.get(6..8)?.parse().ok()?;

    NaiveTime::from_hms_opt(hour, min, sec)
}

/// Get the default epoch datetime (01JAN70:00:00:00).
///
/// This is the default used when no datetime is specified.
#[must_use]
pub fn default_datetime() -> NaiveDateTime {
    NaiveDate::from_ymd_opt(1970, 1, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
}

/// Format a default datetime string.
#[must_use]
pub fn default_datetime_string() -> String {
    "01JAN70:00:00:00".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn test_parse_valid_datetime() {
        let dt = parse_xpt_datetime("15MAR24:14:30:45").unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 3);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 45);
    }

    #[test]
    fn test_parse_lowercase_month() {
        let dt = parse_xpt_datetime("01jan24:00:00:00").unwrap();
        assert_eq!(dt.month(), 1);
    }

    #[test]
    fn test_parse_mixed_case() {
        let dt = parse_xpt_datetime("01Jan24:00:00:00").unwrap();
        assert_eq!(dt.month(), 1);
    }

    #[test]
    fn test_parse_all_months() {
        let months = [
            "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
        ];
        for (i, month) in months.iter().enumerate() {
            let s = format!("01{month}24:00:00:00");
            let dt = parse_xpt_datetime(&s).unwrap();
            assert_eq!(dt.month(), (i + 1) as u32);
        }
    }

    #[test]
    fn test_parse_year_2000s() {
        for y in 0..=29 {
            let s = format!("01JAN{y:02}:00:00:00");
            let dt = parse_xpt_datetime(&s).unwrap();
            assert_eq!(dt.year(), 2000 + y);
        }
    }

    #[test]
    fn test_parse_year_1900s() {
        for y in 30..=99 {
            let s = format!("01JAN{y:02}:00:00:00");
            let dt = parse_xpt_datetime(&s).unwrap();
            assert_eq!(dt.year(), 1900 + y);
        }
    }

    #[test]
    fn test_parse_default_datetime() {
        let dt = parse_xpt_datetime("01JAN70:00:00:00").unwrap();
        assert_eq!(dt, default_datetime());
    }

    #[test]
    fn test_parse_invalid() {
        assert!(parse_xpt_datetime("").is_none());
        assert!(parse_xpt_datetime("invalid").is_none());
        assert!(parse_xpt_datetime("01XXX24:00:00:00").is_none());
        assert!(parse_xpt_datetime("01JAN24 00:00:00").is_none()); // wrong separator
        assert!(parse_xpt_datetime("01JAN24:25:00:00").is_none()); // invalid hour
    }

    #[test]
    fn test_format_datetime() {
        let dt = NaiveDate::from_ymd_opt(2024, 3, 15)
            .unwrap()
            .and_hms_opt(14, 30, 45)
            .unwrap();
        assert_eq!(format_xpt_datetime(dt), "15MAR24:14:30:45");
    }

    #[test]
    fn test_format_single_digit_day() {
        let dt = NaiveDate::from_ymd_opt(2024, 1, 5)
            .unwrap()
            .and_hms_opt(9, 5, 0)
            .unwrap();
        assert_eq!(format_xpt_datetime(dt), "05JAN24:09:05:00");
    }

    #[test]
    fn test_roundtrip() {
        let original = "15MAR24:14:30:45";
        let dt = parse_xpt_datetime(original).unwrap();
        let formatted = format_xpt_datetime(dt);
        assert_eq!(formatted, original);
    }

    #[test]
    fn test_default_datetime_string() {
        assert_eq!(default_datetime_string(), "01JAN70:00:00:00");
    }
}
