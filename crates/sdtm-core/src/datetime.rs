//! SDTM Date/Time utilities conforming to ISO 8601 requirements.
//!
//! This module implements date/time parsing, validation, and normalization
//! per SDTMIG v3.4 Chapter 4, Section 4.4 "Timing Variable Assumptions".
//!
//! # SDTMIG v3.4 Chapter 4 Reference
//!
//! - Section 4.4.1: Formats for Date/Time Variables
//! - Section 4.4.2: Date/Time Precision
//! - Section 4.4.3: Intervals of Time and Use of Duration for --DUR Variables
//! - Section 4.4.4: Use of the Study Day Variables
//! - Section 4.4.7: Use of Relative Timing Variables --STRF and --ENRF
//! - Section 4.4.8: Date and Time Reported in a Domain Based on Findings
//!
//! # Key Requirements
//!
//! - SDTM requires the ISO 8601 **extended format** (with delimiters):
//!   - Dates: `YYYY-MM-DD` (hyphens required)
//!   - Times: `hh:mm:ss` (colons required)
//!   - The ISO 8601 **basic format** (without delimiters) is NOT allowed
//! - Partial/incomplete dates are represented by right truncation or hyphens
//! - Durations use the format `PnYnMnDTnHnMnS` or `PnW`
//! - Intervals use the format `datetime/datetime` or `datetime/duration`
//!
//! # Timing Variable Types
//!
//! - `--DTC`: Date/time of collection (Findings) or single time point
//! - `--STDTC`: Start date/time (Events/Interventions)
//! - `--ENDTC`: End date/time (Events/Interventions/Interval Findings)
//! - `--DUR`: Duration (when start/end not collected)

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use std::fmt;

/// Precision level for ISO 8601 date/time values.
///
/// Per SDTMIG v3.4 Section 4.4.2, precision is indicated by the presence
/// or absence of components in the date/time value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateTimePrecision {
    /// Year only: `YYYY`
    Year,
    /// Year and month: `YYYY-MM`
    Month,
    /// Full date: `YYYY-MM-DD`
    Day,
    /// Date and hour: `YYYY-MM-DDThh`
    Hour,
    /// Date, hour, and minute: `YYYY-MM-DDThh:mm`
    Minute,
    /// Date, hour, minute, and second: `YYYY-MM-DDThh:mm:ss`
    Second,
    /// Date, hour, minute, second, and fractional seconds: `YYYY-MM-DDThh:mm:ss.nnn`
    FractionalSecond,
}

impl DateTimePrecision {
    /// Returns whether this precision level includes complete date information.
    pub fn has_complete_date(&self) -> bool {
        matches!(
            self,
            Self::Day | Self::Hour | Self::Minute | Self::Second | Self::FractionalSecond
        )
    }

    /// Returns whether this precision level includes any time information.
    pub fn has_time(&self) -> bool {
        matches!(
            self,
            Self::Hour | Self::Minute | Self::Second | Self::FractionalSecond
        )
    }
}

/// Represents a parsed ISO 8601 date/time value with precision tracking.
///
/// This struct preserves the original precision of the parsed value,
/// which is critical for SDTM compliance where partial dates are common.
#[derive(Debug, Clone, PartialEq)]
pub struct Iso8601DateTime {
    /// Year component (always present for valid dates)
    pub year: Option<i32>,
    /// Month component (1-12)
    pub month: Option<u32>,
    /// Day component (1-31)
    pub day: Option<u32>,
    /// Hour component (0-23)
    pub hour: Option<u32>,
    /// Minute component (0-59)
    pub minute: Option<u32>,
    /// Second component (0-59)
    pub second: Option<u32>,
    /// Fractional seconds (nanoseconds)
    pub nanosecond: Option<u32>,
    /// Timezone offset in minutes (e.g., +05:30 = 330, -08:00 = -480)
    pub tz_offset_minutes: Option<i32>,
    /// Whether the time is in UTC (indicated by 'Z')
    pub is_utc: bool,
    /// The original string representation
    pub original: String,
}

impl Iso8601DateTime {
    /// Returns the precision level of this date/time value.
    pub fn precision(&self) -> Option<DateTimePrecision> {
        self.year?;

        if self.nanosecond.is_some() {
            Some(DateTimePrecision::FractionalSecond)
        } else if self.second.is_some() {
            Some(DateTimePrecision::Second)
        } else if self.minute.is_some() {
            Some(DateTimePrecision::Minute)
        } else if self.hour.is_some() {
            Some(DateTimePrecision::Hour)
        } else if self.day.is_some() {
            Some(DateTimePrecision::Day)
        } else if self.month.is_some() {
            Some(DateTimePrecision::Month)
        } else {
            Some(DateTimePrecision::Year)
        }
    }

