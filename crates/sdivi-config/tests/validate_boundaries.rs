//! Tests for the `validate_boundaries` function in `sdivi-config/src/load.rs`.
//!
//! `validate_boundaries` is called inside `load_with_paths` after deserialising
//! the merged TOML.  These tests drive it through the error branches
//! (`leiden_min_compression_ratio >= 1.0`, `leiden_max_recursion_depth == 0`)
//! and through the happy path using the crate's default values.

use sdivi_config::{load_with_paths, ConfigError};
use std::io::Write as _;
use tempfile::NamedTempFile;

/// Writes TOML content to a temporary file and returns the file handle.
fn toml_tempfile(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("tempfile creation must succeed");
    f.write_all(content.as_bytes())
        .expect("writing tempfile must succeed");
    f
}

// ── Error branch: leiden_min_compression_ratio ────────────────────────────────

#[test]
fn validate_boundaries_rejects_compression_ratio_at_1_0() {
    let f = toml_tempfile(
        r#"
[boundaries]
leiden_min_compression_ratio = 1.0
"#,
    );
    let err = load_with_paths(Some(f.path()), None)
        .expect_err("leiden_min_compression_ratio = 1.0 must be rejected");
    match err {
        ConfigError::InvalidValue { key, .. } => {
            assert_eq!(
                key, "boundaries.leiden_min_compression_ratio",
                "error key must name the offending field"
            );
        }
        other => panic!("expected InvalidValue, got {other:?}"),
    }
}

#[test]
fn validate_boundaries_rejects_compression_ratio_above_1_0() {
    let f = toml_tempfile(
        r#"
[boundaries]
leiden_min_compression_ratio = 1.5
"#,
    );
    let err = load_with_paths(Some(f.path()), None)
        .expect_err("leiden_min_compression_ratio = 1.5 must be rejected");
    match err {
        ConfigError::InvalidValue { key, message } => {
            assert_eq!(key, "boundaries.leiden_min_compression_ratio");
            assert!(
                message.contains("1.5"),
                "error message must echo the supplied value, got: {message}"
            );
        }
        other => panic!("expected InvalidValue, got {other:?}"),
    }
}

#[test]
fn validate_boundaries_rejects_compression_ratio_exactly_negative() {
    // -0.1 is below the valid range [0.0, 1.0) — must also be rejected.
    let f = toml_tempfile(
        r#"
[boundaries]
leiden_min_compression_ratio = -0.1
"#,
    );
    let err = load_with_paths(Some(f.path()), None)
        .expect_err("leiden_min_compression_ratio = -0.1 must be rejected");
    assert!(
        matches!(err, ConfigError::InvalidValue { .. }),
        "expected InvalidValue, got {err:?}"
    );
}

// ── Error branch: leiden_max_recursion_depth ──────────────────────────────────

#[test]
fn validate_boundaries_rejects_max_recursion_depth_zero() {
    // TOML integers are i64, but the field is u32 — 0 deserialises fine as u32
    // and must be caught by validate_boundaries.
    let f = toml_tempfile(
        r#"
[boundaries]
leiden_max_recursion_depth = 0
"#,
    );
    let err = load_with_paths(Some(f.path()), None)
        .expect_err("leiden_max_recursion_depth = 0 must be rejected");
    match err {
        ConfigError::InvalidValue { key, message } => {
            assert_eq!(
                key, "boundaries.leiden_max_recursion_depth",
                "error key must name the offending field"
            );
            assert!(
                message.contains("1"),
                "error message must mention the minimum value, got: {message}"
            );
        }
        other => panic!("expected InvalidValue, got {other:?}"),
    }
}

// ── Happy path ────────────────────────────────────────────────────────────────

#[test]
fn validate_boundaries_accepts_default_values() {
    // No project config file → all defaults.  Must succeed.
    let config =
        load_with_paths(None, None).expect("load with no config files must succeed using defaults");
    assert_eq!(config.boundaries.leiden_min_compression_ratio, 0.1);
    assert_eq!(config.boundaries.leiden_max_recursion_depth, 32);
}

#[test]
fn validate_boundaries_accepts_valid_custom_values() {
    let f = toml_tempfile(
        r#"
[boundaries]
leiden_min_compression_ratio = 0.05
leiden_max_recursion_depth = 8
"#,
    );
    let config = load_with_paths(Some(f.path()), None)
        .expect("valid boundary values must load without error");
    assert!((config.boundaries.leiden_min_compression_ratio - 0.05).abs() < 1e-12);
    assert_eq!(config.boundaries.leiden_max_recursion_depth, 8);
}

#[test]
fn validate_boundaries_accepts_compression_ratio_zero() {
    // 0.0 is the lower bound of the valid range [0.0, 1.0) — must be accepted.
    let f = toml_tempfile(
        r#"
[boundaries]
leiden_min_compression_ratio = 0.0
"#,
    );
    let config = load_with_paths(Some(f.path()), None)
        .expect("leiden_min_compression_ratio = 0.0 must be accepted");
    assert_eq!(config.boundaries.leiden_min_compression_ratio, 0.0);
}

#[test]
fn validate_boundaries_accepts_depth_1() {
    let f = toml_tempfile(
        r#"
[boundaries]
leiden_max_recursion_depth = 1
"#,
    );
    let config = load_with_paths(Some(f.path()), None)
        .expect("leiden_max_recursion_depth = 1 (minimum valid) must be accepted");
    assert_eq!(config.boundaries.leiden_max_recursion_depth, 1);
}
