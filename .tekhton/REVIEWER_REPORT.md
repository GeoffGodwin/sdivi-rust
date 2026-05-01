# Reviewer Report
Review cycle: 1 of 2

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `crates/sdi-pipeline/src/commit_extract.rs:46` — All three security findings (MEDIUM: rev-parse missing `--` separator; LOW: tar missing `--no-absolute-filenames`; LOW: stderr truncation) are marked `fixable:yes` in the security report but were explicitly not addressed. The coder's claim "no code change required" is incorrect — the security agent flagged these as code changes. The MEDIUM finding (adding `"--"` between `"--verify"` and `reference`) is a one-liner. Recommend addressing in the next cleanup pass.
- `crates/sdi-core/src/input/mod.rs:16-17` — `pub use edge_weight::{...}` appears before its `mod edge_weight;` declaration. Valid Rust, but conventional order is `mod` declaration first, then re-exports. Low friction to reorder.

## Coverage Gaps
- `edge_weight_key` wrong-order path: no test passes `source > target` to `edge_weight_key` and exercises the `boundaries.rs:109` normalisation fallback (`if si < ti { (si, ti) } else { (ti, si) }`). Should be a unit test in `leiden_config_serde.rs` or `boundaries.rs` tests to confirm the graceful fallback is intentional.

## Drift Observations
- `crates/sdi-core/src/input/edge_weight.rs:14` — doc says `source < target` is required but the invariant is not enforced at runtime. `boundaries.rs:109` already normalises index order, so a mis-ordered single key still works. Only two keys mapping to the same edge pair would silently collide (last iteration wins in `BTreeMap::collect`). The doc is misleading — either enforce with a debug_assert or rewrite the doc to say "callers should canonicalise; detection normalises".
- Pre-existing compiler warnings not introduced by this task but noted by the coder as out-of-scope: unused `pub(crate) validate_and_prune_overrides` (`sdi-config/src/thresholds.rs:46`), unused import `tracing::debug` (`sdi-graph/src/dependency_graph.rs:9`), dead code in `sdi-patterns/src/catalog.rs`. These are accumulating and worth a dedicated cleanup pass.
- `crates/sdi-core/src/compute/boundaries.rs:174` — `let _ = &current_communities;` with comment "used for future extension" is dead code and a TODO stub that should live in the issue tracker, not the source.