    /// Returns whether this represents a complete date (year, month, day all present).
    pub fn has_complete_date(&self) -> bool {
        self.year.is_some() && self.month.is_some() && self.day.is_some()
    }

    /// Attempts to extract a NaiveDate if the date is complete.
    pub fn to_naive_date(&self) -> Option<NaiveDate> {
        if let (Some(year), Some(month), Some(day)) = (self.year, self.month, self.day) {
            NaiveDate::from_ymd_opt(year, month, day)
        } else {
            None
        }
    }

    /// Attempts to extract a NaiveDateTime if date and time are complete.
    pub fn to_naive_datetime(&self) -> Option<NaiveDateTime> {
        let date = self.to_naive_date()?;
        let time = NaiveTime::from_hms_nano_opt(
            self.hour.unwrap_or(0),
            self.minute.unwrap_or(0),
            self.second.unwrap_or(0),
            self.nanosecond.unwrap_or(0),
        )?;
        Some(NaiveDateTime::new(date, time))
    }

    /// Returns a normalized ISO 8601 string representation.
    ///
    /// This preserves the original precision by only including components
    /// that were present in the original value.
    pub fn to_iso8601_string(&self) -> String {
        let mut result = String::new();

        // Year (or "--" for missing year per SDTMIG)
        if let Some(year) = self.year {
            result.push_str(&format!("{:04}", year));
        } else {
            result.push_str("--");
        }

        // Month
        if let Some(month) = self.month {
            result.push('-');
            result.push_str(&format!("{:02}", month));
        } else if self.day.is_some() {
            // Missing month with known day
            result.push_str("--");
        } else {
            return result;
        }

        // Day
        if let Some(day) = self.day {
            result.push('-');
            result.push_str(&format!("{:02}", day));
        } else if self.hour.is_some() {
            // Missing day with known time
            result.push('-');
        } else {
            return result;
        }

        // Time components
        if self.hour.is_some() {
            result.push('T');

            if let Some(hour) = self.hour {
                result.push_str(&format!("{:02}", hour));
            } else {
                result.push('-');
            }

            if let Some(minute) = self.minute {
                result.push(':');
                result.push_str(&format!("{:02}", minute));

                if let Some(second) = self.second {
                    result.push(':');
                    result.push_str(&format!("{:02}", second));

                    if let Some(nano) = self.nanosecond
                        && nano > 0
                    {
                        // Convert nanoseconds to decimal fraction
                        let frac = format!("{:09}", nano);
                        let frac = frac.trim_end_matches('0');
                        if !frac.is_empty() {
                            result.push('.');
                            result.push_str(frac);
                        }
                    }
                }
            } else if self.second.is_some() {
                // Missing minute with known second
                result.push_str(":-");
            }

            // Timezone
            if self.is_utc {
                result.push('Z');
            } else if let Some(offset) = self.tz_offset_minutes {
                let sign = if offset >= 0 { '+' } else { '-' };
                let offset_abs = offset.abs();
                let hours = offset_abs / 60;
                let minutes = offset_abs % 60;
                result.push_str(&format!("{}{:02}:{:02}", sign, hours, minutes));
            }
        }

        result
    }
}

impl fmt::Display for Iso8601DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_iso8601_string())
    }
}

/// Represents a parsed ISO 8601 duration value.
///
/// Per SDTMIG v3.4 Section 4.4.3, durations follow the format:
/// - `PnYnMnDTnHnMnS` (general duration)
/// - `PnW` (weeks only, cannot be mixed with other components)
#[derive(Debug, Clone, PartialEq)]
pub struct Iso8601Duration {
    /// Number of years
    pub years: Option<f64>,
    /// Number of months
    pub months: Option<f64>,
    /// Number of weeks (cannot be combined with other components)
    pub weeks: Option<f64>,
    /// Number of days
    pub days: Option<f64>,
    /// Number of hours
    pub hours: Option<f64>,
    /// Number of minutes
    pub minutes: Option<f64>,
    /// Number of seconds
    pub seconds: Option<f64>,
    /// The original string representation
    pub original: String,
}

impl Iso8601Duration {
    /// Returns whether this is a week-based duration.
    pub fn is_week_duration(&self) -> bool {
        self.weeks.is_some()
    }

