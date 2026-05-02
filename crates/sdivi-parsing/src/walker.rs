//! File discovery: breadth-first stable-sorted walk with `.gitignore` and glob exclusion.

use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use sdivi_config::Config;

/// Collects all source files under `repo_root`, applying `.gitignore` rules and
/// `config.core.exclude` glob patterns.
///
/// The returned list is **stable-sorted** by path. Sorting happens before any
/// parallel processing so that downstream stages produce deterministic output
/// regardless of rayon's internal scheduling.
///
/// # Examples
///
/// ```no_run
/// use sdivi_config::Config;
/// use sdivi_parsing::walker::collect_files;
/// use std::path::Path;
///
/// let config = Config::default();
/// let files = collect_files(&config, Path::new("."));
/// // Returns sorted list of all non-excluded source files
/// ```
pub fn collect_files(config: &Config, repo_root: &Path) -> Vec<PathBuf> {
    let exclude = build_exclude_set(&config.core.exclude);

    let mut files: Vec<PathBuf> = WalkBuilder::new(repo_root)
        .hidden(false)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(false)
        .build()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
        .map(|entry| entry.into_path())
        .filter(|path| !is_excluded(path, repo_root, exclude.as_ref()))
        .collect();

    files.sort();
    files
}

fn is_excluded(path: &Path, repo_root: &Path, exclude: Option<&GlobSet>) -> bool {
    let Some(gs) = exclude else { return false };
    let relative = path.strip_prefix(repo_root).unwrap_or(path);
    gs.is_match(relative)
}

fn build_exclude_set(patterns: &[String]) -> Option<GlobSet> {
    if patterns.is_empty() {
        return None;
    }
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
        }
    }
    builder.build().ok()
}
