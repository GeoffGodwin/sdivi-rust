//! Callee-text classification for runtime schema / validation declarations.
//!
//! Detects schema-library call sites in TypeScript/JavaScript (Zod, Yup,
//! Valibot, io-ts, Superstruct) and Pydantic field functions in Python.
//! Detection is callee-text only — no tree-sitter node-kind matching.
//! `call_expression`/`call` nodes are already collected by the TS/JS/Python
//! adapters; this module provides callee-text discrimination in `CALL_DISPATCH`
//! at slot P4.
//!
//! ## Language support
//!
//! - **TypeScript / JavaScript:** Zod (`z.object`, `z.string`, `z.enum`),
//!   Yup (`yup.object()`, `yup.string()`), Valibot (`v.object`, `v.pipe`),
//!   Superstruct (`s.object`). Detected via the namespace prefix regex
//!   `^(z|yup|v|s)\.\w`. Additionally, `.safeParse(` is matched as a
//!   Zod-specific validated-parse call.
//!
//! - **Python:** Pydantic `Field(...)`, `constr(...)`, `conint(...)` calls.
//!   Note: `class Foo(BaseModel)` is a `class_definition` already counted
//!   under `class_hierarchy`; only the *call* forms are captured here.
//!
//! ## Precision over recall
//!
//! The TS/JS regex is anchored to the schema library *namespace* (`z.`/`yup.`/`v.`/`s.`)
//! rather than method names alone. This deliberately avoids bare `.string()`/`.object()`
//! calls on arbitrary receivers — do not (re)introduce `\.(object|string|array|…)\(`,
//! which floods the bucket. The trade-off: `SomeSchema.parse(x)` where the receiver
//! name is arbitrary is not captured — receiver-type info SDIVI does not compute.
//! Document this known recall gap.
//!
//! ## Pydantic class coverage
//!
//! `class Foo(BaseModel)` is a `class_definition` node, already counted under
//! `class_hierarchy` (M6). Python coverage here is intentionally partial:
//! only call forms (`Field(...)`, `constr(...)`, `conint(...)`) are classified.
//! class-validator decorators (`@IsString()`) belong to `decorators` (M36.1/M36.2);
//! see `docs/pattern-categories.md` for the intentional split.

use std::sync::LazyLock;

use regex::Regex;

/// Tree-sitter node kinds for schema-validation patterns.
///
/// Empty — this category is detected entirely via callee-text inspection in
/// [`matches_callee`]. The `call_expression` and `call` node kinds are already
/// collected by the TypeScript/JavaScript/Python adapters; classification happens
/// in `classify_hint`'s `CALL_DISPATCH` loop at slot P4.
pub const NODE_KINDS: &[&str] = &[];

// TypeScript / JavaScript:
//   ^(z|yup|v|s)\.\w  — namespace-anchored: Zod (z.), Yup (yup.), Valibot (v.),
//                        Superstruct (s.) followed by any word character.
//   \.safeParse\(     — Zod-specific validated-parse call; receiver is arbitrary
//                        but `.safeParse(` uniquely identifies schema parsing.
static TS_JS_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(z|yup|v|s)\.\w|\.safeParse\(").expect("schema_validation TS/JS regex is valid")
});

// Python:
//   \bField\(   — Pydantic Field() constructor
//   \bconstr\(  — Pydantic string constraint helper
//   \bconint\(  — Pydantic integer constraint helper
static PYTHON_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bField\(|\bconstr\(|\bconint\(").expect("schema_validation Python regex is valid")
});

/// Return `true` when `text` looks like a schema-validation callee for `language`.
///
/// TypeScript and JavaScript share one regex table (namespace-anchored library
/// prefixes and `.safeParse(`); Python detects Pydantic field-constraint calls.
/// Rust, Go, and Java always return `false` in v0 — schema-library detection
/// for those languages is deferred.
///
/// # Examples
///
/// ```rust
/// use sdivi_patterns::queries::schema_validation::matches_callee;
///
/// assert!(matches_callee("z.object({})", "typescript"));
/// assert!(matches_callee("yup.string().required()", "javascript"));
/// assert!(matches_callee("UserSchema.safeParse(input)", "typescript"));
/// assert!(matches_callee("Field(default=0)", "python"));
/// assert!(!matches_callee("Math.max(a, b)", "typescript"));
/// assert!(!matches_callee("len(x)", "python"));
/// ```
pub fn matches_callee(text: &str, language: &str) -> bool {
    match language {
        "typescript" | "javascript" => TS_JS_RE.is_match(text),
        "python" => PYTHON_RE.is_match(text),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zod_namespace_matches_typescript() {
        for callee in [
            "z.object({})",
            "z.string()",
            "z.enum(['a','b'])",
            "z.union([])",
        ] {
            assert!(
                matches_callee(callee, "typescript"),
                "{callee:?} should match for typescript"
            );
        }
    }

    #[test]
    fn yup_namespace_matches() {
        assert!(matches_callee("yup.object().shape({})", "typescript"));
        assert!(matches_callee("yup.string().required()", "javascript"));
    }

    #[test]
    fn valibot_namespace_matches() {
        assert!(matches_callee("v.object({})", "typescript"));
        assert!(matches_callee(
            "v.pipe(v.string(), v.minLength(1))",
            "javascript"
        ));
    }

    #[test]
    fn superstruct_namespace_matches() {
        assert!(matches_callee("s.object({})", "typescript"));
    }

    #[test]
    fn safe_parse_matches() {
        assert!(matches_callee("UserSchema.safeParse(input)", "typescript"));
        assert!(matches_callee("schema.safeParse(data)", "javascript"));
    }

    #[test]
    fn pydantic_field_matches_python() {
        assert!(matches_callee("Field(default=0)", "python"));
        assert!(matches_callee("Field(...)", "python"));
        assert!(matches_callee("constr(min_length=1)", "python"));
        assert!(matches_callee("conint(gt=0)", "python"));
    }

    #[test]
    fn bare_method_call_does_not_match() {
        assert!(!matches_callee(".string()", "typescript"));
        assert!(!matches_callee(".object()", "typescript"));
        assert!(!matches_callee("parse(x)", "typescript"));
        assert!(!matches_callee("Math.max(a, b)", "typescript"));
    }

    #[test]
    fn generic_python_calls_do_not_match() {
        assert!(!matches_callee("len(x)", "python"));
        assert!(!matches_callee("open(path)", "python"));
    }

    #[test]
    fn other_languages_return_false() {
        for lang in ["rust", "go", "java"] {
            assert!(
                !matches_callee("z.object({})", lang),
                "z.object should not match for {lang}"
            );
        }
    }

    #[test]
    fn node_kinds_is_empty() {
        assert!(NODE_KINDS.is_empty());
    }
}
