//! Time helpers.

use chrono::{DateTime, Utc};

/// Returns a RFC3339 UTC timestamp string.
pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

/// Parses an RFC3339 timestamp to a UTC datetime.
pub fn parse_rfc3339(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}
