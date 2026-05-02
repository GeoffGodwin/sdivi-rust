use sdivi_config::{ColorChoice, Config, OutputFormat};

/// Verifies that every field of `Config::default()` matches the values
/// documented in DESIGN and CLAUDE.md.
#[test]
fn core_defaults() {
    let cfg = Config::default();
    assert_eq!(cfg.core.languages, "auto");
    assert_eq!(cfg.core.random_seed, 42);
    let expected_excludes = vec![
        "**/vendor/**",
        "**/node_modules/**",
        "**/__pycache__/**",
        "**/dist/**",
        "**/build/**",
        "**/target/**",
        "**/.git/**",
    ];
    assert_eq!(cfg.core.exclude, expected_excludes);
}

#[test]
fn snapshot_defaults() {
    let cfg = Config::default();
    assert_eq!(cfg.snapshots.dir, ".sdivi/snapshots");
    assert_eq!(cfg.snapshots.retention, 100);
}

#[test]
fn boundaries_defaults() {
    let cfg = Config::default();
    assert_eq!(cfg.boundaries.spec_file, ".sdivi/boundaries.yaml");
    assert!((cfg.boundaries.leiden_gamma - 1.0).abs() < f64::EPSILON);
    assert_eq!(cfg.boundaries.stability_threshold, 3);
    assert!(!cfg.boundaries.weighted_edges);
}

#[test]
fn patterns_defaults() {
    let cfg = Config::default();
    assert_eq!(cfg.patterns.categories, "auto");
    assert_eq!(cfg.patterns.min_pattern_nodes, 5);
    assert!(cfg.patterns.scope_exclude.is_empty());
}

#[test]
fn thresholds_defaults() {
    let cfg = Config::default();
    assert!((cfg.thresholds.pattern_entropy_rate - 2.0).abs() < f64::EPSILON);
    assert!((cfg.thresholds.convention_drift_rate - 3.0).abs() < f64::EPSILON);
    assert!((cfg.thresholds.coupling_delta_rate - 0.15).abs() < f64::EPSILON);
    assert!((cfg.thresholds.boundary_violation_rate - 2.0).abs() < f64::EPSILON);
    assert!(cfg.thresholds.overrides.is_empty());
}

#[test]
fn change_coupling_defaults() {
    let cfg = Config::default();
    assert!((cfg.change_coupling.min_frequency - 0.6).abs() < f64::EPSILON);
    assert_eq!(cfg.change_coupling.history_depth, 500);
}

#[test]
fn output_defaults() {
    let cfg = Config::default();
    assert_eq!(cfg.output.format, OutputFormat::Text);
    assert_eq!(cfg.output.color, ColorChoice::Auto);
}

#[test]
fn determinism_defaults() {
    let cfg = Config::default();
    assert!(cfg.determinism.enforce_btree_order);
}

#[test]
fn load_or_default_returns_defaults() {
    let cfg = sdivi_config::load_or_default(std::path::Path::new(".")).unwrap();
    assert_eq!(cfg.core.random_seed, 42);
}
