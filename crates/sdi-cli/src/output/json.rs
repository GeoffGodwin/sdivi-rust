// JSON output formatter.

use anyhow::Result;
use sdi_core::{DivergenceSummary, Snapshot, ThresholdCheckResult, TrendResult};
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

/// Prints `sdi check` result as pretty-printed JSON to stdout.
///
/// JSON shape: `{ exit_code, exceeded: [string], summary: DivergenceSummary,
/// applied_overrides: {} }` — stable contract for CI consumers (Seeds Forward).
pub fn print_check(result: &ThresholdCheckResult, summary: &DivergenceSummary) -> Result<()> {
    let exit_code: i32 = if result.breached { 10 } else { 0 };
    let exceeded: Vec<&str> = result
        .breaches
        .iter()
        .map(|b| b.dimension.as_str())
        .collect();
    let output = serde_json::json!({
        "exit_code": exit_code,
        "exceeded": exceeded,
        "summary": summary,
        "applied_overrides": {},
    });
    let json = serde_json::to_string_pretty(&output)?;
    println!("{json}");
    Ok(())
}

/// Prints `sdi trend` result as pretty-printed JSON to stdout.
pub fn print_trend(result: &TrendResult) -> Result<()> {
    let json = serde_json::to_string_pretty(result)?;
    println!("{json}");
    Ok(())
}
