//! `sdi snapshot` — run the full analysis pipeline and write a snapshot.

use std::path::Path;

use anyhow::Result;
use sdi_config::Config;
use sdi_pipeline::{Pipeline, current_timestamp};

use crate::output;

/// Runs `sdi snapshot` against `repo_root` using the given configuration.
///
/// Executes all five pipeline stages (parsing, graph, detection, patterns,
/// snapshot assembly), writes the snapshot atomically to `.sdi/snapshots/`,
/// and prints a summary to stdout.  Logs and progress go to stderr per
/// CLAUDE.md Rule 8.
///
/// # Errors
///
/// Returns an error if the pipeline fails or if output serialization fails.
pub fn run(repo_root: &Path, config: &Config, commit: Option<&str>, format: &str) -> Result<()> {
    let adapters = super::all_adapters();
    let pipeline = Pipeline::new(config.clone(), adapters);

    let timestamp = current_timestamp();

    eprintln!("sdi: snapshotting repository at {}", repo_root.display());

    let snapshot = pipeline.snapshot(repo_root, commit, &timestamp)?;

    eprintln!(
        "sdi: snapshot complete — nodes={} edges={} communities={}",
        snapshot.graph.node_count,
        snapshot.graph.edge_count,
        snapshot.partition.community_count(),
    );

    match format {
        "json" => output::json::print_snapshot(&snapshot)?,
        _ => output::text::print_snapshot(&snapshot),
    }

    Ok(())
}
