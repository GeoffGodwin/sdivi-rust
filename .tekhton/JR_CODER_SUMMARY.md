# Junior Coder Summary — M12 Review Cycle 1

## What Was Fixed

All items under **Simple Blockers (jr coder)** from REVIEWER_REPORT.md have been completed:

1. **`bindings/sdi-wasm/src/exports.rs:170-175` — Hardcoded violation_count**
   - Added `violation_count: Option<u32>` field to `WasmAssembleSnapshotInput` in `types.rs` with `#[tsify(optional)]` attribute
   - Updated `assemble_snapshot` export to use `input.violation_count.unwrap_or(0) as usize` instead of hardcoding `0` in `IntentDivergenceInfo`
   - Now WASM consumers calling `compute_boundary_violations` will have correct violation counts in assembled snapshots, fixing false `boundary_violation_delta = 0` in downstream `compute_delta` calls

2. **`.github/workflows/wasm.yml:62-64` — Broken CI cross-platform hash determinism check**
   - Added dedicated `#[wasm_bindgen_test]` function `normalize_hash_deterministic` in `bindings/sdi-wasm/tests/wasm_smoke.rs`
   - Test calls `normalize_and_hash("try_expression", vec![])` and prints `println!("CI_HASH:{}", hash)` for CI to extract
   - Updated CI grep pattern from `[0-9a-f]{64}` to `CI_HASH:[0-9a-f]{64}` with `sed 's/CI_HASH://'` extraction
   - Now CI correctly captures and compares `normalize_and_hash` output across platforms per Critical System Rule 23

## Files Modified

- `bindings/sdi-wasm/src/types.rs` — Added `violation_count: Option<u32>` field to `WasmAssembleSnapshotInput`
- `bindings/sdi-wasm/src/exports.rs` — Updated `violation_count: 0` hardcode to use `input.violation_count.unwrap_or(0) as usize`
- `bindings/sdi-wasm/tests/wasm_smoke.rs` — Added `normalize_hash_deterministic` test function
- `.github/workflows/wasm.yml` — Fixed CI grep pattern and hash capture bash logic
