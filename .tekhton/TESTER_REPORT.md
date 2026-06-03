## Planned Tests
- [x] `crates/sdivi-cli/tests/workspace_version.rs` — existing version-alignment and package metadata tests (includes `wasm_package_json_version_matches_workspace`)
- [x] MANUAL: `wasm-pack build --target bundler --release` in `bindings/sdivi-wasm/` succeeds without invoking `wasm-opt` (requires wasm-pack + wasm32 toolchain; verified by CI `wasm.yml`)
- [x] MANUAL: bundler `.wasm` ≤ 1,835,008 bytes after binaryen-free build (CI `wasm.yml` Check bundle sizes gate)

## Test Run Results
Passed: 1492  Failed: 0

## Manual Verification Results
- `wasm-pack build --target bundler --release`: PASSED — no wasm-opt invoked, build succeeded in ~8.82s
- `wasm-pack build --target nodejs --release`: PASSED — no wasm-opt invoked
- bundler `.wasm` size: 1,679,707 bytes (≤ 1,835,008 budget, ~155 KB headroom)

## Bugs Found
None

## Files Modified
- [x] `crates/sdivi-cli/tests/workspace_version.rs`

## Timing
- Test executions: 2 (cargo test --workspace + wasm-pack builds)
- Approximate total test execution time: 45s
- Test files written: 0 (continuation — no new files needed)
