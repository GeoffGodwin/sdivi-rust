//! Tests for import/export extraction and pattern hints in the Rust language adapter.

use sdivi_lang_rust::RustAdapter;
use sdivi_parsing::adapter::LanguageAdapter;
use sdivi_parsing::feature_record::FeatureRecord;
use std::path::Path;

fn parse(source: &str) -> FeatureRecord {
    RustAdapter.parse_file(Path::new("test.rs"), source.to_string())
}

#[test]
fn adapter_language_name_is_rust() {
    assert_eq!(RustAdapter.language_name(), "rust");
}

#[test]
fn adapter_handles_rs_extension() {
    assert!(RustAdapter.file_extensions().contains(&".rs"));
}

// ── imports ───────────────────────────────────────────────────────────────────

#[test]
fn use_declaration_yields_import_path() {
    let record = parse("use std::collections::BTreeMap;\n");
    assert_eq!(record.imports, &["std::collections::BTreeMap"]);
}

#[test]
fn multiple_use_declarations_are_extracted() {
    let record = parse("use std::fmt;\nuse std::io;\n");
    assert_eq!(record.imports.len(), 2);
    assert!(record.imports.contains(&"std::fmt".to_string()));
    assert!(record.imports.contains(&"std::io".to_string()));
}

// ── exports ───────────────────────────────────────────────────────────────────

#[test]
fn public_function_is_exported() {
    let record = parse("pub fn hello() {}\n");
    assert!(
        record.exports.contains(&"hello".to_string()),
        "pub fn must appear in exports, got: {:?}",
        record.exports
    );
}

#[test]
fn private_function_is_not_exported() {
    let record = parse("fn hidden() {}\n");
    assert!(
        !record.exports.contains(&"hidden".to_string()),
        "private fn must not appear in exports, got: {:?}",
        record.exports
    );
}

#[test]
fn public_struct_is_exported() {
    let record = parse("pub struct Foo {}\n");
    assert!(
        record.exports.contains(&"Foo".to_string()),
        "pub struct must appear in exports, got: {:?}",
        record.exports
    );
}

// ── class_hierarchy pattern hints (M31) ──────────────────────────────────────

#[test]
fn inherent_impl_captured_as_class_hierarchy_hint() {
    let record = parse("struct Foo;\nimpl Foo { pub fn new() -> Self { Foo } }\n");
    let has_impl = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "impl_item");
    assert!(
        has_impl,
        "impl_item must appear in pattern_hints for an inherent impl block"
    );
}

#[test]
fn trait_impl_captured_as_class_hierarchy_hint() {
    let record = parse(
        "use std::fmt;\nstruct Foo;\nimpl fmt::Display for Foo { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, \"Foo\") } }\n",
    );
    let has_impl = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "impl_item");
    assert!(
        has_impl,
        "impl_item must appear in pattern_hints for a trait impl block"
    );
}

#[test]
fn multiple_impl_blocks_all_collected() {
    let source = concat!(
        "struct Foo;\n",
        "trait Bar { fn bar(&self); }\n",
        "impl Foo { pub fn new() -> Self { Foo } }\n",
        "impl Bar for Foo { fn bar(&self) {} }\n",
    );
    let record = parse(source);
    let impl_hints: Vec<_> = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "impl_item")
        .collect();
    assert_eq!(
        impl_hints.len(),
        2,
        "both impl blocks must appear as impl_item hints, got: {impl_hints:?}"
    );
}

#[test]
fn pattern_hints_text_does_not_exceed_256_bytes() {
    let fill = "x".repeat(300);
    let source = format!("struct S;\nimpl S {{ pub fn f(&self) -> &str {{ \"{fill}\" }} }}\n");
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