    /// Converts to a string representation in ISO 8601 format.
    pub fn to_iso8601_string(&self) -> String {
        let mut result = String::from("P");

        if let Some(weeks) = self.weeks {
            result.push_str(&format_duration_component(weeks, "W"));
            return result;
        }

        let mut has_date = false;

        if let Some(years) = self.years {
            result.push_str(&format_duration_component(years, "Y"));
            has_date = true;
        }
        if let Some(months) = self.months {
            result.push_str(&format_duration_component(months, "M"));
            has_date = true;
        }
        if let Some(days) = self.days {
            result.push_str(&format_duration_component(days, "D"));
            has_date = true;
        }

        let has_time = self.hours.is_some() || self.minutes.is_some() || self.seconds.is_some();

        if has_time {
            result.push('T');
            if let Some(hours) = self.hours {
                result.push_str(&format_duration_component(hours, "H"));
            }
            if let Some(minutes) = self.minutes {
                result.push_str(&format_duration_component(minutes, "M"));
            }
            if let Some(seconds) = self.seconds {
                result.push_str(&format_duration_component(seconds, "S"));
            }
        }

        if !has_date && !has_time {
            result.push_str("0D");
        }

        result
    }
}

fn format_duration_component(value: f64, suffix: &str) -> String {
    if value.fract() == 0.0 {
        format!("{}{}", value as i64, suffix)
    } else {
        // Per SDTMIG: decimals allowed only in lowest-order component
        format!("{}{}", value, suffix)
    }
}

impl fmt::Display for Iso8601Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_iso8601_string())
    }
}

/// Result of validating an ISO 8601 date/time value.
#[derive(Debug, Clone)]
pub enum DateTimeValidation {
    /// Valid ISO 8601 extended format date/time
    Valid(Iso8601DateTime),
    /// Empty or null value
    Empty,
    /// Invalid format with error description
    Invalid(DateTimeError),
}

impl DateTimeValidation {
    /// Returns true if the validation result is valid.
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid(_))
    }

    /// Returns true if the value is empty/null.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns the parsed date/time if valid.
    pub fn as_datetime(&self) -> Option<&Iso8601DateTime> {
        match self {
            Self::Valid(dt) => Some(dt),
            _ => None,
        }
    }

    /// Returns the error if invalid.
    pub fn as_error(&self) -> Option<&DateTimeError> {
        match self {
            Self::Invalid(err) => Some(err),
            _ => None,
        }
    }
}

/// Result of validating an ISO 8601 duration value.
#[derive(Debug, Clone)]
pub enum DurationValidation {
    /// Valid ISO 8601 duration
    Valid(Iso8601Duration),
    /// Empty or null value
    Empty,
    /// Invalid format with error description
    Invalid(DurationError),
}

impl DurationValidation {
    /// Returns true if the validation result is valid.
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid(_))
    }

    /// Returns the parsed duration if valid.
    pub fn as_duration(&self) -> Option<&Iso8601Duration> {
        match self {
            Self::Valid(dur) => Some(dur),
            _ => None,
        }
    }
}

/// Represents an ISO 8601 interval of time.
///
/// Per SDTMIG v3.4 Section 4.4.3.1, intervals can be represented as:
/// - `datetime/datetime` (start and end)
/// - `datetime/duration` (start and duration after)
/// - `duration/datetime` (duration before and end)
///
/// Intervals are used to represent uncertainty or elapsed time.
#[derive(Debug, Clone, PartialEq)]
pub struct Iso8601Interval {
    /// Start date/time (if specified)
    pub start: Option<Iso8601DateTime>,
    /// End date/time (if specified)
    pub end: Option<Iso8601DateTime>,
    /// Duration component (if specified)
    pub duration: Option<Iso8601Duration>,
    /// The original string representation
    pub original: String,
}

impl Iso8601Interval {
    /// Returns whether both start and end are complete dates.
    pub fn has_complete_dates(&self) -> bool {
        match (&self.start, &self.end) {
            (Some(s), Some(e)) => s.has_complete_date() && e.has_complete_date(),
            _ => false,
        }
    }

    /// Returns the start date if available and complete.
    pub fn start_date(&self) -> Option<NaiveDate> {
        self.start.as_ref().and_then(|s| s.to_naive_date())
    }

    /// Returns the end date if available and complete.
    pub fn end_date(&self) -> Option<NaiveDate> {
        self.end.as_ref().and_then(|e| e.to_naive_date())
    }
}

impl fmt::Display for Iso8601Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.original)
    }
}

/// Result of validating an ISO 8601 interval value.
#[derive(Debug, Clone)]
pub enum IntervalValidation {
    /// Valid ISO 8601 interval
    Valid(Box<Iso8601Interval>),
    /// Empty or null value
    Empty,
    /// Invalid format with error description
    Invalid(IntervalError),
}

impl IntervalValidation {
    /// Returns true if the validation result is valid.
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid(_))
    }

    /// Returns the parsed interval if valid.
    pub fn as_interval(&self) -> Option<&Iso8601Interval> {
        match self {
            Self::Valid(int) => Some(int),
            _ => None,
        }
    }
}

