//! M40 acceptance-criterion tests for the `collection_pipelines` pattern category.
//!
//! Verifies:
//! - `xs.map(f)` → `["collection_pipelines"]`
//! - `db.query(sql)` → `["data_access"]` (not collection_pipelines)
//! - Disjointness with `data_access` and `async_patterns`
//! - Wrong-language negative (Python, Rust)

use sdivi_patterns::queries::classify_hint;
use sdivi_patterns::PatternHintInput;

fn hint(text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: text.to_string(),
    }
}

// ── Acceptance criteria ───────────────────────────────────────────────────────

#[test]
fn xs_map_f_is_collection_pipelines() {
    assert_eq!(
        classify_hint(&hint("xs.map(f)"), "typescript"),
        vec!["collection_pipelines"],
        "xs.map(f) must resolve to collection_pipelines (M40 acceptance criterion)"
    );
}

#[test]
fn db_query_sql_is_data_access() {
    assert_eq!(
        classify_hint(&hint("db.query(sql)"), "typescript"),
        vec!["data_access"],
        "db.query(sql) must resolve to data_access, not collection_pipelines"
    );
}

// ── Each method name positive ─────────────────────────────────────────────────

#[test]
fn filter_is_collection_pipelines() {
    assert_eq!(
        classify_hint(&hint("xs.filter(p)"), "typescript"),
        vec!["collection_pipelines"]
    );
}

#[test]
fn reduce_is_collection_pipelines() {
    assert_eq!(
        classify_hint(&hint("xs.reduce(g, 0)"), "javascript"),
        vec!["collection_pipelines"]
    );
}

#[test]
fn flat_map_is_collection_pipelines() {
    assert_eq!(
        classify_hint(&hint("xs.flatMap(fn)"), "typescript"),
        vec!["collection_pipelines"]
    );
}

#[test]
fn for_each_is_collection_pipelines() {
    assert_eq!(
        classify_hint(&hint("xs.forEach(cb)"), "typescript"),
        vec!["collection_pipelines"]
    );
}

#[test]
fn find_is_collection_pipelines() {
    assert_eq!(
        classify_hint(&hint("xs.find(p)"), "typescript"),
        vec!["collection_pipelines"]
    );
}

#[test]
fn find_index_is_collection_pipelines() {
    assert_eq!(
        classify_hint(&hint("xs.findIndex(p)"), "javascript"),
        vec!["collection_pipelines"]
    );
}

#[test]
fn some_is_collection_pipelines() {
    assert_eq!(
        classify_hint(&hint("xs.some(p)"), "typescript"),
        vec!["collection_pipelines"]
    );
}

#[test]
fn every_is_collection_pipelines() {
    assert_eq!(
        classify_hint(&hint("xs.every(p)"), "typescript"),
        vec!["collection_pipelines"]
    );
}

#[test]
fn flat_is_collection_pipelines() {
    assert_eq!(
        classify_hint(&hint("xs.flat()"), "javascript"),
        vec!["collection_pipelines"]
    );
}

// ── Disjointness with data_access ─────────────────────────────────────────────

#[test]
fn data_access_methods_are_not_collection_pipelines() {
    for text in ["db.query(sql)", "client.read(buf)", "client.fetch(url)"] {
        let result = classify_hint(&hint(text), "typescript");
        assert!(
            !result.contains(&"collection_pipelines"),
            "{text:?} must not be classified as collection_pipelines"
        );
    }
}

// ── Disjointness with async_patterns ─────────────────────────────────────────

#[test]
fn async_pattern_methods_are_not_collection_pipelines() {
    for text in [
        "promise.then(resolve)",
        "p.catch(err => {})",
        "p.finally(() => {})",
    ] {
        let result = classify_hint(&hint(text), "typescript");
        assert!(
            !result.contains(&"collection_pipelines"),
            "{text:?} must not be classified as collection_pipelines"
        );
    }
}

// ── Wrong-language negatives ──────────────────────────────────────────────────

#[test]
fn python_and_rust_return_empty_for_collection_pipeline_text() {
    for lang in ["python", "rust"] {
        let result = classify_hint(&hint("xs.map(f)"), lang);
        assert!(
            result.is_empty(),
            "xs.map(f) must not match for {lang} (collection_pipelines is TS/JS primary)"
        );
    }
}

// ── list_categories includes collection_pipelines ────────────────────────────

#[test]
fn list_categories_includes_collection_pipelines() {
    let catalog = sdivi_core::list_categories();
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"collection_pipelines"),
        "list_categories must include the 'collection_pipelines' category (M40)"
    );
}
