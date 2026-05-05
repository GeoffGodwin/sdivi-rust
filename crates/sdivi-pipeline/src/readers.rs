//! Filesystem reader helpers for Stage 2 (graph) configuration.
//!
//! Reads `go.mod` and `tsconfig.json` / `jsconfig.json` from the repository
//! root and returns parsed values for the dependency-graph builder.  All FS
//! I/O is isolated here per Rule 22.

use std::path::Path;

use sdivi_graph::{parse_tsconfig_content, TsConfigPaths};

/// Reads `go.mod` from `root` and returns the module path, or `None` if
/// absent or unparseable (Go imports treated as external on failure).
pub(crate) fn read_go_mod_prefix(root: &Path) -> Option<String> {
    let content = std::fs::read_to_string(root.join("go.mod")).ok()?;
    for line in content.lines() {
        if let Some(rest) = line.trim().strip_prefix("module") {
            let trimmed = rest.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('(') {
                return trimmed.split_whitespace().next().map(str::to_string);
            }
        }
    }
    None
}

/// Reads `tsconfig.json` (or `jsconfig.json`) from `root` and parses path aliases.
///
/// Returns `None` when neither file exists or when the JSON is unparseable
/// (a `WARN` is logged in the latter case).  Alias resolution is then disabled
/// for this snapshot run — other resolution paths are unaffected (Rule 15).
pub(crate) fn read_tsconfig_paths(root: &Path) -> Option<TsConfigPaths> {
    let tsconfig_path = if root.join("tsconfig.json").exists() {
        root.join("tsconfig.json")
    } else if root.join("jsconfig.json").exists() {
        root.join("jsconfig.json")
    } else {
        return None;
    };
    let content = std::fs::read_to_string(&tsconfig_path).ok()?;
    parse_tsconfig_content(&content, Path::new(""))
}
