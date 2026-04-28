#### Milestone 10: Documentation, Examples, Determinism Polish

**Scope:** Stand up the documentation surfaces (`README.md`, `docs/*.md`, rustdoc with `#![deny(missing_docs)]`). Doc tests in CI. Runnable examples. Tighten determinism with `proptest` regression suite and FMA documentation.

**Deliverables:**
- `README.md` quick start, install paths (cargo, brew, binary), one-paragraph SDI overview, links — under 200 lines
- `docs/cli-integration.md` with `cargo install sdi && sdi check` GHA snippet and exit-code reference
- `docs/library-embedding.md` with minimal embedder
- `docs/migrating-from-sdi-py.md` with carry/change matrix
- `docs/determinism.md` with `BTreeMap` discipline, seed contract, FMA notes
- `examples/embed_pipeline.rs` and `examples/custom_config.rs`
- `#![deny(missing_docs)]` enabled on `sdi-core` with docs for every public item
- `cargo test --doc` runs in CI; no broken doc tests
- `proptest` regression directory checked in; `prop_test_pipeline_deterministic`, `prop_test_delta_pure`, `prop_test_leiden_seeded` all permanent

**Files to create or modify:**
- `README.md`, `docs/{cli-integration,library-embedding,migrating-from-sdi-py,determinism}.md`
- `examples/{embed_pipeline,custom_config}.rs`
- Doc comments throughout `crates/sdi-core/src/**/*.rs`
- `crates/sdi-core/src/lib.rs` enables `#![deny(missing_docs)]`
- `proptest-regressions/` directories per crate (committed)

**Acceptance criteria:**
- `cargo doc --workspace --no-deps` produces no warnings
- `cargo test --doc` passes; broken examples fail CI
- `cargo run --example embed_pipeline` succeeds against a fixture
- README under 200 lines
- All four `docs/*.md` files exist with non-trivial content
- `proptest` regression files exist and are loaded by tests

**Tests:**
- Doc tests on every public function with an `# Examples` block
- `examples/` runnable as `cargo run --example <name>`
- A fresh `cargo test` from a clean checkout passes including doctests

**Watch For:**
- `#![deny(missing_docs)]` will surface every undocumented public item — expect a substantial doc-writing pass
- Doc tests are slow to compile; group related examples to avoid linking overhead
- Examples must not require network or external services
- The migration guide must be honest about the snapshot clean break — don't oversell read-compat

**Seeds Forward:**
- The doc structure is the canonical reference for embedders. Bindings (Milestone 12) link to `docs/library-embedding.md`
- `proptest` regressions stay in CI permanently — a regression file commit is mandatory after a flaky test surfaces a real shrinkage

---
