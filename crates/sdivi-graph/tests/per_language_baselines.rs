//! Pinned per-language edge-count baselines for the `tests/fixtures/simple-*` repos.
//!
//! Any PR that changes resolver logic causing a baseline to shift must update
//! these counts deliberately — that is the intent of `assert_eq!`.
//!
//! Post-M26 baselines:
//!
//! | Fixture               | Edges | Notes                                        |
//! |-----------------------|-------|----------------------------------------------|
//! | simple-rust           |     0 | all imports are external std/serde            |
//! | simple-python         |     2 | main.py → utils.py + models.py (bare dotted) |
//! | simple-typescript     |     3 | ./utils, ./models edges resolve (unchanged)  |
//! | simple-javascript     |     3 | ES6 + require() edges (unchanged)            |
//! | simple-go             |     0 | no go.mod at test cwd; all Go external        |
//! | simple-java           |     0 | java.util.* is stdlib, not in graph           |
//! | simple-python-relative|     1 | .models → pkg/models.py (.→self dropped)     |

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

#[test]
fn baseline_simple_rust() {
    assert_eq!(edge_count("rust"), 0, "simple-rust: all imports external");
}

#[test]
fn baseline_simple_python() {
    // main.py imports "utils" and "models" which exist in the fixture.
    assert_eq!(
        edge_count("python"),
        2,
        "simple-python: 2 bare-dotted edges"
    );
}

#[test]
fn baseline_simple_typescript() {
    assert_eq!(
        edge_count("typescript"),
        3,
        "simple-typescript: 3 relative edges"
    );
}

#[test]
fn baseline_simple_javascript() {
    assert_eq!(
        edge_count("javascript"),
        3,
        "simple-javascript: 3 relative edges"
    );
}

#[test]
fn baseline_simple_go() {
    // No go.mod at the test process's cwd (sdivi-rust workspace root is a Rust
    // project). All Go imports treated as external.
    assert_eq!(
        edge_count("go"),
        0,
        "simple-go: all imports external (no go.mod)"
    );
}

#[test]
fn baseline_simple_java() {
    // java.util.* stdlib imports have no matching files in the graph.
    assert_eq!(
        edge_count("java"),
        0,
        "simple-java: stdlib imports external"
    );
}

#[test]
fn baseline_simple_python_relative() {
    let config = Config::default();
    let root = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/simple-python-relative"
    ));
    let adapters: Vec<Box<dyn LanguageAdapter>> = vec![Box::new(PythonAdapter)];
    let records = parse_repository(&config, &root, &adapters);
    let dg = build_dependency_graph(&records);
    // ".models" → pkg/models.py (1 edge); "." → pkg/__init__.py is a self-loop, dropped.
    assert_eq!(
        dg.edge_count(),
        1,
        "simple-python-relative: 1 edge (.models → models.py)"
    );
}
