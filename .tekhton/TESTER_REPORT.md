## Planned Tests
- [x] `crates/sdivi-patterns/tests/prop_classify_hint.rs` — proptest fall-through consistency: classify_hint equals category_for_node_kind for all non-call/macro node kinds
- [x] `crates/sdivi-patterns/tests/classify_hint.rs` — disjoint-regex invariant tests for Python, Go, and Rust (call and macro paths)
- [x] `crates/sdivi-pipeline/tests/snapshot_m32_unchanged.rs` — M32 regression guard: pipeline output is byte-identical for fixed params (bit-identity claim)
- [x] `crates/sdivi-patterns/tests/simple_go_fixture.rs` — build_catalog integration test covering simple-go acceptance criterion: fmt.Print* → logging, db.* → data_access, non-matching → dropped

## Test Run Results
Passed: 42  Failed: 0

## Bugs Found
None

## Files Modified
- [x] `crates/sdivi-patterns/tests/prop_classify_hint.rs`
- [x] `crates/sdivi-patterns/tests/classify_hint.rs`
- [x] `crates/sdivi-pipeline/tests/snapshot_m32_unchanged.rs`
- [x] `crates/sdivi-patterns/tests/simple_go_fixture.rs`
