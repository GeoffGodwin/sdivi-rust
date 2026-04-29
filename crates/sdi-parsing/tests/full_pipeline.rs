//! End-to-end pipeline test: parse the simple-rust fixture via parse_repository.
//!
//! NOTE: The milestone spec places this in `tests/full_pipeline.rs` (workspace
//! root), but the root Cargo.toml has no `[package]` section, so Cargo cannot
//! compile tests there. This file lives in `crates/sdi-parsing/tests/` which
//! has the required dev-dependencies via sdi-parsing's Cargo.toml.

use sdi_config::Config;
use sdi_lang_rust::RustAdapter;
use sdi_parsing::parse::parse_repository;
use std::path::Path;

fn fixture_root() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/simple-rust"
    ))
}

#[test]
fn parse_fixture_returns_five_records() {
    let config = Config::default();
    let adapters: Vec<Box<dyn sdi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];

    let records = parse_repository(&config, fixture_root(), &adapters);

    // The simple-rust fixture has exactly 5 .rs files.
    assert_eq!(
        records.len(),
        5,
        "simple-rust fixture must parse to exactly 5 FeatureRecords, got: {:?}",
        records.iter().map(|r| &r.path).collect::<Vec<_>>()
    );
}

#[test]
fn records_are_sorted_by_path() {
    let config = Config::default();
    let adapters: Vec<Box<dyn sdi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];

    let records = parse_repository(&config, fixture_root(), &adapters);
    let paths: Vec<_> = records.iter().map(|r| &r.path).collect();
    let mut sorted = paths.clone();
    sorted.sort();

    assert_eq!(paths, sorted, "records must be sorted by path");
}

#[test]
fn parse_same_fixture_twice_identical() {
    let config = Config::default();
    let make_adapters = || -> Vec<Box<dyn sdi_parsing::adapter::LanguageAdapter>> {
        vec![Box::new(RustAdapter)]
    };

    let first = parse_repository(&config, fixture_root(), &make_adapters());
    let second = parse_repository(&config, fixture_root(), &make_adapters());

    let first_paths: Vec<_> = first.iter().map(|r| &r.path).collect();
    let second_paths: Vec<_> = second.iter().map(|r| &r.path).collect();
    assert_eq!(first_paths, second_paths);

    for (a, b) in first.iter().zip(second.iter()) {
        assert_eq!(a.imports, b.imports, "imports must be deterministic for {:?}", a.path);
        assert_eq!(a.exports, b.exports, "exports must be deterministic for {:?}", a.path);
        assert_eq!(a.signatures, b.signatures, "sigs must be deterministic for {:?}", a.path);
    }
}

#[test]
fn parse_empty_dir_returns_no_records() {
    let dir = tempfile::tempdir().unwrap();
    let config = Config::default();
    let adapters: Vec<Box<dyn sdi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];

    let records = parse_repository(&config, dir.path(), &adapters);
    assert!(records.is_empty(), "empty directory must produce zero records");
}

#[test]
fn lib_rs_has_two_imports() {
    let config = Config::default();
    let adapters: Vec<Box<dyn sdi_parsing::adapter::LanguageAdapter>> =
        vec![Box::new(RustAdapter)];

    let records = parse_repository(&config, fixture_root(), &adapters);
    let lib = records
        .iter()
        .find(|r| r.path.ends_with("lib.rs"))
        .expect("lib.rs must be in records");

    // lib.rs has: use std::collections::BTreeMap; use std::fmt;
    assert_eq!(lib.imports.len(), 2, "lib.rs must have exactly 2 imports");
}
