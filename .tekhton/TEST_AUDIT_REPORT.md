## Test Audit Report

### Audit Summary
Tests audited: 3 files, 18 test functions
Verdict: PASS

### Findings

#### COVERAGE: ConfigError::Io variant not tested
- File: crates/sdi-config/tests/config_errors.rs
- Issue: The `config_error_variants_are_debug_formattable` test constructs three of four `ConfigError` variants (Parse, InvalidValue, MissingExpiresOnOverride) but omits the `Io` variant, which wraps `std::io::Error` via `#[from]`. No test constructs `ConfigError::Io` or checks its Display output (format: `"I/O error reading config: {0}"`). This variant is reachable via `ConfigError::Io(std::io::Error::new(...))` and could be included in the existing parametric tests.
- Severity: LOW
- Action: Add a test constructing `ConfigError::Io(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "cannot read config"))` and asserting `msg.contains("cannot read config")`. Include the variant in `config_error_variants_are_debug_formattable`.

#### COVERAGE: Weak Display assertions in pipeline_smoke.rs
- File: crates/sdi-core/tests/pipeline_smoke.rs:15 and :37
- Issue: `no_grammars_available_is_constructable_and_has_message` asserts `!msg.is_empty()` — any single character would pass. The actual message is `"no grammar available for any detected language in the repository"` (error.rs:16). `analysis_error_accessible_via_prelude` has the same issue: the assertion `!err.to_string().is_empty()` does nothing beyond confirming the code compiled. In both cases a substring check on the actual message text would meaningfully distinguish a regression (e.g., an accidental `#[error("")]` change) from a passing test.
- Severity: LOW
- Action: Replace `assert!(!msg.is_empty())` with `assert!(msg.contains("grammar"), "...")` and similarly for the prelude test. The file's own doc comment already explains this is a placeholder, so the weakness is partially justified by milestone scope; nonetheless the fix is one line per test.

#### COVERAGE: Exit-code assertions partially duplicate existing tests
- File: crates/sdi-core/tests/pipeline_smoke.rs:47 and :55
- Issue: `analysis_error_exit_code_is_three` and `threshold_exceeded_exit_code_is_ten_and_exclusive_to_check` test `ExitCode::as_i32()` values. The coder's pre-existing `crates/sdi-core/tests/exit_code_contract.rs` (6 tests) already covers every `ExitCode` variant via both `as i32` cast and `.as_i32()` / `i32::from()`. The duplication is harmless (the tester's tests use `.as_i32()` which the contract file also covers in `as_i32_matches_cast`) but adds no new surface coverage.
- Severity: LOW
- Action: No immediate action required. If `pipeline_smoke.rs` is refactored when Pipeline arrives in M06, consolidate exit-code assertions into `exit_code_contract.rs` and keep `pipeline_smoke.rs` focused on Pipeline-level integration.

#### NAMING: File name misrepresents current test scope
- File: crates/sdi-core/tests/pipeline_smoke.rs
- Issue: The file is named `pipeline_smoke.rs` but contains no Pipeline tests — it tests `AnalysisError` variants and `ExitCode` values. The file's own module doc explains the mismatch ("Placeholder tracking file … Pipeline arrives in M06"), but a reader navigating the test directory will expect Pipeline invocations. This creates a discovery gap that grows as M06 approaches.
- Severity: LOW
- Action: Either rename to `analysis_error_and_exit_code.rs` now and add the Pipeline tests to a new `pipeline_smoke.rs` in M06, or add a `// TODO(M06): expand with Pipeline invocations` comment at the top (already partially done) and accept the name until M06. Either is acceptable for M01.

#### None: Assertion honesty, implementation exercise, test weakening, isolation
- All 18 tests construct or call real implementation types (`ConfigError`, `Config`, `AnalysisError`, `ExitCode`) without mocks. Assertions are derived from actual function calls; no hard-coded values appear that are disconnected from implementation logic. No existing tests were modified (all three files are new), so no weakening occurred. No test reads `.tekhton/` files, `.claude/` logs, or any other mutable project-state artifact — all fixture data is constructed inline.
