## Planned Tests
- [x] `crates/sdi-config/tests/config_errors.rs` — ConfigError variants display correctly (MissingExpiresOnOverride, InvalidValue, Parse)
- [x] `crates/sdi-config/tests/serde_round_trip.rs` — Config::default() survives serde_json round-trip; parametric seed/retention property checks
- [x] `crates/sdi-core/tests/pipeline_smoke.rs` — Placeholder tracking file: AnalysisError and ExitCode accessible from sdi-core (Pipeline arrives in M06)

## Test Run Results
Passed: 18  Failed: 0

## Bugs Found
None

## Files Modified
- [x] `crates/sdi-config/tests/config_errors.rs`
- [x] `crates/sdi-config/tests/serde_round_trip.rs`
- [x] `crates/sdi-core/tests/pipeline_smoke.rs`
- [x] `crates/sdi-config/Cargo.toml` — added serde_json dev-dependency; proptest commented out pending MSRV resolution
