//! Tests for import/export extraction in the Java language adapter.

use sdivi_lang_java::JavaAdapter;
use sdivi_parsing::adapter::LanguageAdapter;
use sdivi_parsing::feature_record::FeatureRecord;
use std::path::Path;

fn parse(source: &str) -> FeatureRecord {
    JavaAdapter.parse_file(Path::new("Test.java"), source.to_string())
}

#[test]
fn adapter_language_name_is_java() {
    assert_eq!(JavaAdapter.language_name(), "java");
}

#[test]
fn adapter_handles_java_extension() {
    assert!(JavaAdapter.file_extensions().contains(&".java"));
}

// ── regular imports ───────────────────────────────────────────────────────────

#[test]
fn plain_import_yields_qualified_name() {
    let record = parse("import java.util.List;\npublic class A {}\n");
    assert_eq!(record.imports, &["java.util.List"]);
}

#[test]
fn multiple_imports_are_extracted() {
    let record = parse("import java.util.List;\nimport java.util.Map;\npublic class A {}\n");
    assert_eq!(record.imports.len(), 2);
    assert!(record.imports.contains(&"java.util.List".to_string()));
    assert!(record.imports.contains(&"java.util.Map".to_string()));
}

// ── wildcard imports ──────────────────────────────────────────────────────────

#[test]
fn wildcard_import_appends_star() {
    let record = parse("import java.util.*;\npublic class A {}\n");
    assert_eq!(record.imports, &["java.util.*"]);
}

// ── static imports ────────────────────────────────────────────────────────────

#[test]
fn static_import_strips_member_name() {
    let record = parse("import static org.junit.Assert.assertEquals;\npublic class A {}\n");
    assert_eq!(record.imports, &["org.junit.Assert"]);
}

#[test]
fn static_wildcard_import_yields_class_specifier() {
    let record = parse("import static org.junit.Assert.*;\npublic class A {}\n");
    assert_eq!(record.imports, &["org.junit.Assert"]);
}

// ── exports ──────────────────────────────────────────────────────────────────

#[test]
fn public_class_is_exported() {
    let record = parse("public class Greeter {}\n");
    assert!(
        record.exports.contains(&"Greeter".to_string()),
        "public class name must appear in exports, got: {:?}",
        record.exports
    );
}

#[test]
fn package_private_class_is_not_exported() {
    let record = parse("class Hidden {}\n");
    assert!(
        !record.exports.contains(&"Hidden".to_string()),
        "package-private class must not appear in exports, got: {:?}",
        record.exports
    );
}

// ── pattern hints ─────────────────────────────────────────────────────────────

#[test]
fn try_statement_captured_as_pattern_hint() {
    let record = parse("public class A { public void f() { try {} catch(Exception e) {} } }\n");
    let has_try = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "try_statement");
    assert!(has_try, "try_statement must appear in pattern_hints");
}

#[test]
fn lambda_expression_captured_as_pattern_hint() {
    let record =
        parse("import java.util.function.Function;\npublic class A { Runnable r = () -> {}; }\n");
    let has_lambda = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "lambda_expression");
    assert!(has_lambda, "lambda_expression must appear in pattern_hints");
}

#[test]
fn pattern_hints_text_does_not_exceed_256_bytes() {
    let fill = "á".repeat(200);
    let source = format!(
        "public class A {{ public void f() {{ try {{ String s = \"{fill}\"; }} catch(Exception e) {{}} }} }}\n"
    );
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
