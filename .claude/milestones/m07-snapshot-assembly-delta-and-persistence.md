#### Milestone 7: Snapshot Assembly, Delta, and Persistence

**Scope:** Assemble the `Snapshot` from graph + partition + catalog + boundary spec. Implement `compute_delta` as a pure function. Atomic snapshot write to `.sdi/snapshots/`. Retention enforcement. Wire `sdi snapshot` and `sdi diff` end-to-end. The pipeline is now usable.

**Deliverables:**
- `Snapshot` struct in `sdi-snapshot::snapshot` with `snapshot_version: "1.0"` and all fields from DESIGN's snapshot composition
- `compute_delta(prev: &Snapshot, curr: &Snapshot) -> DivergenceSummary` pure function; first-snapshot returns `null` per-dimension
- `Pipeline::snapshot` and `Pipeline::delta` methods on `sdi-core::pipeline::Pipeline`
- Atomic write: tempfile in `.sdi/snapshots/`, then rename. Retention enforced synchronously after write
- `sdi snapshot [--commit REF] [--format json|text]` command; `sdi diff <prev> <curr>` command
- Snapshot file naming `snapshot_<timestamp>_<sha>.json` (KDD per Open Q #8)

**Files to create or modify:**
- `crates/sdi-snapshot/src/{snapshot.rs,delta.rs,store.rs,retention.rs}`
- `crates/sdi-core/src/pipeline.rs` (real implementation; `Pipeline::new` cheap, `snapshot` runs all five stages)
- `crates/sdi-cli/src/commands/{snapshot.rs,diff.rs}`
- `tests/full_pipeline.rs` (extended end-to-end)

**Acceptance criteria:**
- `sdi snapshot` on `simple-rust` fixture writes a JSON file matching schema 1.0; running again produces a second file
- Delta on identical consecutive snapshots: all dimensions `0` (not `null`)
- Delta on first snapshot: all dimensions `null` (not `0`)
- Killing the process mid-write (simulated by injecting a panic before rename in a test) leaves the target directory free of half-written `.json` files
- `retention = 3` keeps only the 3 most recent snapshots after the third write
- Same input + config → bit-identical snapshot JSON (proptest `prop_test_pipeline_deterministic`)
- `sdi diff <prev> <curr>` prints the divergence summary; exits 0
- Missing boundary spec: snapshot still produced, intent divergence fields absent — no warning

**Tests:**
- `crates/sdi-snapshot/tests/atomic_write.rs`: simulate panic before rename, assert no leftover tempfile in target dir
- `crates/sdi-snapshot/tests/retention.rs`: write N+1 with retention N, assert oldest deleted
- `crates/sdi-snapshot/tests/delta_pure.rs`: `prop_test_delta_pure` referential transparency
- `crates/sdi-snapshot/tests/null_vs_zero.rs`: first snapshot null, second-identical zero
- `tests/full_pipeline.rs`: end-to-end on every fixture
- `crates/sdi-cli/tests/snapshot_diff.rs`: `assert_cmd` integration

**Watch For:**
- The tempfile must be created in the **same directory** as the final file — cross-filesystem rename is not atomic on POSIX. Reject `tempfile::NamedTempFile::new()` (defaults to `/tmp`); use `tempfile_in(snapshot_dir)`
- Retention enforcement must run after the rename succeeds, not before — otherwise a failed write leaves the directory short
- `null` vs missing field in JSON: use `Option<f64>` and serde `skip_serializing_none = false` so `null` is explicit in output
- Identical consecutive snapshots produce zero deltas, and the test must use deterministic timestamps to avoid the timestamp itself making them non-identical

**Seeds Forward:**
- `Snapshot` JSON schema 1.0 is the wire contract from here. Field additions must default-deserialize on old snapshots; renames are breaking
- `Pipeline::{snapshot,delta}` are now the canonical library entry points — bindings (Milestone 12) call these
- The atomic-write pattern is reused for `boundaries.yaml` writes in Milestone 9
- Trend computation in Milestone 8 reads the on-disk snapshot directory established here

---
