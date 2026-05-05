use std::path::Path;

use sdivi_config::{load_with_paths, BoundarySpec};

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

/// Python POC config.toml loads without error and values are preserved.
#[test]
fn legacy_config_toml_loads_cleanly() {
    let config_path = Path::new(FIXTURES).join("legacy_config.toml");
    let config = load_with_paths(Some(&config_path), None)
        .expect("Python POC style config.toml must load without error");

    // Values from the fixture (not defaults) are preserved.
    assert_eq!(config.snapshots.retention, 50);
    assert!((config.boundaries.leiden_gamma - 1.2).abs() < f64::EPSILON);
    assert_eq!(config.boundaries.stability_threshold, 5);
    assert_eq!(config.patterns.min_pattern_nodes, 3);
    assert_eq!(config.patterns.scope_exclude, vec!["**/tests/**"]);
    assert!((config.thresholds.pattern_entropy_rate - 3.0).abs() < f64::EPSILON);
    assert!((config.change_coupling.min_frequency - 0.7).abs() < f64::EPSILON);
    assert_eq!(config.change_coupling.history_depth, 200);
}

/// Python POC boundaries.yaml loads without error and boundaries are preserved.
#[test]
fn legacy_boundaries_yaml_loads_cleanly() {
    let path = Path::new(FIXTURES).join("legacy_boundaries.yaml");
    let spec = BoundarySpec::load(&path).expect("BoundarySpec::load must not error");
    let spec = spec.expect("fixture boundaries.yaml must be found");

    assert_eq!(spec.version.as_deref(), Some("1"));
    assert_eq!(spec.boundaries.len(), 3);

    let api = spec
        .boundaries
        .iter()
        .find(|b| b.name == "api")
        .expect("api boundary");
    assert_eq!(api.allow_imports_from, vec!["models", "utils"]);
    assert!(api.description.as_deref().is_some());

    let models = spec
        .boundaries
        .iter()
        .find(|b| b.name == "models")
        .expect("models boundary");
    assert!(models.allow_imports_from.is_empty());
}

/// Missing boundaries.yaml is normal — returns None, not an error.
#[test]
fn missing_boundaries_yaml_returns_none() {
    let path = Path::new(FIXTURES).join("nonexistent_boundaries.yaml");
    let result = BoundarySpec::load(&path).expect("missing file must not error");
    assert!(result.is_none(), "missing file must return None");
}

/// sdivi-rust-only config sections ([determinism], [bindings]) are absent from
/// Python POC configs — must still load with built-in defaults for those sections.
#[test]
fn legacy_config_without_rust_only_sections_uses_defaults() {
    let config_path = Path::new(FIXTURES).join("legacy_config.toml");
    let config = load_with_paths(Some(&config_path), None).unwrap();
    // [determinism] not in Python POC config — default applies.
    assert!(config.determinism.enforce_btree_order);
}

/// `load_with_paths` does not error on an empty config file (all defaults apply).
#[test]
fn empty_config_file_succeeds() {
    use std::io::Write;
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(b"").unwrap();
    let config = load_with_paths(Some(f.path()), None).expect("empty config file must succeed");
    assert_eq!(config.core.random_seed, 42);
}

/// `BoundarySpec::load` with a file that exists but contains invalid YAML returns
/// `ConfigError::BoundaryParse`, not a panic or a generic I/O error.
///
/// Covers the reviewer-flagged gap: no test exercised `ConfigError::BoundaryParse`.
#[test]
fn boundary_spec_load_invalid_yaml_returns_boundary_parse_error() {
    use sdivi_config::ConfigError;
    use std::io::Write;

    let mut f = tempfile::NamedTempFile::new().unwrap();
    // `boundaries` expects Vec<BoundaryDef>; a plain string forces a serde_yml error.
    f.write_all(b"boundaries: \"this must be a sequence, not a string\"\n")
        .expect("write fixture");

    let err = BoundarySpec::load(f.path()).expect_err("load of invalid YAML must return an error");

    match err {
        ConfigError::BoundaryParse(ref msg) => {
            assert!(
                !msg.is_empty(),
                "BoundaryParse error message must be non-empty"
            );
        }
        other => panic!("expected ConfigError::BoundaryParse, got: {other:?}"),
    }
}

/// The error message from `BoundaryParse` contains enough detail to be actionable.
#[test]
fn boundary_parse_error_message_is_non_empty() {
    use std::io::Write;

    let mut f = tempfile::NamedTempFile::new().unwrap();
    // Syntactically invalid YAML: a bare tab is illegal in flow context.
    f.write_all(b"key: [unclosed bracket\n")
        .expect("write fixture");

    let err =
        BoundarySpec::load(f.path()).expect_err("syntactically invalid YAML must return an error");

    let msg = err.to_string();
    assert!(
        !msg.is_empty(),
        "error message must be non-empty for actionable diagnostics"
    );
    assert!(
        msg.contains("boundary") || msg.contains("parse") || msg.contains("failed"),
        "error message should reference parsing failure; got: {msg}"
    );
}
