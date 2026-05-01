//! `sdi show` — inspect a stored snapshot.

use std::path::Path;

use anyhow::{Context, Result};
use sdi_config::Config;
use sdi_pipeline::store::{latest_snapshot, read_snapshot_by_id};

use crate::output;

/// Runs `sdi show [<id>] [--format json|text]`.
///
/// With no `id`, displays the most recent snapshot (lexicographically last
/// `snapshot_*.json` in the configured snapshot directory).  With an `id`,
/// loads that specific snapshot by filename stem (without `.json` extension).
///
/// `--format json` writes the raw [`sdi_core::Snapshot`] JSON to stdout, which
/// can be piped to `jq` without stderr contamination.
///
/// # Errors
///
/// Returns an error if the snapshot cannot be read or no snapshots exist.
pub fn run(
    repo_root: &Path,
    config: &Config,
    id: Option<&str>,
    format: &str,
) -> Result<()> {
    let snapshot_dir = repo_root.join(&config.snapshots.dir);

    let snapshot = match id {
        Some(id) => read_snapshot_by_id(&snapshot_dir, id)
            .with_context(|| format!("snapshot '{}' not found in {}", id, snapshot_dir.display()))?,
        None => latest_snapshot(&snapshot_dir)
            .with_context(|| {
                format!("failed to read snapshot dir: {}", snapshot_dir.display())
            })?
            .ok_or_else(|| {
                anyhow::anyhow!("no snapshots found in {}", snapshot_dir.display())
            })?,
    };

    match format {
        "json" => output::json::print_snapshot(&snapshot)?,
        _ => output::text::print_snapshot(&snapshot),
    }

    Ok(())
}
