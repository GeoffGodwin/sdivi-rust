//! `sdi trend` — aggregate trend statistics across stored snapshots.

use std::path::Path;

use anyhow::{Context, Result};
use sdi_config::Config;
use sdi_core::compute_trend;
use sdi_pipeline::store::read_snapshots;

use crate::output;

/// Runs `sdi trend [--last N] [--format json|text]`.
///
/// Reads stored snapshots from the configured snapshot directory and computes
/// trend statistics via [`sdi_core::compute_trend`].  With fewer than 2
/// snapshots, prints a friendly message to stderr and exits 0.
///
/// `last_n = None` uses all available snapshots; `Some(n)` takes the `n` most
/// recent (silently clamped to the available count, no error).
///
/// # Errors
///
/// Returns an error if the snapshot directory cannot be read.
pub fn run(
    repo_root: &Path,
    config: &Config,
    last_n: Option<usize>,
    format: &str,
) -> Result<()> {
    let snapshot_dir = repo_root.join(&config.snapshots.dir);
    let snapshots = read_snapshots(&snapshot_dir)
        .with_context(|| format!("failed to read snapshot dir: {}", snapshot_dir.display()))?;

    if snapshots.len() < 2 {
        eprintln!("sdi trend: not enough snapshots (need \u{2265}2)");
        return Ok(());
    }

    let result = compute_trend(&snapshots, last_n);

    match format {
        "json" => output::json::print_trend(&result)?,
        _ => output::text::print_trend(&result),
    }

    Ok(())
}
