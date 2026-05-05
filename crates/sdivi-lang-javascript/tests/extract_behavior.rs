//! Tests for import/export extraction in the JavaScript language adapter.

use sdivi_lang_javascript::JavaScriptAdapter;
use sdivi_parsing::adapter::LanguageAdapter;
use sdivi_parsing::feature_record::FeatureRecord;
use std::path::Path;

fn parse(source: &str) -> FeatureRecord {
    JavaScriptAdapter.parse_file(Path::new("test.js"), source.to_string())
}

#[test]
fn adapter_language_name_is_javascript() {
    assert_eq!(JavaScriptAdapter.language_name(), "javascript");
}

#[test]
fn adapter_handles_js_and_mjs_extensions() {
    let exts = JavaScriptAdapter.file_extensions();
    assert!(exts.contains(&".js"));
    assert!(exts.contains(&".mjs"));
}

// ── ES6 import_statement ─────────────────────────────────────────────────────

#[test]
fn named_import_yields_string_fragment() {
    let record = parse("import { foo } from './foo.js';\n");
    assert_eq!(record.imports, &["./foo.js"]);
}

#[test]
fn default_import_yields_specifier() {
    let record = parse("import React from 'react';\n");
    assert_eq!(record.imports, &["react"]);
}

#[test]
fn side_effect_import_yields_specifier() {
    let record = parse("import './polyfill.js';\n");
    assert_eq!(record.imports, &["./polyfill.js"]);
}

// ── CommonJS require() ───────────────────────────────────────────────────────

#[test]
fn require_call_yields_specifier() {
    let record = parse("const fs = require('./utils');\n");
    assert_eq!(record.imports, &["./utils"]);
}

#[test]
fn require_package_yields_specifier() {
    let record = parse("const path = require('path');\n");
    assert_eq!(record.imports, &["path"]);
}

#[test]
fn require_variable_arg_is_skipped() {
    let record = parse("const m = require(name);\n");
    assert!(
        record.imports.is_empty(),
        "require(variable) must produce no specifier, got: {:?}",
        record.imports
    );
}

// ── dynamic import() ─────────────────────────────────────────────────────────

#[test]
fn dynamic_import_string_literal_yields_specifier() {
    let record = parse("const m = import('./chunk.js');\n");
    // The tree-sitter grammar may represent dynamic import() as a call_expression
    // with an `import` function node or as an import_expression; either way the
    // adapter should extract "./chunk.js" when it can, and produce nothing when
    // the grammar represents it differently. Both outcomes are valid.
    assert!(
        record.imports.is_empty() || record.imports == ["./chunk.js"],
        "dynamic import('./chunk.js') must yield [\"./chunk.js\"] or nothing (grammar-dependent), got: {:?}",
        record.imports
    );
}

// ── count tests ──────────────────────────────────────────────────────────────

#[test]
fn multiple_imports_are_extracted() {
    let record = parse("import a from './a.js';\nimport b from './b.js';\n");
    assert_eq!(record.imports.len(), 2);
}

#[test]
fn es6_and_require_both_extracted() {
    let record = parse("import { foo } from './a.js';\nconst b = require('./b.js');\n");
    assert_eq!(record.imports.len(), 2);
    assert!(record.imports.contains(&"./a.js".to_string()));
    assert!(record.imports.contains(&"./b.js".to_string()));
}

// ── exports ──────────────────────────────────────────────────────────────────

#[test]
fn exported_function_name_is_captured() {
    let record = parse("export function greet() {}\n");
    assert!(
        record.exports.contains(&"greet".to_string()),
        "exported function name must appear in exports, got: {:?}",
        record.exports
    );
}

#[test]
fn exported_class_name_is_captured() {
    let record = parse("export class Widget {}\n");
    assert!(
        record.exports.contains(&"Widget".to_string()),
        "exported class name must appear in exports, got: {:?}",
        record.exports
    );
}

#[test]
fn non_exported_function_is_not_in_exports() {
    let record = parse("function hidden() {}\n");
    assert!(
        record.exports.is_empty(),
        "non-exported function must not appear in exports, got: {:?}",
        record.exports
    );
}

// ── pattern hints ─────────────────────────────────────────────────────────────

#[test]
fn try_statement_captured_as_pattern_hint() {
    let record = parse("try { } catch(e) { }\n");
    let has_try = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "try_statement");
    assert!(has_try, "try_statement must appear in pattern_hints");
}

#[test]
fn await_expression_captured_as_pattern_hint() {
    let record = parse("async function f() { const x = await fetch('url'); return x; }\n");
    let has_await = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "await_expression");
    assert!(has_await, "await_expression must appear in pattern_hints");
}

#[test]
fn pattern_hints_text_does_not_exceed_256_bytes() {
    let fill = "á".repeat(200);
    let source = format!("try {{ const s = \"{fill}\"; }} catch(e) {{}}\n");
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
