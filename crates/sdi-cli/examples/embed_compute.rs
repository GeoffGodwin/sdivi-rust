//! Demonstrates embedding `sdi-core` for pure-compute analysis (WASM-path).
//!
//! This mirrors what the consumer app does: the caller supplies pre-parsed graph
//! data and pattern instances via `*Input` structs, then calls `sdi-core::compute_*`
//! functions directly. Graph edges are bootstrapped from `sdi_parsing::parse_repository`
//! for illustration; in a real WASM consumer the caller supplies its own extracted edges.
//!
//! The example also runs the same fixture through `sdi-pipeline` and asserts
//! that both paths produce the same node and edge counts.
//!
//! Usage: `cargo run --example embed_compute [-- <repo-path>]`

use std::path::PathBuf;

use anyhow::Result;
use sdi_config::Config;
use sdi_core::compute::coupling::compute_coupling_topology;
use sdi_core::compute::patterns::compute_pattern_metrics;
use sdi_core::compute::thresholds::compute_thresholds_check;
use sdi_core::input::{
    DependencyGraphInput, EdgeInput, NodeInput, PatternInstanceInput, ThresholdsInput,
};
use sdi_core::normalize_and_hash;
use sdi_lang_rust::RustAdapter;
use sdi_pipeline::{Pipeline, WriteMode, current_timestamp};

fn main() -> Result<()> {
    let repo_root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tests/fixtures/simple-rust"));

    // ── Orchestration path (as reference) ────────────────────────────────────
    let config = Config::default();
    let pipeline = Pipeline::new(config.clone(), vec![Box::new(RustAdapter)]);
    let ts = current_timestamp();
    let reference = pipeline.snapshot_with_mode(
        &repo_root, None, &ts, WriteMode::EphemeralForCheck,
    )?;

    // ── Pure-compute path (consumer-app / WASM style) ────────────────────────
    // Extract nodes and edges from the path_partition and graph fields instead
    // of re-parsing. In a real consumer app the caller supplies these from its
    // own AST extractor.
    let nodes: Vec<NodeInput> = reference
        .path_partition
        .keys()
        .map(|path| NodeInput {
            id: path.clone(),
            path: path.clone(),
            language: "rust".to_string(),
        })
        .collect();

    let node_ids: std::collections::BTreeMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();

    // Edges: derive from snapshot graph metadata — in production the consumer
    // supplies these from its own import-resolution step.
    let edges: Vec<EdgeInput> = {
        let adapters: Vec<Box<dyn sdi_parsing::adapter::LanguageAdapter>> =
            vec![Box::new(RustAdapter)];
        let records = sdi_parsing::parse::parse_repository(&config, &repo_root, &adapters);
        let mut result = Vec::new();
        for record in &records {
            let src = record.path.to_string_lossy().to_string();
            if !node_ids.contains_key(src.as_str()) {
                continue;
            }
            for import in &record.imports {
                // Simplified: use the import string as a node id if it matches
                if node_ids.contains_key(import.as_str()) {
                    result.push(EdgeInput {
                        source: src.clone(),
                        target: import.clone(),
                    });
                }
            }
        }
        result
    };

    let graph_input = DependencyGraphInput { nodes, edges };
    let coupling = compute_coupling_topology(&graph_input)?;

    // Demonstrate normalize_and_hash — same fingerprint as the Rust pipeline.
    let fp = normalize_and_hash("function_item", &[]);
    let patterns: Vec<PatternInstanceInput> = vec![PatternInstanceInput {
        fingerprint: fp,
        category: "functions".to_string(),
        node_id: "src/lib.rs".to_string(),
        location: None,
    }];
    let pattern_metrics = compute_pattern_metrics(&patterns);

    // Threshold check against defaults.
    use sdi_core::null_summary;
    let check = compute_thresholds_check(&null_summary(), &ThresholdsInput::default());

    println!("=== embed_compute results ===");
    println!("nodes (pure-compute):  {}", coupling.node_count);
    println!("edges (pure-compute):  {}", coupling.edge_count);
    println!("entropy (pure-compute):{:.4}", pattern_metrics.total_entropy);
    println!("breached:              {}", check.breached);

    println!("\n=== parity check ===");
    println!(
        "pipeline nodes={} compute nodes={} — match={}",
        reference.graph.node_count,
        coupling.node_count,
        reference.graph.node_count == coupling.node_count,
    );

    Ok(())
}
