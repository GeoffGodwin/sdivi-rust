//! Verifies that the high-entropy fixture produces a higher entropy score
//! than the simple-rust fixture across at least one pattern category.

use std::path::Path;

use sdivi_config::{Config, PatternsConfig};
use sdivi_lang_rust::RustAdapter;
use sdivi_parsing::parse::parse_repository;
use sdivi_patterns::{build_catalog, compute_entropy};

fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

fn total_entropy(fixture: &str, config: &Config, patterns_config: &PatternsConfig) -> f64 {
    let root = fixture_path(fixture);
    let adapters: Vec<Box<dyn sdivi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];
    let records = parse_repository(config, &root, &adapters);
    let catalog = build_catalog(&records, patterns_config);
    catalog.entries.values().map(compute_entropy).sum::<f64>()
}

/// The high-entropy fixture has a higher total entropy score than simple-rust.
#[test]
fn high_entropy_greater_than_simple_rust() {
    let config = Config::default();
    let mut patterns_config = config.patterns.clone();
    // Use min=1 so no patterns are filtered out, making the comparison meaningful.
    patterns_config.min_pattern_nodes = 1;

    let simple_entropy = total_entropy("simple-rust", &config, &patterns_config);
    let high_entropy = total_entropy("high-entropy", &config, &patterns_config);

    assert!(
        high_entropy > simple_entropy,
        "high-entropy fixture ({high_entropy:.4}) must exceed simple-rust ({simple_entropy:.4})"
    );
}

/// simple-rust has zero entropy in error_handling (only one distinct shape).
#[test]
fn simple_rust_error_handling_entropy_is_zero() {
    let config = Config::default();
    let mut patterns_config = config.patterns.clone();
    patterns_config.min_pattern_nodes = 1;

    let root = fixture_path("simple-rust");
    let adapters: Vec<Box<dyn sdivi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];
    let records = parse_repository(&config, &root, &adapters);
    let catalog = build_catalog(&records, &patterns_config);

    // simple-rust only has match_expression → one distinct shape → entropy 0
    let h = catalog
        .entries
        .get("error_handling")
        .map(compute_entropy)
        .unwrap_or(0.0);
    assert!(
        h < 1e-10,
        "simple-rust error_handling entropy must be 0 (only one distinct shape), got {h}"
    );
}

/// high-entropy has positive entropy in error_handling (two distinct shapes).
#[test]
fn high_entropy_error_handling_entropy_is_positive() {
    let config = Config::default();
    let mut patterns_config = config.patterns.clone();
    patterns_config.min_pattern_nodes = 1;

    let root = fixture_path("high-entropy");
    let adapters: Vec<Box<dyn sdivi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];
    let records = parse_repository(&config, &root, &adapters);
    let catalog = build_catalog(&records, &patterns_config);

    let h = catalog
        .entries
        .get("error_handling")
        .map(compute_entropy)
        .unwrap_or(0.0);
    assert!(
        h > 0.0,
        "high-entropy fixture must have positive error_handling entropy (try + match), got {h}"
    );
}
