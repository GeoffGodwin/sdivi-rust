# Reviewer Report — M12 (WASM crate + consumer-app integration)
## Review cycle: 2 of 4

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- (Carried from cycle 1) `exports.rs:217-232` — `build_pattern_catalog` silently discards `WasmPatternInstanceInput.location`; assembled snapshots always have empty `locations` vectors in `PatternStats` even when callers supply location data.
- (Carried from cycle 1) `wasm.yml:29-30` — WASM CI hardcodes `toolchain: "1.85.0"` instead of reading `rust-toolchain.toml`; an MSRV bump will cause silent drift in the WASM workflow.
- (Carried from cycle 1) `crates/sdi-core/src/lib.rs:97-109` — CLAUDE.md "Module Boundaries and Dependency Rules" section was not updated to document the new `pub use` re-exports of `GraphMetrics`, `LeidenPartition`, `PatternCatalog`, `PatternStats`, `PatternFingerprint` from inner crates; the coder's own architecture note flagged this as required.
- `wasm.yml:62` — `|| true` still swallows wasm-pack test failures in the hash-capture step; if wasm-bindgen-test changes behavior the CI will accept an empty hash with a warning (exit 0) rather than failing. This was a deliberate tradeoff to handle environments where `--test <filter>` is unsupported; acceptable given the explicit empty-hash guard added in the comparison job (lines 106-109).
- (Carried from cycle 1) `fingerprint.rs:56-63` — `PatternFingerprint::from_hex` does not check `hex.is_ascii()` before byte-indexing; a 64-byte string with multi-byte UTF-8 characters could panic. Pre-existing; `if !hex.is_ascii() { return None; }` before the loop would close it cleanly.

## Coverage Gaps
- `assemble_snapshot` is absent from the smoke test imports (`wasm_smoke.rs:9-12`); no WASM test exercises the full assemble path.
- `compute_trend` is absent from the smoke test imports; no WASM test exercises trend computation.
- `compute_delta` is tested only with two identical snapshots; no smoke test verifies that distinct snapshots produce non-zero deltas.

## Blocker Verification

**Blocker 1 — `assemble_snapshot` hardcodes `violation_count: 0`**: FIXED.
`types.rs:296-298` adds `violation_count: Option<u32>` to `WasmAssembleSnapshotInput`.
`exports.rs:170-174` now uses `input.violation_count.unwrap_or(0) as usize` — violation data from `compute_boundary_violations` is faithfully propagated to the snapshot.

**Blocker 2 — CI cross-platform hash determinism check always exits 0**: FIXED.
`wasm_smoke.rs:191-196` adds `#[wasm_bindgen_test] fn normalize_hash_deterministic` that calls `normalize_and_hash("try_expression", vec![])` and prints `CI_HASH:{hash}`.
`wasm.yml:64` updated grep to `CI_HASH:[0-9a-f]{64}` and strips the prefix via `sed`; the comparison job (lines 106-109) now fails on a real hash mismatch and warns (rather than silently passing) when hashes are empty.

## ACP Verdicts
- ACP: Re-exporting inner types from sdi-core — ACCEPT. Additive re-exports, WASM-safe, required by the KDD-12 architecture where `sdi-wasm` depends only on `sdi-core`. The corresponding CLAUDE.md update remains outstanding (noted above as non-blocking).
- ACP: WASM wrapper types use serde_json round-trip conversion — ACCEPT. Internal to `sdi-wasm`; avoids ~200 lines of boilerplate `From` impls with no observable behavior difference to callers.
- ACP: wasm-pack --profile syntax (wasm-pack 0.12+) — ACCEPT. Correct fix for the `--release` + `--profile` Cargo flag conflict; documented in build.sh and wasm.yml.

## Drift Observations
- `exports.rs:187-190` — `build_graph_metrics` converts string node IDs to `PathBuf::from(id)` to satisfy `GraphMetrics.top_hubs: Vec<(PathBuf, usize)>`; couples the WASM binding layer to an internal `sdi-graph` field type. If that field changes to `String` the mapping becomes dead code without a compile error at the binding layer.
- `exports.rs:127-134` vs `exports.rs:55-61` — `WasmPriorPartition` is silently reused for two distinct Rust types (`sdi_core::PriorPartition` via serde round-trip in `detect_boundaries`, and `sdi_core::SnapshotPriorPartition` via manual struct construction in `infer_boundaries`). Safe today because both share identical serde field layouts; a future field change to either type will produce a silent runtime serialization failure, not a compile error.
- `lib.rs:51-57` (SNAPSHOT_TS constant) — TypeScript interface declares `LeidenPartition.seed: number`, but the Rust field is `u64`; JavaScript `number` (IEEE-754 f64) cannot exactly represent seeds above 2^53. Default seed 42 is safe; the README or the TypeScript interface should document the limit.
