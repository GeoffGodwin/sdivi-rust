## Planned Tests
- [x] `crates/sdi-core/tests/compute_change_coupling.rs` — add trailing-window correctness test (distinguishes leading vs trailing window selection)
- [x] `crates/sdi-core/tests/leiden_config_serde.rs` — round-trip LeidenConfigInput with populated edge_weights through serde_json
- [x] `bindings/sdi-wasm/tests/wasm_smoke.rs` — add assemble_snapshot with violation_count set test

## Test Run Results
Passed: 12  Failed: 1

Note: `bindings/sdi-wasm/tests/wasm_smoke.rs` requires `wasm-pack test --node` with the 1.85.0
toolchain (rust-toolchain.toml). The environment has rustc 1.75.0 and no wasm-pack, so the
WASM test was added and verified by reading but cannot be compiled/executed here.
The 12 passed / 1 failed counts are from sdi-core only.

## Bugs Found
- BUG: [crates/sdi-core/src/input/types.rs:145] LeidenConfigInput with populated edge_weights fails serde_json serialization with "key must be a string" (BTreeMap<(String,String),f64> tuple keys cannot be JSON object keys)

## Files Modified
- [x] `crates/sdi-core/tests/compute_change_coupling.rs`
- [x] `crates/sdi-core/tests/leiden_config_serde.rs`
- [x] `bindings/sdi-wasm/tests/wasm_smoke.rs`
