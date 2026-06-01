## Planned Tests
- [x] `crates/sdivi-lang-java/tests/extract_behavior.rs` — real tree-sitter parse of try_with_resources_statement fills the Java adapter coverage gap
- [x] `crates/sdivi-lang-python/tests/extract_behavior.rs` — adapter emits `except_clause` from real Python source (M45.2)
- [x] `crates/sdivi-lang-java/tests/extract_behavior.rs` — adapter emits `catch_clause` and `throw_statement` from real Java source (M45.2)

## Test Run Results
Passed: 1444  Failed: 1 (pre-existing: wasm_package_json_version_matches_workspace — package.json at 0.2.23 vs workspace 0.2.37; not introduced by M45.2)

## Bugs Found
None

## Files Modified
- [x] `crates/sdivi-lang-java/tests/extract_behavior.rs`
- [x] `crates/sdivi-lang-python/tests/extract_behavior.rs`
