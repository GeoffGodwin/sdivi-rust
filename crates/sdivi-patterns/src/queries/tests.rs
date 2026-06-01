use crate::hint_input::PatternHintInput;

use super::*;

#[test]
fn try_expression_is_error_handling() {
    assert_eq!(
        category_for_node_kind("try_expression", "rust"),
        Some("error_handling")
    );
}

#[test]
fn await_expression_is_async_patterns() {
    assert_eq!(
        category_for_node_kind("await_expression", "rust"),
        Some("async_patterns")
    );
}

#[test]
fn closure_expression_is_state_management() {
    assert_eq!(
        category_for_node_kind("closure_expression", "rust"),
        Some("state_management")
    );
}

#[test]
fn macro_invocation_is_resource_management() {
    assert_eq!(
        category_for_node_kind("macro_invocation", "rust"),
        Some("resource_management")
    );
}

#[test]
fn unknown_node_kind_returns_none() {
    assert_eq!(category_for_node_kind("unknown_xyz", "rust"), None);
}

#[test]
fn all_categories_has_nineteen_entries() {
    assert_eq!(ALL_CATEGORIES.len(), 19);
    assert!(ALL_CATEGORIES.contains(&"comprehensions"));
    assert!(ALL_CATEGORIES.contains(&"concurrency"));
    assert!(ALL_CATEGORIES.contains(&"collection_pipelines"));
    assert!(ALL_CATEGORIES.contains(&"framework_hooks"));
    assert!(ALL_CATEGORIES.contains(&"http_routing"));
    assert!(ALL_CATEGORIES.contains(&"decorators"));
    assert!(ALL_CATEGORIES.contains(&"testing"));
    assert!(ALL_CATEGORIES.contains(&"schema_validation"));
    assert!(ALL_CATEGORIES.contains(&"serialization"));
}

#[test]
fn logging_is_in_all_categories() {
    assert!(ALL_CATEGORIES.contains(&"logging"));
}

#[test]
fn class_hierarchy_is_in_all_categories() {
    assert!(ALL_CATEGORIES.contains(&"class_hierarchy"));
}

#[test]
fn class_declaration_is_class_hierarchy() {
    assert_eq!(
        category_for_node_kind("class_declaration", "typescript"),
        Some("class_hierarchy")
    );
}

#[test]
fn class_definition_is_class_hierarchy() {
    assert_eq!(
        category_for_node_kind("class_definition", "python"),
        Some("class_hierarchy")
    );
}

#[test]
fn impl_item_is_class_hierarchy() {
    assert_eq!(
        category_for_node_kind("impl_item", "rust"),
        Some("class_hierarchy")
    );
}

#[test]
fn interface_declaration_is_class_hierarchy() {
    assert_eq!(
        category_for_node_kind("interface_declaration", "java"),
        Some("class_hierarchy")
    );
}

#[test]
fn abstract_class_declaration_is_class_hierarchy() {
    assert_eq!(
        category_for_node_kind("abstract_class_declaration", "typescript"),
        Some("class_hierarchy")
    );
}

// M30 sentinel: `category_for_node_kind` is node-kind-only; M33 promoted `logging`
// via `classify_hint` without changing this function. See m33_sentinels.rs.
#[test]
fn category_for_node_kind_never_returns_logging() {
    for kind in ["call_expression", "call", "macro_invocation"] {
        for lang in ["rust", "python", "typescript", "javascript", "go", "java"] {
            assert_ne!(
                category_for_node_kind(kind, lang),
                Some("logging"),
                "category_for_node_kind never returns logging; callee-text routing via classify_hint \
                 ({kind}, {lang})"
            );
        }
    }
}

#[test]
fn call_expression_is_data_access() {
    assert_eq!(
        category_for_node_kind("call_expression", "typescript"),
        Some("data_access")
    );
    assert_eq!(
        category_for_node_kind("call", "python"),
        Some("data_access")
    );
}

