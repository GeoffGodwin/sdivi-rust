//! `sdivi diff` — compare two snapshots and display the divergence summary.

use std::path::Path;

use anyhow::{Context, Result};
use sdivi_core::{compute_delta, Snapshot, SNAPSHOT_VERSION};

use crate::output;

/// Runs `sdivi diff` by loading two snapshot files and printing the divergence summary.
///
/// Loads `prev_path` and `curr_path` as [`Snapshot`] JSON files, computes the
/// per-dimension delta via [`sdivi_core::compute_delta`], and writes the result to
/// stdout in `format` (either `"json"` or `"text"`).  Logs and progress go to
/// stderr per CLAUDE.md Rule 8.
///
/// If either snapshot has an incompatible `snapshot_version`, a warning is
/// printed to stderr but processing continues (Rule 17).
///
/// # Errors
///
/// Returns an error if either snapshot file cannot be read or deserialized.
pub fn run(prev_path: &Path, curr_path: &Path, format: &str) -> Result<()> {
    let prev: Snapshot = load_snapshot(prev_path)
        .with_context(|| format!("failed to load previous snapshot: {}", prev_path.display()))?;
    let curr: Snapshot = load_snapshot(curr_path)
        .with_context(|| format!("failed to load current snapshot: {}", curr_path.display()))?;

    // Warn on incompatible snapshot versions but continue (Rule 17).
    if prev.snapshot_version != SNAPSHOT_VERSION {
        eprintln!(
            "sdivi: warning: previous snapshot version {:?} != {:?} — treating as baseline",
            prev.snapshot_version, SNAPSHOT_VERSION,
        );
    }
    if curr.snapshot_version != SNAPSHOT_VERSION {
        eprintln!(
            "sdivi: warning: current snapshot version {:?} != {:?} — treating as baseline",
            curr.snapshot_version, SNAPSHOT_VERSION,
        );
    }

    let delta = compute_delta(&prev, &curr);

    match format {
        "json" => output::json::print_divergence(&delta)?,
        _ => output::text::print_divergence(&delta),
    }

    Ok(())
}

fn load_snapshot(path: &Path) -> Result<Snapshot> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("cannot parse snapshot JSON from {}", path.display()))
}