/// Errors that can occur when parsing interval values.
#[derive(Debug, Clone, PartialEq)]
pub enum IntervalError {
    /// Missing solidus separator
    MissingSolidus,
    /// Invalid start component
    InvalidStart(String),
    /// Invalid end component
    InvalidEnd(String),
    /// Invalid duration component
    InvalidDuration(String),
    /// Empty interval components
    EmptyComponents,
    /// General parse error
    ParseError(String),
}

impl fmt::Display for IntervalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSolidus => write!(f, "Interval must contain '/' separator"),
            Self::InvalidStart(msg) => write!(f, "Invalid interval start: {}", msg),
            Self::InvalidEnd(msg) => write!(f, "Invalid interval end: {}", msg),
            Self::InvalidDuration(msg) => write!(f, "Invalid interval duration: {}", msg),
            Self::EmptyComponents => write!(f, "Interval cannot have empty components"),
            Self::ParseError(msg) => write!(f, "Interval parse error: {}", msg),
        }
    }
}

impl std::error::Error for IntervalError {}

/// Errors that can occur when parsing/validating date/time values.
#[derive(Debug, Clone, PartialEq)]
pub enum DateTimeError {
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

/// Errors that can occur when parsing duration values.
#[derive(Debug, Clone, PartialEq)]
pub enum DurationError {
    /// Missing 'P' prefix
    MissingPPrefix,
    /// Weeks mixed with other components
    WeeksMixedWithOtherComponents,
    /// Invalid component format
    InvalidComponent(String),
    /// General parse error
    ParseError(String),
}

impl fmt::Display for DurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPPrefix => write!(f, "Duration must start with 'P'"),
            Self::WeeksMixedWithOtherComponents => {
                write!(f, "Weeks cannot be mixed with other duration components")
            }
            Self::InvalidComponent(comp) => write!(f, "Invalid duration component: {}", comp),
            Self::ParseError(msg) => write!(f, "Duration parse error: {}", msg),
        }
    }
}

impl std::error::Error for DurationError {}

/// Parses and validates an ISO 8601 date/time string per SDTMIG v3.4 requirements.
///
/// # SDTMIG Requirements (Chapter 4, Section 4.4.1)
///
/// - Extended format required (YYYY-MM-DD, not YYYYMMDD)
/// - Spaces are not allowed
/// - Time designator 'T' required between date and time
/// - Supports partial/incomplete dates via right truncation
///
/// # Examples
///
/// ```
/// use sdtm_core::datetime::parse_iso8601_datetime;
///
/// // Complete date/time
/// let result = parse_iso8601_datetime("2003-12-15T13:14:17");
/// assert!(result.is_valid());
///
/// // Partial date (year and month only)
/// let result = parse_iso8601_datetime("2003-12");
/// assert!(result.is_valid());
///
/// // Empty value
/// let result = parse_iso8601_datetime("");
/// assert!(result.is_empty());
/// ```
pub fn parse_iso8601_datetime(value: &str) -> DateTimeValidation {
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

/// Parses and validates an ISO 8601 duration string.
///
/// # SDTMIG Requirements (Chapter 4, Section 4.4.3)
///
/// - Format: `PnYnMnDTnHnMnS` or `PnW`
/// - Weeks (W) cannot be mixed with other components
/// - Only the lowest-order component may have decimals
///
/// # Examples
///
/// ```
/// use sdtm_core::datetime::parse_iso8601_duration;
///
/// let result = parse_iso8601_duration("P2Y3M14D");
/// assert!(result.is_valid());
///
/// let result = parse_iso8601_duration("PT4H30M");
/// assert!(result.is_valid());
///
/// let result = parse_iso8601_duration("P4.5W");
/// assert!(result.is_valid());
/// ```
pub fn parse_iso8601_duration(value: &str) -> DurationValidation {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return DurationValidation::Empty;
    }

    // Must start with 'P'
    if !trimmed.starts_with('P') {
        return DurationValidation::Invalid(DurationError::MissingPPrefix);
    }

    let original = trimmed.to_string();
    let rest = &trimmed[1..];

    // Check for week format
    if rest.contains('W') {
        return parse_week_duration(rest, original);
    }

    parse_general_duration(rest, original)
}

/// Parses a week-based duration (PnW format).
fn parse_week_duration(rest: &str, original: String) -> DurationValidation {
    // Should be just a number followed by W
    if !rest.ends_with('W') {
        return DurationValidation::Invalid(DurationError::InvalidComponent(
            "Week duration must end with W".to_string(),
        ));
    }

    let num_str = &rest[..rest.len() - 1];

    // Check that there are no other components
    if num_str.chars().any(|c| c.is_ascii_alphabetic()) {
        return DurationValidation::Invalid(DurationError::WeeksMixedWithOtherComponents);
    }

    let weeks: f64 = match num_str.parse() {
        Ok(w) => w,
        Err(_) => {
            return DurationValidation::Invalid(DurationError::InvalidComponent(format!(
                "Invalid week value: {}",
                num_str
            )));
        }
    };

    DurationValidation::Valid(Iso8601Duration {
        years: None,
        months: None,
        weeks: Some(weeks),
        days: None,
        hours: None,
        minutes: None,
        seconds: None,
        original,
    })
}

