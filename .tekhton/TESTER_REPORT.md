## Planned Tests
- [x] `crates/sdi-config/tests/config_errors.rs` — ConfigError variants display correctly (MissingExpiresOnOverride, InvalidValue, Parse)
- [x] `crates/sdi-config/tests/serde_round_trip.rs` — Config::default() survives serde_json round-trip; parametric seed/retention property checks
- [x] `crates/sdi-core/tests/pipeline_smoke.rs` — Placeholder tracking file: AnalysisError and ExitCode accessible from sdi-core (Pipeline arrives in M06)
- [x] `crates/sdi-config/tests/sdi_py_compat.rs` — BoundarySpec::load with invalid YAML returns ConfigError::BoundaryParse
- [x] `crates/sdi-config/tests/threshold_overrides.rs` — validate_and_prune_overrides with non-String expires returns ConfigError::InvalidValue
- [x] `crates/sdi-config/tests/precedence.rs` — load_or_default reads global config via $XDG_CONFIG_HOME env var
- [x] `crates/sdi-parsing/tests/proptest.rs` — add Unicode arbitrary-content proptest to exercise char-safe truncation in collect_hints
- [x] `crates/sdi-parsing/tests/extract_behavior.rs` — new file: pub use import extraction, export double-count assertion, hint truncation boundary test

## Test Run Results
Passed: 36  Failed: 3

## Bugs Found
- BUG: [crates/sdi-lang-rust/src/extract.rs:108] collect_hints truncation uses last char whose start < 256 then extends by char width; can produce 257-byte hint text violating the documented 256-byte cap
- BUG: [crates/sdi-lang-rust/src/extract.rs:66] extract_exports recurses into EXPORTABLE_KINDS children without stopping; pub fn nested inside pub mod appears as a top-level export instead of only being reachable as mod::fn
- BUG: [crates/sdi-lang-rust/src/extract.rs:37] extract_imports strip_prefix("use ") fails on pub use declarations; captured import is "pub use foo::bar" not "foo::bar"
- BUG: [crates/sdi-parsing/tests/memory_invariant.rs:22] tree_counter_zero_after_each_parse reads shared global ACTIVE_TREES without isolation; fails under default parallel test execution due to race with parse_many_large_files_completes (passes with --test-threads=1)

## Files Modified
- [x] `crates/sdi-config/tests/config_errors.rs`
- [x] `crates/sdi-config/tests/serde_round_trip.rs`
- [x] `crates/sdi-core/tests/pipeline_smoke.rs`
- [x] `crates/sdi-config/Cargo.toml` — added serde_json dev-dependency; proptest commented out pending MSRV resolution
- [x] `crates/sdi-config/tests/sdi_py_compat.rs`
- [x] `crates/sdi-config/tests/threshold_overrides.rs`
- [x] `crates/sdi-config/tests/precedence.rs`
- [x] `crates/sdi-parsing/tests/proptest.rs`
- [x] `crates/sdi-parsing/tests/extract_behavior.rs`
