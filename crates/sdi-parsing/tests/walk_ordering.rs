//! Verifies that `collect_files` produces a stable-sorted, deterministic path list.

use sdi_config::Config;
use sdi_parsing::walker::collect_files;
use std::path::Path;

fn fixture_path() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/simple-rust"
    ))
}

#[test]
fn walk_twice_identical_order() {
    let config = Config::default();
    let root = fixture_path();

    let first = collect_files(&config, root);
    let second = collect_files(&config, root);

    assert_eq!(
        first, second,
        "collect_files must return the same order on repeated calls"
    );
}

#[test]
fn walk_empty_dir_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let config = Config::default();
    let files = collect_files(&config, dir.path());
    assert!(files.is_empty(), "empty directory should produce no files");
}

#[test]
fn walk_result_is_sorted() {
    let config = Config::default();
    let root = fixture_path();
    let files = collect_files(&config, root);

    let mut sorted = files.clone();
    sorted.sort();

    assert_eq!(files, sorted, "collect_files must return a sorted list");
}

#[test]
fn walk_returns_five_rs_files() {
    let config = Config::default();
    let root = fixture_path();
    let files = collect_files(&config, root);

    let rs_files: Vec<_> = files
        .iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
        .collect();

    // lib.rs, models.rs, utils.rs, errors.rs, config.rs
    assert_eq!(rs_files.len(), 5, "fixture must have exactly 5 .rs files");
}

#[test]
fn walk_exclude_glob_suppresses_files() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("main.rs"), b"fn main() {}").unwrap();
    std::fs::write(src.join("skip.rs"), b"fn skip() {}").unwrap();

    let mut config = Config::default();
    config.core.exclude = vec!["**/skip.rs".to_string()];

    let files = collect_files(&config, dir.path());
    assert!(!files.iter().any(|p| p.ends_with("skip.rs")), "skip.rs must be excluded");
    assert!(files.iter().any(|p| p.ends_with("main.rs")), "main.rs must be included");
}
