## Test Audit Report

### Audit Summary
Tests audited: 12 files, 51 test functions
(exit_codes.rs:9, version.rs:2, check_thresholds.rs:5, stdout_stderr_split.rs:4,
show_format.rs:5, trend_format.rs:5, boundaries_stub.rs:3, no_color.rs:4,
write_boundary_spec.rs:6, boundaries_comment_loss_warning.rs:3, path_partition.rs:5,
boundary_lifecycle.rs:0 — comment-only placeholder)

Freshness samples examined: tests/boundary_lifecycle.rs,
tests/fixtures/simple-rust/.sdi/snapshots/ (two snapshot files).

Implementation files cross-referenced: crates/sdi-pipeline/src/pipeline.rs,
crates/sdi-cli/src/main.rs, crates/sdi-pipeline/src/error.rs,
crates/sdi-pipeline/src/store.rs, crates/sdi-config/src/load.rs,
crates/sdi-config/src/thresholds.rs.

Verdict: PASS

### Findings

#### SCOPE: tests/boundary_lifecycle.rs is a comment-only placeholder that is never executed
- File: `tests/boundary_lifecycle.rs`
- Issue: The file contains only a block comment pointing to
  `crates/sdi-cli/tests/boundary_lifecycle.rs`. Cargo requires a `[package]` section to compile
  workspace-root `tests/*.rs` files; without it this file is silently ignored and never run.
  The coder noted it in the summary ("should be removed in a future cleanup pass") but it was
  not actioned in M11.
- Severity: LOW
- Action: Delete `tests/boundary_lifecycle.rs`. The real test lives at
  `crates/sdi-cli/tests/boundary_lifecycle.rs` and is unaffected.

#### NAMING: boundaries_stub.rs file name misrepresents current test content
- File: `crates/sdi-cli/tests/boundaries_stub.rs`
- Issue: All three tests in this file are substantive behavioral contracts — they capture
  stdout/stderr, assert stdout emptiness, assert stderr non-emptiness, and (for ratify) verify
  no `boundaries.yaml` is created. The `_stub` suffix signals to future readers that these are
  scaffolding tests rather than canonical coverage, which could lead to them being overlooked
  or replaced without preserving the behavioral assertions.
- Severity: LOW
- Action: Rename the file to `boundaries_no_snapshots.rs`. No test logic needs to change.

#### NAMING: show_format.rs uses .failure() where exit code 1 is the known contract
- File: `crates/sdi-cli/tests/show_format.rs:162` (`show_no_snapshots_fails`)
- Issue: `.assert().failure()` accepts any non-zero exit code. The corresponding test in
  `exit_codes.rs:186` (`show_no_snapshots_exits_one`) correctly uses `.code(1)`. If `sdi show`
  with no snapshots were to regress from exit 1 to exit 2 or 3, `show_format.rs` would
  silently pass while `exit_codes.rs` would catch it. Having both tests diverge in strictness
  for the same code path reduces confidence.
- Severity: LOW
- Action: Change `.failure()` to `.code(1)` at `show_format.rs:162`.

#### INTEGRITY: boundaries_comment_loss_warning.rs relies on hand-crafted Snapshot JSON
- File: `crates/sdi-cli/tests/boundaries_comment_loss_warning.rs:26` (`minimal_snapshot_json`)
- Issue: The helper builds a `Snapshot`-compatible JSON string by string interpolation, omitting
  fields not required for the current `Snapshot` deserializer. `store::read_snapshots` calls
  `serde_json::from_str::<Snapshot>` and silently skips malformed files (logs via `tracing::warn!`,
  does not error). If a future `Snapshot` field is added without `#[serde(default)]`, the
  hand-crafted JSON fails to deserialize; `infer_from_snapshots` returns empty proposals;
  `run_ratify` exits early without calling `write_boundary_spec`; and the assertion
  `stderr.contains("comments will be lost")` fails — with a message pointing to the test
  assertion rather than the schema drift. The tester acknowledged this and explicitly deferred it.
  All 580 tests currently pass, confirming the JSON is presently valid.
- Severity: LOW (forward-looking fragility; tester deferred with justification)
- Action: When Snapshot evolves, replace `minimal_snapshot_json` with a helper that constructs
  a `Snapshot` value programmatically and serializes it via `serde_json::to_string`, eliminating
  schema coupling. Deferred to a future refactor pass per tester note.

#### COVERAGE: path_partition_entry_count_matches_graph_node_count elides the UTF-8 precondition
- File: `crates/sdi-pipeline/tests/path_partition.rs:70`
- Issue: The assertion `snap.path_partition.len() == snap.graph.node_count` is correct only
  when every parsed file has a valid UTF-8 path. `compute_path_partition` (pipeline.rs:261-262)
  silently drops nodes failing `path.to_str()`; those nodes remain in `graph.node_count`. On the
  `simple-rust` fixture all paths are ASCII, so the assertion holds — but no comment documents
  this precondition. The tester acknowledged and deferred this.
- Severity: LOW (tester deferred with justification)
- Action: Add a comment stating: "Holds for simple-rust (all ASCII paths). Non-UTF-8 paths are
  silently dropped by compute_path_partition, so path_partition.len() < graph.node_count in that
  case." No assertion change required for the current fixture.

