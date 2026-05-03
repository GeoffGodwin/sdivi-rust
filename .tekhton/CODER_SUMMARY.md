# Coder Summary
## Status: COMPLETE

## What Was Implemented

- **`bindings/sdivi-wasm/src/types.rs`**: Added `pub edge_weights: Option<BTreeMap<String, f64>>` to `WasmLeidenConfigInput` with `#[serde(default)]` and `#[tsify(optional)]`. Replaced the 3-line "INTENTIONAL GAP (ADL-4)" comment with a 1-line doc. Keyed by `"source:target"` colon-separated strings; first colon splits source from target (node IDs containing colons are supported).
- **`bindings/sdivi-wasm/src/weight_keys.rs`** (NEW): `parse_wasm_edge_weights` helper. Converts colon-separated WASM keys to NUL-separated native keys via `edge_weight_key`. Returns `Result<_, String>` (not `JsError`) so native unit tests can run. Validates: NaN, negative, no colon, empty source/target → error; 0.0 accepted.
- **`bindings/sdivi-wasm/src/exports.rs`**: `detect_boundaries` now extracts `edge_weights` before the `to_core` serde round-trip (native format uses NUL keys; WASM callers use colon keys — direct round-trip would silently discard all weights). Converted keys are injected into `LeidenConfigInput.edge_weights` and passed to the existing `detect_boundaries` native path which already handles the weighted/unweighted branch.
- **`bindings/sdivi-wasm/src/lib.rs`**: Added `pub(crate) mod weight_keys;`.
- **`bindings/sdivi-wasm/tests/wasm_weighted_leiden.rs`** (NEW): 6 WASM integration tests: `test_detect_boundaries_weighted_differs_from_unweighted`, `test_detect_boundaries_rejects_malformed_weight_key`, `test_detect_boundaries_rejects_negative_weight`, `test_detect_boundaries_rejects_nan_weight`, `test_detect_boundaries_accepts_zero_weight`, `test_detect_boundaries_weighted_deterministic`.
- **`bindings/sdivi-wasm/tests/wasm_smoke.rs`**: Updated `default_leiden_cfg()` to include `edge_weights: None` (required now that the field exists on `WasmLeidenConfigInput`).
- **`bindings/sdivi-wasm/tests/wasm_snapshot.rs`**: Updated `test_adl4_wasm_leiden_config_input_omits_edge_weights` → `test_m21_wasm_leiden_config_input_edge_weights_optional` reflecting that M21 implements the previously-deferred weighted Leiden.
- **`bindings/sdivi-wasm/README.md`**: Added weighted Leiden usage snippet with `edge_weights` and colon-key documentation.
- **`.tekhton/ARCHITECTURE_LOG.md`**: Added ADL-9 documenting the colon-key design choice and the serde round-trip bypass.
- **`CHANGELOG.md`**: Added `[0.1.11]` entry under Added.

## Root Cause (bugs only)
N/A — feature milestone.

## Files Modified
- `bindings/sdivi-wasm/src/types.rs` — added `edge_weights` field to `WasmLeidenConfigInput`; removed ADL-4 comment
- `bindings/sdivi-wasm/src/weight_keys.rs` (NEW) — `parse_wasm_edge_weights` helper + 7 native unit tests
- `bindings/sdivi-wasm/src/exports.rs` — `detect_boundaries` routes edge_weights via colon→NUL conversion
- `bindings/sdivi-wasm/src/lib.rs` — added `pub(crate) mod weight_keys;`
- `bindings/sdivi-wasm/tests/wasm_weighted_leiden.rs` (NEW) — 6 WASM integration tests
- `bindings/sdivi-wasm/tests/wasm_smoke.rs` — added `edge_weights: None` to `default_leiden_cfg()`
- `bindings/sdivi-wasm/tests/wasm_snapshot.rs` — renamed ADL-4 test to reflect M21 completion
- `bindings/sdivi-wasm/README.md` — weighted Leiden usage example
- `.tekhton/ARCHITECTURE_LOG.md` — ADL-9 entry
- `CHANGELOG.md` — `[0.1.11]` entry

## Human Notes Status
- Non-Blocking Note (threshold_types.rs:98 doc-test import path): NOT_ADDRESSED — out of scope for M21
- Coverage Gaps (prop_breach additional dimension property tests): NOT_ADDRESSED — already addressed by tester in prior run (Tester Report: Passed 9, Failed 0)
- Drift Observations (compute/mod.rs:9): NOT_ADDRESSED — out of scope for M21

## Docs Updated
- `bindings/sdivi-wasm/README.md` — weighted Leiden usage example
- `CHANGELOG.md` — `[0.1.11]` entry for M21

## Architecture Change Proposals

### New test file `wasm_weighted_leiden.rs` instead of extending `wasm_smoke.rs`
- **Current constraint**: Scout report says all new tests go in `wasm_smoke.rs`.
- **What triggered this**: `wasm_smoke.rs` was 240 lines; adding ~80 new lines would exceed the 300-line ceiling.
- **Proposed change**: Created `bindings/sdivi-wasm/tests/wasm_weighted_leiden.rs` for the 6 new weighted Leiden tests.
- **Backward compatible**: Yes — no existing tests moved or changed.
- **ARCHITECTURE.md update needed**: No.

### New module `weight_keys.rs` instead of inlining helper in `exports.rs`
- **Current constraint**: Helper could be a private function in `exports.rs`.
- **What triggered this**: `exports.rs` hit 309 lines after adding the helper inline; 300-line ceiling exceeded.
- **Proposed change**: `weight_keys.rs` module with `parse_wasm_edge_weights` returning `Result<_, String>` (not `JsError`) so native unit tests work without hitting the "cannot call wasm-bindgen imported functions on non-wasm targets" panic.
- **Backward compatible**: Yes — internal module, no public API change.
- **ARCHITECTURE.md update needed**: No.

## Observed Issues (out of scope)
- `crates/sdivi-cli/tests/workspace_version.rs:113` — `wasm_package_json_version_matches_workspace` was already failing before M21: wasm `package.json` at `0.1.8` vs workspace `0.1.10`. Pre-existing version sync issue.
- `crates/sdivi-config/src/thresholds.rs:52` — `validate_and_prune_overrides` dead code warning (pre-existing).

## Files Modified (auto-detected)
- `.claude/milestones/MANIFEST.cfg`
- `.claude/milestones/m21-weighted-leiden-wasm.md`
- `.tekhton/ARCHITECTURE_LOG.md`
- `.tekhton/CODER_SUMMARY.md`
- `.tekhton/test_dedup.fingerprint`
- `CHANGELOG.md`
- `Cargo.lock`
- `bindings/sdivi-wasm/README.md`
- `bindings/sdivi-wasm/src/exports.rs`
- `bindings/sdivi-wasm/src/lib.rs`
- `bindings/sdivi-wasm/src/types.rs`
- `bindings/sdivi-wasm/tests/wasm_smoke.rs`
- `bindings/sdivi-wasm/tests/wasm_snapshot.rs`
