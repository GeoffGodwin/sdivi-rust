//! Tests for import/export extraction in the Python language adapter.

use sdivi_lang_python::PythonAdapter;
use sdivi_parsing::adapter::LanguageAdapter;
use sdivi_parsing::feature_record::FeatureRecord;
use std::path::Path;

fn parse(source: &str) -> FeatureRecord {
    PythonAdapter.parse_file(Path::new("test.py"), source.to_string())
}

#[test]
fn adapter_language_name_is_python() {
    assert_eq!(PythonAdapter.language_name(), "python");
}

#[test]
fn adapter_handles_py_extension() {
    assert!(PythonAdapter.file_extensions().contains(&".py"));
}

// ── import_statement ─────────────────────────────────────────────────────────

#[test]
fn import_statement_emits_specifier() {
    let record = parse("import os\n");
    assert_eq!(record.imports, &["os"]);
}

#[test]
fn import_with_alias_drops_alias() {
    let record = parse("import os.path as p\n");
    assert_eq!(record.imports, &["os.path"]);
}

#[test]
fn multi_name_import_yields_multiple_specifiers() {
    let record = parse("import os, sys, re\n");
    assert_eq!(record.imports, &["os", "sys", "re"]);
}

#[test]
fn import_dotted_name() {
    let record = parse("import os.path\n");
    assert_eq!(record.imports, &["os.path"]);
}

// ── import_from_statement ────────────────────────────────────────────────────

#[test]
fn from_import_emits_module_specifier() {
    let record = parse("from os.path import join\n");
    assert_eq!(record.imports, &["os.path"]);
}

#[test]
fn from_import_multi_names_still_one_specifier() {
    let record = parse("from os.path import join, exists\n");
    assert_eq!(record.imports, &["os.path"]);
}

#[test]
fn relative_single_dot_import() {
    let record = parse("from . import utils\n");
    assert_eq!(record.imports, &["."]);
}

#[test]
fn relative_double_dot_import() {
    let record = parse("from .. import shared\n");
    assert_eq!(record.imports, &[".."]);
}

#[test]
fn relative_dot_with_name_import() {
    let record = parse("from .models import User\n");
    assert_eq!(record.imports, &[".models"]);
}

#[test]
fn relative_double_dot_with_name_import() {
    let record = parse("from ..pkg import helper\n");
    assert_eq!(record.imports, &["..pkg"]);
}

// ── future_import_statement ──────────────────────────────────────────────────

#[test]
fn future_import_yields_nothing() {
    let record = parse("from __future__ import annotations\n");
    assert!(
        record.imports.is_empty(),
        "from __future__ import must produce no specifier, got: {:?}",
        record.imports
    );
}

// ── count tests ──────────────────────────────────────────────────────────────

#[test]
fn multiple_imports_are_all_extracted() {
    let record = parse("import os\nimport sys\n");
    assert_eq!(record.imports.len(), 2);
}

// ── exports ──────────────────────────────────────────────────────────────────

#[test]
fn top_level_public_function_is_exported() {
    let record = parse("def public_fn():\n    pass\n");
    assert!(
        record.exports.contains(&"public_fn".to_string()),
        "public function must appear in exports, got: {:?}",
        record.exports
    );
}

#[test]
fn underscore_function_is_not_exported() {
    let record = parse("def _private():\n    pass\n");
    assert!(
        !record.exports.contains(&"_private".to_string()),
        "underscore-prefixed function must not be exported, got: {:?}",
        record.exports
    );
}

#[test]
fn top_level_class_is_exported() {
    let record = parse("class MyClass:\n    pass\n");
    assert!(
        record.exports.contains(&"MyClass".to_string()),
        "class name must appear in exports, got: {:?}",
        record.exports
    );
}

#[test]
fn nested_function_is_not_in_exports() {
    let record = parse("def outer():\n    def inner():\n        pass\n");
    assert!(
        record.exports.contains(&"outer".to_string()),
        "outer function should be exported"
    );
    assert!(
        !record.exports.contains(&"inner".to_string()),
        "nested inner function must not be in top-level exports"
    );
}

// ── pattern hints ─────────────────────────────────────────────────────────────

#[test]
fn try_statement_captured_as_pattern_hint() {
    let record = parse("try:\n    pass\nexcept Exception:\n    pass\n");
    let has_try = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "try_statement");
    assert!(has_try, "try_statement must appear in pattern_hints");
}

#[test]
fn lambda_captured_as_pattern_hint() {
    let record = parse("f = lambda x: x + 1\n");
    let has_lambda = record.pattern_hints.iter().any(|h| h.node_kind == "lambda");
    assert!(has_lambda, "lambda must appear in pattern_hints");
}

#[test]
fn pattern_hints_text_does_not_exceed_256_bytes() {
    let fill = "á".repeat(200);
    let source = format!("try:\n    x = \"{fill}\"\nexcept Exception:\n    pass\n");
    let record = parse(&source);
    for hint in &record.pattern_hints {
        assert!(
            hint.text.len() <= 256,
            "hint text must be ≤ 256 bytes, got {} for {:?}",
            hint.text.len(),
            hint.node_kind
        );
        assert!(hint.text.is_char_boundary(hint.text.len()));
    }
}
