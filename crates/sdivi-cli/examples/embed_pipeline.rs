//! Demonstrates embedding `sdivi-pipeline` for full FS-orchestrated analysis.
//!
//! Usage: `cargo run --example embed_pipeline [-- <repo-path>]`
//!
//! If no path is given, defaults to `tests/fixtures/simple-rust`.

use std::path::PathBuf;

use anyhow::Result;
use sdivi_config::Config;
use sdivi_lang_rust::RustAdapter;
use sdivi_pipeline::{current_timestamp, Pipeline};

fn main() -> Result<()> {
    let repo_root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tests/fixtures/simple-rust"));

    let config = Config::default();
    let pipeline = Pipeline::new(config, vec![Box::new(RustAdapter)]);

    let ts = current_timestamp();
    let snapshot = pipeline.snapshot(&repo_root, None, &ts)?;

    println!("=== embed_pipeline results ===");
    println!("nodes:       {}", snapshot.graph.node_count);
    println!("edges:       {}", snapshot.graph.edge_count);
    println!("communities: {}", snapshot.partition.community_count());
    println!("entropy:     {:.4}", snapshot.pattern_metrics.total_entropy);
    println!(
        "drift:       {:.4}",
        snapshot.pattern_metrics.convention_drift
    );

    Ok(())
}
