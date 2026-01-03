//! ISO 8601 duration formatting per SDTMIG 4.4.4.
//!
//! Duration format: PnYnMnDTnHnMnS or PnW
//! Where:
//! - P = period designator (required)
//! - nY = years, nM = months (before T), nD = days
//! - T = time separator
//! - nH = hours, nM = minutes (after T), nS = seconds
//! - nW = weeks (cannot mix with other components)

/// Format duration as ISO 8601.
///
/// Attempts to parse various input formats and convert to ISO 8601 duration.
/// Returns None if the value cannot be parsed as a duration.
///
/// # Supported input formats:
/// - Numeric days (e.g., "5", "3.5")
/// - Text descriptions (e.g., "5 days", "2 hours 30 minutes")
/// - Already ISO 8601 (e.g., "P5D", "PT2H30M")
pub fn format_iso8601_duration(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Check if already ISO 8601 duration
    if is_iso8601_duration(trimmed) {
        return Some(trimmed.to_string());
    }

    // Try to parse as numeric days
    if let Some(duration) = try_parse_numeric_days(trimmed) {
        return Some(duration);
    }

    // Try to parse text description
    if let Some(duration) = try_parse_text_duration(trimmed) {
        return Some(duration);
    }

    // Cannot parse - return None
    None
}

/// Check if a string is already valid ISO 8601 duration format.
fn is_iso8601_duration(value: &str) -> bool {
    if !value.starts_with('P') {
        return false;
    }

    // Simple validation: starts with P, contains valid components
    let rest = &value[1..];

    // Week format: PnW
    if let Some(stripped) = rest.strip_suffix('W') {
        return stripped.parse::<f64>().is_ok();
    }

    // Check for valid date/time components
    let valid_chars = rest
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || "YMDHMS".contains(c) || c == 'T');

    valid_chars && !rest.is_empty()
}

/// Try to parse a numeric value as days.
fn try_parse_numeric_days(value: &str) -> Option<String> {
    let cleaned = value.replace(',', "").trim().to_string();

    if let Ok(days) = cleaned.parse::<f64>() {
        return Some(format_duration_from_days(days));
    }

    None
}

/// Format a duration from decimal days.
fn format_duration_from_days(days: f64) -> String {
    if days == 0.0 {
        return "P0D".to_string();
    }

    let abs_days = days.abs();
    let sign = if days < 0.0 { "-" } else { "" };

    // Check if it's a whole number of days
    if abs_days == abs_days.floor() {
        return format!("{sign}P{}D", abs_days as i64);
    }

    // Convert fractional days to hours
    let whole_days = abs_days.floor() as i64;
    let fractional_days = abs_days - abs_days.floor();
    let hours = (fractional_days * 24.0).round() as i64;

    if whole_days == 0 {
        format!("{sign}PT{hours}H")
    } else if hours == 0 {
        format!("{sign}P{whole_days}D")
    } else {
        format!("{sign}P{whole_days}DT{hours}H")
    }
}

/// Try to parse text duration description.
fn try_parse_text_duration(value: &str) -> Option<String> {
    let lower = value.to_lowercase();
    let mut years = 0i64;
    let mut months = 0i64;
    let mut weeks = 0i64;
    let mut days = 0i64;
    let mut hours = 0i64;
    let mut minutes = 0i64;
    let mut seconds = 0i64;

    // Split by spaces and parse each component
    let parts: Vec<&str> = lower.split_whitespace().collect();
    let mut i = 0;

    while i < parts.len() {
        let part = parts[i];

        // Try to get numeric value
        if let Ok(num) = part.parse::<i64>() {
            // Look for unit in next part
            if i + 1 < parts.len() {
                let unit = parts[i + 1];
                match unit {
                    u if u.starts_with("year") => years = num,
                    u if u.starts_with("month") => months = num,
                    u if u.starts_with("week") => weeks = num,
                    u if u.starts_with("day") => days = num,
                    u if u.starts_with("hour") || u == "h" || u == "hr" || u == "hrs" => {
                        hours = num
                    }
                    u if u.starts_with("minute") || u == "min" || u == "mins" => minutes = num,
                    u if u.starts_with("second") || u == "sec" || u == "secs" || u == "s" => {
                        seconds = num
                    }
                    _ => {}
                }
                i += 2;
                continue;
            }
        }

        // Check for combined format like "5days" or "2h"
        if let Some(duration) = parse_combined_duration(part) {
            match duration {
                DurationComponent::Years(n) => years += n,
                DurationComponent::Months(n) => months += n,
                DurationComponent::Weeks(n) => weeks += n,
                DurationComponent::Days(n) => days += n,
                DurationComponent::Hours(n) => hours += n,
                DurationComponent::Minutes(n) => minutes += n,
                DurationComponent::Seconds(n) => seconds += n,
            }
        }

        i += 1;
    }

    // Build ISO 8601 duration string
    build_iso8601_duration(years, months, weeks, days, hours, minutes, seconds)
}

#[derive(Debug)]
enum DurationComponent {
    Years(i64),
    Months(i64),
    Weeks(i64),
    Days(i64),
    Hours(i64),
    Minutes(i64),
    Seconds(i64),
}