/// Parses a general duration (PnYnMnDTnHnMnS format).
fn parse_general_duration(rest: &str, original: String) -> DurationValidation {
    let mut dur = Iso8601Duration {
        years: None,
        months: None,
        weeks: None,
        days: None,
        hours: None,
        minutes: None,
        seconds: None,
        original,
    };

    // Split into date and time parts
    let (date_part, time_part) = if let Some(t_pos) = rest.find('T') {
        (&rest[..t_pos], Some(&rest[t_pos + 1..]))
    } else {
        (rest, None)
    };

    // Parse date components
    if let Err(e) = parse_duration_date_part(date_part, &mut dur) {
        return DurationValidation::Invalid(e);
    }

    // Parse time components
    if let Some(time_str) = time_part
        && let Err(e) = parse_duration_time_part(time_str, &mut dur)
    {
        return DurationValidation::Invalid(e);
    }

    DurationValidation::Valid(dur)
}

/// Parses date components of a duration (nYnMnD).
fn parse_duration_date_part(
    date_str: &str,
    dur: &mut Iso8601Duration,
) -> Result<(), DurationError> {
    if date_str.is_empty() {
        return Ok(());
    }

    let mut current_num = String::new();

    for c in date_str.chars() {
        if c.is_ascii_digit() || c == '.' {
            current_num.push(c);
        } else {
            let value: f64 = current_num.parse().map_err(|_| {
                DurationError::InvalidComponent(format!("Invalid number: {}", current_num))
            })?;

            match c {
                'Y' => dur.years = Some(value),
                'M' => dur.months = Some(value),
                'D' => dur.days = Some(value),
                _ => {
                    return Err(DurationError::InvalidComponent(format!(
                        "Unknown date component: {}",
                        c
                    )));
                }
            }
            current_num.clear();
        }
    }

    if !current_num.is_empty() {
        return Err(DurationError::InvalidComponent(
            "Trailing number without designator".to_string(),
        ));
    }

    Ok(())
}

/// Parses time components of a duration (nHnMnS).
fn parse_duration_time_part(
    time_str: &str,
    dur: &mut Iso8601Duration,
) -> Result<(), DurationError> {
    if time_str.is_empty() {
        return Ok(());
    }

    let mut current_num = String::new();

    for c in time_str.chars() {
        if c.is_ascii_digit() || c == '.' {
            current_num.push(c);
        } else {
            let value: f64 = current_num.parse().map_err(|_| {
                DurationError::InvalidComponent(format!("Invalid number: {}", current_num))
            })?;

            match c {
                'H' => dur.hours = Some(value),
                'M' => dur.minutes = Some(value),
                'S' => dur.seconds = Some(value),
                _ => {
                    return Err(DurationError::InvalidComponent(format!(
                        "Unknown time component: {}",
                        c
                    )));
                }
            }
            current_num.clear();
        }
    }

    if !current_num.is_empty() {
        return Err(DurationError::InvalidComponent(
            "Trailing number without designator".to_string(),
        ));
    }

    Ok(())
}

// =============================================================================
// Convenience functions for common operations
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

/// Validates an ISO 8601 date/time string and returns an error message if invalid.
pub fn validate_iso8601(value: &str) -> Option<String> {
    match parse_iso8601_datetime(value) {
        DateTimeValidation::Valid(_) | DateTimeValidation::Empty => None,
        DateTimeValidation::Invalid(err) => Some(err.to_string()),
    }
}

