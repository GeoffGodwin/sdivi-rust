//! Tests for import/export extraction in the Go language adapter.

use sdivi_lang_go::GoAdapter;
use sdivi_parsing::adapter::LanguageAdapter;
use sdivi_parsing::feature_record::FeatureRecord;
use std::path::Path;

fn parse(source: &str) -> FeatureRecord {
    GoAdapter.parse_file(Path::new("test.go"), source.to_string())
}

#[test]
fn adapter_language_name_is_go() {
    assert_eq!(GoAdapter.language_name(), "go");
}

#[test]
fn adapter_handles_go_extension() {
    assert!(GoAdapter.file_extensions().contains(&".go"));
}

#[test]
fn import_declaration_is_extracted() {
    let record = parse("package main\nimport \"fmt\"\n");
    assert_eq!(record.imports.len(), 1);
    assert!(record.imports[0].contains("import"));
}

#[test]
fn grouped_import_is_extracted_as_single_entry() {
    let record = parse("package main\nimport (\n    \"fmt\"\n    \"os\"\n)\n");
    assert_eq!(
        record.imports.len(),
        1,
        "grouped import is one import_declaration, got: {:?}",
        record.imports
    );
}

#[test]
fn exported_function_capitalized_name_is_captured() {
    let record = parse("package main\nfunc Hello() {}\n");
    assert!(
        record.exports.contains(&"Hello".to_string()),
        "capitalized function name must appear in exports, got: {:?}",
        record.exports
    );
}

#[test]
fn unexported_function_lowercase_name_is_not_exported() {
    let record = parse("package main\nfunc helper() {}\n");
    assert!(
        !record.exports.contains(&"helper".to_string()),
        "lowercase function must not appear in exports, got: {:?}",
        record.exports
    );
}

#[test]
fn go_statement_captured_as_pattern_hint() {
    let record = parse("package main\nfunc f() { go func() {}() }\n");
    let has_go = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "go_statement");
    assert!(has_go, "go_statement must appear in pattern_hints");
}

#[test]
fn defer_statement_captured_as_pattern_hint() {
    let record = parse("package main\nfunc f() { defer cleanup() }\n");
    let has_defer = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "defer_statement");
    assert!(has_defer, "defer_statement must appear in pattern_hints");
}

#[test]
fn pattern_hints_text_does_not_exceed_256_bytes() {
    let fill = "á".repeat(200);
    let source = format!("package main\nfunc f() {{ go func() {{ _ = \"{fill}\" }}() }}\n");
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
