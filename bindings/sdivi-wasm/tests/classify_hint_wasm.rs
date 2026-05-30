//! WASM integration tests for `classify_hint` — run via `wasm-pack test --node`.

use sdivi_wasm::classify_hint;
use sdivi_wasm::types::WasmPatternHintInput;
use wasm_bindgen_test::wasm_bindgen_test;

fn hint(node_kind: &str, text: &str) -> WasmPatternHintInput {
    WasmPatternHintInput {
        node_kind: node_kind.to_string(),
        text: text.to_string(),
    }
}

/// JS-side signature test: `classify_hint` accepts a `WasmPatternHintInput` and a
/// language string, returns `Vec<String>`.
#[wasm_bindgen_test]
fn classify_hint_console_log_returns_logging() {
    let result = classify_hint(hint("call_expression", "console.log(\"x\")"), "typescript");
    assert_eq!(result, vec!["logging".to_string()]);
}

#[wasm_bindgen_test]
fn classify_hint_tracing_macro_returns_logging() {
    let result = classify_hint(hint("macro_invocation", "tracing::info!(\"hi\")"), "rust");
    assert_eq!(result, vec!["logging".to_string()]);
}

#[wasm_bindgen_test]
fn classify_hint_vec_macro_returns_resource_management() {
    let result = classify_hint(hint("macro_invocation", "vec![1, 2, 3]"), "rust");
    assert_eq!(result, vec!["resource_management".to_string()]);
}

#[wasm_bindgen_test]
fn classify_hint_unrecognised_call_returns_empty() {
    let result = classify_hint(hint("call_expression", "Math.max(a, b)"), "typescript");
    assert!(result.is_empty());
}

#[wasm_bindgen_test]
fn classify_hint_fetch_returns_data_access() {
    let result = classify_hint(hint("call_expression", "fetch(\"/api\")"), "typescript");
    assert_eq!(result, vec!["data_access".to_string()]);
}

#[wasm_bindgen_test]
fn classify_hint_promise_then_returns_async_patterns() {
    let result = classify_hint(hint("call_expression", "p.then(resolve)"), "javascript");
    assert_eq!(result, vec!["async_patterns".to_string()]);
}

/// Cross-platform determinism: native and WASM must return identical results
/// for the same PatternHintInput + language combinations.
#[wasm_bindgen_test]
fn classify_hint_wasm_matches_native_results() {
    let cases: &[(&str, &str, &str, &[&str])] = &[
        (
            "call_expression",
            "console.log(\"x\")",
            "typescript",
            &["logging"],
        ),
        (
            "call_expression",
            "fetch(\"/api\")",
            "typescript",
            &["data_access"],
        ),
        (
            "call_expression",
            "p.then(r)",
            "javascript",
            &["async_patterns"],
        ),
        ("call_expression", "Math.max(a, b)", "typescript", &[]),
        (
            "macro_invocation",
            "tracing::info!(\"x\")",
            "rust",
            &["logging"],
        ),
        (
            "macro_invocation",
            "vec![1]",
            "rust",
            &["resource_management"],
        ),
        ("try_expression", "foo()?", "rust", &["error_handling"]),
    ];
    for (node_kind, text, lang, expected) in cases {
        let result = classify_hint(hint(node_kind, text), lang);
        let expected_owned: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            result, expected_owned,
            "WASM/native mismatch for ({node_kind}, {text}, {lang})"
        );
    }
}
