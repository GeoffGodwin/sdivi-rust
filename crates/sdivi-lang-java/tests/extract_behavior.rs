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

// ── try_with_resources_statement pattern hints (M45.1) ───────────────────────

/// M45.1: `try_with_resources_statement` is emitted by the Java adapter from
/// real Java source — confirming the real tree-sitter parse path that the
/// synthetic-FeatureRecord tests in `resource_management_fixture.rs` do not cover.
#[test]
fn try_with_resources_statement_captured_as_pattern_hint() {
    let record = parse(concat!(
        "import java.io.InputStream;\n",
        "public class A {\n",
        "    public void f() throws Exception {\n",
        "        try (InputStream is = A.class.getResourceAsStream(\"f\")) {\n",
        "        }\n",
        "    }\n",
        "}\n",
    ));
    let has_twr = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "try_with_resources_statement");
    assert!(
        has_twr,
        "try_with_resources_statement must appear in pattern_hints when parsing real Java source; \
         got node kinds: {:?}",
        record
            .pattern_hints
            .iter()
            .map(|h| h.node_kind.as_str())
            .collect::<Vec<_>>()
    );
}

/// M45.1: The captured hint's text is non-empty and starts with "try".
#[test]
fn try_with_resources_hint_text_starts_with_try() {
    let record = parse(concat!(
        "import java.io.InputStream;\n",
        "public class A {\n",
        "    public void f() throws Exception {\n",
        "        try (InputStream is = A.class.getResourceAsStream(\"f\")) {\n",
        "        }\n",
        "    }\n",
        "}\n",
    ));
    let twr = record
        .pattern_hints
        .iter()
        .find(|h| h.node_kind == "try_with_resources_statement")
        .expect("try_with_resources_statement hint must be present");
    assert!(
        twr.text.starts_with("try"),
        "try_with_resources_statement text must start with 'try'; got: {:?}",
        twr.text
    );
    assert!(twr.text.len() <= 256, "hint text must not exceed 256 bytes");
}

/// M45.1: `try_with_resources_statement` and plain `try_statement` are
/// distinct node kinds — the adapter does not conflate them.
#[test]
fn try_with_resources_is_distinct_from_try_statement() {
    let source_twr = concat!(
        "public class A {\n",
        "    public void f() throws Exception {\n",
        "        try (var r = java.io.InputStream.class.getResourceAsStream(\"f\")) {\n",
        "        }\n",
        "    }\n",
        "}\n",
    );
    let source_try = concat!(
        "public class A {\n",
        "    public void f() {\n",
        "        try { } catch (Exception e) { }\n",
        "    }\n",
        "}\n",
    );
    let rec_twr = parse(source_twr);
    let rec_try = parse(source_try);

    // try-with-resources source must produce try_with_resources_statement, not try_statement
    assert!(
        rec_twr
            .pattern_hints
            .iter()
            .any(|h| h.node_kind == "try_with_resources_statement"),
        "source with try-with-resources must produce try_with_resources_statement hint"
    );
    // plain try source must produce try_statement, not try_with_resources_statement
    assert!(
        rec_try
            .pattern_hints
            .iter()
            .any(|h| h.node_kind == "try_statement"),
        "source with plain try must produce try_statement hint"
    );
    assert!(
        !rec_try
            .pattern_hints
            .iter()
            .any(|h| h.node_kind == "try_with_resources_statement"),
        "plain try source must NOT produce try_with_resources_statement hint"
    );
}

// ── M45.2: catch_clause and throw_statement pattern hints ────────────────────

/// M45.2: `catch_clause` is emitted by the Java adapter from real Java source —
/// confirming the real tree-sitter parse path that the synthetic-FeatureRecord tests
/// in `error_handling_fixture.rs` do not cover.
#[test]
fn catch_clause_captured_as_pattern_hint() {
    let record = parse(concat!(
        "public class A {\n",
        "    public void f() {\n",
        "        try { } catch (IOException e) { }\n",
        "    }\n",
        "}\n",
    ));
    let has_catch = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "catch_clause");
    assert!(
        has_catch,
        "catch_clause must appear in pattern_hints when parsing real Java source; \
         got node kinds: {:?}",
        record
            .pattern_hints
            .iter()
            .map(|h| h.node_kind.as_str())
            .collect::<Vec<_>>()
    );
}

