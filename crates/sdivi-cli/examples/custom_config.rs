//! Demonstrates programmatic `Config` construction for embedding `sdivi-pipeline`.
//!
//! Shows how to override defaults without a config file — useful in CI pipelines
//! and test harnesses that need reproducible builds.
//!
//! Usage: `cargo run --example custom_config [-- <repo-path>]`

use std::path::PathBuf;

use anyhow::Result;
use sdivi_config::{Config, ThresholdsConfig};
use sdivi_lang_rust::RustAdapter;
use sdivi_pipeline::{current_timestamp, Pipeline, WriteMode};

fn main() -> Result<()> {
    let repo_root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tests/fixtures/simple-rust"));

    // Build a custom Config programmatically.
    let mut config = Config::default();

    // Use a fixed random seed for reproducible Leiden partitioning.
    config.core.random_seed = 99;

    // Tighten the pattern entropy threshold.
    config.thresholds = ThresholdsConfig {
        pattern_entropy_rate: 1.5,
        convention_drift_rate: 2.0,
        coupling_delta_rate: 0.10,
        boundary_violation_rate: 1.0,
        overrides: std::collections::BTreeMap::new(),
    };

    // Keep only 10 snapshots on disk.
    config.snapshots.retention = 10;

    println!(
        "Config: seed={} retention={}",
        config.core.random_seed, config.snapshots.retention
    );

    let pipeline = Pipeline::new(config, vec![Box::new(RustAdapter)]);
    let ts = current_timestamp();
    let snapshot =
        pipeline.snapshot_with_mode(&repo_root, None, &ts, WriteMode::EphemeralForCheck)?;

    println!("=== custom_config results ===");
    println!("seed:        {}", snapshot.partition.seed);
    println!("nodes:       {}", snapshot.graph.node_count);
    println!("communities: {}", snapshot.partition.community_count());
    println!("entropy:     {:.4}", snapshot.pattern_metrics.total_entropy);

    Ok(())
}
