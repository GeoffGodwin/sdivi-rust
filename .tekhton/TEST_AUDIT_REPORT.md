## Test Audit Report

### Audit Summary
Tests audited: 6 files, 45 test functions
Verdict: CONCERNS

---

### Findings

#### ISOLATION: Two env-var tests read from the live project filesystem
- File: `crates/sdi-config/tests/precedence.rs:86-93` (`env_var_snapshot_dir_overrides_file_config`) and `precedence.rs:97-106` (`env_var_no_color_sets_never`)
- Issue: Both tests call `load_or_default(std::path::Path::new("."))`. `load_or_default` reads `./.sdi/config.toml` from the real project directory when that file exists. Milestone 2 introduces `sdi init`, which creates `.sdi/config.toml` in the current directory. Once any developer (or CI runner) runs `sdi init` at the project root, both tests acquire a dependency on that file's contents. If the file contains a threshold override with a missing `expires`, `load_or_default` returns `Err` and `.unwrap()` panics — the test fails for reasons entirely unrelated to the env-var behavior under test. Neither test snapshots or restores `SDI_CONFIG_PATH`, so an inherited value from the caller's environment would further redirect the config read. The two long-form XDG tests added immediately below them in the same file (`load_or_default_reads_global_config_via_xdg_config_home`, `global_config_is_lower_precedence_than_project_config_via_load_or_default`) correctly avoid all of this by using `tempfile::TempDir` as the repo root and by snapshotting/restoring every relevant env var. The two shorter tests must be brought up to the same standard.
- Severity: HIGH
- Action: In both tests, replace `std::path::Path::new(".")` with a `tempfile::TempDir::new().unwrap()` repo root. Also snapshot and restore `SDI_CONFIG_PATH` before/after the `load_or_default` call, following the pattern already established in `load_or_default_reads_global_config_via_xdg_config_home`.

#### COVERAGE: `AnalysisError::Config` variant untested
- File: `crates/sdi-core/tests/pipeline_smoke.rs`
- Issue: `AnalysisError` has three variants — `Io`, `NoGrammarsAvailable`, and `Config` (wrapping `sdi_config::ConfigError` via `#[from]`, defined at `crates/sdi-core/src/error.rs:21`). The first two are tested; `Config` is not. Its Display format is `"configuration error: {0}"`. The file's stated purpose is to verify that `AnalysisError` is correctly exported and reachable from `sdi_core` before Pipeline arrives in M06; leaving one of three variants unexercised is a gap in that coverage.
- Severity: LOW
- Action: Add a test constructing `AnalysisError::Config(sdi_config::ConfigError::Parse("bad".into()))` and asserting `err.to_string().contains("configuration error")`.

#### COVERAGE: Weak Display assertions in `pipeline_smoke.rs`
- File: `crates/sdi-core/tests/pipeline_smoke.rs:18` (`no_grammars_available_is_constructable_and_has_message`) and `:40` (`analysis_error_accessible_via_prelude`)
- Issue: Both tests assert `!msg.is_empty()` — any single character satisfies this. The actual message is `"no grammar available for any detected language in the repository"` (error.rs:17). A substring check against a key word (e.g., `"grammar"`) would catch a regression from an accidental `#[error("")]` or `#[error(" ")]` change; `!is_empty()` would not. The weakness is partially justified by the placeholder milestone scope, but the fix is one line each.
- Severity: LOW
- Action: Replace `assert!(!msg.is_empty(), ...)` with `assert!(msg.contains("grammar"), ...)` in `no_grammars_available_is_constructable_and_has_message`, and similarly for the prelude test.

#### COVERAGE: `ConfigError::Io` variant still untested in config_errors.rs
- File: `crates/sdi-config/tests/config_errors.rs`
- Issue: The prior audit (M01) flagged that `config_error_variants_are_debug_formattable` omits `ConfigError::Io`. That finding was not addressed this run: the `Io` variant (`#[error("I/O error reading config: {0}")]`) still has no Display assertion and is still absent from the Debug-formattability test. `BoundarySpec::load` surfaces `ConfigError::Io` on read failures; the path is exercisable without filesystem mocking via `std::io::Error::new(...)`.
- Severity: LOW
- Action: Add `ConfigError::Io(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "cannot read"))` to the `config_error_variants_are_debug_formattable` loop, and add a standalone test asserting `err.to_string().contains("cannot read")`.

---

### Rubric Assessment (non-finding dimensions)

**Assertion Honesty — PASS.** All assertions derive from actual function return values. Hard-coded check values (`random_seed == 42`, `retention == 100`, `leiden_gamma == 1.2`, etc.) match `Config::default()` and the fixture files verbatim. No tautological assertions detected across any of the 45 test functions.

**Edge Case Coverage — PASS.** `threshold_overrides.rs` covers missing, malformed-string, integer, and boolean `expires` values, plus the expired/valid split and mixed-expiry batches. `precedence.rs` covers absent files, empty TOML, list replacement, and full global/project/env-var precedence ordering. `sdi_py_compat.rs` covers missing file (`None`), valid file, and two distinct invalid-YAML shapes.

**Implementation Exercise — PASS.** All test files call real implementation code. No dependency mocking observed. Fixture files (`sdi_py_config.toml`, `sdi_py_boundaries.yaml`) exist and their values match exactly what the tests assert.

**Test Weakening — PASS.** The tester added new tests throughout. No assertions were removed or broadened in any pre-existing test function. `pipeline_smoke.rs` gained two new functions (`io_variant_wraps_std_io_error`, `analysis_error_accessible_via_prelude`) over its prior state; `config_errors.rs` and `serde_round_trip.rs` gained additional parametric coverage; no functions were deleted or weakened.

**Test Naming — PASS.** All 45 test function names encode both the scenario and the expected outcome (e.g., `expired_override_is_silently_ignored`, `integer_expires_returns_invalid_value`, `global_config_wins_when_project_absent`, `config_default_serde_round_trip_is_identity`). No vague names detected.

**Scope Alignment — PASS.** No orphaned imports. The deleted `.tekhton/test_dedup.fingerprint` is not referenced by any test file. The duplicate entries in the audit context's modified-file list are list repetitions — both `config_errors.rs` and `serde_round_trip.rs` exist once on disk and are distinct files.
