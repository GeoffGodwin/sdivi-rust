#### Milestone 10: Boundaries — Infer, Ratify, Show

**Scope:** Implement the boundary lifecycle: infer modules from a `LeidenPartition` (using the pure `sdi_core::infer_boundaries` function delivered in M08), ratify them into `.sdi/boundaries.yaml`, and inspect via `show`. Comment loss on programmatic write is accepted per KDD-6.

**Deliverables:**
- `sdi boundaries infer` proposes module groupings from the most recent partition, using `sdi_core::infer_boundaries` for the proposal logic
- `sdi boundaries ratify` writes (or merges) accepted groupings into `.sdi/boundaries.yaml`
- `sdi boundaries show` prints the current spec
- YAML write path via `serde_yaml` with documented comment-loss behavior — lives in `sdi-pipeline::store` (I/O), not in `sdi-config` (pure)
- Stability tracking: `Config::boundaries.stability_threshold` (default 3) gates which clusters are mature enough to propose. Stability is computed inside `sdi_core::infer_boundaries` against the partition history (oldest→newest, latest is the proposal source) that the caller assembles. CLI loads it from `.sdi/cache/`; the consumer app supplies its own list.

**Files to create or modify:**
- `crates/sdi-pipeline/src/boundaries.rs` — read the last N stored `Snapshot`s from `.sdi/snapshots/` (each carries its `LeidenPartition`), assemble a `Vec<PriorPartition>` ordered oldest→newest, call `sdi_core::infer_boundaries`, atomic-write YAML to `.sdi/boundaries.yaml`. `N = stability_threshold + 1` is sufficient (need the proposal source plus enough history to gate it). No separate partition-history cache is introduced — the snapshot store already retains partitions per `Config::snapshots.retention`.
- `crates/sdi-config/src/boundary.rs` — add pure YAML serialization helper `BoundarySpec::to_yaml(&self) -> String` (no FS). Lives outside the M08 `loader` feature gate so it's available in WASM. The FS-touching `BoundarySpec::write(&self, path)` lives in `sdi-pipeline::store` (gated by `loader` if any sdi-config helper is needed there).
- `crates/sdi-cli/src/commands/boundaries.rs` — full subcommand impl (parent stub from M09)
- `docs/migrating-from-sdi-py.md` — **create** this file with at minimum the YAML comment-loss section (full migration guide is finished in Milestone 11)

**Acceptance criteria:**
- `sdi boundaries infer` on a multi-snapshot history proposes groupings only for clusters present in `stability_threshold` consecutive snapshots
- `sdi boundaries ratify` writes a valid YAML file; reading it back produces an equivalent `BoundarySpec`
- A user-written `boundaries.yaml` with comments loses comments on the next ratify; behavior documented in `docs/migrating-from-sdi-py.md` (file is created here as a stub with the comment-loss section; Milestone 11 fills out the rest of the migration guide)
- `sdi boundaries show` prints the spec in either YAML or JSON format
- `sdi_core::infer_boundaries` is callable directly by the consumer app (via WASM in M12) with a caller-supplied prior-partition history

**Tests:**
- `tests/boundary_lifecycle.rs`: build evolving fixture, run `infer`/`ratify`/`show` end-to-end
- `crates/sdi-config/tests/boundary_roundtrip.rs`: write then read; equivalent spec
- `crates/sdi-cli/tests/boundaries_show.rs`: format flags work
- `crates/sdi-core/tests/infer_boundaries.rs`: pure-compute infer test (no FS); same proposal as the CLI for an equivalent prior-history

**Watch For:**
- The `sdi-py` `boundaries.yaml` schema is read-compatible — DO NOT introduce sdi-rust-only fields here without an explicit `tekhton` design discussion
- Comment loss surprises users; `sdi boundaries ratify` should print a stderr warning the first time it overwrites a file with comments
- Atomic write applies here too — same tempfile + rename pattern as snapshots, in `sdi-pipeline::store`
- Inference must respect the `stability_threshold` over historical snapshots, not just propose every cluster from the latest one. The history is supplied to `sdi_core::infer_boundaries` as `&[PriorPartition]` — the caller (CLI or the consumer app) loads it.

**Seeds Forward:**
- `BoundarySpec` write path established here is the only programmatic write point. Future write-back features (e.g., editor plugin, post-1.0) reuse it.
- The decision on comment preservation can be revisited post-MVP without breaking the schema.
- The consumer app gets boundary inference + violation detection via `sdi_core::{infer_boundaries, compute_boundary_violations}` — same surface, different consumer.

---
