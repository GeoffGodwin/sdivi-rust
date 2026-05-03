## Planned Tests
- [x] `bindings/sdivi-wasm/tests/m22_native.rs` — Native serde round-trip tests for WasmCoChangePairInput and WasmChangeCouplingInput (run without wasm-pack)

## Test Run Results
Passed: 6  Failed: 0

## Bugs Found
- BUG: [bindings/sdivi-wasm/src/weight_keys.rs:25] parse_wasm_edge_weights accepts f64::INFINITY when doc contract says "weights must be finite"; is_infinite() check is missing alongside the existing is_nan() check

## Files Modified
- [x] `bindings/sdivi-wasm/tests/m22_native.rs`
