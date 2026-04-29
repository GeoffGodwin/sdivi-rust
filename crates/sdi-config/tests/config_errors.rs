use sdi_config::ConfigError;

#[test]
fn missing_expires_on_override_display_contains_category() {
    let err = ConfigError::MissingExpiresOnOverride {
        category: "error_handling".to_string(),
    };
    let msg = err.to_string();
    assert!(
        msg.contains("error_handling"),
        "expected category name in message, got: {msg}"
    );
    assert!(
        msg.contains("expires"),
        "expected 'expires' in message, got: {msg}"
    );
}

#[test]
fn missing_expires_on_override_uses_supplied_category_name() {
    let categories = ["async_patterns", "convention_drift", "boundary_violations"];
    for cat in categories {
        let err = ConfigError::MissingExpiresOnOverride {
            category: cat.to_string(),
        };
        assert!(
            err.to_string().contains(cat),
            "category '{cat}' missing from error message: {}",
            err
        );
    }
}

#[test]
fn invalid_value_display_contains_key_and_message() {
    let err = ConfigError::InvalidValue {
        key: "core.random_seed".to_string(),
        message: "must be a non-negative integer".to_string(),
    };
    let msg = err.to_string();
    assert!(
        msg.contains("core.random_seed"),
        "expected key in message, got: {msg}"
    );
    assert!(
        msg.contains("must be a non-negative integer"),
        "expected message text in output, got: {msg}"
    );
}

#[test]
fn invalid_value_display_uses_supplied_key() {
    let keys = ["thresholds.pattern_entropy_rate", "snapshots.retention", "boundaries.leiden_gamma"];
    for key in keys {
        let err = ConfigError::InvalidValue {
            key: key.to_string(),
            message: "test".to_string(),
        };
        assert!(
            err.to_string().contains(key),
            "key '{key}' missing from error display: {}",
            err
        );
    }
}

#[test]
fn parse_error_display_contains_detail() {
    let detail = "unexpected end of input at line 7";
    let err = ConfigError::Parse(detail.to_string());
    let msg = err.to_string();
    assert!(
        msg.contains(detail),
        "expected parse detail in message, got: {msg}"
    );
}

#[test]
fn config_error_variants_are_debug_formattable() {
    let errors: Vec<ConfigError> = vec![
        ConfigError::Parse("bad TOML".to_string()),
        ConfigError::InvalidValue {
            key: "k".to_string(),
            message: "m".to_string(),
        },
        ConfigError::MissingExpiresOnOverride {
            category: "cat".to_string(),
        },
    ];
    for err in errors {
        let debug = format!("{err:?}");
        assert!(!debug.is_empty(), "Debug output must be non-empty");
    }
}
