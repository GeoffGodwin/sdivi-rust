# Coder Summary
## Status: COMPLETE

## What Was Implemented

### Previous run (continuation context):
- `bindings/sdi-wasm` WASM crate wrapping sdi-core with wasm-bindgen + tsify
- Re-exports of inner types in sdi-core: `GraphMetrics`, `LeidenPartition`, `PatternCatalog`, `PatternStats`, `PatternFingerprint`
- `PatternFingerprint::from_hex` method in sdi-patterns
- Workspace Cargo.toml updated with `bindings/sdi-wasm` member and pinned WASM deps
- CI workflow `.github/workflows/wasm.yml` with bundle-size check and cross-platform hash determinism
- `examples/binding_node.ts` consumer example

### This run:
- Fixed `wasm-pack build` command: replaced `--release -- --profile release-wasm` with `--profile release-wasm` in both `build.sh` and `wasm.yml` to avoid the Cargo conflict between `--release` and a custom `--profile` flag
- Added `leiden_seed: Option<u64>` to `WasmAssembleSnapshotInput` so callers can record the exact seed used in `detect_boundaries`; `build_leiden_partition` uses it instead of hardcoded 42
- Updated `examples/binding_node.ts` to pass `leiden_seed: cfg.seed` in `assemble_snapshot` call

## Root Cause (bugs only)
N/A — feature implementation (M12: WASM crate and consumer app integration)

## Files Modified
- `Cargo.toml` (workspace) — add `bindings/sdi-wasm` member + WASM deps in `[workspace.dependencies]`
- `crates/sdi-core/src/lib.rs` — add inner-type re-exports (`GraphMetrics`, `LeidenPartition`, `PatternCatalog`, `PatternStats`, `PatternFingerprint`)
- `crates/sdi-patterns/src/fingerprint.rs` — add `from_hex` method + doc test
- `bindings/sdi-wasm/Cargo.toml` (NEW)
- `bindings/sdi-wasm/src/lib.rs` (NEW) — wasm-bindgen `#[start]`, TypeScript custom section for Snapshot
- `bindings/sdi-wasm/src/types.rs` (NEW) — tsify-derived wrapper types; added `leiden_seed` field
- `bindings/sdi-wasm/src/exports.rs` (NEW) — `#[wasm_bindgen]` exported functions; fixed `build_leiden_partition` to use `leiden_seed`
- `bindings/sdi-wasm/tests/wasm_smoke.rs` (NEW) — wasm-bindgen-test smoke tests for every export
- `bindings/sdi-wasm/package.json` (NEW)
- `bindings/sdi-wasm/README.md` (NEW)
- `bindings/sdi-wasm/build.sh` (NEW) — fixed: `wasm-pack build --target bundler --profile release-wasm`
- `examples/binding_node.ts` (NEW) — consumer example; passes `leiden_seed: cfg.seed`
- `.github/workflows/wasm.yml` (NEW) — fixed: `wasm-pack build --target bundler --profile release-wasm`

## Human Notes Status
N/A — no human notes in this task

## Architecture Decisions

### Re-exporting inner types from sdi-core
**Current constraint**: sdi-core only re-exports types it defines directly.
**What triggered this**: sdi-wasm depends on sdi-core only, but `assemble_snapshot` requires `GraphMetrics`, `LeidenPartition`, `PatternCatalog` from inner crates.
**Proposed change**: sdi-core adds `pub use` re-exports for these types. They're already transitively available (sdi-core directly depends on sdi-graph, sdi-detection, sdi-patterns with default-features=false).
**Backward compatible**: Yes — additive only, no existing code breaks.
**ARCHITECTURE.md update needed**: Yes — "Module Boundaries and Dependency Rules" section, note that sdi-core re-exports inner-crate types.

### WASM wrapper types use serde_json round-trip conversion
**Current constraint**: None specific — architecture doesn't prescribe conversion strategy.
**What triggered this**: Input/output wrapper types in sdi-wasm mirror sdi-core types exactly (same serde field names). Using serde_json for conversion avoids writing ~200 lines of From impls.
**Proposed change**: A pair of generic helper functions `to_core<T>` and `from_core<T>` in sdi-wasm exports use serde_json for type conversion.
**Backward compatible**: Yes — internal to sdi-wasm.
**ARCHITECTURE.md update needed**: No.

### wasm-pack --profile syntax (wasm-pack 0.12+)
**Current constraint**: None documented.
**What triggered this**: Using `--release -- --profile release-wasm` in wasm-pack would pass conflicting `--release` and `--profile` flags to Cargo.
**Proposed change**: Use `wasm-pack build --target bundler --profile release-wasm` (no `--release` flag). wasm-pack 0.12+ supports `--profile` directly.
**Backward compatible**: Yes — only affects build tooling.
**ARCHITECTURE.md update needed**: No.

## Docs Updated
- `bindings/sdi-wasm/README.md` (NEW) — covers install, init pattern, every export, strict-TS guarantees, normalize_and_hash determinism, local build instructions, tsify version note

## Observed Issues (out of scope)
- `tools/validate-against-bifl-tracker.sh:144` — sibling `open('$f')` injection pattern (same as the fixed line 119); noted by reviewer as drift observation for next maintenance cycle; not fixed here per scope rules.
