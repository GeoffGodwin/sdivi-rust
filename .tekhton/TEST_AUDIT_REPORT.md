## Test Audit Report

### Audit Summary
Tests audited: 8 files, 62 test functions (4 proptest blocks × 256 cases + 58 unit tests)
Verdict: CONCERNS

---

### Findings

#### INTEGRITY: Failing test committed without #[ignore] — pub use import path
- File: `crates/sdi-parsing/tests/extract_behavior.rs:50`
- Issue: `pub_use_import_path_excludes_pub_keyword` asserts `!import.starts_with("pub ")`. The implementation in `crates/sdi-lang-rust/src/extract.rs:36-39` falls back to `text.trim()` when `strip_prefix("use ")` fails on `"pub use crate::foo::Bar;"`, yielding the import string `"pub use crate::foo::Bar"`. The assertion is `false` on every run — this is one of the three confirmed suite failures. The inline comment correctly identifies the deficiency but the test carries no `#[ignore]` attribute, so the failure appears as a normal regression in CI.
- Severity: HIGH
- Action: Add `#[ignore = "known bug: extract_imports does not strip 'pub' prefix from pub use (extract.rs:37)"]` to this function. Add a companion passing test that asserts the actual current captured value `"pub use crate::foo::Bar"` to detect regressions. Remove `#[ignore]` when the implementation is fixed.

#### INTEGRITY: Failing test committed without #[ignore] — nested pub fn leaks into top-level exports
- File: `crates/sdi-parsing/tests/extract_behavior.rs:126`
- Issue: `pub_fn_inside_pub_mod_not_in_top_level_exports` asserts `!record.exports.contains(&"inner".to_string())`. The implementation in `crates/sdi-lang-rust/src/extract.rs:57-73` does not stop recursing when it finds an `EXPORTABLE_KINDS` node: after collecting `"outer"` (mod_item), it unconditionally enqueues all children, eventually visiting the nested `function_item` `"inner"` which is public and also in `EXPORTABLE_KINDS`. So `"inner"` lands in `exports`, the assertion is `false`, and the test always fails. No `#[ignore]` attribute is present.
- Severity: HIGH
- Action: Add `#[ignore = "known bug: extract_exports recurses past exportable nodes; nested pub fn appears as top-level export (extract.rs:66)"]`. Add a companion passing test asserting both `"outer"` and `"inner"` are present in exports to serve as a regression anchor. Remove `#[ignore]` when `extract_exports` stops recursing into the children of matched exportable nodes.

#### COVERAGE: Unicode truncation test does not exercise the documented 257-byte edge case
- File: `crates/sdi-parsing/tests/extract_behavior.rs:153`
- Issue: `collect_hints_long_unicode_text_truncated_at_char_boundary` tests with `"á"` (2 bytes) × 200 = 400 bytes. Every `'á'` starts at an even byte index, so `take_while(|(i, _)| *i < 256)` terminates at index 254 and `end = 254 + 2 = 256` — exactly at the cap. The assertion `hint.text.len() <= 256` passes. The tester-reported bug (a two-byte char whose start byte index is 255 produces `end = 257`, exceeding the documented cap) is never triggered by this input. The proptest in `proptest.rs` also does not bound-check hint length.
- Severity: MEDIUM
- Action: Add a targeted case whose pattern node source text consists of 255 ASCII characters immediately followed by a two-byte Unicode character (e.g. `"a".repeat(255) + "á"` embedded in a match arm), then assert `hint.text.len() <= 256`. This will reproduce the documented bug and guard against off-by-one regressions after the fix.

#### COVERAGE: `no_export_name_appears_more_than_once` does not detect the documented recursion bug
- File: `crates/sdi-parsing/tests/extract_behavior.rs:110`
- Issue: The test comment describes a guard against extract_exports "double-counting" due to unchecked recursion. With input `pub mod outer { pub fn inner() {} }`, the current bug produces `exports = ["outer", "inner"]` — two *different* names, zero duplicates. The `HashSet::insert` loop therefore always passes, regardless of the recursion bug. The test does not detect what it claims to detect; it is superseded by `pub_fn_inside_pub_mod_not_in_top_level_exports` for that purpose.
- Severity: MEDIUM
- Action: Either rename to `export_names_contain_no_duplicates` and remove the misleading double-counting comment (leaving the test as a uniqueness guard only), or delete it as redundant with the stronger `pub_fn_inside_pub_mod_not_in_top_level_exports` test.

