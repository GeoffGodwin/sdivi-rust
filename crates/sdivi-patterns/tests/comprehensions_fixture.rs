//! M46 integration tests: Python comprehension detection via real tree-sitter parsing.
//!
//! Verifies the full path: Python source → `PythonAdapter.parse_file` (tree-sitter) →
//! `build_catalog` → `comprehensions` bucket. This covers the tree-sitter parse path
//! that the synthetic-FeatureRecord tests in `category_contract_m46.rs` do not exercise.
//!
//! Confirms the `generator_expression` node-kind spelling used by the pinned
//! tree-sitter-python grammar matches what the adapter and classifier expect.

use sdivi_config::PatternsConfig;
use sdivi_lang_python::PythonAdapter;
use sdivi_parsing::adapter::LanguageAdapter;
use sdivi_patterns::build_catalog;
use std::path::Path;

fn parse_python(source: &str) -> sdivi_parsing::feature_record::FeatureRecord {
    PythonAdapter.parse_file(Path::new("test.py"), source.to_string())
}

fn min1_config() -> PatternsConfig {
    PatternsConfig {
        min_pattern_nodes: 1,
        ..PatternsConfig::default()
    }
}

// ── single-form tests: each comprehension kind in isolation ───────────────────

/// M46 integration: `list_comprehension` parsed from real Python source routes to
/// the `comprehensions` catalog bucket.
#[test]
fn python_list_comprehension_routes_to_comprehensions() {
    let source = "xs = [1, 2, 3]\nsquares = [x * x for x in xs]\n";
    let records = vec![parse_python(source)];

    // Verify the adapter collected the hint from real tree-sitter output.
    let list_comp_count = records[0]
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "list_comprehension")
        .count();
    assert_eq!(
        list_comp_count,
        1,
        "PythonAdapter must emit exactly 1 list_comprehension hint from tree-sitter parse; \
         got {list_comp_count}. Hints: {:?}",
        records[0]
            .pattern_hints
            .iter()
            .map(|h| h.node_kind.as_str())
            .collect::<Vec<_>>()
    );

    let catalog = build_catalog(&records, &min1_config());
    assert!(
        catalog.entries.contains_key("comprehensions"),
        "build_catalog must produce a `comprehensions` bucket for list_comprehension (M46). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    let total: u32 = catalog.entries["comprehensions"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 1,
        "comprehensions bucket must contain exactly 1 list_comprehension instance; got {total}"
    );
}

/// M46 integration: `set_comprehension` parsed from real Python source routes to
/// the `comprehensions` catalog bucket.
#[test]
fn python_set_comprehension_routes_to_comprehensions() {
    let source = "xs = [1, 2, 3]\nunique = {x for x in xs}\n";
    let records = vec![parse_python(source)];

    let set_comp_count = records[0]
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "set_comprehension")
        .count();
    assert_eq!(
        set_comp_count,
        1,
        "PythonAdapter must emit exactly 1 set_comprehension hint; \
         got {set_comp_count}. Hints: {:?}",
        records[0]
            .pattern_hints
            .iter()
            .map(|h| h.node_kind.as_str())
            .collect::<Vec<_>>()
    );

    let catalog = build_catalog(&records, &min1_config());
    assert!(
        catalog.entries.contains_key("comprehensions"),
        "build_catalog must produce a `comprehensions` bucket for set_comprehension (M46). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    let total: u32 = catalog.entries["comprehensions"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 1,
        "comprehensions bucket must contain exactly 1 set_comprehension instance; got {total}"
    );
}

/// M46 integration: `dictionary_comprehension` parsed from real Python source routes to
/// the `comprehensions` catalog bucket.
#[test]
fn python_dictionary_comprehension_routes_to_comprehensions() {
    let source = "items = [(\"a\", 1), (\"b\", 2)]\nmapping = {k: v for k, v in items}\n";
    let records = vec![parse_python(source)];

    let dict_comp_count = records[0]
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "dictionary_comprehension")
        .count();
    assert_eq!(
        dict_comp_count,
        1,
        "PythonAdapter must emit exactly 1 dictionary_comprehension hint; \
         got {dict_comp_count}. Hints: {:?}",
        records[0]
            .pattern_hints
            .iter()
            .map(|h| h.node_kind.as_str())
            .collect::<Vec<_>>()
    );

    let catalog = build_catalog(&records, &min1_config());
    assert!(
        catalog.entries.contains_key("comprehensions"),
        "build_catalog must produce a `comprehensions` bucket for dictionary_comprehension (M46). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    let total: u32 = catalog.entries["comprehensions"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 1,
        "comprehensions bucket must contain exactly 1 dictionary_comprehension instance; got {total}"
    );
}

