# Coder Summary
**Date:** 2026-05-01
**Audit cycle:** post-M16 (non-blocking notes sweep)
**Scope:** All 9 open items in `.tekhton/NON_BLOCKING_LOG.md` + simple blocker from prior reviewer report

---

## Status: COMPLETE

## What Was Implemented

### Simple Blocker (reviewer report)
- `crates/sdi-core/src/compute/boundaries.rs` — removed unused `current`
  parameter from `compute_historical_stability` and updated its call site.
  This eliminates the `cargo clippy -- -D warnings` unused-variable error.

### Non-Blocking Notes — All 9 Items Resolved

| # | File | Action |
|---|------|--------|
| 1 | `commit_extract.rs:81` — misleading `RefResolutionFailed` | Already fixed in prior run: `CommitDateParseFailed { sha, raw }` variant added and used. |
| 2 | `commit_extract.rs:94-113` — `git archive` before tar check | Already fixed in prior run: tar `--version` check precedes `git archive` spawn. |
| 3 | Security findings MEDIUM/LOW | Informational; handled by security pipeline. No code change. |
| 4 | `sdi-core/.../change_coupling.rs:96-100` — dead `else` branch | Already fixed in prior run: direct `(sorted_files[i].clone(), sorted_files[j].clone())`. |
| 5 | `sdi-pipeline/.../change_coupling.rs:101-103,123` — double empty-SHA guard | Already fixed in prior run: inner guard removed; single outer guard. |
| 6 | `exports.rs:168` — `change_coupling: None` hardcoded | Tracked with TODO comment at exports.rs:160-162; MVP intentional. |
| 7 | `types.rs:53-58` — `WasmLeidenConfigInput` no `edge_weights` | Tracked with INTENTIONAL GAP doc comment at types.rs:46-48 (ADL-4). |
| 8 | `input/types.rs:145` — tuple-keyed map unserializable via serde_json | Already fixed in prior run: `edge_weights` uses `Option<BTreeMap<String, f64>>` with NUL-delimited keys. |
| 9 | `change_coupling.rs:83` — `HashSet` vs `BTreeSet` convention | Already fixed in prior run: line 83 uses `BTreeSet`. |

## Root Cause (bugs only)
The simple blocker (`current` unused param) was introduced when dead code stubs
referencing `current` were removed but the parameter itself was not cleaned up.

## Files Modified
- `crates/sdi-core/src/compute/boundaries.rs` — removed `current` param from
  `compute_historical_stability`; updated call site at line 136
- `.tekhton/NON_BLOCKING_LOG.md` — moved all 9 open items to Resolved

## Human Notes Status
N/A — no Human Notes section in this task.

## Docs Updated
None — no public-surface changes in this task. (`compute_historical_stability` is
`pub(crate)` — not public API.)

## Observed Issues (out of scope)
- `crates/sdi-config/src/thresholds.rs:46` — `validate_and_prune_overrides` is `pub(crate)` but never called; dead code warning (pre-existing).
- `crates/sdi-graph/src/dependency_graph.rs:9` — unused import `tracing::debug` (pre-existing).
- `crates/sdi-patterns/src/catalog.rs:6,8,10,11` — four unused imports (`Glob`, `GlobSet`, `GlobSetBuilder`, `PatternsConfig`, `fingerprint_node_kind`, `crate::queries`) (pre-existing).
