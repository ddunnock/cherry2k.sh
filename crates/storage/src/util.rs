//! Shared utilities for the storage crate.

use chrono::{DateTime, Utc};

/// Parses a SQLite datetime string into a DateTime<Utc>.
///
/// SQLite stores datetimes as "YYYY-MM-DD HH:MM:SS" strings.
/// Returns the current time if parsing fails.
pub fn parse_datetime(s: &str) -> DateTime<Utc> {
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc())
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn parses_sqlite_format() {
        let dt = parse_datetime("2026-01-30 14:23:45");
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 30);
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 23);
        assert_eq!(dt.second(), 45);
    }

    #[test]
    fn returns_now_for_invalid() {
        let dt = parse_datetime("invalid");
        let now = Utc::now();
        let diff = (now - dt).num_seconds().abs();
        assert!(diff < 5);
    }
}
