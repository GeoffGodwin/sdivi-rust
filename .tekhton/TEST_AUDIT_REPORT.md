## Test Audit Report

### Audit Summary
Tests audited: 8 files, 41 test functions (version.rs:2, exit_codes.rs:12, check_thresholds.rs:6,
stdout_stderr_split.rs:4, show_format.rs:5, trend_format.rs:5, boundaries_stub.rs:3, no_color.rs:4)
Verdict: CONCERNS

---

### Findings

#### COVERAGE: Exit code 10 never triggered
- File: `crates/sdi-cli/tests/exit_codes.rs` (entire file)
- Issue: `exit_codes.rs` was created specifically to satisfy CLAUDE.md Testing Strategy §CLI:
  "exhaustively covering 0/1/2/3/10." Every `check` test in that file asserts `.success()` (exit 0).
  No test forces `sdi check` to exit 10 (ThresholdExceeded). Exit 10 is the only exit code unique
  to this milestone and is the primary behavioral invariant of `sdi check`. One achievable approach:
  write a `.sdi/config.toml` into the temp repo with `pattern_entropy_rate = 0.0` and
  `convention_drift_rate = 0.0`, create a prior snapshot, then run `sdi check` — any non-null delta
  on an empty repo will breach a zero threshold.
- Severity: HIGH
- Action: Add `check_exits_ten_when_threshold_breached` to `exit_codes.rs`. Assert `.code(10)` (not
  just `.failure()`). Optionally cross-check with `--format json` to verify `exit_code == 10` and
  `exceeded` is non-empty.

#### COVERAGE: Exit codes 2 and 3 absent from exit_codes.rs
- File: `crates/sdi-cli/tests/exit_codes.rs` (entire file)
- Issue: CLAUDE.md mandates "exhaustively covering 0/1/2/3/10." Exit 2 (ConfigError — e.g., a
  `[thresholds.overrides.x]` block missing the required `expires` field, CLAUDE.md Rule 12) and
  exit 3 (all detected languages lack grammars — testable with a repo containing only `.xyzunknown`
  files and `languages = ["xyzunknown"]` in config) are absent. Since `exit_codes.rs` is a new file
  with declared exhaustive scope, these gaps are in-scope for M09.
- Severity: MEDIUM
- Action: Add `config_error_exits_two` (write a malformed config.toml to the temp repo's `.sdi/`
  directory before invoking `sdi snapshot`) and `snapshot_exits_three_on_all_unknown_languages`
  (repo with only unrecognized file extensions) to `exit_codes.rs`.

#### NAMING: `no_color_flag_suppresses_ansi_in_show` sets no flag or env variable
- File: `crates/sdi-cli/tests/no_color.rs:82`
- Issue: The test name says "flag_suppresses" but neither `--no-color` nor `NO_COLOR=1` is set.
  The test runs plain `sdi show` and asserts no ANSI codes — it verifies that the default text
  formatter produces plain text, not that flag-based suppression works. Additionally, the
  `--no-color` CLI flag (CLAUDE.md Config table: `--no-color → output.color = "never"`) is not
  exercised by any test in the file.
- Severity: MEDIUM
- Action: Rename the test to `default_show_output_has_no_ansi_codes` to match actual behavior.
  Once any formatter gains color output, add a proper `no_color_flag_suppresses_ansi_in_show` that
  passes `.arg("--no-color")` and a companion test that verifies ANSI codes ARE present without
  the flag (so the suppression is observable).

#### NAMING: Tests named `_exits_one` use `.failure()` not `.code(1)`
- File: `crates/sdi-cli/tests/exit_codes.rs:128` (`show_no_snapshots_exits_one`) and
  `crates/sdi-cli/tests/exit_codes.rs:141` (`diff_nonexistent_file_exits_one`)
- Issue: `.failure()` asserts any non-zero exit code. The test names encode exit code 1 specifically.
  If the implementation changed to exit 2 for these paths (e.g., if a future change reclassifies
  "no snapshots found" as a ConfigError), the tests would still pass, masking a regression.
- Severity: LOW
- Action: Replace `.failure()` with `.code(1)` in both tests, or rename the tests to
  `_exits_nonzero` if exit code 1 is not contractually required for these error paths.

