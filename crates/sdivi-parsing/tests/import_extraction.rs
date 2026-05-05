//! Multi-language import-specifier extraction regression test.
//!
//! Parses each `tests/fixtures/simple-<lang>/` fixture with the matching language
//! adapter, builds a dependency graph, and asserts the exact edge count.
//!
//! These tests are the regression sentinel for M25+M26. Expected edge counts:
//!
//! | Fixture | Edges | Notes |
//! |---------|-------|-------|
//! | simple-rust | 0 | all imports are external std/serde |
//! | simple-python | 2 | main.py → utils.py + models.py (bare dotted, M26) |
//! | simple-typescript | 3 | ./utils, ./models edges resolve |
//! | simple-javascript | 3 | ES6 + require() edges resolve |
//! | simple-go | 0 | no go.mod at test cwd; all Go external |
//! | simple-java | 0 | java.util.* is stdlib, not in fixture graph |
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
    // main.py has `from utils import helper` → "utils" and `from models import User` → "models".
    // Both resolve to files in the fixture (utils.py, models.py). M26 bare-dotted resolution.
    assert_eq!(
        edge_count("python"),
        2,
        "simple-python: main.py → utils.py + models.py (2 bare-dotted edges)"
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
    // simple-go only imports "fmt" and "os" (stdlib). No go.mod present at the
    // test cwd (sdivi-rust workspace root is a Rust project), so all Go imports
    // are treated as external regardless of M26.
    assert_eq!(
        edge_count("go"),
        0,
        "simple-go: stdlib + no go.mod → all external"
    );
}

// ── simple-java ───────────────────────────────────────────────────────────────

#[test]
fn simple_java_edge_count() {
    // simple-java only imports java.util.List and java.util.ArrayList (stdlib).
    // These are not in the fixture graph even with M26 Java resolution.
    assert_eq!(
        edge_count("java"),
        0,
        "simple-java: java.util.* stdlib, not in graph"
    );
}

// ── simple-python-relative ───────────────────────────────────────────────────

/// End-to-end regression sentinel for Python relative-import specifiers.
///
/// `pkg/__init__.py` contains `from . import models` (specifier `"."`) and
/// `from .models import User` (specifier `".models"`).
///
/// Post-M26: `".models"` resolves to `pkg/models.py` (1 edge). `"."` resolves
/// to `pkg/__init__.py` — a self-loop that is dropped — so edge count is 1.
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

    // ".models" → pkg/models.py (edge); "." → pkg/__init__.py is a self-loop, dropped.
    let dg = build_dependency_graph(&records);
    assert_eq!(
        dg.edge_count(),
        1,
        "Python '.models' resolves to pkg/models.py; '.' self-loop is dropped"
    );
}
