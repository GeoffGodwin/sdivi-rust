//! Tests for import/export extraction correctness in the Rust language adapter.
//!
//! All tests exercise behavior via the public `RustAdapter::parse_file` API;
//! the internal `extract_imports` / `extract_exports` / `collect_hints` helpers
//! are `pub(crate)` and are verified here indirectly.

use sdi_lang_rust::RustAdapter;
use sdi_parsing::adapter::LanguageAdapter;
use sdi_parsing::feature_record::FeatureRecord;
use std::path::Path;

fn parse(source: &str) -> FeatureRecord {
    RustAdapter.parse_file(Path::new("test.rs"), source.to_string())
}

// ── import extraction — plain `use` (regression guard) ────────────────────────

#[test]
fn plain_use_import_is_extracted_without_keyword_prefix() {
    let record = parse("use std::collections::BTreeMap;\n");
    assert_eq!(record.imports.len(), 1, "one use statement → one import");
    assert_eq!(
        record.imports[0], "std::collections::BTreeMap",
        "import must not include 'use' keyword or trailing semicolon"
    );
}

#[test]
fn multiple_use_declarations_each_produce_one_import() {
    let record = parse("use std::fmt;\nuse std::collections::BTreeMap;\n");
    assert_eq!(record.imports.len(), 2, "two use statements → two imports");
}

// ── import extraction — `pub use` forms ───────────────────────────────────────

#[test]
fn pub_use_import_is_captured_as_one_entry() {
    // tree-sitter parses `pub use` as a `use_declaration` node; the entry must
    // be captured regardless of the visibility modifier.
    let record = parse("pub use crate::foo::Bar;\n");
    assert_eq!(
        record.imports.len(),
        1,
        "pub use must produce exactly one import entry, got: {:?}",
        record.imports
    );
}

#[test]
fn pub_use_import_path_excludes_pub_keyword() {
    // The import string must contain only the path, not the "pub use" prefix.
    // Currently the extractor falls back to the full declaration text when
    // strip_prefix("use ") fails on "pub use …", so this test documents a
    // known deficiency: the import is captured as "pub use crate::foo::Bar"
    // rather than "crate::foo::Bar".
    let record = parse("pub use crate::foo::Bar;\n");
    assert_eq!(record.imports.len(), 1);
    let import = &record.imports[0];
    assert!(
        !import.starts_with("pub "),
        "import path must not include 'pub' keyword prefix, got: {import:?}"
    );
}

// ── export extraction — basic correctness ─────────────────────────────────────

#[test]
fn pub_fn_is_exported_by_name() {
    let record = parse("pub fn hello() {}\n");
    assert!(
        record.exports.contains(&"hello".to_string()),
        "pub fn name must appear in exports, got: {:?}",
        record.exports
    );
}

#[test]
fn private_fn_is_not_exported() {
    let record = parse("fn private_fn() {}\n");
    assert!(
        !record.exports.contains(&"private_fn".to_string()),
        "private fn must not appear in exports, got: {:?}",
        record.exports
    );
}

#[test]
fn pub_struct_is_exported_by_name() {
    let record = parse("pub struct Point { pub x: f64, pub y: f64 }\n");
    assert!(
        record.exports.contains(&"Point".to_string()),
        "pub struct name must appear in exports, got: {:?}",
        record.exports
    );
}

// ── export extraction — nested pub items (double-count guard) ─────────────────

#[test]
fn pub_mod_name_appears_in_exports() {
    let record = parse("pub mod outer { pub fn inner() {} }\n");
    assert!(
        record.exports.contains(&"outer".to_string()),
        "pub mod name must appear in exports, got: {:?}",
        record.exports
    );
}

#[test]
fn no_export_name_appears_more_than_once() {
    // Verifies that extract_exports does not produce duplicate entries for any
    // single exported name. A bug where traversal continues into EXPORTABLE_KINDS
    // children without stopping could cause double-counting.
    let record = parse("pub mod outer { pub fn inner() {} }\n");
    let mut seen = std::collections::HashSet::new();
    for export in &record.exports {
        assert!(
            seen.insert(export.clone()),
            "export {export:?} appeared more than once — double-counting in extract_exports, full list: {:?}",
            record.exports
        );
    }
}

#[test]
fn pub_fn_inside_pub_mod_not_in_top_level_exports() {
    // A pub fn nested inside a pub mod is reachable as `outer::inner`, not as a
    // top-level export. Only the module name should appear at the top level.
    // This test documents a known latent issue: the traversal currently recurses
    // into mod_item children, so `inner` is also captured as an export.
    let record = parse("pub mod outer { pub fn inner() {} }\n");
    assert!(
        !record.exports.contains(&"inner".to_string()),
        "nested pub fn 'inner' must not appear as a top-level export; \
         it is accessible via outer::inner. Got exports: {:?}",
        record.exports
    );
}

// ── collect_hints — char-safe truncation at > 256 bytes with Unicode ──────────

#[test]
fn collect_hints_match_expression_captured() {
    let record = parse("fn f(x: u8) -> u8 { match x { 0 => 0, _ => 1 } }\n");
    let has_match = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "match_expression");
    assert!(has_match, "match_expression must appear in pattern_hints");
}

#[test]
fn collect_hints_long_unicode_text_truncated_at_char_boundary() {
    // 'á' is 2 bytes in UTF-8; 200 repetitions = 400 bytes > 256-byte limit.
    // The char-safe truncation path must not panic and must produce valid UTF-8.
    let unicode_fill = "á".repeat(200);
    let source = format!(
        "fn f(x: u8) -> u8 {{\n    match x {{\n        0 => {{ let _s = \"{unicode_fill}\"; 0 }}\n        _ => 1,\n    }}\n}}\n"
    );
    let record = parse(&source);

    let match_hints: Vec<_> = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "match_expression")
        .collect();
    assert!(
        !match_hints.is_empty(),
        "expected at least one match_expression hint for Unicode source"
    );

    for hint in &record.pattern_hints {
        // Truncated text must not exceed the 256-byte cap.
        assert!(
            hint.text.len() <= 256,
            "hint text must be ≤ 256 bytes after truncation, got {} bytes for node {:?}",
            hint.text.len(),
            hint.node_kind
        );
        // hint.text is a String, so always valid UTF-8 — if char_indices slicing
        // had produced a non-boundary the parse call above would have panicked.
        assert!(
            hint.text.is_char_boundary(hint.text.len()),
            "hint text must end at a valid char boundary"
        );
    }
}

#[test]
fn collect_hints_try_expression_captured() {
    let record = parse(
        "fn f() -> Result<u8, ()> { let _x = some_fn()?; Ok(0) }\n",
    );
    let has_try = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "try_expression");
    assert!(has_try, "try_expression (?) must appear in pattern_hints");
}
