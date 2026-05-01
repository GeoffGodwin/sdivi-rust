## Test Audit Report

### Audit Summary
Tests audited: 3 files, 18 test functions
Verdict: PASS

Files:
- `crates/sdi-pipeline/tests/commit_snapshot.rs` — 9 integration tests
- `crates/sdi-pipeline/src/commit_extract.rs` (unit test block, `#[cfg(test)]`) — 6 unit tests
- `crates/sdi-pipeline/tests/pipeline_smoke.rs` — 3 integration tests

Implementation files cross-referenced: `crates/sdi-pipeline/src/commit_extract.rs`,
`crates/sdi-pipeline/src/pipeline.rs`, `crates/sdi-pipeline/src/error.rs`,
`crates/sdi-pipeline/src/helpers.rs`.

---

### Findings

#### NAMING: `change_coupling_ends_at_commit_not_head` name does not match its assertions
- File: `crates/sdi-pipeline/tests/commit_snapshot.rs:185`
- Issue: The test name implies that the change-coupling analysis window is verified to
  stop at the named commit rather than at HEAD. The two assertions (lines 203–204) only
  confirm that `snap_hist.commit` equals `sha_head1` and that the two commit fields
  differ — both are SHA-labeling checks, not coupling-window checks. The actual
  windowing behavior promised by the name is covered by the separate
  `change_coupling_window_clamped_to_commit_not_head` test. Additionally, the SHA-label
  assertions are redundant with the stronger `commit_field_is_resolved_sha_not_ref_name`
  and `different_commits_produce_distinct_snapshots` tests.
- Severity: MEDIUM
- Action: Either (a) rename the test to `historical_commit_sha_label_matches_expected`
  to accurately describe what it asserts, or (b) remove it entirely as a redundant test.
  Do NOT add coupling-window assertions here to justify the current name;
  `change_coupling_window_clamped_to_commit_not_head` already covers that path
  more thoroughly.

#### ISOLATION: `pipeline_smoke.rs` writes snapshot files to a checked-in fixture directory
- File: `crates/sdi-pipeline/tests/pipeline_smoke.rs:27` (`snapshot_on_simple_rust_fixture`)
  and `crates/sdi-pipeline/tests/pipeline_smoke.rs:48` (`delta_of_same_snapshot_is_all_zero`)
- Issue: Both tests call `pipeline.snapshot(root, None, …)` where `root` resolves to
  `tests/fixtures/simple-rust/` (a checked-in directory). `Pipeline::snapshot` defaults
  to `WriteMode::Persist`, so each invocation writes to
  `tests/fixtures/simple-rust/.sdi/snapshots/` and runs `enforce_retention` against that
  directory. This violates the project's explicit testing strategy (CLAUDE.md: "Use real
  filesystem via `tempfile` for any test that touches `.sdi/`"). Consequences: (1) snapshot
  files accumulate in the source tree on every `cargo test` run, leaving the working tree
  dirty; (2) parallel test execution creates a race between the two tests on the shared
  `enforce_retention` path; (3) if any future test reads from `tests/fixtures/simple-rust/
  .sdi/`, it will observe run-dependent state. This issue predates M16 and was inherited
  by the tester, who only changed the `commit` argument.
- Severity: MEDIUM
- Action: Either (a) copy the `simple-rust` fixture into a fresh `TempDir` before
  calling `pipeline.snapshot` so writes are isolated and cleaned up on drop, or (b)
  replace the `Pipeline::snapshot` calls with `pipeline.snapshot_with_mode(root, None,
  …, WriteMode::EphemeralForCheck)` — these smoke tests do not require persistence and
  `EphemeralForCheck` avoids all `.sdi/` writes.

#### COVERAGE: `timestamp_is_commit_date_not_wall_clock` uses only a negative assertion for the date value
- File: `crates/sdi-pipeline/tests/commit_snapshot.rs:121`
- Issue: The test passes `"2099-12-31T23:59:59Z"` as the wall-clock argument and asserts
  `!snap.timestamp.starts_with("2099")`. This confirms the timestamp is not the supplied
  wall-clock value but does not verify it is the actual commit date. The assertion would
  pass for any malformed non-2099 timestamp. A complementary positive assertion — that
  the timestamp equals the value produced by `commit_date_iso` for the same SHA — would
  close the gap.
- Severity: LOW
- Action: Add a positive assertion. Resolve the expected timestamp independently via
  `get_sha(repo.path(), "HEAD")` followed by `commit_date_iso(repo.path(), &sha)`, then
  assert `snap.timestamp == expected_utc_ts`. This converts the negative sanity check
  into a precise behavioral assertion.

---

### Passing Items

**INTEGRITY — PASS.** All assertions derive expected values from fixture construction or
documented implementation behavior. Node counts (1/2/3) match the number of `.rs` files
committed at each revision. Co-change counts and frequencies in
`change_coupling_window_clamped_to_commit_not_head` are derived from the explicit
`setup_cochange_repo()` fixture design (2/3 commits where both a.rs and b.rs appear →
frequency ≈ 0.67 ≥ 0.6). SHA length check (40 hex chars) matches the invariant enforced
by `resolve_ref_to_sha`. No hard-coded magic values; no tautological assertions.

**WEAKENING — PASS.** The sole modification to an existing test (`pipeline_smoke.rs`)
changed `commit=Some("some-label")` to `commit=None`. This is a correct adaptation to the
M16 API change (where `Some(ref)` now triggers real git resolution), not a weakening: the
test still exercises the same five-stage pipeline and the `snap.commit.is_none()` assertion
is a new, honest behavioral claim about the no-commit path.

**EXERCISE — PASS.** All tests call real `Pipeline::snapshot` (integration) or real
`normalize_to_utc` (unit). No internal dependencies are mocked. The commit-extract unit
tests access `normalize_to_utc` via `use super::*` in the same file, calling the actual
private implementation directly.

**SCOPE — PASS.** All imports (`sdi_pipeline::Pipeline`, `sdi_config::Config`,
`sdi_lang_rust::RustAdapter`, `sdi_snapshot::snapshot::SNAPSHOT_VERSION`) match symbols
present in the current codebase. No orphaned references to deleted or renamed symbols.
The `PipelineError::CommitExtract` variant referenced in
`crates/sdi-cli/tests/exit_codes.rs` (coder-added, not tester-modified) is confirmed
present in `crates/sdi-pipeline/src/error.rs:28`.

**ISOLATION — PASS (for `commit_snapshot.rs` and `commit_extract.rs`).** All nine tests
in `commit_snapshot.rs` create their own `TempDir` git repos via factory functions. The
six unit tests in `commit_extract.rs` operate on string literals with no filesystem
access. The fixture-write issue is confined to `pipeline_smoke.rs` and noted above.

**COVERAGE — PASS (for `commit_extract.rs`).** The six `normalize_to_utc` unit tests
cover: UTC passthrough, negative-offset forward shift, positive-offset backward shift,
malformed-input `None` return, positive-offset crossing midnight backward, and
negative-offset crossing midnight forward. Both day-boundary directions are exercised,
directly satisfying reviewer gap 2 from the TESTER_REPORT.

**COVERAGE — PASS (for windowing, reviewer gap 1).** `change_coupling_window_clamped_
to_commit_not_head` uses a purpose-built `setup_cochange_repo()` fixture and asserts
exact `commits_analyzed` counts, pair counts, `cochange_count`, `frequency`, and
lexicographic source/target ordering at both HEAD~1 and HEAD. This is the strongest test
in the suite and directly satisfies reviewer gap 1.
