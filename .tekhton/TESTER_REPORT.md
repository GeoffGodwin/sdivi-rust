## Planned Tests
- [x] `crates/sdivi-patterns/tests/dispatch_disjointness_supplement.rs` — add Promise.allSettled and asyncio.create_task corpus entries absent from main dispatch_disjointness.rs CORPUS

## Test Run Results
Passed: 10  Failed: 0

Focused run of `dispatch_disjointness_supplement`: 10 passed (8 pre-existing + 2 new).
M44 category_contract tests: 16 passed. Concurrency Go fixture: 4 passed. Dispatch disjointness: 4 passed.
Full workspace (`cargo test --workspace`): 121 passed, 1 pre-existing failure (`wasm_package_json_version_matches_workspace` — package.json at 0.2.23 vs workspace 0.2.35; not introduced by M44).

## Bugs Found
None

## Files Modified
- [x] `crates/sdivi-patterns/tests/dispatch_disjointness_supplement.rs`
