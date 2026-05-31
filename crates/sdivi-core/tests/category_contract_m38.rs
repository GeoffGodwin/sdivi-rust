//! M38 acceptance-criteria tests for `schema_validation`.
//!
//! Kept separate from `category_contract.rs` to avoid exceeding the 300-line
//! ceiling in that file. Uses the same public API surface.

use sdivi_patterns::queries::classify_hint;
use sdivi_patterns::PatternHintInput;

fn call_hint(text: &str) -> PatternHintInput {
    PatternHintInput {
        node_kind: "call_expression".to_string(),
        text: text.to_string(),
    }
}

// ── Category catalog ──────────────────────────────────────────────────────────

#[test]
fn list_categories_includes_schema_validation() {
    let catalog = sdivi_core::list_categories();
    let names: Vec<&str> = catalog.categories.iter().map(|c| c.name.as_str()).collect();
    assert!(
        names.contains(&"schema_validation"),
        "list_categories must include 'schema_validation'"
    );
}

// ── M38 acceptance criteria ───────────────────────────────────────────────────

#[test]
fn zod_object_typescript_is_schema_validation() {
    assert_eq!(
        classify_hint(&call_hint("z.object({})"), "typescript"),
        vec!["schema_validation"],
        "z.object({{...}}) must map to schema_validation (M38 acceptance criterion)"
    );
}

#[test]
fn math_max_returns_empty_not_schema_validation() {
    let result = classify_hint(&call_hint("Math.max(a, b)"), "typescript");
    assert!(
        result.is_empty(),
        "Math.max(a,b) → [] (M38 acceptance criterion): got {result:?}"
    );
}

// ── Zod / Yup / Valibot / Superstruct positives ───────────────────────────────

#[test]
fn zod_string_is_schema_validation() {
    assert_eq!(
        classify_hint(&call_hint("z.string()"), "typescript"),
        vec!["schema_validation"]
    );
}

#[test]
fn yup_object_shape_is_schema_validation() {
    assert_eq!(
        classify_hint(&call_hint("yup.object().shape({})"), "javascript"),
        vec!["schema_validation"]
    );
}

#[test]
fn valibot_object_is_schema_validation() {
    assert_eq!(
        classify_hint(&call_hint("v.object({})"), "typescript"),
        vec!["schema_validation"]
    );
}

#[test]
fn superstruct_object_is_schema_validation() {
    assert_eq!(
        classify_hint(&call_hint("s.object({})"), "typescript"),
        vec!["schema_validation"]
    );
}

#[test]
fn safe_parse_is_schema_validation() {
    assert_eq!(
        classify_hint(&call_hint("UserSchema.safeParse(input)"), "typescript"),
        vec!["schema_validation"]
    );
}

// ── Python (Pydantic) positives ───────────────────────────────────────────────

#[test]
fn pydantic_field_is_schema_validation() {
    let hint = PatternHintInput {
        node_kind: "call".to_string(),
        text: "Field(default=0)".to_string(),
    };
    assert_eq!(classify_hint(&hint, "python"), vec!["schema_validation"]);
}

#[test]
fn pydantic_constr_is_schema_validation() {
    let hint = PatternHintInput {
        node_kind: "call".to_string(),
        text: "constr(min_length=1)".to_string(),
    };
    assert_eq!(classify_hint(&hint, "python"), vec!["schema_validation"]);
}

#[test]
fn pydantic_conint_is_schema_validation() {
    let hint = PatternHintInput {
        node_kind: "call".to_string(),
        text: "conint(gt=0)".to_string(),
    };
    assert_eq!(classify_hint(&hint, "python"), vec!["schema_validation"]);
}

// ── Negatives ─────────────────────────────────────────────────────────────────

#[test]
fn bare_string_method_does_not_match_schema_validation() {
    let result = classify_hint(&call_hint(".string()"), "typescript");
    assert!(
        !result.contains(&"schema_validation"),
        ".string() on arbitrary receiver must not match schema_validation"
    );
}

#[test]
fn other_languages_do_not_match_schema_validation() {
    for lang in ["rust", "go", "java"] {
        let result = classify_hint(&call_hint("z.object({})"), lang);
        assert!(
            result.is_empty(),
            "z.object() should not match for {lang}: got {result:?}"
        );
    }
}

// ── Disjointness: no overlap with data_access ─────────────────────────────────

#[test]
fn schema_validation_and_data_access_are_disjoint_for_parse() {
    // `parse` is in the schema-validation namespace (via .safeParse), but NOT
    // in the data_access regex — verify they don't overlap on this example.
    let result = classify_hint(&call_hint("UserSchema.safeParse(input)"), "typescript");
    assert_eq!(
        result,
        vec!["schema_validation"],
        ".safeParse( must resolve to schema_validation, not data_access"
    );
}
