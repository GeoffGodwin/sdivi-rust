// JSON output formatter.

use anyhow::Result;
use sdi_core::{DivergenceSummary, Snapshot};
use sdi_patterns::PatternCatalog;

/// Prints `catalog` as pretty-printed JSON to stdout.
///
/// Logs and progress messages go to stderr (CLAUDE.md Rule 8).
pub fn print_catalog(catalog: &PatternCatalog) -> Result<()> {
    let json = serde_json::to_string_pretty(catalog)?;
    println!("{json}");
    Ok(())
}

/// Prints `snapshot` as pretty-printed JSON to stdout.
///
/// Logs and progress messages go to stderr (CLAUDE.md Rule 8).
pub fn print_snapshot(snapshot: &Snapshot) -> Result<()> {
    let json = serde_json::to_string_pretty(snapshot)?;
    println!("{json}");
    Ok(())
}

/// Prints `summary` as pretty-printed JSON to stdout.
///
/// All four delta fields are serialized as explicit `null` when absent so
/// that CI consumers can distinguish "not computed" from "zero change"
/// (CLAUDE.md Rule 14).
pub fn print_divergence(summary: &DivergenceSummary) -> Result<()> {
    let json = serde_json::to_string_pretty(summary)?;
    println!("{json}");
    Ok(())
}
