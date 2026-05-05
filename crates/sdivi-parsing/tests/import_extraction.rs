//! Multi-language import-specifier extraction regression test.
//!
//! Parses each `tests/fixtures/simple-<lang>/` fixture with the matching language
//! adapter, builds a dependency graph, and asserts the exact edge count.
//!
//! These tests are the regression sentinel for M25. Expected edge counts:
//!
//! | Fixture | Edges | Notes |
//! |---------|-------|-------|
//! | simple-rust | 0 | all imports are external std/serde |
//! | simple-python | 0 | bare names not resolved until M26 |
//! | simple-typescript | 3 | ./utils, ./models edges resolve |
//! | simple-javascript | 3 | ES6 + require() edges resolve |
//! | simple-go | 0 | Go module paths not resolved until M26 |
//! | simple-java | 0 | Java dotted paths not resolved until M26 |
//!
//! If grammar updates change adapter output and edges shift, update the pinned
//! counts deliberately — that is the intent of `assert_eq!` rather than `>=`.

use sdivi_config::Config;
use sdivi_graph::dependency_graph::build_dependency_graph;
use sdivi_lang_go::GoAdapter;
use sdivi_lang_java::JavaAdapter;
use sdivi_lang_javascript::JavaScriptAdapter;
use sdivi_lang_python::PythonAdapter;
use sdivi_lang_rust::RustAdapter;
use sdivi_lang_typescript::TypeScriptAdapter;
use sdivi_parsing::adapter::LanguageAdapter;
use sdivi_parsing::parse::parse_repository;
use std::path::PathBuf;

fn fixture(lang: &str) -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/"
    ))
    .join(format!("simple-{lang}"))
}

fn adapters_for(lang: &str) -> Vec<Box<dyn LanguageAdapter>> {
    match lang {
        "rust" => vec![Box::new(RustAdapter)],
        "python" => vec![Box::new(PythonAdapter)],
        "typescript" => vec![Box::new(TypeScriptAdapter)],
        "javascript" => vec![Box::new(JavaScriptAdapter)],
        "go" => vec![Box::new(GoAdapter)],
        "java" => vec![Box::new(JavaAdapter)],
        other => panic!("unknown lang: {other}"),
    }
}

fn edge_count(lang: &str) -> usize {
    let config = Config::default();
    let root = fixture(lang);
    let records = parse_repository(&config, &root, &adapters_for(lang));
    let dg = build_dependency_graph(&records);
    dg.edge_count()
}

// ── simple-rust ───────────────────────────────────────────────────────────────

#[test]
fn simple_rust_edge_count() {
    assert_eq!(
        edge_count("rust"),
        0,
        "simple-rust: all imports are external"
    );
}

// ── simple-python ─────────────────────────────────────────────────────────────

#[test]
fn simple_python_edge_count() {
    // bare module names ("os", "utils") not resolved until M26
    assert_eq!(
        edge_count("python"),
        0,
        "simple-python: bare names not resolved until M26"
    );
}

// ── simple-typescript ─────────────────────────────────────────────────────────

#[test]
fn simple_typescript_edge_count() {
    // app.ts→utils.ts, app.ts→models.ts, utils.ts→models.ts
    assert_eq!(
        edge_count("typescript"),
        3,
        "simple-typescript: 3 ./relative edges must resolve"
    );
}

// ── simple-javascript ─────────────────────────────────────────────────────────

#[test]
fn simple_javascript_edge_count() {
    // index.js→utils.js (import), index.js→helpers.js (require), utils.js→helpers.js (import)
    assert_eq!(
        edge_count("javascript"),
        3,
        "simple-javascript: 3 ./relative edges must resolve"
    );
}

// ── simple-go ─────────────────────────────────────────────────────────────────

#[test]
fn simple_go_edge_count() {
    // Go module paths ("fmt", "os") not resolved until M26
    assert_eq!(
        edge_count("go"),
        0,
        "simple-go: module paths not resolved until M26"
    );
}

// ── simple-java ───────────────────────────────────────────────────────────────

#[test]
fn simple_java_edge_count() {
    // Java dotted names not resolved until M26
    assert_eq!(
        edge_count("java"),
        0,
        "simple-java: dotted paths not resolved until M26"
    );
}

// ── simple-python-relative ───────────────────────────────────────────────────

/// End-to-end regression sentinel for Python relative-import specifiers.
///
/// `pkg/__init__.py` contains `from . import models` (specifier `"."`) and
/// `from .models import User` (specifier `".models"`).  Neither specifier
/// starts with `"./"` or `"../"`, so the current resolver (M25) drops them;
/// edges will be 0 until M26 adds dot-relative path navigation.
///
/// When M26 lands, update this count to 2 (one edge per resolved specifier).
#[test]
fn simple_python_relative_import_edge_count() {
    let config = Config::default();
    let root = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/simple-python-relative"
    ));
    let adapters: Vec<Box<dyn LanguageAdapter>> = vec![Box::new(PythonAdapter)];
    let records = parse_repository(&config, &root, &adapters);

    // Verify the adapter extracted relative specifiers from __init__.py.
    let init_record = records
        .iter()
        .find(|r| r.path.ends_with("__init__.py"))
        .expect("__init__.py must be present in fixture");
    assert!(
        init_record.imports.contains(&".".to_string())
            || init_record.imports.contains(&".models".to_string()),
        "relative specifiers '.' or '.models' must appear in __init__.py imports, got: {:?}",
        init_record.imports
    );

    // Graph resolver drops dot-relative specifiers until M26.
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        0,
        "Python dot-relative imports (\".\", \".models\") not resolved until M26"
    );
}