#[test]
fn decorator_is_decorators() {
    assert_eq!(
        category_for_node_kind("decorator", "typescript"),
        Some("decorators")
    );
    assert_eq!(
        category_for_node_kind("decorator", "javascript"),
        Some("decorators")
    );
}
// ── M41: http_routing ────────────────────────────────────────────────────────

#[test]
fn app_get_is_http_routing_not_data_access() {
    let hint = PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: "app.get('/users', handler)".to_string(),
    };
    assert_eq!(classify_hint(&hint, "typescript"), vec!["http_routing"]);
}

#[test]
fn axios_get_is_data_access_not_http_routing() {
    let hint = PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: "axios.get(url)".to_string(),
    };
    assert_eq!(classify_hint(&hint, "typescript"), vec!["data_access"]);
}

#[test]
fn go_http_handle_func_is_http_routing() {
    let hint = PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: "http.HandleFunc(\"/\", h)".to_string(),
    };
    assert_eq!(classify_hint(&hint, "go"), vec!["http_routing"]);
}

// ── M40: collection_pipelines ────────────────────────────────────────────────

#[test]
fn xs_map_is_collection_pipelines() {
    let hint = PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: "xs.map(f)".to_string(),
    };
    assert_eq!(
        classify_hint(&hint, "typescript"),
        vec!["collection_pipelines"]
    );
}

#[test]
fn db_query_is_data_access_not_collection_pipelines() {
    let hint = PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: "db.query(sql)".to_string(),
    };
    assert_eq!(classify_hint(&hint, "typescript"), vec!["data_access"]);
}

#[test]
fn promise_then_is_async_not_collection_pipelines() {
    let hint = PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: "promise.then(resolve)".to_string(),
    };
    assert_eq!(classify_hint(&hint, "typescript"), vec!["async_patterns"]);
}

// ── M39: state_store ─────────────────────────────────────────────────────────

#[test]
fn use_selector_is_state_store_not_framework_hooks() {
    let hint = PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: "useSelector(s => s.user)".to_string(),
    };
    let result = classify_hint(&hint, "typescript");
    assert_eq!(
        result,
        vec!["state_store"],
        "useSelector must resolve to state_store (P5 beats framework_hooks P6)"
    );
}

#[test]
fn create_slice_is_state_store() {
    let hint = PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: "createSlice({})".to_string(),
    };
    assert_eq!(classify_hint(&hint, "typescript"), vec!["state_store"]);
}

#[test]
fn use_effect_is_still_framework_hooks() {
    let hint = PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: "useEffect(fn, [])".to_string(),
    };
    assert_eq!(classify_hint(&hint, "typescript"), vec!["framework_hooks"]);
}

// ── M38: schema_validation ───────────────────────────────────────────────────

#[test]
fn zod_call_expression_is_schema_validation() {
    let hint = PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: "z.object({})".to_string(),
    };
    assert_eq!(
        classify_hint(&hint, "typescript"),
        vec!["schema_validation"]
    );
}

#[test]
fn math_max_is_not_schema_validation() {
    let hint = PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: "Math.max(a, b)".to_string(),
    };
    let result = classify_hint(&hint, "typescript");
    assert!(
        !result.contains(&"schema_validation"),
        "Math.max must not match schema_validation"
    );
}

// ── M37: null_safety ──────────────────────────────────────────────────────────

#[test]
fn optional_chain_is_null_safety() {
    assert_eq!(
        category_for_node_kind("optional_chain", "typescript"),
        Some("null_safety")
    );
    assert_eq!(
        category_for_node_kind("optional_chain", "javascript"),
        Some("null_safety")
    );
}

#[test]
fn non_null_expression_is_null_safety() {
    assert_eq!(
        category_for_node_kind("non_null_expression", "typescript"),
        Some("null_safety")
    );
}

#[test]
fn category_for_node_kind_is_language_unaware_optional_chain_always_maps_to_null_safety() {
    for lang in ["rust", "python", "go", "java"] {
        assert_eq!(
            category_for_node_kind("optional_chain", lang),
            Some("null_safety"),
            "optional_chain maps to null_safety regardless of language parameter"
        );
    }
}
