//! Time utilities for the pipeline — clock access lives here, not in `sdi-core`.

/// Returns the current UTC time as an ISO 8601 string (`YYYY-MM-DDTHH:MM:SSZ`).
///
/// # Examples
///
/// ```rust
/// use sdi_pipeline::current_timestamp;
///
/// let ts = current_timestamp();
/// assert!(ts.ends_with('Z'));
/// assert_eq!(ts.len(), 20);
/// ```
pub fn current_timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    unix_to_iso8601(secs)
}

fn unix_to_iso8601(secs: u64) -> String {
    let secs = secs as i64;
    let days_since_epoch = secs / 86400;
    let time_secs = secs % 86400;
    let j = days_since_epoch + 2440588;
    let f = j + 1401 + (((4 * j + 274277) / 146097) * 3) / 4 - 38;
    let e = 4 * f + 3;
    let g = (e % 1461) / 4;
    let h = 5 * g + 2;
    let day = (h % 153) / 5 + 1;
    let month = (h / 153 + 2) % 12 + 1;
    let year = e / 1461 - 4716 + (14 - month) / 12;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day,
        time_secs / 3600, (time_secs % 3600) / 60, time_secs % 60,
    )
}

#[cfg(test)]
mod tests {
    use super::unix_to_iso8601;

    #[test]
    fn epoch() {
        assert_eq!(unix_to_iso8601(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn y2k() {
        assert_eq!(unix_to_iso8601(946684800), "2000-01-01T00:00:00Z");
    }

    #[test]
    fn year_end_second() {
        assert_eq!(unix_to_iso8601(1704067199), "2023-12-31T23:59:59Z");
    }

    #[test]
    fn leap_day_2024() {
        assert_eq!(unix_to_iso8601(1709164800), "2024-02-29T00:00:00Z");
    }
}
