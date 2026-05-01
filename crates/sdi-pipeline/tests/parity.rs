//! Parity test: `compute_pattern_metrics_from_catalog` (pipeline path) and
//! `sdi_core::compute_pattern_metrics` (pure-compute path) produce identical
//! `convention_drift_per_category` values for the same input.

use std::path::Path;

use sdi_config::Config;
use sdi_core::{compute_pattern_metrics, PatternInstanceInput};
use sdi_lang_rust::RustAdapter;
use sdi_pipeline::Pipeline;

fn fixture_root() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/simple-rust"
    ))
}

/// The pipeline's `PatternMetricsResult` must match the pure-compute path for
/// `convention_drift_per_category` on real fixture data.
#[test]
fn convention_drift_per_category_matches_pure_compute() {
    let root = fixture_root();
    let adapters: Vec<Box<dyn sdi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];
    let pipeline = Pipeline::new(Config::default(), adapters);

    let snap = pipeline
        .snapshot(root, None, "2026-05-01T00:00:00Z")
        .expect("snapshot must succeed on simple-rust fixture");

    // The pipeline-computed per-category map.
    let pipeline_map = &snap.pattern_metrics.convention_drift_per_category;

    // Build the pure-compute path result from the catalog entries.
    // We convert catalog fingerprints into PatternInstanceInput to feed compute_pattern_metrics.
    let instances: Vec<PatternInstanceInput> = snap
        .catalog
        .entries
        .iter()
        .flat_map(|(cat, fps)| {
            fps.iter().flat_map(move |(fp, stats)| {
                std::iter::repeat_with(move || PatternInstanceInput {
                    fingerprint: fp.to_hex(),
                    category: cat.clone(),
                    node_id: "fixture".to_string(),
                    location: None,
                })
                .take(stats.count as usize)
            })
        })
        .collect();

    let pure_result = compute_pattern_metrics(&instances);

    // Per-category keys and drift values must match.
    assert_eq!(
        pipeline_map.len(),
        pure_result.convention_drift_per_category.len(),
        "category count must match between pipeline and pure-compute paths"
    );

    for (cat, pipeline_val) in pipeline_map {
        let pure_val = pure_result
            .convention_drift_per_category
            .get(cat)
            .unwrap_or_else(|| panic!("category {cat} missing from pure-compute result"));
        assert!(
            (pipeline_val - pure_val).abs() < 1e-10,
            "convention_drift_per_category[{cat}]: pipeline={pipeline_val}, pure={pure_val}"
        );
    }
}
