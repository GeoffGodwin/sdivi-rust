// Text/table output formatter.

use sdi_core::{DivergenceSummary, Snapshot};
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

/// Prints a human-readable summary of `snapshot` to stdout.
///
/// Shows key structural metrics: schema version, timestamp, node/edge counts,
/// community count, and the number of pattern categories in the catalog.
pub fn print_snapshot(snapshot: &Snapshot) {
    println!("snapshot_version:  {}", snapshot.snapshot_version);
    println!("timestamp:         {}", snapshot.timestamp);
    if let Some(commit) = &snapshot.commit {
        println!("commit:            {commit}");
    }
    println!("nodes:             {}", snapshot.graph.node_count);
    println!("edges:             {}", snapshot.graph.edge_count);
    println!("density:           {:.6}", snapshot.graph.density);
    println!("communities:       {}", snapshot.partition.community_count());
    println!("modularity:        {:.6}", snapshot.partition.modularity);
    println!("pattern_categories:{}", snapshot.catalog.entries.len());
    if let Some(id) = &snapshot.intent_divergence {
        println!("boundaries:        {}", id.boundary_count);
        println!("violations:        {}", id.violation_count);
    }
}

/// Prints the per-dimension divergence summary as a labeled table to stdout.
///
/// Fields that could not be computed (no prior snapshot) are shown as `null`.
/// `0` or `0.00` means "compared and no change observed" (CLAUDE.md Rule 14).
pub fn print_divergence(summary: &DivergenceSummary) {
    fn fmt_opt_f64(v: Option<f64>) -> String {
        match v {
            Some(x) => format!("{x:.6}"),
            None => "null".to_string(),
        }
    }
    fn fmt_opt_i64(v: Option<i64>) -> String {
        match v {
            Some(x) => x.to_string(),
            None => "null".to_string(),
        }
    }

    println!(
        "pattern_entropy_delta:   {}",
        fmt_opt_f64(summary.pattern_entropy_delta)
    );
    println!(
        "coupling_delta:          {}",
        fmt_opt_f64(summary.coupling_delta)
    );
    println!(
        "community_count_delta:   {}",
        fmt_opt_i64(summary.community_count_delta)
    );
    println!(
        "boundary_violation_delta:{}",
        fmt_opt_i64(summary.boundary_violation_delta)
    );
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
