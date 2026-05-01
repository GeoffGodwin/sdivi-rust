// Text/table output formatter.

use sdi_core::{DivergenceSummary, Snapshot, ThresholdCheckResult, TrendResult};
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

/// Prints `sdi check` result as human-readable text to stdout.
pub fn print_check(result: &ThresholdCheckResult, summary: &DivergenceSummary) {
    if result.breached {
        println!("check: FAILED — {} threshold(s) exceeded", result.breaches.len());
        for b in &result.breaches {
            println!(
                "  {}: {:.6} > {:.6} (limit)",
                b.dimension, b.actual, b.limit
            );
        }
    } else {
        println!("check: OK — all thresholds within limits");
    }
    println!();
    print_divergence(summary);
}

/// Prints `sdi trend` result as human-readable text to stdout.
pub fn print_trend(result: &TrendResult) {
    fn fmt_slope(v: Option<f64>) -> String {
        match v {
            Some(x) => format!("{x:+.6}"),
            None => "null".to_string(),
        }
    }
    println!("snapshots in window: {}", result.snapshot_count);
    println!(
        "pattern_entropy_slope:   {}",
        fmt_slope(result.pattern_entropy_slope)
    );
    println!(
        "convention_drift_slope:  {}",
        fmt_slope(result.convention_drift_slope)
    );
    println!(
        "coupling_slope:          {}",
        fmt_slope(result.coupling_slope)
    );
    println!(
        "community_count_slope:   {}",
        fmt_slope(result.community_count_slope)
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
