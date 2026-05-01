## Planned Tests
- [x] `crates/sdi-pipeline/tests/commit_snapshot.rs` — add test verifying coupling window actually clamped to historical commit (reviewer gap 1)
- [x] `crates/sdi-pipeline/src/commit_extract.rs` — add normalize_to_utc day-boundary edge case unit tests (reviewer gap 2)
- [x] `crates/sdi-pipeline/tests/pipeline_smoke.rs` — fix M16 API breakage (commit=Some(label) no longer valid; updated to None path)

## Test Run Results
Passed: 52  Failed: 0

## Bugs Found
None

## Files Modified
- [x] `crates/sdi-pipeline/tests/commit_snapshot.rs`
- [x] `crates/sdi-pipeline/src/commit_extract.rs`
- [x] `crates/sdi-pipeline/tests/pipeline_smoke.rs`
