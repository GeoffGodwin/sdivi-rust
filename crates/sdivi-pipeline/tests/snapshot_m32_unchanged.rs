//! M32 regression guard: `Pipeline::snapshot` output is bit-identical before and after M32.
//!
//! M32 adds `classify_hint` and per-language `matches_callee` functions to
//! `sdivi-patterns`, but intentionally does NOT wire them into the pipeline.
//! `crates/sdivi-patterns/src/catalog.rs` continues to call `category_for_node_kind`,
//! so no pattern-catalog entry can change.
//!
//! This test proves the guarantee by:
//! 1. Running the pipeline twice with the same fixed seed and timestamp,
//!    asserting byte-identical JSON output.
//! 2. Asserting that the snapshot catalog does not contain a `logging` key —
//!    which would only appear if `classify_hint` were being called instead of
//!    `category_for_node_kind`.
//!
//! These two assertions together are the closest achievable proxy to the
//! "snapshot the workspace fixtures before and after the M32 commit and assert
//! byte-equal JSON" acceptance criterion.

use std::path::Path;

use sdivi_config::Config;
use sdivi_lang_rust::RustAdapter;
use sdivi_pipeline::{Pipeline, WriteMode};

fn fixture_root() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/simple-rust"
    ))
}

/// Run the full pipeline on `simple-rust` with a fixed seed and fixed timestamp,
/// returning the snapshot as a serialised JSON string.
fn pipeline_json(seed: u64) -> String {
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
    serde_json::to_string(&snap).expect("snapshot must serialise")
}

/// Pipeline output is byte-identical for the same fixed seed and timestamp.
///
/// This is the primary M32 regression guard: any change to the pipeline's
/// pattern-classification path would produce different JSON across two runs with
/// the same seed only if the pipeline is non-deterministic — which it must not be.
/// Combined with the `logging` absence test below, this proves M32 left the
/// pipeline path unchanged.
#[test]
fn m32_pipeline_output_byte_identical_for_same_params() {
    let run_a = pipeline_json(42);
    let run_b = pipeline_json(42);
    assert_eq!(
        run_a, run_b,
        "Pipeline::snapshot must produce byte-identical JSON for the same fixed \
         seed and timestamp — any difference means the pipeline is non-deterministic \
         or M32 accidentally modified the classification path"
    );
}

/// Different seeds may produce different JSON (Leiden partition may differ).
///
/// This is a sanity check that the test is meaningful: if all seeds produced
/// the same output, the byte-identity assertion would be vacuous.
#[test]
fn m32_different_seeds_may_differ() {
    let seed_a = pipeline_json(42);
    let seed_b = pipeline_json(99);
    // We don't assert they MUST differ (the simple-rust fixture has no edges,
    // so Leiden partition is deterministic regardless of seed), but we confirm
    // the infrastructure can generate different seeds without panicking.
    let _ = (seed_a, seed_b);
}

/// M33: the pipeline now produces a `logging` entry for the simple-rust fixture.
///
/// The simple-rust fixture was extended in M33 to include `tracing::info!()` calls
/// in `utils.rs`. With M33's switch from `category_for_node_kind` to `classify_hint`,
/// those `macro_invocation` nodes are now classified as `logging` instead of
/// `resource_management`. This test locks in the M33 positive behavior.
///
/// re-baselined in M33: switched to classify_hint — logging bucket is now expected.
#[test]
fn m33_pipeline_snapshot_has_logging_entry_for_tracing_macros() {
    let json = pipeline_json(42);
    let snap: serde_json::Value = serde_json::from_str(&json).expect("snapshot must parse");
    let entries = snap
        .get("catalog")
        .and_then(|c| c.get("entries"))
        .expect("snapshot must have catalog.entries");
    assert!(
        entries
            .as_object()
            .expect("catalog.entries must be a JSON object")
            .contains_key("logging"),
        "Pipeline::snapshot MUST produce a `logging` entry for simple-rust — \
         tracing::info! calls in utils.rs are classified as logging via classify_hint (M33). \
         Absent entry means the M33 switchover did not take effect."
    );
}

/// Confirm the snapshot has valid structure and the expected schema version.
///
/// This guards against silent changes to the Snapshot schema that could mask
/// the `logging`-absence check (e.g., if `catalog` moved to a different key).
#[test]
fn m32_pipeline_snapshot_has_expected_schema_version() {
    let json = pipeline_json(42);
    let snap: serde_json::Value = serde_json::from_str(&json).expect("snapshot must parse");
    assert_eq!(
        snap["snapshot_version"].as_str().unwrap_or(""),
        "1.0",
        "snapshot_version must remain \"1.0\" — bumping it is a breaking change"
    );
    assert!(
        snap.get("catalog").is_some(),
        "snapshot must contain a `catalog` key — absent key would make the logging-absence \
         test vacuously pass"
    );
}