#### EXERCISE: path_partition_community_ids_are_valid uses partition.stability.keys() as proxy for all known communities
- File: `crates/sdi-pipeline/tests/path_partition.rs:92`
- Issue: `known_communities` is built from `snap.partition.stability.keys()`. Community IDs in
  `path_partition` originate from `partition.assignments`, not `stability`. If a community
  appears in `assignments` but not `stability` (e.g., a degenerate Leiden run producing a
  community with no stability score), the assertion would produce a false failure unrelated to
  the `comm_id as u32` cast it intends to guard. For the `simple-rust` fixture the two maps
  are consistent, so the test passes correctly.
- Severity: LOW
- Action: Build `known_communities` from `snap.partition.assignments.values()` (the authoritative
  source for which communities exist) rather than from `snap.partition.stability.keys()`. This
  directly guards the numeric cast without coupling to the stability map structure.

#### COVERAGE: no_color.rs check and trend tests may be trivially true when commands produce empty stdout
- File: `crates/sdi-cli/tests/no_color.rs:43` (`no_color_env_suppresses_ansi_in_check`),
         `crates/sdi-cli/tests/no_color.rs:63` (`no_color_env_suppresses_ansi_in_trend_insufficient_snapshots`)
- Issue: Both tests assert `!has_ansi(&stdout)` on the stdout of commands that may produce no
  stdout output in their default text format. `sdi check` with no prior snapshots and no
  `--format json` might write its report to stderr only; `sdi trend` with zero snapshots writes
  "not enough snapshots" to stderr (confirmed by `trend_format.rs`). If stdout is empty, the
  `\x1b[` scan trivially passes regardless of whether ANSI suppression is working, making the
  test a no-op rather than an active guard. The `no_color_env_suppresses_ansi_in_show` test does
  not have this problem because `sdi show` definitively produces a snapshot summary on stdout.
- Severity: LOW
- Action: Add an assertion that stdout is non-empty before checking for ANSI codes, or switch to
  testing commands that unconditionally produce stdout output in text mode (e.g., `sdi check` with
  a prior snapshot and a breached threshold, or `sdi trend` with ≥2 snapshots).

### Passing Items

**Assertion Honesty — PASS.** No test asserts hard-coded magic values unconnected to
implementation logic. `check_exits_ten_when_threshold_breached` uses `coupling_delta_rate = -1.0`
and relies on `0.0 > -1.0` — the invariant is documented in the test comment and verified against
`pipeline.rs` and `main.rs`. `config_error_exits_two` produces a real `ConfigError::MissingExpiresOnOverride`
via the config loading path at `load.rs:101` before any subcommand dispatch. No `assertTrue(True)`
or self-referential assertions found.

**Test Weakening — PASS.** The one renamed test (`snapshot_exits_zero_on_all_unknown_languages`
→ `snapshot_exits_three_on_all_unknown_languages`) is a STRENGTHENING: the old assertion
`.success()` accepted the buggy pre-M11 exit-0 behavior; the new `.code(3)` enforces the
Rule 15 / System Rule 7 contract, made possible by the matching coder fix in `pipeline.rs:142-150`
and `main.rs:153-157`. The `no_color` rename is a name correction with unchanged assertion body.
No assertion was broadened; no edge case was removed.

**Implementation Exercise — PASS.** All tests call real binaries via `assert_cmd` or real library
functions (`write_boundary_spec`, `Pipeline::snapshot_with_mode`). No mocking of any internal.
`path_partition.rs` exercises the private `compute_path_partition` helper via the only correct
public seam (`Pipeline::snapshot_with_mode`). `write_boundary_spec.rs` calls the function
directly and verifies disk state.

**Test Naming — PASS (with two LOW exceptions above).** All function names encode the scenario
and expected outcome. No `test_1()` or `test_thing()` patterns present.

**Scope Alignment — PASS.** No orphaned imports. `PipelineError::NoGrammarsAvailable` exists in
`sdi-pipeline/src/error.rs:14`. `write_boundary_spec` is public in `sdi-pipeline/src/store.rs:111`.
`Pipeline::snapshot_with_mode` and `WriteMode::EphemeralForCheck` are public in
`sdi-pipeline/src/pipeline.rs:127,35`. The deleted `.tekhton/test_dedup.fingerprint` is not
referenced by any test under audit.

**Test Isolation — PASS.** All CLI integration tests create their own `tempfile::TempDir`. The
`write_boundary_spec.rs` tests create isolated `TempDir` instances with no shared state. No test
reads `.tekhton/`, `.claude/`, build artifacts, or any mutable pipeline state file. The
`path_partition.rs` tests use `WriteMode::EphemeralForCheck` — no snapshots are written to the
fixture directory. (The partition cache write at `pipeline.rs:162` does touch the fixture's
`.sdi/cache/` directory, but this is an idempotent pre-existing behavior shared across all
pipeline tests and is not unique to M11.)

**Prior HIGH/MEDIUM Findings Resolved — PASS.** All issues from the prior audit rework pass were
addressed: `check_exits_ten_when_threshold_breached` and `config_error_exits_two` added to
`exit_codes.rs`; `no_color_flag_suppresses_ansi_in_show` renamed to
`default_show_output_has_no_ansi_codes`; `.failure()` replaced with `.code(1)` for `show` and
`diff` tests in `exit_codes.rs`; `coupling_slope` assertion strengthened to `.as_f64().is_some()`;
three check-test duplicates removed from `exit_codes.rs`.
