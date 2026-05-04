//! Dedicated integration test target for the CI cross-platform hash
//! determinism check.
//!
//! The WASM workflow runs `wasm-pack test --node -- --test normalize_hash_deterministic`
//! and greps the test output for a `CI_HASH:<hex>` line, then compares the
//! captured hash across runners (Linux vs macOS) to assert
//! `normalize_and_hash` is platform-stable.
//!
//! The test stays in its own file so `--test normalize_hash_deterministic`
//! resolves to a dedicated target (the dispatch is by integration-test file
//! name, not test function name).

use sdivi_wasm::normalize_and_hash;
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn normalize_hash_deterministic() {
    let hash = normalize_and_hash("try_expression", vec![]).unwrap();
    println!("CI_HASH:{}", hash);
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}
