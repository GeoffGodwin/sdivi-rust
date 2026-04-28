#### Milestone 5: Dependency Graph + Native Leiden Port

**Scope:** Build the dependency graph from `FeatureRecord` outputs (`sdi-graph`) and implement the native Leiden community detection (`sdi-detection`). This is the KD11 hot zone — the largest single milestone. Verification suite against `leidenalg` is set up here, gated behind feature `verify-leiden`.

**Deliverables:**
- `DependencyGraph` built on `petgraph::Graph<NodeId, EdgeWeight>`; node-per-file (or per-top-level-module per adapter); directed edges from resolved imports
- Graph metrics: density, cycle count via DFS (excluding self-loops), top-degree hubs, connected component count
- Native Leiden implementation in `sdi-detection::leiden`: Modularity and CPM quality functions; ~1500–2500 LOC total across `mod.rs`, `modularity.rs`, `cpm.rs`, `refine.rs`, `aggregate.rs`
- `LeidenPartition` struct: cluster assignments + per-cluster stability score
- Warm-start path reading `.sdi/cache/partition.json`; cold-start uses `Config::random_seed`
- `verify-leiden` feature: cross-check fixture suite (50, 500, 5000 nodes, including parsed bifl-tracker) against `leidenalg` via a Python harness; assert modularity within 1% and community count within ±10%
- New CI job `verify-leiden.yml` running the gated suite (skipped on PRs without the feature)

**Files to create or modify:**
- `crates/sdi-graph/src/{dependency_graph.rs,metrics.rs}` (real implementation)
- `crates/sdi-detection/src/leiden/{mod.rs,modularity.rs,cpm.rs,refine.rs,aggregate.rs}`
- `crates/sdi-detection/src/{partition.rs,warm_start.rs}`
- `crates/sdi-detection/Cargo.toml` (feature `verify-leiden`)
- `crates/sdi-detection/tests/leiden_quality.rs` (gated)
- `tests/fixtures/leiden-graphs/{small,medium,large}/` (parsed adjacency lists + reference modularities from leidenalg)
- `.github/workflows/verify-leiden.yml`

**Acceptance criteria:**
- `build_dependency_graph` on `simple-rust` fixture produces the hand-known node/edge counts
- Same input + same seed → bit-identical `LeidenPartition` JSON across 100 runs (proptest `prop_test_leiden_seeded`)
- On every fixture in `tests/fixtures/leiden-graphs/`: modularity within 1% of leidenalg's, community count within ±10%
- No community larger than 50% of node count for fixtures leidenalg partitions sensibly
- Warm-start with a stale partition file: first iteration starts from those clusters; result quality matches cold-start within 1%
- Disconnected components are partitioned independently; result is the union
- `verify-leiden` CI job passes; default `cargo test` does not require Python or leidenalg

**Tests:**
- `crates/sdi-graph/tests/metrics.rs`: hand-built graphs with known density, cycles, hubs
- `crates/sdi-detection/tests/proptest_seeded.rs`: same seed → same partition (`prop_test_leiden_seeded`)
- `crates/sdi-detection/tests/leiden_quality.rs` (gated `verify-leiden`): the cross-check suite
- `crates/sdi-detection/tests/warm_start.rs`: stale cache path → first-iteration honors prior assignments

**Watch For:**
- This is the milestone with the highest implementation risk. Profile early, before optimizing
- If `petgraph`'s adjacency representation dominates Leiden's hot loops, build the `csr_view` in `sdi-graph` (KDD-5 says decide here)
- Cycle detection must exclude self-loops (DESIGN edge case); a depth-first cycle finder that doesn't filter self-loops will inflate counts
- Unresolved imports must be **dropped silently**, logged at DEBUG only — failing parses must not propagate as graph errors
- The verification harness's Python dependency is allowed but must not be a default-test dep — the gating is non-negotiable
- Modularity float comparisons must use `f64::abs_diff <= 0.01 * leidenalg_value`, not `==`. Document the FMA caveat

**Seeds Forward:**
- `DependencyGraph` and `LeidenPartition` are inputs to snapshot assembly in Milestone 7
- The verification suite stays in CI from here forward; regressions are blocking
- The optional CSR view, if built, becomes a permanent path; document in `docs/determinism.md`
- `.sdi/cache/partition.json` schema is fixed here; bumping it requires care to keep warm-start beneficial across snapshots

---
