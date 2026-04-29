use std::io::Write;

use tempfile::NamedTempFile;

use sdi_config::{load_with_paths, ConfigError};

fn write_toml(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("tempfile");
    f.write_all(content.as_bytes()).expect("write tempfile");
    f
}

// ── Missing expires → ConfigError::MissingExpiresOnOverride ──────────────────

#[test]
fn missing_expires_returns_typed_error() {
    let toml = r#"
[thresholds.overrides.error_handling]
pattern_entropy_rate = 5.0
reason = "migrating"
"#;
    let file = write_toml(toml);
    let err = load_with_paths(Some(file.path()), None).unwrap_err();
    match err {
        ConfigError::MissingExpiresOnOverride { ref category } => {
            assert_eq!(category, "error_handling");
        }
        other => panic!("expected MissingExpiresOnOverride, got: {other:?}"),
    }
}

#[test]
fn missing_expires_error_names_category() {
    let toml = r#"
[thresholds.overrides.async_patterns]
convention_drift_rate = 4.0
"#;
    let file = write_toml(toml);
    let err = load_with_paths(Some(file.path()), None).unwrap_err();
    assert!(
        err.to_string().contains("async_patterns"),
        "error message must name the category, got: {err}"
    );
}

// ── Expired override → silently ignored ──────────────────────────────────────

#[test]
fn expired_override_is_silently_ignored() {
    let toml = r#"
[thresholds.overrides.error_handling]
pattern_entropy_rate = 99.0
expires = "2000-01-01"
reason = "very old migration"
"#;
    let file = write_toml(toml);
    let config = load_with_paths(Some(file.path()), None)
        .expect("expired override must not cause an error");
    // The override should have been pruned: default threshold applies.
    assert!(
        config.thresholds.overrides.is_empty(),
        "expired override must be removed: {:?}",
        config.thresholds.overrides
    );
    assert!(
        (config.thresholds.pattern_entropy_rate - 2.0).abs() < f64::EPSILON,
        "default threshold must be restored after expiry"
    );
}

#[test]
fn multiple_overrides_some_expired() {
    let toml = r#"
[thresholds.overrides.expired_cat]
pattern_entropy_rate = 10.0
expires = "2000-01-01"

[thresholds.overrides.valid_cat]
convention_drift_rate = 5.0
expires = "2099-12-31"
"#;
    let file = write_toml(toml);
    let config = load_with_paths(Some(file.path()), None).unwrap();
    assert!(
        !config.thresholds.overrides.contains_key("expired_cat"),
        "expired category must be removed"
    );
    assert!(
        config.thresholds.overrides.contains_key("valid_cat"),
        "valid category must be kept"
    );
    let valid = &config.thresholds.overrides["valid_cat"];
    assert!(
        (valid.convention_drift_rate.unwrap() - 5.0).abs() < f64::EPSILON
    );
}

// ── Valid override → applied ──────────────────────────────────────────────────

#[test]
fn valid_override_is_applied() {
    let toml = r#"
[thresholds.overrides.error_handling]
pattern_entropy_rate = 5.0
expires = "2099-12-31"
reason = "active migration"
"#;
    let file = write_toml(toml);
    let config = load_with_paths(Some(file.path()), None).unwrap();
    assert!(
        config.thresholds.overrides.contains_key("error_handling"),
        "valid override must be present"
    );
    let ov = &config.thresholds.overrides["error_handling"];
    assert!(
        (ov.pattern_entropy_rate.unwrap() - 5.0).abs() < f64::EPSILON
    );
    assert_eq!(ov.expires, "2099-12-31");
    assert_eq!(ov.reason.as_deref(), Some("active migration"));
}

// ── Invalid expires format → ConfigError::InvalidValue ───────────────────────

#[test]
fn malformed_expires_returns_invalid_value() {
    let toml = r#"
[thresholds.overrides.bad_format]
pattern_entropy_rate = 3.0
expires = "not-a-date"
"#;
    let file = write_toml(toml);
    let err = load_with_paths(Some(file.path()), None).unwrap_err();
    match err {
        ConfigError::InvalidValue { ref key, .. } => {
            assert!(key.contains("expires"), "error key must reference expires: {key}");
        }
        other => panic!("expected InvalidValue, got: {other:?}"),
    }
}

// ── Non-String expires (e.g. integer) → ConfigError::InvalidValue ─────────────
//
// Covers the reviewer-flagged gap: the `Some(other)` branch of
// `validate_and_prune_overrides` — where `expires` is a TOML value that is
// present but not a String — was not exercised by any prior test.

#[test]
fn integer_expires_returns_invalid_value() {
    let toml = r#"
[thresholds.overrides.int_expires_cat]
pattern_entropy_rate = 3.0
expires = 20261231
"#;
    let file = write_toml(toml);
    let err = load_with_paths(Some(file.path()), None)
        .expect_err("integer expires must cause an error");

    match err {
        ConfigError::InvalidValue { ref key, ref message } => {
            assert!(
                key.contains("expires"),
                "error key must reference the expires field, got: {key}"
            );
            assert!(
                key.contains("int_expires_cat"),
                "error key must name the category, got: {key}"
            );
            assert!(
                !message.is_empty(),
                "error message must be non-empty"
            );
        }
        other => panic!("expected ConfigError::InvalidValue, got: {other:?}"),
    }
}

#[test]
fn boolean_expires_returns_invalid_value() {
    let toml = r#"
[thresholds.overrides.bool_expires_cat]
convention_drift_rate = 4.0
expires = true
"#;
    let file = write_toml(toml);
    let err = load_with_paths(Some(file.path()), None)
        .expect_err("boolean expires must cause an error");

    match err {
        ConfigError::InvalidValue { ref key, .. } => {
            assert!(
                key.contains("bool_expires_cat"),
                "error key must name the category, got: {key}"
            );
        }
        other => panic!("expected ConfigError::InvalidValue for boolean expires, got: {other:?}"),
    }
}

#[test]
fn non_string_expires_error_message_includes_actual_value() {
    // Verifies the error message quotes the actual bad value for diagnostics.
    let toml = r#"
[thresholds.overrides.diag_cat]
pattern_entropy_rate = 2.5
expires = 99
"#;
    let file = write_toml(toml);
    let err = load_with_paths(Some(file.path()), None)
        .expect_err("non-string expires must error");

    let msg = err.to_string();
    assert!(
        msg.contains("99") || msg.contains("expected a string"),
        "error message should include the actual value or expected type, got: {msg}"
    );
}