/// Calculates study day per SDTMIG v3.4 Section 4.4.4.
///
/// # Formula
///
/// - If observation date is on or after reference date: DY = (obs - ref) + 1
/// - If observation date is before reference date: DY = (obs - ref)
/// - There is no Day 0 in SDTM
///
/// Returns `None` if either date is incomplete or cannot be parsed.
///
/// # Examples
///
/// ```
/// use sdtm_core::datetime::calculate_study_day;
///
/// // Reference date is Study Day 1
/// assert_eq!(calculate_study_day("2003-12-15", "2003-12-15"), Some(1));
///
/// // Day after reference
/// assert_eq!(calculate_study_day("2003-12-16", "2003-12-15"), Some(2));
///
/// // Day before reference (no Day 0)
/// assert_eq!(calculate_study_day("2003-12-14", "2003-12-15"), Some(-1));
/// ```
pub fn calculate_study_day(obs_date: &str, ref_date: &str) -> Option<i64> {
    let obs = parse_date(obs_date)?;
    let reference = parse_date(ref_date)?;

    let delta = obs.signed_duration_since(reference).num_days();

    // Per SDTMIG: there is no Day 0
    Some(if delta >= 0 { delta + 1 } else { delta })
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
pub fn compare_iso8601(a: &str, b: &str) -> Option<std::cmp::Ordering> {
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
// ISO 8601 Interval Parsing
// =============================================================================

/// Parses and validates an ISO 8601 interval string per SDTMIG v3.4 requirements.
///
/// # SDTMIG Requirements (Chapter 4, Section 4.4.3)
///
/// Intervals can be represented in three formats:
/// - `datetime/datetime` - Start and end date/time
/// - `datetime/duration` - Start date/time and duration after
/// - `duration/datetime` - Duration and end date/time
///
/// # Examples
///
/// ```
/// use sdtm_core::datetime::parse_iso8601_interval;
///
/// // Date/time to date/time interval
/// let result = parse_iso8601_interval("2003-12-01/2003-12-10");
/// assert!(result.is_valid());
///
/// // Date/time to duration interval
/// let result = parse_iso8601_interval("2003-12-15T10:00/PT30M");
/// assert!(result.is_valid());
/// ```
pub fn parse_iso8601_interval(value: &str) -> IntervalValidation {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return IntervalValidation::Empty;
    }

    // Must contain solidus separator
    let Some(solidus_pos) = trimmed.find('/') else {
        return IntervalValidation::Invalid(IntervalError::MissingSolidus);
    };

    let original = trimmed.to_string();
    let first = &trimmed[..solidus_pos];
    let second = &trimmed[solidus_pos + 1..];

    if first.is_empty() || second.is_empty() {
        return IntervalValidation::Invalid(IntervalError::EmptyComponents);
    }

    // Determine the format based on whether components start with 'P' (duration)
    let first_is_duration = first.starts_with('P');
    let second_is_duration = second.starts_with('P');

    match (first_is_duration, second_is_duration) {
        // datetime/datetime
        (false, false) => {
            let start = match parse_iso8601_datetime(first) {
                DateTimeValidation::Valid(dt) => dt,
                DateTimeValidation::Invalid(err) => {
                    return IntervalValidation::Invalid(IntervalError::InvalidStart(
                        err.to_string(),
                    ));
                }
                DateTimeValidation::Empty => {
                    return IntervalValidation::Invalid(IntervalError::EmptyComponents);
                }
            };

            let end = match parse_iso8601_datetime(second) {
                DateTimeValidation::Valid(dt) => dt,
                DateTimeValidation::Invalid(err) => {
                    return IntervalValidation::Invalid(IntervalError::InvalidEnd(err.to_string()));
                }
                DateTimeValidation::Empty => {
                    return IntervalValidation::Invalid(IntervalError::EmptyComponents);
                }
            };

            IntervalValidation::Valid(Box::new(Iso8601Interval {
                start: Some(start),
                end: Some(end),
                duration: None,
                original,
            }))
        }

        // datetime/duration
        (false, true) => {
            let start = match parse_iso8601_datetime(first) {
                DateTimeValidation::Valid(dt) => dt,
                DateTimeValidation::Invalid(err) => {
                    return IntervalValidation::Invalid(IntervalError::InvalidStart(
                        err.to_string(),
                    ));
                }
                DateTimeValidation::Empty => {
                    return IntervalValidation::Invalid(IntervalError::EmptyComponents);
                }
            };

            let duration = match parse_iso8601_duration(second) {
                DurationValidation::Valid(dur) => dur,
                DurationValidation::Invalid(err) => {
                    return IntervalValidation::Invalid(IntervalError::InvalidDuration(
                        err.to_string(),
                    ));
                }
                DurationValidation::Empty => {
                    return IntervalValidation::Invalid(IntervalError::EmptyComponents);
                }
            };

            IntervalValidation::Valid(Box::new(Iso8601Interval {
                start: Some(start),
                end: None,
                duration: Some(duration),
                original,
            }))
        }

        // duration/datetime
        (true, false) => {
            let duration = match parse_iso8601_duration(first) {
                DurationValidation::Valid(dur) => dur,
                DurationValidation::Invalid(err) => {
                    return IntervalValidation::Invalid(IntervalError::InvalidDuration(
                        err.to_string(),
                    ));
                }
                DurationValidation::Empty => {
                    return IntervalValidation::Invalid(IntervalError::EmptyComponents);
                }
            };

            let end = match parse_iso8601_datetime(second) {
                DateTimeValidation::Valid(dt) => dt,
                DateTimeValidation::Invalid(err) => {
                    return IntervalValidation::Invalid(IntervalError::InvalidEnd(err.to_string()));
                }
                DateTimeValidation::Empty => {
                    return IntervalValidation::Invalid(IntervalError::EmptyComponents);
                }
            };

            IntervalValidation::Valid(Box::new(Iso8601Interval {
                start: None,
                end: Some(end),
                duration: Some(duration),
                original,
            }))
        }

        // duration/duration - not valid per ISO 8601
        (true, true) => IntervalValidation::Invalid(IntervalError::ParseError(
            "Both interval components cannot be durations".to_string(),
        )),
    }
}

// =============================================================================
// Timing Variable Validation
// =============================================================================

/// Type of SDTM timing variable per SDTMIG v3.4 Chapter 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingVariableType {
    /// Collection date/time (--DTC) - used in Findings for specimen collection
    CollectionDateTime,
    /// Start date/time (--STDTC) - used in Events/Interventions
    StartDateTime,
    /// End date/time (--ENDTC) - used in Events/Interventions and interval collections
    EndDateTime,
    /// Duration (--DUR) - used when start/end not collected
    Duration,
    /// Reference start date/time (RFSTDTC) - from Demographics
    ReferenceStartDateTime,
    /// Reference end date/time (RFENDTC) - from Demographics
    ReferenceEndDateTime,
}