#### ISOLATION: Two env-var tests use `Path::new(".")` without controlling the file environment
- File: `crates/sdi-config/tests/precedence.rs:86` (`env_var_snapshot_dir_overrides_file_config`) and `precedence.rs:98` (`env_var_no_color_sets_never`)
- Issue: Both tests call `load_or_default(std::path::Path::new("."))`. During `cargo test`, `.` resolves to `crates/sdi-config`. `load_or_default` also resolves a global config via `dirs::config_dir()` (typically `~/.config/sdi/config.toml`), which may exist on a developer's machine. Additionally, neither test snapshots or restores `SDI_CONFIG_PATH`: if that variable is set in the caller's environment, `load_or_default` reads that path instead of `.sdi/config.toml`. The assertions likely hold in the common case (env var overrides apply after file loading), but a malformed global or `SDI_CONFIG_PATH`-redirected config can cause `.unwrap()` to panic with an error unrelated to the tested behaviour. The immediately following `load_or_default_reads_global_config_via_xdg_config_home` and `global_config_is_lower_precedence_than_project_config_via_load_or_default` tests in the same file correctly use `tempfile::TempDir` and snapshot all relevant env vars.
- Severity: MEDIUM
- Action: Replace `Path::new(".")` with a `tempfile::TempDir::new().unwrap()` repo root. Snapshot and restore `SDI_CONFIG_PATH`, `XDG_CONFIG_HOME`, and `NO_COLOR` in both tests, following the pattern in `load_or_default_reads_global_config_via_xdg_config_home`.

#### NAMING: `no_export_name_appears_more_than_once` misrepresents what it checks
- File: `crates/sdi-parsing/tests/extract_behavior.rs:110`
- Issue: The test name and comment both reference "double-counting" due to the extract_exports recursion bug. The assertion only checks that no name appears twice in the list — a uniqueness check, not a correctness check. A reader relying on this name as documentation will incorrectly believe the recursion scenario is covered.
- Severity: LOW
- Action: Rename to `export_names_contain_no_duplicates` and drop the double-counting comment. See COVERAGE finding above for the stronger guard needed.

---

### Rubric Assessment (non-finding dimensions)

**Assertion Honesty — PASS.** Assertions are grounded in actual function return values. Hard-coded check values (`random_seed == 42`, `retention == 100`, `leiden_gamma == 1.2`, exit codes 3 and 10) match `Config::default()`, `ExitCode` repr discriminants, and fixture files verbatim. No tautological or always-true assertions detected among the passing tests.

**Edge Case Coverage — PASS.** `threshold_overrides.rs` covers missing, malformed-string-format, integer, and boolean `expires` values, plus expired/valid split and mixed-expiry batches. `precedence.rs` covers absent files, empty TOML, list replacement, global/project/env-var precedence. `sdi_py_compat.rs` covers missing file, valid sdi-py fixture, and two distinct invalid-YAML shapes.

**Implementation Exercise — PASS.** All test files call real implementation code. No dependency mocking observed. Fixture files (`sdi_py_config.toml`, `sdi_py_boundaries.yaml`) exist at `crates/sdi-config/tests/fixtures/` and their values exactly match the assertions in `sdi_py_compat.rs`.

**Test Weakening — PASS.** The tester added new tests throughout. No assertions were removed or broadened in any pre-existing function. `pipeline_smoke.rs`, `config_errors.rs`, and `serde_round_trip.rs` all gained additional coverage over their prior state.

**Scope Alignment — PASS.** No orphaned imports. The deleted `.tekhton/test_dedup.fingerprint` is not referenced by any test file. The `sdi-config/Cargo.toml` change (adding `serde_json` dev-dependency, commenting out `proptest` pending MSRV) is consistent with the tests: `serde_round_trip.rs` uses `serde_json`; no `proptest` import appears in `sdi-config` tests.
