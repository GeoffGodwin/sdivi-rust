//! `sdi snapshot` — run the full analysis pipeline and write a snapshot.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use sdi_config::Config;
use sdi_core::Pipeline;
use sdi_parsing::adapter::LanguageAdapter;

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
    let adapters = all_adapters();
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

/// Returns one instance of every built-in language adapter.
fn all_adapters() -> Vec<Box<dyn LanguageAdapter>> {
    vec![
        Box::new(sdi_lang_rust::RustAdapter),
        Box::new(sdi_lang_python::PythonAdapter),
        Box::new(sdi_lang_typescript::TypeScriptAdapter),
        Box::new(sdi_lang_javascript::JavaScriptAdapter),
        Box::new(sdi_lang_go::GoAdapter),
        Box::new(sdi_lang_java::JavaAdapter),
    ]
}

/// Returns the current UTC time as an ISO 8601 string (`"YYYY-MM-DDTHH:MM:SSZ"`).
fn current_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    unix_to_iso8601(secs)
}

/// Converts a Unix timestamp (seconds since epoch) to an ISO 8601 UTC string.
///
/// Uses the Richards algorithm via Julian Day Number to compute the Gregorian
/// calendar date without external date/time crates.
fn unix_to_iso8601(secs: u64) -> String {
    let secs = secs as i64;
    let days_since_epoch = secs / 86400;
    let time_secs = secs % 86400;

    // Julian Day Number: 2440588 = JDN for 1970-01-01
    let j = days_since_epoch + 2440588;
    let f = j + 1401 + (((4 * j + 274277) / 146097) * 3) / 4 - 38;
    let e = 4 * f + 3;
    let g = (e % 1461) / 4;
    let h = 5 * g + 2;

    let day = (h % 153) / 5 + 1;
    let month = (h / 153 + 2) % 12 + 1;
    let year = e / 1461 - 4716 + (14 - month) / 12;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year,
        month,
        day,
        time_secs / 3600,
        (time_secs % 3600) / 60,
        time_secs % 60,
    )
}

#[cfg(test)]
mod tests {
    use super::unix_to_iso8601;

    #[test]
    fn epoch_is_1970_01_01() {
        assert_eq!(unix_to_iso8601(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn known_timestamp() {
        // 2026-04-29T00:00:00Z = 1777420800 seconds since epoch
        assert_eq!(unix_to_iso8601(1777420800), "2026-04-29T00:00:00Z");
    }

    #[test]
    fn time_components_correct() {
        // 2026-04-29T12:34:56Z = 1777420800 + 12*3600 + 34*60 + 56 = 1777466096
        assert_eq!(unix_to_iso8601(1777466096), "2026-04-29T12:34:56Z");
    }
}
