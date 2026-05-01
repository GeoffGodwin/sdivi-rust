#### Milestone 11: Documentation, Examples, Determinism Polish, bifl-tracker Validation
<!-- milestone-meta
id: "11"
status: "done"
-->


**Scope:** Stand up the documentation surfaces (`README.md`, `docs/*.md`, rustdoc with `#![deny(missing_docs)]` on `sdi-core`). Doc tests in CI. Runnable examples covering both the orchestration path (`sdi-pipeline`) and the pure-compute path (`sdi-core`). Tighten determinism with `proptest` regression suite and FMA documentation. **Run sdi-rust end-to-end against bifl-tracker as the v0 validation gate** — this is the user-facing acceptance for "the rewrite produces the same answers." Failures here block 0.1.0 release.

**Deliverables:**
- `README.md` quick start, install paths (cargo, brew, binary, npm for the WASM bundle), one-paragraph SDI overview, links — under 200 lines
- `docs/cli-integration.md` with `cargo install sdi && sdi check` GHA snippet and exit-code reference
- `docs/library-embedding.md` with **two** sections: Rust embedders using `sdi-pipeline::Pipeline` for FS-based runs, and pure-compute embedders (Rust + JS via WASM) using `sdi-core::compute_*` functions. Includes a consumer-app-style example: caller has its own AST extractors, computes `normalize_and_hash` per node, ships hashes + graph to `sdi-core`.
- `docs/migrating-from-sdi-py.md` with full carry/change matrix (file stub from M10)
- `docs/determinism.md` with `BTreeMap` discipline, seed contract, FMA notes, and a section on "feeding sdi-core canonical input from a foreign extractor" (the `NodeId` canonicalization rule, `normalize_and_hash` invariant)
- `examples/embed_pipeline.rs` — Rust orchestration via `sdi-pipeline`
- `examples/embed_compute.rs` — Rust pure-compute via `sdi-core` (mirrors what the consumer app does, minus the WASM boundary)
- `examples/custom_config.rs` — programmatic Config building
- `#![deny(missing_docs)]` enabled on `sdi-core` with docs for every public item; rustdoc on `sdi-pipeline` highly recommended but not enforced
- `cargo test --doc` runs in CI; no broken doc tests
- `proptest` regression directory checked in; `prop_test_pipeline_deterministic`, `prop_test_delta_pure`, `prop_test_leiden_seeded`, `prop_test_normalize_and_hash_stable`, `prop_test_compute_thresholds_check_pure` all permanent
- **bifl-tracker validation harness** at `tools/validate-against-bifl-tracker.sh` — uses local checkout of `~/workspace/geoffgodwin/bifl-tracker`, runs `sdi snapshot` at a fixed set of commits across its history (the same commits sdi-py validated against), and compares snapshot output to sdi-py's stored snapshots. Acceptable variance per KD11: modularity within 1%, community count within ±10%, pattern entropy within 5%. The compared sdi-py snapshots are pinned in `tests/fixtures/bifl-tracker-baselines/`.
- **Pure-compute parity check** in the same harness: for each fixture commit, run `sdi-pipeline::Pipeline::snapshot` (Snapshot A); separately construct `DependencyGraphInput` + `Vec<PatternInstanceInput>` from the same parsed `FeatureRecord`s and call `sdi-core::compute_*` (results B); assert A's per-dimension metrics match B's within FMA tolerance. This validates that the consumer app's WASM-mediated path (M12) produces the same answers as the native CLI given equivalent input.

**Files to create or modify:**
- `README.md`, `docs/{cli-integration,library-embedding,migrating-from-sdi-py,determinism}.md` — `migrating-from-sdi-py.md` already exists from Milestone 10 with the comment-loss section; expand with the full carry/change matrix
- `examples/{embed_pipeline,embed_compute,custom_config}.rs`
- Doc comments throughout `crates/sdi-core/src/**/*.rs`
- `crates/sdi-core/src/lib.rs` enables `#![deny(missing_docs)]`
- `proptest-regressions/` directories per crate (committed)
- `tools/validate-against-bifl-tracker.sh`, `tests/fixtures/bifl-tracker-baselines/` (pinned sdi-py snapshots from the bifl-tracker validation history)

**Acceptance criteria:**
- `cargo doc --workspace --no-deps` produces no warnings
- `cargo test --doc` passes; broken examples fail CI
- `cargo run --example embed_pipeline` succeeds against a fixture
- `cargo run --example embed_compute` succeeds against a fixture, producing equivalent results to `embed_pipeline` for the metrics they share
- README under 200 lines
- All four `docs/*.md` files exist with non-trivial content
- `proptest` regression files exist and are loaded by tests
- `tools/validate-against-bifl-tracker.sh` runs end-to-end against the local bifl-tracker checkout and passes within the documented tolerances. Pure-compute parity check also passes. A failing comparison blocks the milestone.

**Tests:**
- Doc tests on every public function in `sdi-core` with an `# Examples` block
- `examples/` runnable as `cargo run --example <name>`
- A fresh `cargo test` from a clean checkout passes including doctests
- Pure-compute parity test inside the bifl-tracker harness

**Watch For:**
- `#![deny(missing_docs)]` will surface every undocumented public item — expect a substantial doc-writing pass on the new M08 surface (`compute_*`, input structs, `normalize_and_hash`)
- Doc tests are slow to compile; group related examples to avoid linking overhead
- Examples must not require network or external services
- The migration guide must be honest about the snapshot clean break — don't oversell read-compat
- The consumer-app-style example in `docs/library-embedding.md` should not import the actual consumer-app repo; build a synthetic mini-extractor inline so the example is self-contained

**Seeds Forward:**
- The doc structure is the canonical reference for embedders. M12 (WASM) cross-links `docs/library-embedding.md` from the `sdi-wasm` README.
- `proptest` regressions stay in CI permanently — a regression file commit is mandatory after a flaky test surfaces a real shrinkage.
- The bifl-tracker baselines are the v0 acceptance gate; updating them after 0.1.0 is a deliberate decision (any tolerance breach during 0.x patch work blocks the patch).

---
