# Coder Summary
## Status: COMPLETE (rework cycle 1)

## What Was Implemented

### M13 — Release Pipeline and Distribution

- **Workspace `Cargo.toml`**: version bumped `0.0.16` → `0.1.0`; added `sdi-cli` to `[workspace.dependencies]`; added `[profile.release]` with `lto = "thin"`, `strip = true`, `panic = "abort"`.
- **`.github/workflows/release.yml`**: Full tag-driven release workflow (replaces stub):
  - `check-version` job: verifies tag base matches workspace Cargo.toml version (supports `-rc.N` pre-release tags)
  - `audit` job: `rustsec/audit-check` on every release tag
  - `build` job (5-platform matrix): Linux x86_64 (native), Linux aarch64 (via `cross`), macOS x86_64 (`macos-13`), macOS aarch64 (`macos-latest`), Windows x86_64; stripped LTO binaries uploaded as artifacts
  - `create-release` job: downloads all artifacts, creates GitHub Release (prerelease for `-rc` tags); binary sizes logged
  - `publish-crates` job: manual approval gate via `environment: crates-io-publish`; publishes 11 crates in dependency order with 30s index propagation sleeps
  - `publish-npm` job: manual approval gate via `environment: npm-publish`; dry-run on RC tags, real publish on release tags; WASM bundle size logged
- **`CHANGELOG.md`**: Added `[0.1.0]` entry covering all 13 milestones with binary/WASM size placeholders.
- **`bindings/sdi-wasm/package.json`**: version `0.0.15` → `0.1.0`.
- **All published crate `Cargo.toml` files**: Added `readme`, `keywords`, `categories` metadata to all 17 crates (11 workspace crates + 6 lang adapters + sdi-wasm excluded since npm-only).
- **Minimal `README.md`** created for each published crate (required for the `readme` field): `sdi-config`, `sdi-core`, `sdi-pipeline`, `sdi-cli`, `sdi-parsing`, `sdi-graph`, `sdi-detection`, `sdi-patterns`, `sdi-snapshot`, `sdi-rust`, `sdi-lang-rust`, `sdi-lang-python`, `sdi-lang-typescript`, `sdi-lang-javascript`, `sdi-lang-go`, `sdi-lang-java`.
- **`crates/sdi-cli/src/lib.rs`** (NEW): Extracts all CLI logic into a library target exposing `pub fn run()`. The `Cli`/`Commands` structs and `error_exit_code` remain crate-private; only `run()` is public.
- **`crates/sdi-cli/src/main.rs`**: Slimmed to 3 lines calling `sdi_cli::run()`.
- **`crates/sdi-rust/src/main.rs`** (NEW): 3-line binary entry point calling `sdi_cli::run()`, enabling `cargo install sdi-rust` to produce a working `sdi` binary.
- **`crates/sdi-rust/Cargo.toml`**: Added `[[bin]] name = "sdi"` and `sdi-cli` dependency.
- **`.github/workflows/audit.yml`**: Already had weekly Monday cron — verified correct, no changes needed.

## Rework (cycle 1) — Changes from Reviewer Feedback

### Complex Blocker Fixed: `publish-crates` publish order
- **Problem**: `sdi-lang-*` crates were published before `sdi-parsing` in the `publish-crates` job. All six language adapter crates depend on `sdi-parsing`, so crates.io would reject the publish with a registry resolution error.
- **Fix**: Moved the `sdi-parsing` publish step (and its 30s sleep) to immediately after `sdi-config`, before the language adapters block. Also corrected the comment at lines 13–16 of `release.yml` to show the accurate order.
- **Correct order**: `sdi-config → sdi-parsing → sdi-lang-* → sdi-graph → sdi-detection → sdi-patterns → sdi-snapshot → sdi-core → sdi-pipeline → sdi-cli → sdi-rust`
- **Files changed**: `.github/workflows/release.yml`

## Root Cause (bugs only)
N/A — feature implementation (M13: Release Pipeline and Distribution)

