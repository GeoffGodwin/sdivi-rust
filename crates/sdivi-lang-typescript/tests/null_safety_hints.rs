//! Tests for null-safety pattern hints in the TypeScript language adapter (M37).

use sdivi_lang_typescript::TypeScriptAdapter;
use sdivi_parsing::adapter::LanguageAdapter;
use std::path::Path;

fn parse_ts(source: &str) -> sdivi_parsing::feature_record::FeatureRecord {
    TypeScriptAdapter.parse_file(Path::new("test.ts"), source.to_string())
}

#[test]
fn optional_chain_captured_as_pattern_hint() {
    let record = parse_ts("const x = user?.name;\n");
    let has_opt = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "optional_chain");
    assert!(
        has_opt,
        "optional_chain must appear in pattern_hints for user?.name, got hints: {:?}",
        record
            .pattern_hints
            .iter()
            .map(|h| h.node_kind.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn non_null_expression_captured_as_pattern_hint() {
    let record = parse_ts("const x = el!;\n");
    let has_nne = record
        .pattern_hints
        .iter()
        .any(|h| h.node_kind == "non_null_expression");
    assert!(
        has_nne,
        "non_null_expression must appear in pattern_hints for el!, got hints: {:?}",
        record
            .pattern_hints
            .iter()
            .map(|h| h.node_kind.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn ts_fixture_with_optional_chain_and_non_null_yields_two_null_safety_instances() {
    // Acceptance criterion: a?.b + c! → two null_safety-relevant hints.
    let source = "const a = user?.name;\nconst b = el!;\n";
    let record = parse_ts(source);
    let opt_count = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "optional_chain")
        .count();
    let nne_count = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "non_null_expression")
        .count();
    assert!(
        opt_count >= 1,
        "must have at least one optional_chain hint, got: {opt_count}"
    );
    assert!(
        nne_count >= 1,
        "must have at least one non_null_expression hint, got: {nne_count}"
    );
}

#[test]
fn optional_chain_member_access_variants_captured() {
    // a?.b and arr?.[0] produce optional_chain nodes (member_expression / subscript_expression child).
    // fn?.() does NOT — optional calls are call_expression in the grammar, not optional_chain.
    let record = parse_ts("const a = obj?.field;\nconst b = arr?.[0];\n");
    let opt_count = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "optional_chain")
        .count();
    assert!(
        opt_count >= 2,
        "obj?.field and arr?.[0] must each produce an optional_chain hint, got: {opt_count}"
    );
}

#[test]
fn chained_optional_chain_produces_multiple_nodes() {
    // Documents the per-node counting semantics stated in null_safety.rs module doc
    // and MIGRATION_NOTES.md: each `optional_chain` node in a chain like `a?.b?.c`
    // counts independently. This pins the documented behavior so future grammar
    // bumps that change node emission are caught immediately.
    let record = parse_ts("const x = a?.b?.c;\n");
    let opt_count = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "optional_chain")
        .count();
    assert!(
        opt_count >= 2,
        "a?.b?.c must produce at least 2 optional_chain hints (one per chain link); got: {opt_count}"
    );
}

#[test]
fn file_with_no_optional_chain_produces_no_null_safety_hints() {
    let record = parse_ts("const x = user.name;\n");
    let opt_count = record
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "optional_chain" || h.node_kind == "non_null_expression")
        .count();
    assert_eq!(
        opt_count, 0,
        "a file with plain member access must produce zero null_safety hints"
    );
}
