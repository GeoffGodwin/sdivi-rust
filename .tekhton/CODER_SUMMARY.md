# Coder Summary
## Status: COMPLETE

## What Was Implemented
### From prior run (already in place on continuation):
- `crates/sdi-config/src/thresholds.rs` (NEW): date utilities, expires validation, prune expired overrides
- `crates/sdi-config/src/boundary.rs` (NEW): BoundarySpec + BoundaryDef structs, YAML reader
- `crates/sdi-config/src/load.rs` (MODIFIED): real 5-level precedence chain, TOML merging, env var overrides, unknown-key warnings
- `crates/sdi-config/src/error.rs` (MODIFIED): added BoundaryParse variant
- `crates/sdi-config/src/lib.rs` (MODIFIED): export new types
- `crates/sdi-config/Cargo.toml` (MODIFIED): added toml, serde_yaml, dirs dependencies
- `crates/sdi-cli/src/commands/init.rs` (NEW): sdi init command
- `crates/sdi-cli/src/commands/mod.rs` (MODIFIED): registered init subcommand
- `crates/sdi-cli/src/main.rs` (MODIFIED): Commands enum and subcommand dispatch
- `crates/sdi-cli/Cargo.toml` (MODIFIED): added walkdir
- Tests: precedence.rs, threshold_overrides.rs, sdi_py_compat.rs, init.rs (all NEW)
- Fixtures: sdi_py_config.toml, sdi_py_boundaries.yaml (NEW)

### This run (continuation):
- Fixed `tempfile` version pin in `Cargo.toml` (from `"3"` to `">=3.0, <3.20"`) to avoid `getrandom 0.4.2` (edition2024) incompatibility with the Rust 1.75 environment; updated `Cargo.lock` accordingly
- Fixed `version.rs` test: expected "0.0.0" but crate version is "0.0.1"
- `crates/sdi-cli/src/commands/init.rs` (MODIFIED): added `SDI_CONFIG_PATH` env var support for write target; `config_path_for()` now checks env first; existing configs are now validated via `load_with_paths` (enabling exit code 2 for bad configs on re-run)
- `crates/sdi-cli/src/main.rs` (MODIFIED): added `error_exit_code()` helper that downcasts to `ConfigError` → exit 2, other errors → exit 1; uses `sdi_core::ExitCode`
- `crates/sdi-config/src/load.rs` (MODIFIED): removed redundant `ColorChoice` import (was explicitly imported but used as `crate::ColorChoice`)
- `crates/sdi-config/src/thresholds.rs` (MODIFIED): `validate_and_prune_overrides` now calls `is_expired()` instead of duplicating the comparison inline (eliminates dead-code warning)
- `crates/sdi-cli/tests/init.rs` (MODIFIED): added 3 new tests covering `SDI_CONFIG_PATH` env var, exit code 2 for missing expires, and unknown-section stderr warning

## Root Cause (bugs only)
- `version.rs` test expected "0.0.0" but workspace version was "0.0.1": test was written with wrong expected value
- `tempfile 3.27.0` (pulled in by Cargo.lock update) depends on `getrandom 0.4.2` which uses `edition = "2024"`, incompatible with Cargo 1.75 in this environment; fixed with `<3.20` pin

## Architecture Decisions
- `load_with_paths(project, global)` exposed as pub for test isolation (no env var mutation needed in tests)
- `validate_and_prune_overrides` operates on raw toml::Table before deserialization, so MissingExpiresOnOverride is a proper typed error not a generic serde error
- `merge_into` does 2-level merge (section → key) with special 3-level handling for thresholds.overrides (per-category semantics from spec)
- Arrays (core.exclude, patterns.scope_exclude) are replaced not merged, which happens naturally since they are toml::Value::Array not Table
- `init` validates existing config on re-run via `load_with_paths`; this surfaces unknown-key warnings and returns ConfigError for bad configs — enabling the "exit 2 for missing expires" acceptance criterion without adding new commands
- `error_exit_code` in main.rs uses `anyhow::Error::downcast_ref` to detect ConfigError; all other errors fall through to exit 1

## Files Modified
- `Cargo.toml` (MODIFIED): pinned `tempfile = ">=3.0, <3.20"` to avoid edition2024 in environment
- `Cargo.lock` (MODIFIED): updated after tempfile version pin
- `crates/sdi-config/Cargo.toml` (MODIFIED)
- `crates/sdi-config/src/thresholds.rs` (NEW)
- `crates/sdi-config/src/boundary.rs` (NEW)
- `crates/sdi-config/src/load.rs` (MODIFIED)
- `crates/sdi-config/src/error.rs` (MODIFIED)
- `crates/sdi-config/src/lib.rs` (MODIFIED)
- `crates/sdi-config/tests/precedence.rs` (NEW)
- `crates/sdi-config/tests/threshold_overrides.rs` (NEW)
- `crates/sdi-config/tests/sdi_py_compat.rs` (NEW)
- `crates/sdi-config/tests/fixtures/sdi_py_config.toml` (NEW)
- `crates/sdi-config/tests/fixtures/sdi_py_boundaries.yaml` (NEW)
- `crates/sdi-cli/Cargo.toml` (MODIFIED)
- `crates/sdi-cli/src/commands/init.rs` (NEW → MODIFIED)
- `crates/sdi-cli/src/commands/mod.rs` (MODIFIED)
- `crates/sdi-cli/src/main.rs` (MODIFIED)
- `crates/sdi-cli/tests/init.rs` (NEW → MODIFIED)
- `crates/sdi-cli/tests/version.rs` (MODIFIED): fixed expected version string from "0.0.0" to "0.0.1"

## Human Notes Status
- M01 reviewer note: `expires: String` generic serde error — COMPLETED: validates at TOML level before deserialization
- M01 reviewer note: `main()` exit code — COMPLETED: added `error_exit_code()` mapping ConfigError → exit 2, others → exit 1
- M01 reviewer note: `clap` version restriction — NOT_ADDRESSED (deferred to M11)
- M01 reviewer note: `prelude.rs` separate file — NOT_ADDRESSED (out of scope)
- M01 reviewer note: `verify-leiden.yml` missing — NOT_ADDRESSED (deferred to M05)
- M01 reviewer note: `expires: String` exit-2 contract — COMPLETED: wired in main.rs + tested in init.rs

## Docs Updated
None — no public-surface changes beyond what milestone deliverables require. All M02 public surface (ConfigError variants, BoundarySpec, load_or_default, load_with_paths) already has rustdoc on every public item.

## Observed Issues (out of scope)
- `crates/sdi-core/src/lib.rs` — `pub mod prelude` should be a separate `prelude.rs` per CLAUDE.md repo layout (noted by M01 reviewer, still unfixed)
- `Cargo.toml` — `clap = ">=4.4, <4.5"` restricts to 4.4.x only; security patches in 4.5+ are blocked (noted by M01 reviewer, deferred to M11)
- `crates/sdi-cli/src/output/mod.rs` — `pub mod json` and `pub mod text` are empty public bodies (noted by M01 reviewer, should be filled or made private before 0.1.0)
