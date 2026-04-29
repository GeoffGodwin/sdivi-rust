//! Tests for import/export extraction in the JavaScript language adapter.

use sdi_lang_javascript::JavaScriptAdapter;
use sdi_parsing::adapter::LanguageAdapter;
use sdi_parsing::feature_record::FeatureRecord;
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

#[test]
fn import_statement_is_extracted() {
    let record = parse("import { foo } from './foo.js';\n");
    assert_eq!(record.imports.len(), 1);
    assert!(record.imports[0].contains("import"));
}

#[test]
fn multiple_imports_are_extracted() {
    let record = parse("import a from './a.js';\nimport b from './b.js';\n");
    assert_eq!(record.imports.len(), 2);
}

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