/// M46 integration: `generator_expression` parsed from real Python source routes to
/// the `comprehensions` catalog bucket.
///
/// Confirms the node-kind string "generator_expression" matches the pinned
/// tree-sitter-python grammar (Watch For item in the M46 milestone spec).
#[test]
fn python_generator_expression_routes_to_comprehensions() {
    // sum(x for x in xs) — the argument "x for x in xs" is a generator_expression node.
    let source = "xs = [1, 2, 3]\ntotal = sum(x for x in xs)\n";
    let records = vec![parse_python(source)];

    let gen_expr_count = records[0]
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "generator_expression")
        .count();
    assert_eq!(
        gen_expr_count,
        1,
        "PythonAdapter must emit exactly 1 generator_expression hint from `sum(x for x in xs)`; \
         got {gen_expr_count}. \
         If 0: the tree-sitter-python grammar may use a different node-kind spelling. \
         Actual hints: {:?}",
        records[0]
            .pattern_hints
            .iter()
            .map(|h| h.node_kind.as_str())
            .collect::<Vec<_>>()
    );

    let catalog = build_catalog(&records, &min1_config());
    assert!(
        catalog.entries.contains_key("comprehensions"),
        "build_catalog must produce a `comprehensions` bucket for generator_expression (M46). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    let total: u32 = catalog.entries["comprehensions"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 1,
        "comprehensions bucket must contain exactly 1 generator_expression instance; got {total}"
    );
}

// ── combined fixture: all four forms in one file ──────────────────────────────

/// M46 acceptance criterion (integration): a Python file with all four comprehension
/// forms yields exactly four instances in the `comprehensions` catalog bucket.
///
/// Each comprehension kind (list, set, dict, generator) contributes one instance.
#[test]
fn all_four_comprehension_forms_yield_four_instances() {
    let source = concat!(
        "xs = [1, 2, 3]\n",
        "items = [(\"a\", 1), (\"b\", 2)]\n",
        "squares = [x * x for x in xs]\n", // list_comprehension
        "unique = {x for x in xs}\n",      // set_comprehension
        "mapping = {k: v for k, v in items}\n", // dictionary_comprehension
        "total = sum(x for x in xs)\n",    // generator_expression
    );
    let records = vec![parse_python(source)];

    // Verify the adapter collected all four hints from tree-sitter.
    let hint_kinds: Vec<&str> = records[0]
        .pattern_hints
        .iter()
        .map(|h| h.node_kind.as_str())
        .collect();

    for expected_kind in &[
        "list_comprehension",
        "set_comprehension",
        "dictionary_comprehension",
        "generator_expression",
    ] {
        assert!(
            hint_kinds.contains(expected_kind),
            "PythonAdapter must emit a `{expected_kind}` hint from tree-sitter; \
             actual hint kinds: {hint_kinds:?}"
        );
    }

    let catalog = build_catalog(&records, &min1_config());
    assert!(
        catalog.entries.contains_key("comprehensions"),
        "build_catalog must produce a `comprehensions` bucket for a file with all four \
         comprehension forms (M46 acceptance criterion). \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
    let total: u32 = catalog.entries["comprehensions"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 4,
        "one list + one set + one dict + one generator = 4 comprehensions instances; got {total}"
    );
}

// ── nested comprehensions: count semantics ────────────────────────────────────

/// M46 count semantics: nested comprehensions emit one instance per comprehension node.
///
/// `[[x for x in row] for row in matrix]` has an inner `list_comprehension` nested
/// inside an outer `list_comprehension`, producing 2 instances total.
/// This is intentional — more nesting = more comprehension structure = higher entropy.
#[test]
fn nested_list_comprehension_yields_two_instances() {
    let source = "matrix = [[1, 2], [3, 4]]\nflat = [[x for x in row] for row in matrix]\n";
    let records = vec![parse_python(source)];

    let list_comp_count = records[0]
        .pattern_hints
        .iter()
        .filter(|h| h.node_kind == "list_comprehension")
        .count();
    assert_eq!(
        list_comp_count, 2,
        "nested `[[x for x in row] for row in matrix]` must emit 2 list_comprehension hints \
         (one inner, one outer); got {list_comp_count}"
    );

    let catalog = build_catalog(&records, &min1_config());
    let total: u32 = catalog.entries["comprehensions"]
        .values()
        .map(|s| s.count)
        .sum();
    assert_eq!(
        total, 2,
        "nested comprehensions must produce 2 instances in the comprehensions bucket; got {total}"
    );
}

// ── language exclusivity: Python-only ────────────────────────────────────────

/// Comprehensions are Python-only: a file with no comprehension node kinds produces
/// no `comprehensions` catalog bucket (empty Python file).
#[test]
fn python_file_without_comprehensions_produces_no_comprehensions_bucket() {
    let source = "x = 1\ny = x + 2\n";
    let records = vec![parse_python(source)];
    let catalog = build_catalog(&records, &min1_config());
    assert!(
        !catalog.entries.contains_key("comprehensions"),
        "a Python file with no comprehension forms must produce no `comprehensions` bucket. \
         Present categories: {:?}",
        catalog.entries.keys().collect::<Vec<_>>()
    );
}
