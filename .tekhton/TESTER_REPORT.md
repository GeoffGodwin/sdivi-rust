## Planned Tests
- [x] `bindings/sdivi-wasm/src/weight_keys.rs` — add `rejects_negative_infinity_weight` companion test for `f64::NEG_INFINITY`
- [x] `bindings/sdivi-wasm/tests/validate_pkg_template.cjs` — validate pkg-template/package.json is parseable JSON with required exports map structure
- [x] `bindings/sdivi-wasm/tests/node_smoke/package.json` — fix npm test script to use `node index.mjs` directly (aligns local and CI invocation paths)

## Manual Verification Items
- **Node 18 smoke run**: The CI matrix currently uses `node-version: "20"` only. A parallel matrix entry with `node-version: "18"` in `wasm.yml` would confirm the `engines: >=18` claim is real. Adding that matrix entry requires a CI workflow change (implementation code); it is not automated here. Acceptance criterion: both CJS and ESM smoke tests pass under Node 18.

## Test Run Results
Passed: 3  Failed: 0

## Bugs Found
None

## Files Modified
- [x] `bindings/sdivi-wasm/src/weight_keys.rs`
- [x] `bindings/sdivi-wasm/tests/validate_pkg_template.cjs`
- [x] `bindings/sdivi-wasm/tests/node_smoke/package.json`