/// Parse combined format like "5days", "2h", "30min".
fn parse_combined_duration(part: &str) -> Option<DurationComponent> {
    let patterns = [
        (
            "years",
            DurationComponent::Years as fn(i64) -> DurationComponent,
        ),
        ("year", DurationComponent::Years),
        ("months", DurationComponent::Months),
        ("month", DurationComponent::Months),
        ("weeks", DurationComponent::Weeks),
        ("week", DurationComponent::Weeks),
        ("days", DurationComponent::Days),
        ("day", DurationComponent::Days),
        ("hours", DurationComponent::Hours),
        ("hour", DurationComponent::Hours),
        ("hrs", DurationComponent::Hours),
        ("hr", DurationComponent::Hours),
        ("h", DurationComponent::Hours),
        ("minutes", DurationComponent::Minutes),
        ("minute", DurationComponent::Minutes),
        ("mins", DurationComponent::Minutes),
        ("min", DurationComponent::Minutes),
        ("m", DurationComponent::Minutes),
        ("seconds", DurationComponent::Seconds),
        ("second", DurationComponent::Seconds),
        ("secs", DurationComponent::Seconds),
        ("sec", DurationComponent::Seconds),
        ("s", DurationComponent::Seconds),
        ("d", DurationComponent::Days),
        ("w", DurationComponent::Weeks),
    ];

    for (suffix, constructor) in &patterns {
        if let Some(num_str) = part.strip_suffix(suffix)
            && let Ok(num) = num_str.parse::<i64>()
        {
            return Some(constructor(num));
        }
    }

    None
}

/// Build ISO 8601 duration string from components.
fn build_iso8601_duration(
    years: i64,
    months: i64,
    weeks: i64,
    days: i64,
    hours: i64,
    minutes: i64,
    seconds: i64,
) -> Option<String> {
    // If weeks only, use week format
    if weeks != 0
        && years == 0
        && months == 0
        && days == 0
        && hours == 0
        && minutes == 0
        && seconds == 0
    {
        return Some(format!("P{weeks}W"));
    }

    // Check if we have any components
    let has_date = years != 0 || months != 0 || days != 0 || weeks != 0;
    let has_time = hours != 0 || minutes != 0 || seconds != 0;

    if !has_date && !has_time {
        return None;
    }

    let mut result = String::from("P");

    // Date part
    if years != 0 {
        result.push_str(&format!("{years}Y"));
    }
    if months != 0 {
        result.push_str(&format!("{months}M"));
    }
    // Convert weeks to days
    let total_days = days + weeks * 7;
    if total_days != 0 {
        result.push_str(&format!("{total_days}D"));
    }

    // Time part
    if has_time {
        result.push('T');
        if hours != 0 {
            result.push_str(&format!("{hours}H"));
        }
        if minutes != 0 {
            result.push_str(&format!("{minutes}M"));
        }
        if seconds != 0 {
            result.push_str(&format!("{seconds}S"));
        }
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_already_iso8601() {
        assert_eq!(format_iso8601_duration("P5D"), Some("P5D".to_string()));
        assert_eq!(
            format_iso8601_duration("PT2H30M"),
            Some("PT2H30M".to_string())
        );
        assert_eq!(
            format_iso8601_duration("P1Y2M3D"),
            Some("P1Y2M3D".to_string())
        );
        assert_eq!(format_iso8601_duration("P2W"), Some("P2W".to_string()));
    }

    #[test]
    fn test_numeric_days() {
        assert_eq!(format_iso8601_duration("5"), Some("P5D".to_string()));
        assert_eq!(format_iso8601_duration("0"), Some("P0D".to_string()));
    }

    #[test]
    fn test_fractional_days() {
        assert_eq!(format_iso8601_duration("1.5"), Some("P1DT12H".to_string()));
        assert_eq!(format_iso8601_duration("0.5"), Some("PT12H".to_string()));
    }

    #[test]
    fn test_text_duration() {
        assert_eq!(format_iso8601_duration("5 days"), Some("P5D".to_string()));
        assert_eq!(format_iso8601_duration("2 hours"), Some("PT2H".to_string()));
        assert_eq!(
            format_iso8601_duration("2 hours 30 minutes"),
            Some("PT2H30M".to_string())
        );
        assert_eq!(format_iso8601_duration("1 week"), Some("P1W".to_string()));
        assert_eq!(format_iso8601_duration("2 weeks"), Some("P2W".to_string()));
    }

    #[test]
    fn test_combined_format() {
        assert_eq!(format_iso8601_duration("5d"), Some("P5D".to_string()));
        assert_eq!(format_iso8601_duration("2h"), Some("PT2H".to_string()));
        assert_eq!(format_iso8601_duration("30min"), Some("PT30M".to_string()));
    }

    #[test]
    fn test_empty_and_invalid() {
        assert_eq!(format_iso8601_duration(""), None);
        assert_eq!(format_iso8601_duration("invalid"), None);
    }

    #[test]
    fn test_is_iso8601_duration() {
        assert!(is_iso8601_duration("P5D"));
        assert!(is_iso8601_duration("PT2H"));
        assert!(is_iso8601_duration("P1Y2M3DT4H5M6S"));
        assert!(is_iso8601_duration("P2W"));

        assert!(!is_iso8601_duration("5 days"));
        assert!(!is_iso8601_duration("5"));
        assert!(!is_iso8601_duration(""));
    }
}