/// M45.2: Each `catch` arm in a multi-catch try/catch produces a separate
/// `catch_clause` hint — one hint per arm, not one per try block.
#[test]
fn multi_catch_emits_one_hint_per_arm() {
    let record = parse(concat!(
        "public class A {\n",
        "    public void f() {\n",
        "        try {\n",
        "        } catch (IOException e) {\n",
        "        } catch (SQLException e) {\n",
        "        } catch (RuntimeException e) {\n",
        "        }\n",
        "    }\n",
        "}\n",
    ));
    let catch_count = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "catch_clause")
        .count();
    assert_eq!(
        catch_count, 3,
        "three catch arms must produce exactly 3 catch_clause hints; got: {catch_count}"
    );
}

/// M45.2: `throw_statement` is emitted by the Java adapter from real Java source.
#[test]
fn throw_statement_captured_as_pattern_hint() {
    let record = parse(concat!(
        "public class A {\n",
        "    public void f() {\n",
        "        throw new RuntimeException(\"oops\");\n",
        "    }\n",
        "}\n",
    ));
    let has_throw = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "throw_statement");
    assert!(
        has_throw,
        "throw_statement must appear in pattern_hints when parsing real Java source; \
         got node kinds: {:?}",
        record
            .pattern_hints
            .iter()
            .map(|h| h.node_kind.as_str())
            .collect::<Vec<_>>()
    );
}

/// M45.2: A try block with two catch arms and a throw emits 1 try_statement +
/// 2 catch_clause + 1 throw_statement (double-count semantic verified end-to-end
/// from the real Java adapter rather than synthetic hints).
#[test]
fn try_with_catches_and_throw_emits_all_hint_kinds() {
    let record = parse(concat!(
        "public class A {\n",
        "    public void f() {\n",
        "        try {\n",
        "            risky();\n",
        "        } catch (IOException e) {\n",
        "            throw new RuntimeException(e);\n",
        "        } catch (SQLException e) {\n",
        "            throw new IllegalStateException(e);\n",
        "        }\n",
        "    }\n",
        "    private void risky() throws IOException, SQLException {}\n",
        "}\n",
    ));
    let try_count = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "try_statement")
        .count();
    let catch_count = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "catch_clause")
        .count();
    let throw_count = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "throw_statement")
        .count();
    assert_eq!(
        try_count, 1,
        "one try block must produce exactly 1 try_statement hint; got: {try_count}"
    );
    assert_eq!(
        catch_count, 2,
        "two catch arms must produce exactly 2 catch_clause hints; got: {catch_count}"
    );
    assert_eq!(
        throw_count, 2,
        "two throw statements must produce exactly 2 throw_statement hints; got: {throw_count}"
    );
}

/// M45.2: A method body with no try/catch produces no `catch_clause` hints.
#[test]
fn method_with_no_catch_produces_no_catch_clause_hints() {
    let record = parse(concat!(
        "public class A {\n",
        "    public int add(int a, int b) { return a + b; }\n",
        "}\n",
    ));
    let catch_count = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "catch_clause")
        .count();
    assert_eq!(
        catch_count, 0,
        "method with no catch must produce zero catch_clause hints; got: {catch_count}"
    );
}

/// M45.2: A method body with no throw produces no `throw_statement` hints.
#[test]
fn method_with_no_throw_produces_no_throw_statement_hints() {
    let record = parse(concat!(
        "public class A {\n",
        "    public int add(int a, int b) { return a + b; }\n",
        "}\n",
    ));
    let throw_count = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "throw_statement")
        .count();
    assert_eq!(
        throw_count, 0,
        "method with no throw must produce zero throw_statement hints; got: {throw_count}"
    );
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
