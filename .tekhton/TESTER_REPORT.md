## Planned Tests
- [x] `crates/sdi-cli/tests/lib_target.rs` — primary behavior: sdi-cli library target exposes pub fn run() and sdi binary works via --help
- [x] `crates/sdi-cli/tests/release_publish_order.rs` — Coverage Gap 1: parse release.yml and assert sdi-parsing is published before sdi-lang-* crates
- [x] `crates/sdi-cli/tests/workspace_version.rs` — Coverage Gap 2 partial + metadata: workspace version is 0.1.0, wasm package.json matches, all crates have readme/keywords/categories

## Test Run Results
Passed: 41  Failed: 0

## Bugs Found
None

## Files Modified
- [x] `crates/sdi-cli/tests/lib_target.rs`
- [x] `crates/sdi-cli/tests/release_publish_order.rs`
- [x] `crates/sdi-cli/tests/workspace_version.rs`