impl TimingVariableType {
    /// Determines the timing variable type from a variable name.
    pub fn from_variable_name(name: &str) -> Option<Self> {
        let upper = name.to_uppercase();

        // Exact matches for reference variables
        if upper == "RFSTDTC" {
            return Some(Self::ReferenceStartDateTime);
        }
        if upper == "RFENDTC" {
            return Some(Self::ReferenceEndDateTime);
        }

        // Pattern matches for --DTC, --STDTC, --ENDTC, --DUR
        if upper.ends_with("DUR") {
            Some(Self::Duration)
        } else if upper.ends_with("STDTC") {
            Some(Self::StartDateTime)
        } else if upper.ends_with("ENDTC") {
            Some(Self::EndDateTime)
        } else if upper.ends_with("DTC") {
            Some(Self::CollectionDateTime)
        } else {
            None
        }
    }

    /// Returns whether this variable type should contain a date/time value.
    pub fn is_datetime(&self) -> bool {
        !matches!(self, Self::Duration)
    }

    /// Returns whether this variable type should contain a duration value.
    pub fn is_duration(&self) -> bool {
        matches!(self, Self::Duration)
    }
}

/// Result of validating a timing variable value.
#[derive(Debug, Clone)]
pub struct TimingValidationResult {
    /// The variable name
    pub variable: String,
    /// The variable type
    pub variable_type: Option<TimingVariableType>,
    /// Whether the value is valid
    pub is_valid: bool,
    /// Error message if invalid
    pub error: Option<String>,
    /// Warning message if applicable
    pub warning: Option<String>,
    /// The parsed date/time (if applicable and valid)
    pub datetime: Option<Iso8601DateTime>,
    /// The parsed duration (if applicable and valid)
    pub duration: Option<Iso8601Duration>,
    /// The parsed interval (if applicable and valid)
    pub interval: Option<Iso8601Interval>,
}

/// Validates a timing variable value per SDTMIG v3.4 requirements.
///
/// # SDTMIG Requirements
///
/// - Section 4.4.1: Extended format required (YYYY-MM-DD, not YYYYMMDD)
/// - Section 4.4.2: Partial dates allowed via right truncation
/// - Section 4.4.3: Durations must use P-format, intervals use solidus
/// - Section 4.4.8: Findings use --DTC, not --STDTC
///
/// # Arguments
///
/// * `variable` - The variable name (e.g., "AESTDTC", "LBDTC")
/// * `value` - The value to validate
///
/// # Examples
///
/// ```
/// use sdtm_core::datetime::validate_timing_variable;
///
/// let result = validate_timing_variable("AESTDTC", "2003-12-15");
/// assert!(result.is_valid);
///
/// let result = validate_timing_variable("AEDUR", "P3D");
/// assert!(result.is_valid);
/// ```
pub fn validate_timing_variable(variable: &str, value: &str) -> TimingValidationResult {
    let trimmed = value.trim();
    let var_type = TimingVariableType::from_variable_name(variable);

    // Handle empty values
    if trimmed.is_empty() {
        return TimingValidationResult {
            variable: variable.to_string(),
            variable_type: var_type,
            is_valid: true, // Empty is valid (may be null)
            error: None,
            warning: None,
            datetime: None,
            duration: None,
            interval: None,
        };
    }

    // Determine expected type and validate accordingly
    match var_type {
        Some(TimingVariableType::Duration) => validate_duration_variable(variable, trimmed),
        Some(_) => validate_datetime_variable(variable, trimmed, var_type),
        None => {
            // Unknown variable type - validate as generic ISO 8601
            validate_generic_iso8601(variable, trimmed)
        }
    }
}

