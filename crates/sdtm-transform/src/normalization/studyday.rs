//! Study day calculation per SDTMIG 4.4.4.
//!
//! Study day is calculated relative to RFSTDTC (reference start date):
//! - If event_date >= reference_date: (event - ref) + 1 (Day 1, 2, 3...)
//! - If event_date < reference_date: (event - ref) (Day -1, -2, -3...)
//! - No day 0 exists

use chrono::NaiveDate;

/// Calculate study day per SDTMIG 4.4.4 rules.
///
/// # Arguments
/// * `event_date` - Date of the event/observation
/// * `reference_date` - Reference start date (RFSTDTC from DM)
///
/// # Returns
/// Study day as integer. Day 1 is the reference date.
/// Days before reference are negative (no day 0).
///
/// # Examples
/// ```
/// use chrono::NaiveDate;
/// use sdtm_transform::normalization::studyday::calculate_study_day;
///
/// let ref_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
///
/// // Same day = Day 1
/// assert_eq!(calculate_study_day(ref_date, ref_date), 1);
///
/// // Next day = Day 2
/// let day2 = NaiveDate::from_ymd_opt(2024, 1, 16).unwrap();
/// assert_eq!(calculate_study_day(day2, ref_date), 2);
///
/// // Previous day = Day -1
/// let day_m1 = NaiveDate::from_ymd_opt(2024, 1, 14).unwrap();
/// assert_eq!(calculate_study_day(day_m1, ref_date), -1);
/// ```
pub fn calculate_study_day(event_date: NaiveDate, reference_date: NaiveDate) -> i32 {
    let days = (event_date - reference_date).num_days() as i32;

    if days >= 0 {
        // On or after reference: Day 1, 2, 3...
        days + 1
    } else {
        // Before reference: Day -1, -2, -3... (no day 0)
        days
    }
}

/// Calculate study day from string dates.
///
/// Returns None if either date is unparseable or partial.
/// Both dates must have at least day precision for calculation.
pub fn calculate_study_day_from_strings(
    event_date_str: &str,
    reference_date_str: &str,
) -> Option<i32> {
    let event = parse_date_for_studyday(event_date_str)?;
    let reference = parse_date_for_studyday(reference_date_str)?;
    Some(calculate_study_day(event, reference))
}

/// Parse a date string for study day calculation.
/// Only dates with full day precision can be used.
fn parse_date_for_studyday(value: &str) -> Option<NaiveDate> {
    use super::datetime::parse_date_precision;
    use super::datetime::DateTimePrecision;

    match parse_date_precision(value) {
        DateTimePrecision::DateTime(dt) => Some(dt.date()),
        DateTimePrecision::Date(d) => Some(d),
        DateTimePrecision::Iso8601(s) => {
            // Try to extract date from ISO 8601 string
            if s.len() >= 10 {
                chrono::NaiveDate::parse_from_str(&s[..10], "%Y-%m-%d").ok()
            } else {
                None
            }
        }
        // Partial dates (year-month, year) cannot be used for study day
        DateTimePrecision::YearMonth { .. } | DateTimePrecision::Year(_) => None,
        DateTimePrecision::Unknown(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_day() {
        let ref_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        assert_eq!(calculate_study_day(ref_date, ref_date), 1);
    }

    #[test]
    fn test_day_after() {
        let ref_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let event = NaiveDate::from_ymd_opt(2024, 1, 16).unwrap();
        assert_eq!(calculate_study_day(event, ref_date), 2);
    }

    #[test]
    fn test_day_before() {
        let ref_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let event = NaiveDate::from_ymd_opt(2024, 1, 14).unwrap();
        assert_eq!(calculate_study_day(event, ref_date), -1);
    }

    #[test]
    fn test_two_days_before() {
        let ref_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let event = NaiveDate::from_ymd_opt(2024, 1, 13).unwrap();
        assert_eq!(calculate_study_day(event, ref_date), -2);
    }

    #[test]
    fn test_week_after() {
        let ref_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let event = NaiveDate::from_ymd_opt(2024, 1, 22).unwrap();
        assert_eq!(calculate_study_day(event, ref_date), 8);
    }

    #[test]
    fn test_no_day_zero() {
        // Day -1 and Day 1 are adjacent
        let ref_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let day_before = NaiveDate::from_ymd_opt(2024, 1, 14).unwrap();

        let study_day_before = calculate_study_day(day_before, ref_date);
        let study_day_ref = calculate_study_day(ref_date, ref_date);

        assert_eq!(study_day_before, -1);
        assert_eq!(study_day_ref, 1);
        // No day 0 between them
        assert_ne!(study_day_before, 0);
        assert_ne!(study_day_ref, 0);
    }

    #[test]
    fn test_from_strings() {
        let result = calculate_study_day_from_strings("2024-01-20", "2024-01-15");
        assert_eq!(result, Some(6));
    }

    #[test]
    fn test_from_strings_partial_date() {
        // Partial dates should return None
        let result = calculate_study_day_from_strings("2024-01", "2024-01-15");
        assert_eq!(result, None);
    }

    #[test]
    fn test_from_strings_invalid() {
        let result = calculate_study_day_from_strings("invalid", "2024-01-15");
        assert_eq!(result, None);
    }
}
