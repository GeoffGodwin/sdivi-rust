//! M43 acceptance-criterion tests: `serialization` pattern category.
//!
//! Verifies that `classify_hint` routes (de)serialization boundary calls to
//! `["serialization"]` and that non-serialization calls are not miscategorised.

use sdivi_patterns::queries::classify_hint;
use sdivi_patterns::PatternHintInput;

fn hint(node_kind: &str, text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: node_kind.to_string(),
        text: text.to_string(),
    }
}

// ── Acceptance criteria ───────────────────────────────────────────────────────

#[test]
fn json_parse_s_is_serialization() {
    // Milestone acceptance criterion: `JSON.parse(s)` → `["serialization"]`
    let result = classify_hint(&hint("call_expression", "JSON.parse(s)"), "typescript");
    assert_eq!(result, vec!["serialization"]);
}

#[test]
fn json_dumps_o_is_serialization() {
    // Milestone acceptance criterion: `json.dumps(o)` → `["serialization"]`
    let result = classify_hint(&hint("call", "json.dumps(o)"), "python");
    assert_eq!(result, vec!["serialization"]);
}

// ── TypeScript / JavaScript ───────────────────────────────────────────────────

#[test]
fn json_stringify_is_serialization() {
    let result = classify_hint(
        &hint("call_expression", "JSON.stringify(obj)"),
        "javascript",
    );
    assert_eq!(result, vec!["serialization"]);
}

#[test]
fn structured_clone_is_serialization() {
    let result = classify_hint(
        &hint("call_expression", "structuredClone(data)"),
        "typescript",
    );
    assert_eq!(result, vec!["serialization"]);
}

// ── Python ────────────────────────────────────────────────────────────────────

#[test]
fn json_loads_is_serialization_python() {
    let result = classify_hint(&hint("call", "json.loads(s)"), "python");
    assert_eq!(result, vec!["serialization"]);
}

#[test]
fn pickle_dumps_is_serialization_python() {
    let result = classify_hint(&hint("call", "pickle.dumps(o)"), "python");
    assert_eq!(result, vec!["serialization"]);
}

#[test]
fn pickle_loads_is_serialization_python() {
    let result = classify_hint(&hint("call", "pickle.loads(b)"), "python");
    assert_eq!(result, vec!["serialization"]);
}

// ── Go ────────────────────────────────────────────────────────────────────────

#[test]
fn json_marshal_is_serialization_go() {
    let result = classify_hint(&hint("call_expression", "json.Marshal(v)"), "go");
    assert_eq!(result, vec!["serialization"]);
}

#[test]
fn json_unmarshal_is_serialization_go() {
    let result = classify_hint(&hint("call_expression", "json.Unmarshal(b, &v)"), "go");
    assert_eq!(result, vec!["serialization"]);
}

#[test]
fn json_new_decoder_is_serialization_go() {
    let result = classify_hint(&hint("call_expression", "json.NewDecoder(r)"), "go");
    assert_eq!(result, vec!["serialization"]);
}

// ── Negatives — disjoint from existing categories ────────────────────────────

#[test]
fn schema_parse_is_not_serialization() {
    // Bare `.parse(` on arbitrary receiver must not match — anchor prevents it.
    let result = classify_hint(&hint("call_expression", "schema.parse(x)"), "typescript");
    assert!(
        !result.contains(&"serialization"),
        "schema.parse must not match serialization; got {result:?}"
    );
}

#[test]
fn zod_object_is_schema_validation_not_serialization() {
    let result = classify_hint(&hint("call_expression", "z.object({})"), "typescript");
    assert_eq!(result, vec!["schema_validation"]);
}

#[test]
fn requests_get_is_data_access_not_serialization() {
    let result = classify_hint(&hint("call", "requests.get(url)"), "python");
    assert_eq!(result, vec!["data_access"]);
}

#[test]
fn json_loads_not_data_access_in_python() {
    // json.loads must not match Python data_access regex (^(open\(|requests\.|...)).
    let result = classify_hint(&hint("call", "json.loads(s)"), "python");
    assert!(
        !result.contains(&"data_access"),
        "json.loads must not be data_access; got {result:?}"
    );
}

// ── list_categories includes serialization ────────────────────────────────────

#[test]
fn list_categories_includes_serialization() {
    let catalog = sdivi_core::list_categories();
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"serialization"),
        "list_categories must include 'serialization' (M43)"
    );
}

// ── Remaining Go variants (MarshalIndent and NewEncoder) ─────────────────────

#[test]
fn json_marshal_indent_is_serialization_go() {
    // Reviewer gap: json.MarshalIndent was only in serialization.rs inline tests, not here.
    let result = classify_hint(
        &hint("call_expression", "json.MarshalIndent(v, \"\", \"  \")"),
        "go",
    );
    assert_eq!(result, vec!["serialization"]);
}

#[test]
fn json_new_encoder_is_serialization_go() {
    // Reviewer gap: json.NewEncoder was only in serialization.rs inline tests, not here.
    let result = classify_hint(&hint("call_expression", "json.NewEncoder(w)"), "go");
    assert_eq!(result, vec!["serialization"]);
}

#[test]
fn list_categories_count_after_m43() {
    // M43 added serialization; M46 added comprehensions — total is now 19.
    let catalog = sdivi_core::list_categories();
    assert_eq!(
        catalog.categories.len(),
        19,
        "list_categories must return exactly 19 categories (18 at M43 + comprehensions at M46)"
    );
}
