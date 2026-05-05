## Planned Tests
- [x] `crates/sdivi-graph/tests/tsconfig_alias.rs` — JSONC/parse edge cases + determinism + property-style no-panic tests
- [x] `crates/sdivi-pipeline/tests/tsconfig_pipeline.rs` — pipeline acceptance criteria: malformed tsconfig succeeds; jsconfig.json fallback succeeds

## Test Run Results
Passed: 118  Failed: 4 (pre-existing: wasm_package_json_* in sdivi-cli, bindings/sdivi-wasm/package.json not yet created — M12 scope)

## Bugs Found
None

## Files Modified
- [x] `crates/sdivi-graph/tests/tsconfig_alias.rs`
- [x] `crates/sdivi-pipeline/tests/tsconfig_pipeline.rs`
