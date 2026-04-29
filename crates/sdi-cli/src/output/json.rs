// JSON output formatter.

use anyhow::Result;
use sdi_patterns::PatternCatalog;

/// Prints `catalog` as pretty-printed JSON to stdout.
///
/// Logs and progress messages go to stderr (CLAUDE.md Rule 8).
pub fn print_catalog(catalog: &PatternCatalog) -> Result<()> {
    let json = serde_json::to_string_pretty(catalog)?;
    println!("{json}");
    Ok(())
}
