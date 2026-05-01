## Test Audit Report

### Audit Summary
Tests audited: 2 files, 10 test functions

- `crates/sdi-pipeline/src/commit_extract.rs` — inline `#[cfg(test)] mod tests`: 7 unit tests
  (`utc_passthrough`, `negative_offset_shifts_forward`, `positive_offset_shifts_back`,
  `malformed_returns_none`, `positive_offset_crosses_day_boundary_backward`,
  `negative_offset_crosses_day_boundary_forward`, `commit_date_parse_failed_when_date_unparseable`)
- `crates/sdi-pipeline/tests/commit_extract_security.rs` — 3 integration tests
  (`resolve_ref_includes_double_dash_separator`, `stderr_truncation_prevents_information_leakage`,
  `extract_commit_tree_succeeds_with_tar_flags`)

Note: `.tekhton/ARCHITECTURE_LOG.md` and `.tekhton/NON_BLOCKING_LOG.md` are documentation
files, not test files — audited for consistency with implementation, not for test quality.
The three freshness-sample files (`tests/fixtures/high-entropy/src/lib.rs`,
`tests/fixtures/high-entropy/src/state.rs`, `tests/fixtures/leiden-graphs/large/metadata.json`)
are fixture data with no embedded test code; no issues found.

Implementation files cross-referenced:
- `crates/sdi-pipeline/src/commit_extract.rs` (lines 40-68, 94-151, 155-161)
- `crates/sdi-core/src/compute/boundaries.rs` (lines 136, 152-169)

Deleted test file confirmed: `tests/historical_commit_lifecycle.rs` — intentional removal per
audit context; no orphaned imports detected in remaining test files.

Verdict: **CONCERNS**

---

### Findings

#### INTEGRITY: Test suite was not executed — zero-run report
- File: `.tekhton/TESTER_REPORT.md`
- Issue: The report records "Passed: 0  Failed: 0" while carrying checkmarks for five
  verified items. A count of zero for both pass and fail is mechanically impossible if any
  test ran to completion — even a single panic or compile error would produce a non-zero
  failed count. The 10 test functions across both test files under audit were never run.
  Any assertion failures, compile errors, or wrong behaviors introduced in this cycle are
  currently undetected.
- Severity: HIGH
- Action: Run `cargo test -p sdi-pipeline` and record the actual pass/fail totals in
  TESTER_REPORT.md before closing this cycle. Fix any failures before merging.

#### SCOPE: NON_BLOCKING_LOG.md records two security fixes absent from the implementation
- File: `.tekhton/NON_BLOCKING_LOG.md` (resolved item 3); `crates/sdi-pipeline/src/commit_extract.rs:46,119-124`
- Issue: The resolved entry states "(1) added `--` separator to `git rev-parse` at line 46"
  and "(2) added `--no-absolute-filenames` flag to `tar` at line 120." Neither fix is present:
  line 46 of `commit_extract.rs` reads `.args(["rev-parse", "--verify", reference])` with no
  `--` separator, and the `tar` command (lines 119-124) contains only `.arg("-xC")` — no
  `--no-absolute-filenames`. Only the third sub-item (the `truncate_stderr` helper) was
  actually implemented. CODER_SUMMARY.md compounds the inconsistency by recording item 3 as
  "No code change," which contradicts both the NON_BLOCKING_LOG.md entry and the presence
  of `truncate_stderr` in the implementation. The audit trail is inaccurate on two counts.
- Severity: HIGH
- Action: Either (a) apply both missing security fixes (`--` separator in `resolve_ref_to_sha`
  and `--no-absolute-filenames` in `extract_commit_tree`) and reopen the NON_BLOCKING_LOG.md
  entry as truly fixed, or (b) correct the NON_BLOCKING_LOG.md entry to accurately reflect
  that only `truncate_stderr` was applied and the `--` / `--no-absolute-filenames` items
  remain open. Update CODER_SUMMARY.md to match whichever path is taken.

