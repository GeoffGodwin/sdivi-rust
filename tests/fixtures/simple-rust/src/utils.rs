// simple-rust fixture: utils.rs
// Imports: 2 | Exports: 2
// Extended in M33: added tracing::info! so logging bucket is natively populated.
use std::collections::HashSet;
use std::path::Path;

/// Returns `true` if `path` has the given file extension.
pub fn has_extension(path: &Path, ext: &str) -> bool {
    tracing::info!("checking extension for {:?}", path);
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case(ext))
        .unwrap_or(false)
}

/// Deduplicates a `Vec<String>` while preserving first-occurrence order.
pub fn dedup_preserve_order(items: Vec<String>) -> Vec<String> {
    tracing::debug!("deduplicating {} items", items.len());
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.clone()))
        .collect()
}
