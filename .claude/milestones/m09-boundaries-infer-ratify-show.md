#### Milestone 9: Boundaries — Infer, Ratify, Show

**Scope:** Implement the boundary lifecycle: infer modules from a `LeidenPartition`, ratify them into `.sdi/boundaries.yaml`, and inspect via `show`. Comment loss on programmatic write is accepted per KDD-6.

**Deliverables:**
- `sdi boundaries infer` proposes module groupings from the most recent partition
- `sdi boundaries ratify` writes (or merges) accepted groupings into `.sdi/boundaries.yaml`
- `sdi boundaries show` prints the current spec
- YAML write path via `serde_yaml` with documented comment-loss behavior
- Stability tracking: `Config::boundaries.stability_threshold` (default 3) gates which clusters are mature enough to propose

**Files to create or modify:**
- `crates/sdi-snapshot/src/boundary_inference.rs`
- `crates/sdi-config/src/boundary.rs` (extend with write path)
- `crates/sdi-cli/src/commands/boundaries.rs` (full subcommand impl)

**Acceptance criteria:**
- `sdi boundaries infer` on a multi-snapshot history proposes groupings only for clusters present in `stability_threshold` consecutive snapshots
- `sdi boundaries ratify` writes a valid YAML file; reading it back produces an equivalent `BoundarySpec`
- A user-written boundaries.yaml with comments loses comments on the next ratify; behavior documented in `docs/migrating-from-sdi-py.md`
- `sdi boundaries show` prints the spec in either YAML or JSON format

**Tests:**
- `tests/boundary_lifecycle.rs`: build evolving fixture, run `infer`/`ratify`/`show` end-to-end
- `crates/sdi-config/tests/boundary_roundtrip.rs`: write then read; equivalent spec
- `crates/sdi-cli/tests/boundaries_show.rs`: format flags work

**Watch For:**
- The `sdi-py` `boundaries.yaml` schema is read-compatible — DO NOT introduce sdi-rust-only fields here without an explicit `tekhton` design discussion
- Comment loss surprises users; `sdi boundaries ratify` should print a stderr warning the first time it overwrites a file with comments
- Atomic write applies here too — same tempfile + rename pattern as snapshots
- Inference must respect the `stability_threshold` over historical snapshots, not just propose every cluster from the latest one

**Seeds Forward:**
- `BoundarySpec` write path established here is the only programmatic write point. Future write-back features (e.g., editor plugin Milestone is post-1.0) reuse it
- The decision on comment preservation can be revisited post-MVP without breaking the schema

---