#### COVERAGE: `coupling_slope` key-presence assertion is always true
- File: `crates/sdi-cli/tests/trend_format.rs:87-90` and
  `crates/sdi-cli/tests/stdout_stderr_split.rs:131-134`
- Issue: `parsed.get("coupling_slope").is_some()` checks that the key exists in the JSON object.
  `TrendResult` derives `Serialize` without `skip_serializing_if`, so the key is always present —
  whether the value is a number or `null`. The assertion passes even if `coupling_slope` serialized
  as `null` (which would indicate the trend computation failed to produce a slope). With ≥2
  snapshots the value should be `Some(f64)`, so a stronger assertion is achievable.
- Severity: LOW
- Action: Replace with `assert!(parsed["coupling_slope"].as_f64().is_some(), "coupling_slope must
  be a non-null number with ≥2 snapshots; got: {parsed}")` in both occurrences.

---

### Clean Findings (no issues)

- **version.rs**: `env!("CARGO_PKG_VERSION")` fix is correct; no hardcoded version string remains.
  Both tests invoke the real binary via `assert_cmd`; no mocking.

- **check_thresholds.rs**: All 6 tests exercise real pipeline behavior. Assertions are specific
  (before/after snapshot count comparison, exact JSON field presence, exit_code consistency with
  process status). `check_json_exit_code_matches_process_exit` is a particularly strong test —
  it cross-checks the JSON field against the OS-level exit code, catching any mismatch between
  the two output channels.

- **stdout_stderr_split.rs**: All 4 tests verify the `jq '.'` piping contract (stdout is valid
  JSON; stderr does not parse as JSON). The `diff_json_stdout_is_valid_json` test reads actual
  on-disk snapshot files rather than hard-coded paths. The stderr guard logic
  `is_err() || is_empty()` is correct — it allows informational tracing output while blocking
  accidental JSON on stderr.

- **show_format.rs**: `show_no_id_selects_latest` reads the actual last file from disk and
  cross-checks the `commit` field against the returned snapshot. This is a meaningful, non-trivial
  assertion. `show_with_id_selects_specific_snapshot` exercises the full `read_snapshot_by_id`
  path with a real filename stem.

- **trend_format.rs**: `trend_zero_snapshots_prints_friendly_message` and
  `trend_one_snapshot_prints_friendly_message` both check the specific substring "not enough
  snapshots" against the actual implementation message
  `"sdi trend: not enough snapshots (need ≥2)"`. `trend_last_n_larger_than_available_silently_clamps`
  and `trend_last_n_selects_tail` verify the `--last` clamping behavior described in the
  CODER_SUMMARY via actual `snapshot_count` values in JSON output.

- **boundaries_stub.rs**: Asserting `stdout.is_empty()` alongside
  `stderr.contains("not implemented")` correctly captures the M09 stub contract without
  over-specifying M10 behavior.

- **no_color.rs (tests 1–3)**: `no_color_env_suppresses_ansi_in_show`,
  `no_color_env_suppresses_ansi_in_check`, and `no_color_env_suppresses_ansi_in_trend_insufficient_snapshots`
  all set `NO_COLOR=1` via `.env("NO_COLOR", "1")` and verify stdout is free of `\x1b[` sequences.
  These serve as regression guards for when color support is added.

- **Isolation**: All 8 files use `tempfile::tempdir()` for every repo fixture. No test reads
  `.tekhton/`, `.claude/`, any build artifact, or any mutable project state file.

- **Scope alignment**: No orphaned imports. The deleted `.tekhton/test_dedup.fingerprint` is not
  referenced in any test file. All command paths match what `main.rs` now dispatches.

- **No weakening**: The only modification to a pre-existing test (`version.rs`) strengthened it —
  `contains("0.0.11")` → `contains(env!("CARGO_PKG_VERSION"))` eliminates future stale-string
  failures on version bumps while remaining semantically equivalent at the current version.

- **Assertion honesty**: No `assert_eq!(x, x)`, no `assertTrue(true)`, no hard-coded values
  detached from implementation logic. JSON field names (`exit_code`, `exceeded`, `summary`,
  `snapshot_count`, `coupling_slope`, `snapshot_version`) all correspond to real struct fields
  in `ThresholdCheckResult`, `TrendResult`, and `Snapshot` respectively.