## Files Modified
- `Cargo.toml` (workspace) — version `0.1.0`; `[profile.release]` lto/strip/panic; `sdi-cli` in workspace deps
- `.github/workflows/release.yml` — full tag-driven workflow (replaced stub)
- `CHANGELOG.md` — `[0.1.0]` entry
- `bindings/sdi-wasm/package.json` — version `0.1.0`
- `crates/sdi-config/Cargo.toml` — added `readme`, `keywords`, `categories`
- `crates/sdi-core/Cargo.toml` — added `readme`, `keywords`, `categories`
- `crates/sdi-pipeline/Cargo.toml` — added `readme`, `keywords`, `categories`
- `crates/sdi-cli/Cargo.toml` — added `readme`, `keywords`, `categories`
- `crates/sdi-parsing/Cargo.toml` — added `readme`, `keywords`, `categories`
- `crates/sdi-graph/Cargo.toml` — added `readme`, `keywords`, `categories`
- `crates/sdi-detection/Cargo.toml` — added `readme`, `keywords`, `categories`
- `crates/sdi-patterns/Cargo.toml` — added `readme`, `keywords`, `categories`
- `crates/sdi-snapshot/Cargo.toml` — added `readme`, `keywords`, `categories`
- `crates/sdi-rust/Cargo.toml` — added `readme`, `keywords`, `categories`, `[[bin]]`, `sdi-cli` dep
- `crates/sdi-lang-rust/Cargo.toml` — added `readme`, `keywords`, `categories`
- `crates/sdi-lang-python/Cargo.toml` — added `readme`, `keywords`, `categories`
- `crates/sdi-lang-typescript/Cargo.toml` — added `readme`, `keywords`, `categories`
- `crates/sdi-lang-javascript/Cargo.toml` — added `readme`, `keywords`, `categories`
- `crates/sdi-lang-go/Cargo.toml` — added `readme`, `keywords`, `categories`
- `crates/sdi-lang-java/Cargo.toml` — added `readme`, `keywords`, `categories`
- `crates/sdi-cli/src/lib.rs` (NEW) — `pub fn run()` entry point; moves CLI logic out of binary-only main.rs
- `crates/sdi-cli/src/main.rs` — thinned to 3-line stub calling `sdi_cli::run()`
- `crates/sdi-rust/src/main.rs` (NEW) — 3-line `sdi` binary entry point via `sdi_cli::run()`
- `crates/sdi-config/README.md` (NEW)
- `crates/sdi-core/README.md` (NEW)
- `crates/sdi-pipeline/README.md` (NEW)
- `crates/sdi-cli/README.md` (NEW)
- `crates/sdi-parsing/README.md` (NEW)
- `crates/sdi-graph/README.md` (NEW)
- `crates/sdi-detection/README.md` (NEW)
- `crates/sdi-patterns/README.md` (NEW)
- `crates/sdi-snapshot/README.md` (NEW)
- `crates/sdi-rust/README.md` (NEW)
- `crates/sdi-lang-rust/README.md` (NEW)
- `crates/sdi-lang-python/README.md` (NEW)
- `crates/sdi-lang-typescript/README.md` (NEW)
- `crates/sdi-lang-javascript/README.md` (NEW)
- `crates/sdi-lang-go/README.md` (NEW)
- `crates/sdi-lang-java/README.md` (NEW)

## Human Notes Status
N/A — no human notes in this task

## Architecture Change Proposals

### `sdi-cli` exposed as a library target to enable `cargo install sdi-rust`
- **Current constraint**: `sdi-cli` was a binary-only crate (`[[bin]]` with `src/main.rs`, no `[lib]`). `sdi-rust` was a library-only meta-crate with no binary.
- **What triggered this**: The M13 acceptance criterion requires `cargo install sdi-rust` to produce a working `sdi` binary. Since `sdi-cli` had no library target, `sdi-rust` couldn't depend on its logic.
- **Proposed change**: Moved all CLI logic from `sdi-cli/src/main.rs` to `sdi-cli/src/lib.rs` exposing `pub fn run()`. The `main.rs` is now a 3-line stub calling `sdi_cli::run()`. `sdi-rust` gains `[[bin]] name = "sdi"` and a `main.rs` that also calls `sdi_cli::run()`. Both `cargo install sdi-cli` and `cargo install sdi-rust` now produce the `sdi` binary.
- **Backward compatible**: Yes — `sdi-cli` still produces the `sdi` binary. No existing API changes.
- **ARCHITECTURE.md update needed**: No — this is an internal implementation detail.

## Docs Updated
- Per-crate `README.md` files (16 new files) document each crate's purpose and link to the workspace README.

## Observed Issues (out of scope)
- `CHANGELOG.md` entries for `[0.0.1]`–`[0.0.16]` contain internal development tracking noise (e.g. "Managed the human_action_required...") that could be cleaned up before a public release. Out of scope; user decision.
- The `wasm.yml` non-blocking note from the reviewer: hardcodes `toolchain: "1.85.0"` instead of reading `rust-toolchain.toml`. Still present; out of scope for M13.
