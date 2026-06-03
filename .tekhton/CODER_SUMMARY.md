# Coder Summary

**Agent:** coder
**Date:** 2026-06-03
**Milestone:** M50 — Disable wasm-opt for binaryen-free WASM builds (v0.2.51)

---

## Status: COMPLETE

## What Was Implemented

1. **`bindings/sdivi-wasm/Cargo.toml`** — Added `[package.metadata.wasm-pack.profile.release]` block with `wasm-opt = false` immediately before the trailing release-profile comment. wasm-pack reads this key and skips the binaryen post-processing step, eliminating the toolchain dependency without any source or behaviour change.

2. **`Cargo.toml`** — Bumped `[workspace.package].version` from `0.2.50` to `0.2.51`. All member crates inherit this version.

3. **`bindings/sdivi-wasm/pkg-template/package.json`** — Synced `"version"` to `"0.2.51"` in lockstep with the workspace version bump. Required for `wasm_package_json_version_matches_workspace` to pass.

4. **`CHANGELOG.md`** — Added `[0.2.51] - 2026-06-03` section with a "Changed" entry describing the wasm-opt disable, the ~10% size trade-off, budget headroom, and `snapshot_version` stability.

## Root Cause (bugs only)

N/A — feature/config change milestone.

## Files Modified

- `bindings/sdivi-wasm/Cargo.toml` — added `[package.metadata.wasm-pack.profile.release]` / `wasm-opt = false`
- `Cargo.toml` — bumped workspace version 0.2.50 → 0.2.51
- `bindings/sdivi-wasm/pkg-template/package.json` — synced version 0.2.50 → 0.2.51
- `CHANGELOG.md` — added [0.2.51] section

## Human Notes Status

No Human Notes section in this milestone.

## Docs Updated

None — no public-surface changes in this task. The wasm-opt metadata is a build-tool directive; no CLI flags, exported functions, config keys, or schemas changed.

## Observed Issues (out of scope)

- `crates/sdivi-detection/tests/renumber_delegation.rs:83,85` — pre-existing `clippy::iter_cloned_collect` warnings noted by the M49.2 reviewer (unrelated to M50).
