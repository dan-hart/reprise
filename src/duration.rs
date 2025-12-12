//! Duration parsing for time-based filtering
//!
//! Supports formats:
//! - Relative: 1h, 30m, 2d, 1w
//! - Named: today, yesterday, this-week
//! - ISO 8601: 2025-12-11T09:00:00

use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};

use crate::error::{RepriseError, Result};

/// Parse a duration string and return the UTC datetime threshold
///
/// Builds triggered at or after this threshold will be included.
pub fn parse_since(s: &str) -> Result<DateTime<Utc>> {
    let s = s.trim().to_lowercase();
    let now = Local::now();

    // Try named durations first
    if let Some(dt) = parse_named_duration(&s, now) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try relative duration (e.g., 1h, 30m, 2d)
    if let Some(dt) = parse_relative_duration(&s, now) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try ISO 8601 / date formats
    if let Some(dt) = parse_datetime_string(&s) {
        return Ok(dt);
    }

    Err(RepriseError::InvalidArgument(format!(
        "Invalid duration format: '{}'. Use formats like: 1h, 30m, 2d, 1w, today, yesterday, this-week, or ISO 8601",
        s
    )))
}

/// Parse named duration keywords
fn parse_named_duration(s: &str, now: DateTime<Local>) -> Option<DateTime<Local>> {
    match s {
        "today" => {
            // Start of today (midnight)
            let today = now.date_naive();
            Some(Local.from_local_datetime(&today.and_hms_opt(0, 0, 0)?).single()?)
        }
        "yesterday" => {
            // Start of yesterday (midnight)
            let yesterday = now.date_naive() - Duration::days(1);
            Some(Local.from_local_datetime(&yesterday.and_hms_opt(0, 0, 0)?).single()?)
        }
        "this-week" | "thisweek" | "week" => {
            // Start of current week (Monday midnight)
            let today = now.date_naive();
            let days_since_monday = today.weekday().num_days_from_monday();
            let monday = today - Duration::days(days_since_monday as i64);
            Some(Local.from_local_datetime(&monday.and_hms_opt(0, 0, 0)?).single()?)
        }
        "this-month" | "thismonth" | "month" => {
            // Start of current month
            let today = now.date_naive();
            let first = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)?;
            Some(Local.from_local_datetime(&first.and_hms_opt(0, 0, 0)?).single()?)
        }
        _ => None,
    }
}

/// Parse relative duration (e.g., 1h, 30m, 2d, 1w)
fn parse_relative_duration(s: &str, now: DateTime<Local>) -> Option<DateTime<Local>> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Find where the number ends and the unit begins
    let (num_str, unit) = if s.ends_with(char::is_alphabetic) {
        let split_idx = s.rfind(|c: char| c.is_ascii_digit())?;
        (&s[..=split_idx], &s[split_idx + 1..])
    } else {
        return None;
    };

    let num: i64 = num_str.parse().ok()?;
    if num <= 0 {
        return None;
    }

    let duration = match unit.to_lowercase().as_str() {
        "m" | "min" | "mins" | "minute" | "minutes" => Duration::minutes(num),
        "h" | "hr" | "hrs" | "hour" | "hours" => Duration::hours(num),
        "d" | "day" | "days" => Duration::days(num),
        "w" | "week" | "weeks" => Duration::weeks(num),
        _ => return None,
    };

    Some(now - duration)
}

/// Parse datetime string (ISO 8601 or date-only)
fn parse_datetime_string(s: &str) -> Option<DateTime<Utc>> {
    // Try full ISO 8601 with timezone
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }

    // Try ISO 8601 without timezone (assume local)
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        let local_dt = Local.from_local_datetime(&ndt).single()?;
        return Some(local_dt.with_timezone(&Utc));
    }

    // Try date only (assume start of day, local time)
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let ndt = date.and_hms_opt(0, 0, 0)?;
        let local_dt = Local.from_local_datetime(&ndt).single()?;
        return Some(local_dt.with_timezone(&Utc));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_parse_relative_hours() {
        let result = parse_since("1h").unwrap();
        let expected = Utc::now() - Duration::hours(1);
        // Allow 1 second tolerance
        assert!((result - expected).num_seconds().abs() < 2);
    }

    #[test]
    fn test_parse_relative_minutes() {
        let result = parse_since("30m").unwrap();
        let expected = Utc::now() - Duration::minutes(30);
        assert!((result - expected).num_seconds().abs() < 2);
    }

    #[test]
    fn test_parse_relative_days() {
        let result = parse_since("2d").unwrap();
        let expected = Utc::now() - Duration::days(2);
        assert!((result - expected).num_seconds().abs() < 2);
    }

    #[test]
    fn test_parse_relative_weeks() {
        let result = parse_since("1w").unwrap();
        let expected = Utc::now() - Duration::weeks(1);
        assert!((result - expected).num_seconds().abs() < 2);
    }

    #[test]
    fn test_parse_today() {
        let result = parse_since("today").unwrap();
        let now = Local::now();
        let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let expected = Local.from_local_datetime(&today_start).unwrap().with_timezone(&Utc);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_yesterday() {
        let result = parse_since("yesterday").unwrap();
        let now = Local::now();
        let yesterday = now.date_naive() - Duration::days(1);
        let yesterday_start = yesterday.and_hms_opt(0, 0, 0).unwrap();
        let expected = Local.from_local_datetime(&yesterday_start).unwrap().with_timezone(&Utc);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_this_week() {
        let result = parse_since("this-week").unwrap();
        let now = Local::now();
        let today = now.date_naive();
        let days_since_monday = today.weekday().num_days_from_monday();
        let monday = today - Duration::days(days_since_monday as i64);
        let monday_start = monday.and_hms_opt(0, 0, 0).unwrap();
        let expected = Local.from_local_datetime(&monday_start).unwrap().with_timezone(&Utc);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_iso_date() {
        let result = parse_since("2025-01-15").unwrap();
        let expected_date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        assert_eq!(result.date_naive(), expected_date);
    }

    #[test]
    fn test_invalid_format() {
        assert!(parse_since("invalid").is_err());
        assert!(parse_since("abc123").is_err());
        assert!(parse_since("").is_err());
    }

    #[test]
    fn test_case_insensitive() {
        assert!(parse_since("TODAY").is_ok());
        assert!(parse_since("Yesterday").is_ok());
        assert!(parse_since("1H").is_ok());
        assert!(parse_since("2D").is_ok());
    }
}
