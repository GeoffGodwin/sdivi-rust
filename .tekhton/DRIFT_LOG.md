# Drift Log

## Metadata
- Last audit: never
- Runs since audit: 1

## Unresolved Observations
- [2026-04-28 | "M01"] `Cargo.toml:41` — `serde_yaml = "0.9"` is unmaintained (last release 2023-06). Coder added `# note: unmaintained upstream; revisit in M10` comment. No action needed now; revisit in M02 when the YAML loader is written.
- [2026-04-28 | "M01"] `crates/sdi-cli/src/output/mod.rs` — `pub mod json` and `pub mod text` are exposed with empty bodies. IDEs and rustdoc will surface them as public API with no content. Low-risk for a scaffold; ensure they receive content (or become private) before the first `0.1.0` publish.
- [2026-04-28 | "M01"] `crates/sdi-config/src/lib.rs` — `#![deny(missing_docs)]` is absent. All public items currently have doc comments, but there is no compile-time enforcement. CLAUDE.md mandates it on `sdi-core` only; no action required, but worth adding before the config surface stabilizes.

## Resolved
