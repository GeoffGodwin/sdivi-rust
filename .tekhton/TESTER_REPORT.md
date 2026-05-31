## Planned Tests
- [x] `crates/sdivi-patterns/tests/http_routing_limitations.rs` — document Next.js App Router limitation and idiosyncratic-receiver limitation; cover remaining Go receivers at integration level
- [x] `crates/sdivi-patterns/tests/testing_scope_exclude.rs` — fixture-level integration: testing bucket populates when test files are in-scope, is absent when excluded via scope_exclude

## Test Run Results
Passed: 31  Failed: 0

Full sdivi-patterns suite: 225 tests (142 unit + 83 integration/doc) all green.
Pre-existing failure in wasm crate (`wasm_package_json_version_matches_workspace`) is out-of-scope; unchanged.

## Bugs Found
None

## Files Modified
- [x] `crates/sdivi-patterns/tests/http_routing_limitations.rs`
- [x] `crates/sdivi-patterns/tests/testing_scope_exclude.rs`
