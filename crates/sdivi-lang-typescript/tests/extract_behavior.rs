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

// ── class_hierarchy pattern hints (M31) ──────────────────────────────────────

#[test]
fn plain_class_declaration_captured_as_class_hierarchy_hint() {
    let record = parse_ts("class Foo { bar(): void {} }\n");
    let has_class = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "class_declaration");
    assert!(has_class, "class_declaration must appear in pattern_hints for a plain class");
}

#[test]
fn class_with_extends_captured_as_class_hierarchy_hint() {
    let record = parse_ts("class Child extends Parent { constructor() { super(); } }\n");
    let has_class = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "class_declaration");
    assert!(
        has_class,
        "class_declaration must appear in pattern_hints for a class with extends clause"
    );
}

#[test]
fn abstract_class_declaration_captured_as_class_hierarchy_hint() {
    let record = parse_ts("abstract class Shape { abstract area(): number; }\n");
    let has_abstract = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "abstract_class_declaration");
    assert!(
        has_abstract,
        "abstract_class_declaration must appear in pattern_hints"
    );
}

#[test]
fn interface_declaration_captured_as_class_hierarchy_hint() {
    let record = parse_ts("interface Drawable { draw(): void; }\n");
    let has_iface = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "interface_declaration");
    assert!(has_iface, "interface_declaration must appear in pattern_hints");
}

#[test]
fn all_three_class_hierarchy_kinds_collected_in_one_file() {
    let source = concat!(
        "interface Printable { print(): void; }\n",
        "abstract class Base { abstract run(): void; }\n",
        "class Concrete extends Base implements Printable { run(): void {} print(): void {} }\n",
    );
    let record = parse_ts(source);
    let kinds: Vec<&str> = record
        .pattern_hints
        .iter()
        .map(|h| h.node_kind.as_str())
        .collect();
    assert!(
        kinds.contains(&"interface_declaration"),
        "interface_declaration must appear in pattern_hints, got: {kinds:?}"
    );
    assert!(
        kinds.contains(&"abstract_class_declaration"),
        "abstract_class_declaration must appear in pattern_hints, got: {kinds:?}"
    );
    assert!(
        kinds.contains(&"class_declaration"),
        "class_declaration must appear in pattern_hints, got: {kinds:?}"
    );
}
