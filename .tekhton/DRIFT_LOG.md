# Drift Log

## Metadata
- Last audit: never
- Runs since audit: 2

## Unresolved Observations
- [2026-04-28 | "Implement Milestone 2: Config Loading + Boundary Spec Reader"] `thresholds.rs` is declared `pub(crate)` at module level but its functions (`today_iso8601`, `is_expired`, `validate_date_format`, `validate_and_prune_overrides`) are all `pub`. They are unreachable from outside the crate regardless; tightening to `pub(crate)` removes the mismatch.
- [2026-04-28 | "Implement Milestone 2: Config Loading + Boundary Spec Reader"] `init.rs` writes progress lines (`"sdi: created .sdi/config.toml"`, `"sdi: detected languages: ..."`) to stdout. Rule 8 reserves stdout for snapshot JSON, summaries, and table output and assigns progress/status to stderr. The integration tests pin these on stdout, so this is intentional, but the choice is worth flagging before 0.1.0 stdout/stderr contract is locked.
- [2026-04-28 | "Implement Milestone 2: Config Loading + Boundary Spec Reader"] `load.rs:122-131` has an `else {}` block with only a comment and no code; clippy's `clippy::redundant_else` or empty-block lint may or may not fire depending on the version. Worth verifying in CI.
- [2026-04-28 | "Implement Milestone 2: Config Loading + Boundary Spec Reader"] `Cargo.toml` (carried from M01): `clap = ">=4.4, <4.5"` restricts to 4.4.x only, blocking security patches in 4.5+. Deferred to M11 per coder; must not be forgotten.
- [2026-04-28 | "Implement Milestone 2: Config Loading + Boundary Spec Reader"] `crates/sdi-cli/src/output/mod.rs` (carried from M01): `pub mod json` and `pub mod text` are exposed with empty bodies. Ensure they receive content or become private before 0.1.0 publish.
- [2026-04-28 | "M01"] `Cargo.toml:41` — `serde_yaml = "0.9"` is unmaintained (last release 2023-06). Coder added `# note: unmaintained upstream; revisit in M10` comment. No action needed now; revisit in M02 when the YAML loader is written.
- [2026-04-28 | "M01"] `crates/sdi-cli/src/output/mod.rs` — `pub mod json` and `pub mod text` are exposed with empty bodies. IDEs and rustdoc will surface them as public API with no content. Low-risk for a scaffold; ensure they receive content (or become private) before the first `0.1.0` publish.
- [2026-04-28 | "M01"] `crates/sdi-config/src/lib.rs` — `#![deny(missing_docs)]` is absent. All public items currently have doc comments, but there is no compile-time enforcement. CLAUDE.md mandates it on `sdi-core` only; no action required, but worth adding before the config surface stabilizes.

## Resolved
