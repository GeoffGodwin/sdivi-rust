//! [`parse_repository`] — parallel parsing entry point for the parsing stage.

use std::path::Path;

use rayon::prelude::*;
use sdi_config::Config;

use crate::adapter::LanguageAdapter;
use crate::feature_record::FeatureRecord;
use crate::walker::collect_files;

/// Parses all source files in `repo_root` and returns a sorted `Vec<FeatureRecord>`.
///
/// The file list is stable-sorted **before** parallelization so that rayon's
/// internal scheduling cannot introduce ordering nondeterminism. The returned
/// `Vec` is additionally sorted by path after parallel parsing to guarantee
/// deterministic output.
///
/// Files whose extension is not handled by any adapter are silently skipped.
/// Files that fail to read from disk are silently skipped (a warning should be
/// added to the tracing layer in a later milestone).
///
/// # Examples
///
/// ```no_run
/// use sdi_config::Config;
/// use sdi_parsing::parse::parse_repository;
/// use std::path::Path;
///
/// let config = Config::default();
/// let records = parse_repository(&config, Path::new("."), &[]);
/// assert!(records.is_empty()); // no adapters registered
/// ```
pub fn parse_repository(
    config: &Config,
    repo_root: &Path,
    adapters: &[Box<dyn LanguageAdapter>],
) -> Vec<FeatureRecord> {
    if adapters.is_empty() {
        return Vec::new();
    }

    let files = collect_files(config, repo_root);

    let mut records: Vec<FeatureRecord> = files
        .into_par_iter()
        .filter_map(|path| {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{e}"));
            let Some(ext) = ext else { return None };
            let adapter = adapters
                .iter()
                .find(|a| a.file_extensions().contains(&ext.as_str()))?;
            let content = std::fs::read_to_string(&path).ok()?;
            let relative = path
                .strip_prefix(repo_root)
                .unwrap_or(&path)
                .to_path_buf();
            let mut record = adapter.parse_file(&relative, content);
            record.path = relative;
            Some(record)
        })
        .collect();

    records.sort_by(|a, b| a.path.cmp(&b.path));
    records
}