#### EXERCISE: `resolve_ref_includes_double_dash_separator` may vacuously pass without the fix
- File: `crates/sdi-pipeline/tests/commit_extract_security.rs:7-31`
- Issue: The test asserts that stderr does not contain "unknown option" when `--invalid-ref`
  is passed to `resolve_ref_to_sha`. Git on most distributions treats `--invalid-ref` as a
  revspec rather than a flag even when no `--` separator is present, so git produces output
  like "fatal: not a valid object name" rather than "unknown option". The test would therefore
  pass on a typical git installation regardless of whether `--` is present in the command —
  making it unable to detect the absence of the fix it claims to verify.
- Severity: MEDIUM
- Action: Replace the "does not contain 'unknown option'" assertion with a direct inspection
  of the spawned `git` command's argv (e.g., by exposing a test-only helper that returns the
  argument list, or by constructing a mock git script that echoes its argv and verifying `--`
  appears between `--verify` and the reference). Alternatively, leave the functional
  assertion as-is and add an explicit comment acknowledging it does not prove the `--` is
  present — only that git fails cleanly.

#### EXERCISE: `extract_commit_tree_succeeds_with_tar_flags` does not verify the security flag
- File: `crates/sdi-pipeline/tests/commit_extract_security.rs:109-152`
- Issue: The test verifies that `extract_commit_tree` succeeds and `test.txt` is present in
  the output directory. It does not verify that `--no-absolute-filenames` was passed to tar.
  A comment in the test body explicitly acknowledges this gap ("A comprehensive security test
  would construct a malicious tar with absolute paths"). Since `--no-absolute-filenames` is
  also absent from the implementation (see SCOPE finding), the test currently validates a
  correct-but-insecure extraction and reports it as a security fix verification.
- Severity: MEDIUM
- Action: If the `--no-absolute-filenames` fix is applied, also add a test that constructs a
  synthetic tar archive with an absolute-path entry and verifies the file is not created
  outside the target directory. If the flag is intentionally deferred, document the open risk
  in the NON_BLOCKING_LOG.md rather than claiming it as fixed.

#### NAMING: `commit_date_parse_failed_when_date_unparseable` does not test `commit_date_iso`
- File: `crates/sdi-pipeline/src/commit_extract.rs:237-245`
- Issue: The test name implies it verifies that `commit_date_iso` returns a
  `CommitDateParseFailed` error when git returns an unparseable date string. The body
  only calls the private `normalize_to_utc` helper and asserts it returns `None`. The
  public function `commit_date_iso` and its `ok_or_else(|| CommitDateParseFailed {...})`
  mapping (line 87) are never exercised. The behavior the name promises is already partially
  covered by `malformed_returns_none`; this test adds no distinct coverage.
- Severity: MEDIUM
- Action: Either rename the test to `normalize_to_utc_returns_none_for_bad_input` to match
  what it actually does, or replace it with a test that calls `commit_date_iso` against a
  git repo where the command succeeds but returns a malformed date (constructed via a
  fake-git script in a tempdir), verifying the `CommitDateParseFailed` variant is returned.

#### INTEGRITY: NON_BLOCKING_LOG.md and CODER_SUMMARY.md give inconsistent accounts of item 3
- File: `.tekhton/NON_BLOCKING_LOG.md` (resolved item 3); `.tekhton/CODER_SUMMARY.md` (row 3)
- Issue: NON_BLOCKING_LOG.md marks item 3 as "FIXED in sweep" and lists three specific code
  changes. CODER_SUMMARY.md records the same item as "Informational; handled by security
  pipeline. No code change." Both documents cannot be correct. The actual state (only
  `truncate_stderr` was added) matches neither description exactly, creating an audit trail
  that will mislead any future reviewer who reads only one of the two files.
- Severity: MEDIUM
- Action: Reconcile both documents so they agree on what was done. The NON_BLOCKING_LOG.md
  entry should list only the sub-items that were actually implemented; CODER_SUMMARY.md
  should reflect the actual file changes rather than "No code change."
