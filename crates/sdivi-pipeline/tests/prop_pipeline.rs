//! Property test: the pipeline produces bit-identical Snapshot JSON for the
//! same input (determinism guarantee — Critical Rule 1).
//!
//! Uses the `simple-rust` fixture and a fixed seed so the test is fast and
//! does not require generating synthetic repos.

use std::path::Path;

use proptest::prelude::*;
use sdivi_config::Config;
use sdivi_lang_rust::RustAdapter;
use sdivi_pipeline::{Pipeline, WriteMode};

fn fixture_root() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/simple-rust"
    ))
}

fn run_pipeline(seed: u64) -> String {
    let mut config = Config::default();
    config.core.random_seed = seed;
    let pipeline = Pipeline::new(config, vec![Box::new(RustAdapter)]);
    let snap = pipeline
        .snapshot_with_mode(
            fixture_root(),
            None,
            "2026-01-01T00:00:00Z",
            WriteMode::EphemeralForCheck,
        )
        .expect("pipeline must succeed on simple-rust fixture");
    serde_json::to_string(&snap).expect("snapshot must serialize")
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    /// Same seed → bit-identical Snapshot JSON.
    ///
    /// Runs the full five-stage pipeline twice on the simple-rust fixture with
    /// the same seed and asserts JSON equality.  This validates Critical Rule 1
    /// (pipeline determinism) end-to-end.
    #[test]
    fn prop_test_pipeline_deterministic(seed in 0u64..1000) {
        let a = run_pipeline(seed);
        let b = run_pipeline(seed);
        prop_assert_eq!(a, b, "pipeline must produce bit-identical JSON for seed={}", seed);
    }
}
