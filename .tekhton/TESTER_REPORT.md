## Planned Tests
- [x] `crates/sdivi-core/tests/prop_thresholds.rs` — add three dimension-specific epsilon property tests (convention_drift, coupling_delta, boundary_violation)
- [x] `bindings/sdivi-wasm/src/weight_keys.rs` — native unit tests: NUL-key content verification, colon-in-node-id key correctness, infinity weight rejection, empty-map boundary, value preservation

## Test Run Results
Passed: 20  Failed: 1

## Bugs Found
- BUG: [bindings/sdivi-wasm/src/weight_keys.rs:25] parse_wasm_edge_weights accepts f64::INFINITY even though the doc comment promises "weights must be finite"; is_infinite() check is missing alongside the existing is_nan() check

## Files Modified
- [x] `crates/sdivi-core/tests/prop_thresholds.rs`
- [x] `bindings/sdivi-wasm/src/weight_keys.rs`
