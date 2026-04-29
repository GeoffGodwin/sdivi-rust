use std::time::{SystemTime, UNIX_EPOCH};

use crate::ConfigError;

/// Returns today's date as an ISO-8601 string (`"YYYY-MM-DD"`).
///
/// Uses the Gregorian calendar algorithm from the proleptic calendar system,
/// derived from the Unix epoch. No external dependencies required.
pub fn today_iso8601() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = (secs / 86400) as u32;
    let (year, month, day) = days_since_epoch_to_ymd(days);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Converts days since Unix epoch (1970-01-01) to a Gregorian `(year, month, day)` tuple.
fn days_since_epoch_to_ymd(days: u32) -> (u32, u32, u32) {
    // Algorithm: shift to a civil calendar anchored at Mar 1, year 0
    // Reference: http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d)
}

/// Returns `true` if `expires_str` is a date strictly before today (i.e., the override has expired).
///
/// `expires_str` must be in `"YYYY-MM-DD"` format. An unparseable string is treated as
/// non-expired (a separate validation step handles format errors).
pub fn is_expired(expires_str: &str) -> bool {
    // ISO-8601 date strings in YYYY-MM-DD format compare correctly lexicographically.
    expires_str < today_iso8601().as_str()
}

/// Validates that `expires_str` is a well-formed `"YYYY-MM-DD"` date string.
pub fn validate_date_format(expires_str: &str) -> bool {
    let parts: Vec<&str> = expires_str.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    let Ok(year) = parts[0].parse::<u16>() else { return false };
    let Ok(month) = parts[1].parse::<u8>() else { return false };
    let Ok(day) = parts[2].parse::<u8>() else { return false };
    parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && year >= 1970
        && (1..=12).contains(&month)
        && (1..=31).contains(&day)
}

/// Validates all threshold override entries in `table`, returning an error if any
/// override is missing the mandatory `expires` field. Removes expired overrides in-place.
///
/// `table` is the fully-merged `toml::Table` before deserialization into [`Config`].
pub fn validate_and_prune_overrides(table: &mut toml::Table) -> Result<(), ConfigError> {
    let Some(toml::Value::Table(thresholds)) = table.get_mut("thresholds") else {
        return Ok(());
    };
    let Some(toml::Value::Table(overrides)) = thresholds.get_mut("overrides") else {
        return Ok(());
    };

    let categories: Vec<String> = overrides.keys().cloned().collect();
    let mut to_remove = Vec::new();

    for category in &categories {
        let Some(toml::Value::Table(entry)) = overrides.get(category) else {
            continue;
        };
        match entry.get("expires") {
            None => {
                return Err(ConfigError::MissingExpiresOnOverride {
                    category: category.clone(),
                });
            }
            Some(toml::Value::String(date_str)) => {
                if !validate_date_format(date_str) {
                    return Err(ConfigError::InvalidValue {
                        key: format!("thresholds.overrides.{category}.expires"),
                        message: format!(
                            "expected a date in 'YYYY-MM-DD' format, got: {date_str:?}"
                        ),
                    });
                }
                if is_expired(date_str) {
                    to_remove.push(category.clone());
                }
            }
            Some(other) => {
                return Err(ConfigError::InvalidValue {
                    key: format!("thresholds.overrides.{category}.expires"),
                    message: format!("expected a string date like '2026-09-30', got: {other}"),
                });
            }
        }
    }

    for cat in to_remove {
        overrides.remove(&cat);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn today_iso8601_has_correct_format() {
        let today = today_iso8601();
        assert_eq!(today.len(), 10, "expected YYYY-MM-DD, got: {today}");
        assert_eq!(&today[4..5], "-");
        assert_eq!(&today[7..8], "-");
    }

    #[test]
    fn past_date_is_expired() {
        assert!(is_expired("2000-01-01"));
        assert!(is_expired("1970-01-01"));
    }

    #[test]
    fn future_date_is_not_expired() {
        assert!(!is_expired("2099-12-31"));
    }

    #[test]
    fn validate_date_format_accepts_valid_dates() {
        assert!(validate_date_format("2026-09-30"));
        assert!(validate_date_format("2099-12-31"));
        assert!(validate_date_format("2000-01-01"));
    }

    #[test]
    fn validate_date_format_rejects_invalid() {
        assert!(!validate_date_format("26-09-30"));
        assert!(!validate_date_format("2026-9-30"));
        assert!(!validate_date_format("2026-09-3"));
        assert!(!validate_date_format("not-a-date"));
        assert!(!validate_date_format("2026/09/30"));
    }
}
