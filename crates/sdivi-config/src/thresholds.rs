use crate::ConfigError;

/// Validates that `expires_str` is a well-formed `"YYYY-MM-DD"` date string.
///
/// Does not check leap-year validity; Feb 29 is accepted for any year.
///
/// # Examples
///
/// ```rust
/// use sdivi_config::validate_date_format;
///
/// assert!(validate_date_format("2026-09-30"));
/// assert!(!validate_date_format("26-09-30"));
/// ```
pub fn validate_date_format(expires_str: &str) -> bool {
    let parts: Vec<&str> = expires_str.split('-').collect();
    if parts.len() != 3 {
        return false;
    }
    let Ok(year) = parts[0].parse::<u16>() else {
        return false;
    };
    let Ok(month) = parts[1].parse::<u8>() else {
        return false;
    };
    let Ok(day) = parts[2].parse::<u8>() else {
        return false;
    };
    let max_day: u8 = match month {
        2 => 29,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    parts[0].len() == 4
        && parts[1].len() == 2
        && parts[2].len() == 2
        && year >= 1970
        && (1..=12).contains(&month)
        && (1..=max_day).contains(&day)
}

/// Validates expires format and prunes expired overrides from `table`.
///
/// Validates that every override has a well-formed `expires` field in
/// `"YYYY-MM-DD"` format, then removes any overrides whose expiry is strictly
/// before today.  After expiry the override is silently ignored and defaults
/// resume — per project rules.
///
/// `today` is the current date as a `"YYYY-MM-DD"` string for comparison.
/// `table` is the fully-merged `toml::Table` before deserialization into
/// [`Config`].
pub(crate) fn validate_and_prune_overrides(
    table: &mut toml::Table,
    today: &str,
) -> Result<(), ConfigError> {
    let Some(toml::Value::Table(thresholds)) = table.get_mut("thresholds") else {
        return Ok(());
    };
    let Some(toml::Value::Table(overrides)) = thresholds.get_mut("overrides") else {
        return Ok(());
    };

    let categories: Vec<String> = overrides.keys().cloned().collect();
    let mut to_remove: Vec<String> = Vec::new();

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
                // Prune expired: date_str < today means it expired before today.
                if date_str.as_str() < today {
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

    for cat in &to_remove {
        overrides.remove(cat);
    }

    Ok(())
}

/// Returns today's date as a `"YYYY-MM-DD"` string using the system clock.
#[cfg(feature = "loader")]
pub(crate) fn today_iso8601() -> String {
    use chrono::Datelike;
    let today = chrono::Local::now().date_naive();
    format!(
        "{:04}-{:02}-{:02}",
        today.year(),
        today.month(),
        today.day()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn validate_date_format_rejects_impossible_days() {
        assert!(!validate_date_format("2026-02-30"));
        assert!(!validate_date_format("2026-04-31"));
        assert!(!validate_date_format("2026-06-31"));
        assert!(validate_date_format("2026-02-29"));
        assert!(validate_date_format("2026-12-31"));
    }

    // ── validate_and_prune_overrides: strict-less-than boundary ──────────────

    fn make_override_table(expires: &str) -> toml::Table {
        toml::from_str(&format!(
            "[thresholds.overrides.test_cat]\npattern_entropy_rate = 5.0\nexpires = \"{expires}\"\n"
        ))
        .expect("valid TOML")
    }

    fn override_present(table: &toml::Table) -> bool {
        table
            .get("thresholds")
            .and_then(|t| t.as_table())
            .and_then(|t| t.get("overrides"))
            .and_then(|t| t.as_table())
            .map(|ov| ov.contains_key("test_cat"))
            .unwrap_or(false)
    }

    /// An override whose `expires` equals today must be KEPT.
    ///
    /// The predicate is `date_str < today` (strict less-than), so an override
    /// expiring today is not yet expired.
    #[test]
    fn expires_equal_to_today_is_kept() {
        let today = "2026-04-29";
        let mut table = make_override_table(today);
        validate_and_prune_overrides(&mut table, today).expect("valid expires must not error");
        assert!(
            override_present(&table),
            "override expiring on today ({today}) must be kept because expires == today is not < today"
        );
    }

    /// An override expiring one day before today must be PRUNED.
    ///
    /// "2026-04-28" < "2026-04-29" is true under lexicographic ISO comparison.
    #[test]
    fn expires_one_day_before_today_is_pruned() {
        let today = "2026-04-29";
        let yesterday = "2026-04-28";
        let mut table = make_override_table(yesterday);
        validate_and_prune_overrides(&mut table, today).expect("valid expires must not error");
        assert!(
            !override_present(&table),
            "override expiring yesterday ({yesterday}) must be pruned (strictly before today {today})"
        );
    }

    /// An override expiring one day after today must be KEPT.
    ///
    /// Verifies the boundary from the other direction.
    #[test]
    fn expires_one_day_after_today_is_kept() {
        let today = "2026-04-29";
        let tomorrow = "2026-04-30";
        let mut table = make_override_table(tomorrow);
        validate_and_prune_overrides(&mut table, today).expect("valid expires must not error");
        assert!(
            override_present(&table),
            "override expiring tomorrow ({tomorrow}) must be kept"
        );
    }
}
