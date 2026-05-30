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

// ── class_hierarchy pattern hints (M31) ──────────────────────────────────────

#[test]
fn plain_class_declaration_captured_as_class_hierarchy_hint() {
    let record = parse("public class Greeter { public void hello() {} }\n");
    let has_class = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "class_declaration");
    assert!(
        has_class,
        "class_declaration must appear in pattern_hints for a plain Java class"
    );
}

#[test]
fn class_with_extends_captured_as_class_hierarchy_hint() {
    let record = parse("public class Child extends Parent { }\n");
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
fn interface_declaration_captured_as_class_hierarchy_hint() {
    let record = parse("public interface Printable { void print(); }\n");
    let has_iface = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "interface_declaration");
    assert!(
        has_iface,
        "interface_declaration must appear in pattern_hints"
    );
}

#[test]
fn class_and_interface_both_collected_in_one_file() {
    let source = concat!(
        "public interface Shape { double area(); }\n",
        "public class Circle implements Shape { public double area() { return 3.14; } }\n",
    );
    let record = parse(source);
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
        kinds.contains(&"class_declaration"),
        "class_declaration must appear in pattern_hints, got: {kinds:?}"
    );
}
