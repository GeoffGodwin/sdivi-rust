## Test Audit Report

### Audit Summary
Tests audited: 11 files, 56 test functions
(version.rs:2, exit_codes.rs:12, check_thresholds.rs:6, stdout_stderr_split.rs:4,
show_format.rs:5, trend_format.rs:5, boundaries_stub.rs:3, no_color.rs:4,
write_boundary_spec.rs:7, boundaries_comment_loss_warning.rs:3, path_partition.rs:5)
Verdict: PASS

---

### Findings

#### NAMING: `boundaries_stub.rs` file name misrepresents current content
- File: `crates/sdi-cli/tests/boundaries_stub.rs:1`
- Issue: All three tests were upgraded from M09 stubs to full behavioral contracts — they now capture stdout/stderr, assert emptiness, and (for ratify) verify no `boundaries.yaml` is created. The `_stub` suffix misleads future readers into treating the file as scaffolding rather than canonical coverage. The coder's own summary confirms: "Updated `boundaries_stub.rs` to match new behavior (no longer stubs)."
- Severity: LOW
- Action: Rename to `boundaries_no_snapshots.rs`. No test logic needs to change.

---

#### INTEGRITY: Hand-crafted `Snapshot` JSON in `boundaries_comment_loss_warning.rs`
- File: `crates/sdi-cli/tests/boundaries_comment_loss_warning.rs:26-60`
- Issue: `minimal_snapshot_json` builds a `Snapshot`-compatible JSON string via string interpolation. `store::read_snapshots` deserializes via `serde_json::from_str::<Snapshot>` and silently skips malformed files with a tracing warning, not a hard error. If any future `Snapshot` field is added without `#[serde(default)]`, the hand-crafted JSON fails to deserialize, `infer_from_snapshots` returns empty proposals, `run_ratify` prints "no stable communities found" rather than calling `write_boundary_spec`, and the assertion `stderr.contains("comments will be lost")` fails — with an error that points to the test assertion rather than the schema drift. The tester acknowledged this fragility and deferred it. All 573 tests currently pass, so the JSON is presently valid.
- Severity: LOW
- Action: Extract `write_fake_snapshot` from `crates/sdi-pipeline/src/boundaries.rs:65-91` (currently inside `#[cfg(test)] mod tests`) into a shared `test_support` module or a `tests/support.rs` file, and use it in `boundaries_comment_loss_warning.rs` in place of `minimal_snapshot_json`. This removes the schema-coupling concern without touching production code.

---

#### NAMING: `snapshot_exits_zero_on_all_unknown_languages` name gives no signal it documents a known bug
- File: `crates/sdi-cli/tests/exit_codes.rs:46`
- Issue: The test asserts `.success()` (exit 0) for behavior that CLAUDE.md Rule 15 / System Rule 7 requires to be exit 3. The inline comment block (lines 38–45) is clear about the bug, but the test name alone gives no indication that the assertion intentionally documents a violated invariant. Anyone reading `cargo test -q` output sees `snapshot_exits_zero_on_all_unknown_languages ... ok` with no signal that this is a known-broken contract.
- Severity: LOW
- Action: Rename to `snapshot_exits_zero_on_all_unknown_languages_known_bug` so the bug is visible in test output, or annotate with `#[ignore = "BUG: Rule 15 requires exit 3; PipelineError::NoGrammarsAvailable is never emitted"]` so the discrepancy appears in CI. No assertion logic needs to change.

---

#### NAMING: `show_no_snapshots_fails` uses `.failure()` where `.code(1)` is the stated contract
- File: `crates/sdi-cli/tests/show_format.rs:161`
- Issue: `.failure()` accepts any non-zero exit code. The equivalent test in `exit_codes.rs` (`show_no_snapshots_exits_one`, line 170) was correctly updated to `.code(1)` during the audit rework, but `show_format.rs:161` retains `.failure()`. If `sdi show` with no snapshots were to regress to exit 2 or 3, `show_format.rs` would silently pass while `exit_codes.rs` would catch it — but only if that file happens to be tested in the same run.
- Severity: LOW
- Action: Change `.failure()` to `.code(1)` at `show_format.rs:161`.

---

#### COVERAGE: `path_partition_entry_count_matches_graph_node_count` conflates two independent counts without noting the UTF-8 precondition
- File: `crates/sdi-pipeline/tests/path_partition.rs:70-85`
- Issue: The assertion `path_partition.len() == snap.graph.node_count` holds only when (a) the Leiden partition assigns every graph node a community, and (b) every assigned node has a valid UTF-8 path. `compute_path_partition` silently drops nodes failing `path.to_str()`. On the `simple-rust` fixture both conditions hold, so the assertion is correct, but the test carries no comment explaining the UTF-8 precondition. The tester acknowledged this and deferred it.
- Severity: LOW
- Action: Add a comment: "This holds for the `simple-rust` fixture (all ASCII paths). Non-UTF-8 paths are silently dropped by `compute_path_partition`, which would cause `path_partition.len() < graph.node_count`." No assertion change needed for the current fixture.

