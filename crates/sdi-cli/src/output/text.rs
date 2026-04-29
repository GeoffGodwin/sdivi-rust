// Text/table output formatter.

use sdi_patterns::{PatternCatalog, PatternStats};

/// Prints `catalog` as human-readable text to stdout.
///
/// Logs and progress messages go to stderr (CLAUDE.md Rule 8).
pub fn print_catalog(catalog: &PatternCatalog) {
    if catalog.entries.is_empty() {
        println!("(no patterns found)");
        return;
    }
    for (category, fingerprints) in &catalog.entries {
        println!("=== {category} ===");
        for (fp, stats) in fingerprints {
            print_stats_line(&fp.to_hex(), stats);
        }
    }
}

fn print_stats_line(hex: &str, stats: &PatternStats) {
    let short = &hex[..12];
    let locs: Vec<String> = stats
        .locations
        .iter()
        .take(3)
        .map(|l| format!("{}:{}:{}", l.file.display(), l.start_row, l.start_col))
        .collect();
    let loc_str = if locs.is_empty() {
        String::new()
    } else {
        format!(" | {}", locs.join(", "))
    };
    println!("  {short}… count:{}{}", stats.count, loc_str);
}
