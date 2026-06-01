## Planned Tests
- [x] `crates/sdivi-patterns/tests/comprehensions_fixture.rs` — integration: real Python adapter parses all four comprehension forms and routes each to `comprehensions` bucket
- [x] `crates/sdivi-lang-python/tests/extract_behavior.rs` — M46 adapter-level: all four comprehension node kinds are captured as pattern hints from real Python source

## Test Run Results
Passed: 121 integration + 279 unit + 125 doc = 525 total  Failed: 1 (pre-existing: wasm_package_json_version_matches_workspace — package.json at 0.2.23 vs workspace 0.2.38; not introduced by M46)

## Bugs Found
None

## Files Modified
- [x] `crates/sdivi-patterns/tests/comprehensions_fixture.rs`
- [x] `crates/sdivi-lang-python/tests/extract_behavior.rs`