/// Validates a date/time variable value.
fn validate_datetime_variable(
    variable: &str,
    value: &str,
    var_type: Option<TimingVariableType>,
) -> TimingValidationResult {
    // Check for interval format
    if value.contains('/') {
        return validate_interval_value(variable, value, var_type);
    }

    // Parse as date/time
    match parse_iso8601_datetime(value) {
        DateTimeValidation::Valid(dt) => TimingValidationResult {
            variable: variable.to_string(),
            variable_type: var_type,
            is_valid: true,
            error: None,
            warning: None,
            datetime: Some(dt),
            duration: None,
            interval: None,
        },
        DateTimeValidation::Empty => TimingValidationResult {
            variable: variable.to_string(),
            variable_type: var_type,
            is_valid: true,
            error: None,
            warning: None,
            datetime: None,
            duration: None,
            interval: None,
        },
        DateTimeValidation::Invalid(err) => TimingValidationResult {
            variable: variable.to_string(),
            variable_type: var_type,
            is_valid: false,
            error: Some(err.to_string()),
            warning: None,
            datetime: None,
            duration: None,
            interval: None,
        },
    }
}

/// Validates a duration variable value.
fn validate_duration_variable(variable: &str, value: &str) -> TimingValidationResult {
    match parse_iso8601_duration(value) {
        DurationValidation::Valid(dur) => TimingValidationResult {
            variable: variable.to_string(),
            variable_type: Some(TimingVariableType::Duration),
            is_valid: true,
            error: None,
            warning: None,
            datetime: None,
            duration: Some(dur),
            interval: None,
        },
        DurationValidation::Empty => TimingValidationResult {
            variable: variable.to_string(),
            variable_type: Some(TimingVariableType::Duration),
            is_valid: true,
            error: None,
            warning: None,
            datetime: None,
            duration: None,
            interval: None,
        },
        DurationValidation::Invalid(err) => TimingValidationResult {
            variable: variable.to_string(),
            variable_type: Some(TimingVariableType::Duration),
            is_valid: false,
            error: Some(err.to_string()),
            warning: None,
            datetime: None,
            duration: None,
            interval: None,
        },
    }
}

/// Validates an interval value in a date/time variable.
fn validate_interval_value(
    variable: &str,
    value: &str,
    var_type: Option<TimingVariableType>,
) -> TimingValidationResult {
    match parse_iso8601_interval(value) {
        IntervalValidation::Valid(interval) => TimingValidationResult {
            variable: variable.to_string(),
            variable_type: var_type,
            is_valid: true,
            error: None,
            warning: None,
            datetime: None,
            duration: None,
            interval: Some(*interval),
        },
        IntervalValidation::Empty => TimingValidationResult {
            variable: variable.to_string(),
            variable_type: var_type,
            is_valid: true,
            error: None,
            warning: None,
            datetime: None,
            duration: None,
            interval: None,
        },
        IntervalValidation::Invalid(err) => TimingValidationResult {
            variable: variable.to_string(),
            variable_type: var_type,
            is_valid: false,
            error: Some(err.to_string()),
            warning: None,
            datetime: None,
            duration: None,
            interval: None,
        },
    }
}

/// Validates a value as generic ISO 8601 (unknown variable type).
fn validate_generic_iso8601(variable: &str, value: &str) -> TimingValidationResult {
    // Try duration first (if starts with P)
    if value.starts_with('P') {
        return validate_duration_variable(variable, value);
    }

    // Try interval (if contains solidus)
    if value.contains('/') {
        return validate_interval_value(variable, value, None);
    }

    // Try date/time
    validate_datetime_variable(variable, value, None)
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
/// # Examples
///
/// ```
/// use sdtm_core::datetime::{validate_date_pair, DatePairOrder};
///
/// // Valid: end after start
/// assert_eq!(
///     validate_date_pair("2003-12-01", "2003-12-15"),
///     DatePairOrder::Valid
/// );
///
/// // Invalid: end before start
/// assert_eq!(
///     validate_date_pair("2003-12-15", "2003-12-01"),
///     DatePairOrder::EndBeforeStart
/// );
///
/// // Partial dates
/// assert_eq!(
///     validate_date_pair("2003-12", "2003-12-15"),
///     DatePairOrder::StartIncomplete
/// );
/// ```
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

/// Returns whether study day can be computed from the given date value.
///
/// Per SDTMIG v3.4 Section 4.4.4, study day requires complete dates.
/// Partial dates cannot be used for study day calculation.
pub fn can_compute_study_day(value: &str) -> bool {
    match parse_iso8601_datetime(value) {
        DateTimeValidation::Valid(dt) => dt.has_complete_date(),
        _ => false,
    }
}
