//! Tests for import/export extraction in the TypeScript language adapter.

use sdivi_lang_typescript::TypeScriptAdapter;
use sdivi_parsing::adapter::LanguageAdapter;
use sdivi_parsing::feature_record::FeatureRecord;
use std::path::Path;

fn parse_ts(source: &str) -> FeatureRecord {
    TypeScriptAdapter.parse_file(Path::new("test.ts"), source.to_string())
}

fn parse_tsx(source: &str) -> FeatureRecord {
    TypeScriptAdapter.parse_file(Path::new("test.tsx"), source.to_string())
}

#[test]
fn adapter_language_name_is_typescript() {
    assert_eq!(TypeScriptAdapter.language_name(), "typescript");
}

#[test]
fn adapter_handles_ts_and_tsx_extensions() {
    let exts = TypeScriptAdapter.file_extensions();
    assert!(exts.contains(&".ts"));
    assert!(exts.contains(&".tsx"));
}

// ── import_statement specifier extraction ────────────────────────────────────

#[test]
fn named_import_yields_string_fragment() {
    let record = parse_ts("import { foo } from './foo';\n");
    assert_eq!(record.imports, &["./foo"]);
}

#[test]
fn default_import_yields_specifier() {
    let record = parse_ts("import React from 'react';\n");
    assert_eq!(record.imports, &["react"]);
}

#[test]
fn namespace_import_yields_specifier() {
    let record = parse_ts("import * as ns from './util';\n");
    assert_eq!(record.imports, &["./util"]);
}

#[test]
fn side_effect_import_yields_specifier() {
    let record = parse_ts("import './side-effect';\n");
    assert_eq!(record.imports, &["./side-effect"]);
}

#[test]
fn type_only_import_yields_specifier() {
    let record = parse_ts("import type { T } from './types';\n");
    assert_eq!(record.imports, &["./types"]);
}

#[test]
fn parent_relative_import_yields_specifier() {
    let record = parse_ts("import { x } from '../lib/x';\n");
    assert_eq!(record.imports, &["../lib/x"]);
}

// ── count tests ──────────────────────────────────────────────────────────────

#[test]
fn multiple_imports_are_extracted() {
    let record = parse_ts("import { a } from './a';\nimport { b } from './b';\n");
    assert_eq!(record.imports.len(), 2);
}

// ── exports ──────────────────────────────────────────────────────────────────

#[test]
fn exported_function_name_is_captured() {
    let record = parse_ts("export function hello(): void {}\n");
    assert!(
        record.exports.contains(&"hello".to_string()),
        "exported function name must appear in exports, got: {:?}",
        record.exports
    );
}

#[test]
fn exported_class_name_is_captured() {
    let record = parse_ts("export class Foo {}\n");
    assert!(
        record.exports.contains(&"Foo".to_string()),
        "exported class name must appear in exports, got: {:?}",
        record.exports
    );
}

#[test]
fn non_exported_function_is_not_in_exports() {
    let record = parse_ts("function hidden(): void {}\n");
    assert!(
        record.exports.is_empty(),
        "non-exported function must not appear in exports, got: {:?}",
        record.exports
    );
}

#[test]
fn tsx_file_parses_correctly() {
    let record = parse_tsx("export function App(): JSX.Element { return null as any; }\n");
    assert!(
        record.exports.contains(&"App".to_string()),
        "exported TSX function must appear in exports, got: {:?}",
        record.exports
    );
}

// ── pattern hints ─────────────────────────────────────────────────────────────

#[test]
fn try_statement_captured_as_pattern_hint() {
    let record = parse_ts("try { } catch(e) { }\n");
    let has_try = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "try_statement");
    assert!(has_try, "try_statement must appear in pattern_hints");
}

#[test]
fn await_expression_captured_as_pattern_hint() {
    let record = parse_ts("async function f() { const x = await fetch('url'); return x; }\n");
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
    let record = parse_ts(&source);
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
