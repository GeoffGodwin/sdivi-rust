//! Verify ALL_CATEGORIES doc correctly lists categories by classification path.
//!
//! The doc claims:
//! - Callee-text only: logging, testing, serialization, schema_validation, state_store, framework_hooks, http_routing, collection_pipelines
//! - Node-kind only: class_hierarchy, comprehensions, decorators, error_handling, null_safety, resource_management, state_management, type_assertions
//! - Hybrid: async_patterns, data_access, concurrency
//!
//! This test verifies that actual implementation matches doc claims.

use sdivi_patterns::queries::{
    self, async_patterns, collection_pipelines, concurrency, data_access, framework_hooks,
    http_routing, schema_validation, serialization, state_store, testing, ALL_CATEGORIES,
};

#[test]
fn all_categories_list_has_19_items() {
    assert_eq!(
        ALL_CATEGORIES.len(),
        19,
        "ALL_CATEGORIES must have exactly 19 items"
    );
}

#[test]
fn all_19_categories_are_in_constant() {
    let expected = vec![
        "async_patterns",
        "class_hierarchy",
        "collection_pipelines",
        "comprehensions",
        "concurrency",
        "data_access",
        "decorators",
        "error_handling",
        "framework_hooks",
        "http_routing",
        "logging",
        "null_safety",
        "resource_management",
        "schema_validation",
        "serialization",
        "state_management",
        "state_store",
        "testing",
        "type_assertions",
    ];
    for cat in expected {
        assert!(
            ALL_CATEGORIES.contains(&cat),
            "category '{}' must be in ALL_CATEGORIES",
            cat
        );
    }
}

#[test]
fn callee_only_categories_listed_in_doc_match_real_dispatch() {
    // Doc claims these 8 categories are callee-text only — category_for_node_kind
    // never returns them. Verify by asserting that a representative node kind
    // (call_expression in typescript) maps to data_access (a hybrid), not any
    // of the callee-only names.
    let result = queries::category_for_node_kind("call_expression", "typescript");
    assert_eq!(
        result,
        Some("data_access"),
        "call_expression should map to the hybrid 'data_access', not a callee-only category"
    );
    let callee_only = [
        "logging",
        "testing",
        "serialization",
        "schema_validation",
        "state_store",
        "framework_hooks",
        "http_routing",
        "collection_pipelines",
    ];
    for cat in callee_only {
        assert_ne!(
            result,
            Some(cat),
            "callee-only category '{cat}' must not be returned by category_for_node_kind"
        );
    }
}

#[test]
fn data_access_is_hybrid_both_node_kind_and_callee() {
    // data_access must have node-kind paths
    assert_eq!(
        queries::category_for_node_kind("call_expression", "typescript"),
        Some("data_access"),
        "data_access must be returned by category_for_node_kind for call_expression"
    );

    // data_access must have callee-text paths
    assert!(
        data_access::matches_callee("fetch(\"/api\")", "typescript"),
        "data_access must match callee text for fetch"
    );
}

#[test]
fn concurrency_is_hybrid_both_node_kind_and_callee() {
    // concurrency must have node-kind paths
    assert_eq!(
        queries::category_for_node_kind("go_statement", "go"),
        Some("concurrency"),
        "concurrency must be returned by category_for_node_kind for go_statement"
    );

    // concurrency must have callee-text paths
    assert!(
        concurrency::matches_callee("Promise.all([a, b])", "typescript"),
        "concurrency must match callee text for Promise.all"
    );
}

#[test]
fn async_patterns_is_hybrid_both_node_kind_and_callee() {
    // async_patterns must have a node-kind path (await_expression)
    assert_eq!(
        queries::category_for_node_kind("await_expression", "typescript"),
        Some("async_patterns"),
        "async_patterns must be returned by category_for_node_kind for await_expression"
    );

    // async_patterns must have a callee-text path (Promise chains)
    assert!(
        async_patterns::matches_callee("promise.then(resolve)", "typescript"),
        "async_patterns must match callee text for Promise chain .then()"
    );
    assert!(
        async_patterns::matches_callee("fetch(url).catch(e => {})", "javascript"),
        "async_patterns must match callee text for Promise chain .catch()"
    );
}

#[test]
fn node_kind_only_categories_have_dispatch_entries() {
    // These are detected purely via category_for_node_kind; no matches_callee path.
    let node_kind_only = vec![
        ("class_declaration", "class_hierarchy"),
        ("generator_expression", "comprehensions"),
        ("decorator", "decorators"),
        ("try_expression", "error_handling"),
        ("optional_chain", "null_safety"),
        ("macro_invocation", "resource_management"),
        ("closure_expression", "state_management"),
        ("as_expression", "type_assertions"),
    ];

    for (node_kind, expected_cat) in node_kind_only {
        assert_eq!(
            queries::category_for_node_kind(node_kind, "typescript"),
            Some(expected_cat),
            "node-kind-only category {} must return '{}' for node kind '{}'",
            expected_cat,
            expected_cat,
            node_kind
        );
    }
}

#[test]
fn callee_only_categories_have_empty_node_kinds() {
    // These 7 callee-only categories have empty NODE_KINDS — they are classified
    // exclusively via classify_hint callee-text inspection:
    assert!(collection_pipelines::NODE_KINDS.is_empty());
    assert!(framework_hooks::NODE_KINDS.is_empty());
    assert!(http_routing::NODE_KINDS.is_empty());
    assert!(schema_validation::NODE_KINDS.is_empty());
    assert!(serialization::NODE_KINDS.is_empty());
    assert!(state_store::NODE_KINDS.is_empty());
    assert!(testing::NODE_KINDS.is_empty());
    // Note: logging::NODE_KINDS is intentionally non-empty — it lists the node
    // kinds the module *inspects* (call_expression, call, macro_invocation) but
    // those overlap with data_access and resource_management, so logging is NOT
    // wired into category_for_node_kind. Classification is still callee-text-only
    // via classify_hint; callers must apply callee-text filtering before emitting
    // PatternInstanceInput values with category = "logging".
}