---

### What Passed Without Issue

- **Prior HIGH findings resolved.** All HIGH and MEDIUM issues from the M09 audit were correctly addressed in the rework pass: `check_exits_ten_when_threshold_breached` and `config_error_exits_two` were added to `exit_codes.rs`; `no_color_flag_suppresses_ansi_in_show` was renamed to `default_show_output_has_no_ansi_codes`; `.failure()` was replaced with `.code(1)` in `show_no_snapshots_exits_one` and `diff_nonexistent_file_exits_one` in `exit_codes.rs`; `coupling_slope` assertion was strengthened to `.as_f64().is_some()`; duplicated check tests were removed from `exit_codes.rs`.

- **`version.rs`**: `env!("CARGO_PKG_VERSION")` is compile-time evaluated and robust to version bumps. Both tests invoke the real binary via `assert_cmd`.

- **`exit_codes.rs`**: `check_exits_ten_when_threshold_breached` logic is sound — `coupling_delta_rate = -1.0` guarantees breach because `0.0 > -1.0`; JSON field assertions on `exit_code` and `exceeded` test real output structure. `config_error_exits_two` correctly produces `ConfigError::MissingExpiresOnOverride` via the config loading path before any subcommand dispatch.

- **`check_thresholds.rs`**: Six tests cover distinct scenarios with specific assertions. `check_json_exit_code_matches_process_exit` is a strong bidirectional guard between the JSON payload and OS-level exit code.

- **`stdout_stderr_split.rs`**: All four tests verify the `sdi … --format json | jq '.'` contract. The `serde_json::from_str(...).is_err() || stderr.trim().is_empty()` guard correctly detects JSON contamination while tolerating `tracing` log output.

- **`show_format.rs`**: `show_no_id_selects_latest` cross-checks the returned snapshot's `commit` against the lexicographically-last file on disk — a non-trivial end-to-end assertion of the latest-selection contract.

- **`trend_format.rs`**: Specific-substring assertions (`"not enough snapshots"`) and `--last` clamping tests (snapshot_count verified by actual JSON value, not heuristics). `coupling_slope.as_f64().is_some()` catches null values correctly.

- **`boundaries_stub.rs`**: Correctly tests the real graceful-degradation paths: stdout empty, stderr non-empty, and (for ratify) no `boundaries.yaml` created when proposals are absent. Assertions match the actual `run_infer` / `run_ratify` / `run_show` code paths in `boundaries.rs`.

- **`no_color.rs`**: `NO_COLOR=1` via `.env("NO_COLOR", "1")` exercises the env-var suppression path. The `\x1b[` scan is a concrete assertion. `default_show_output_has_no_ansi_codes` is correctly scoped with a comment explaining when the test should be revisited.

- **`write_boundary_spec.rs`**: Seven tests cover new-file creation, round-trip YAML validity, both `#` detection patterns (leading and inline), atomicity (replace not append), parent-dir creation, and empty spec. All assertions verify post-conditions against real disk state via `tempfile`. This is the strongest new test file in M10.

- **`boundaries_comment_loss_warning.rs`**: The three-test structure (warning present / no warning without comments / no warning on first ratify) covers the positive case and both negative cases of the `store.rs:108-128` warning path. Negative assertions guard against spurious warnings.

- **`path_partition.rs`**: `pipeline_populates_path_partition_on_real_fixture`, `path_partition_keys_are_non_empty_strings`, `path_partition_community_ids_are_valid`, and `path_partition_is_deterministic` exercise the real `Pipeline::snapshot_with_mode` against the `simple-rust` fixture using `WriteMode::EphemeralForCheck` — no fixture pollution, no mocking. The determinism test is the strongest: same config, two runs, exact equality.

- **Isolation**: All 11 files use `tempfile::tempdir()` for every writable repo fixture. The `path_partition.rs` tests use the read-only `simple-rust` fixture via `EphemeralForCheck` — no writes to the fixture directory. No test reads `.tekhton/`, `.claude/`, build artifacts, or any mutable project state file.

- **Scope alignment**: No orphaned imports. The deleted `.tekhton/test_dedup.fingerprint` is not referenced in any test file under audit. All CLI command paths match `main.rs` dispatch.

- **No weakening**: The only modification to pre-existing test content in `boundaries_stub.rs` strengthened the assertions — replacing empty stubs with real behavioral contracts.
