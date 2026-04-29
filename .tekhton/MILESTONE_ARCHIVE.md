# Milestone Archive

Completed milestone definitions archived from CLAUDE.md.
See git history for the commit that completed each milestone.

---

## Archived: 2026-04-28 — Unknown Initiative

#### Milestone 1: Workspace Scaffold and `sdi-core` Skeleton
<!-- milestone-meta
id: "01"
status: "done"
-->


**Scope:** Create the Cargo workspace with all crates as empty shells, wire up CI, finalize MSRV, reserve crate names on crates.io with `0.0.0` placeholders, and stand up the `Config` struct and `ExitCode` enum — the two types every other crate depends on. License (Apache 2.0) and `sdi`-name workaround are already ratified during planning; this milestone just executes them. No real analysis logic yet.

**Deliverables:**
- Cargo workspace with `crates/sdi-core`, `crates/sdi-cli`, `crates/sdi-parsing`, `crates/sdi-graph`, `crates/sdi-detection`, `crates/sdi-patterns`, `crates/sdi-snapshot`, `crates/sdi-config`, and the six `sdi-lang-*` adapter crates as compile-but-empty libraries
- `Config` struct in `sdi-config` with `Default`, full schema mirroring DESIGN, and 5-level precedence loader stub returning defaults
- `ExitCode` closed enum in `sdi-core::exit_code` with explicit `i32` discriminants (`Success=0, RuntimeError=1, ConfigError=2, AnalysisError=3, ThresholdExceeded=10`)
- `sdi-cli` builds a `sdi --version` binary
- `LICENSE` (Apache 2.0) and `NOTICE` already in place from planning; verify contents match upstream; every crate's `Cargo.toml` sets `license = "Apache-2.0"`
- `rust-toolchain.toml` pinning MSRV to "stable latest minus 2"
- GitHub Actions: `ci.yml` (clippy, fmt, test on Linux/macOS/Windows × stable/MSRV); `release.yml` skeleton (no publish yet); `audit.yml` weekly
- Crate names reserved on crates.io with empty `0.0.0` placeholders. Names to publish: `sdi-rust` (the install-discovery meta-crate; users `cargo install sdi-rust`), `sdi-core`, `sdi-cli`, `sdi-config`, `sdi-parsing`, `sdi-graph`, `sdi-detection`, `sdi-patterns`, `sdi-snapshot`, `sdi-lang-rust`, `sdi-lang-python`, `sdi-lang-typescript`, `sdi-lang-javascript`, `sdi-lang-go`, `sdi-lang-java`, `sdi-py`, `sdi-node`. **The bare `sdi` is unavailable** (taken by an unrelated DI library); the binary stays `sdi` via `[[bin]] name = "sdi"` in `sdi-cli`'s `Cargo.toml`

**Files to create or modify:**
- `Cargo.toml` (workspace, pinned dep versions with `.workspace = true`)
- `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`
- `LICENSE`, `NOTICE`, `README.md`, `CHANGELOG.md` (all already exist; `Cargo.toml` workspace metadata wires them in)
- `crates/sdi-core/{Cargo.toml,src/lib.rs,src/exit_code.rs,src/error.rs}`
- `crates/sdi-cli/{Cargo.toml,src/main.rs}`
- `crates/sdi-config/{Cargo.toml,src/lib.rs,src/config.rs,src/load.rs,src/error.rs}`
- Empty `crates/sdi-{parsing,graph,detection,patterns,snapshot}/{Cargo.toml,src/lib.rs}`
- Empty `crates/sdi-lang-{python,typescript,javascript,go,java,rust}/{Cargo.toml,src/lib.rs}`
- `.github/workflows/{ci.yml,release.yml,audit.yml}`

**Acceptance criteria:**
- `cargo build --workspace` succeeds on Linux, macOS, Windows
- `cargo build -p sdi-cli --release` produces an `sdi` binary
- `sdi --version` prints the version from `Cargo.toml`
- `cargo clippy --workspace -- -D warnings` is clean
- `cargo fmt --all --check` is clean
- `Config::default()` returns a struct matching every default in DESIGN's complete config block
- `ExitCode::Success as i32 == 0`, etc., for all five variants
- CI runs green on push and PR

**Tests:**
- `crates/sdi-config/tests/defaults.rs`: assert every field of `Config::default()` matches DESIGN
- `crates/sdi-core/tests/exit_code_contract.rs`: assert each variant casts to its documented `i32`
- `crates/sdi-cli/tests/version.rs`: `assert_cmd::Command::cargo_bin("sdi").arg("--version")` succeeds

**Watch For:**
- Crate name re-check before publishing — availabilities were verified 2026-04-28 but crates.io is first-come; re-run `cargo search` against each name immediately before `cargo publish`
- Publish order matters: leaf crates first (`sdi-config`, `sdi-lang-*`), then `sdi-parsing`/`sdi-graph`/`sdi-detection`/`sdi-patterns`/`sdi-snapshot`, then `sdi-core`, then `sdi-cli`, then `sdi-rust` (meta). For empty `0.0.0` placeholders this ordering is cosmetic but for real publishes in m11 it's load-bearing
- crates.io is **append-only** — once `0.0.0` is published it stays; do not panic about needing version bumps later
- MSRV drift: pin a concrete version in `rust-toolchain.toml` and add an MSRV row to the CI matrix
- Every published crate's `Cargo.toml` needs `license = "Apache-2.0"` and a `description` (crates.io rejects publishes without one)
- Workspace `[workspace.dependencies]` block must list every external dep with a pinned version; member crates use `dep.workspace = true`

**Seeds Forward:**
- Every later milestone consumes `Config` and `ExitCode` — their public shape is now load-bearing
- The empty `LanguageAdapter` trait location (`sdi-parsing::adapter`) is the extension point that all adapter crates will implement
- `crates/sdi-cli/src/commands/` is created in Milestone 8; its skeleton lives here as a directory but is not populated
- The CI matrix established here is extended in later milestones (verify-leiden gate added in Milestone 5, release publish in Milestone 11)

---

---

## Archived: 2026-04-28 — Unknown Initiative

#### Milestone 2: Config Loading + Boundary Spec Reader
<!-- milestone-meta
id: "2"
status: "done"
-->


**Scope:** Make `Config::load_or_default` actually walk the 5-level precedence chain and parse TOML. Implement `BoundarySpec` reader from YAML (read-only — write is Milestone 9). Threshold overrides with `expires` validation. Wire `sdi init` so we have a usable command.

**Deliverables:**
- Working `Config::load_or_default(path)` resolving CLI flags > env > project > global > defaults
- TOML parser with structured `ConfigError` variants (`Parse`, `InvalidValue { key, message }`, `MissingExpiresOnOverride { category }`)
- Per-category threshold overrides parsed; missing `expires` errors; expired overrides silently ignored
- Unknown-key deprecation warnings to stderr (never error)
- `BoundarySpec` reader from `.sdi/boundaries.yaml` via `serde_yaml`
- `sdi init` command writes a default `.sdi/config.toml` and detects languages from file extensions
- Env vars wired: `SDI_LOG_LEVEL`, `SDI_WORKERS`, `SDI_CONFIG_PATH`, `SDI_SNAPSHOT_DIR`, `NO_COLOR`

**Files to create or modify:**
- `crates/sdi-config/src/load.rs` (real implementation)
- `crates/sdi-config/src/thresholds.rs`
- `crates/sdi-config/src/boundary.rs`
- `crates/sdi-config/src/error.rs` (extend variants)
- `crates/sdi-cli/src/commands/init.rs`
- `crates/sdi-cli/src/commands/mod.rs` registers `init`

**Acceptance criteria:**
- `sdi init` in an empty repo writes `.sdi/config.toml` matching the DESIGN default block byte-for-byte
- Config with `[thresholds.overrides.foo]` missing `expires` exits with code 2 and a clear error message naming the category
- An expired override (date in past) is loaded without error and behaves as if absent
- `SDI_CONFIG_PATH=/tmp/x.toml sdi init` reads from that path
- Unknown key like `[unknown_section]` produces a stderr deprecation warning and otherwise succeeds
- `sdi-py`'s real `.sdi/config.toml` (taken from the bifl-tracker fixtures) loads cleanly

**Tests:**
- `crates/sdi-config/tests/precedence.rs`: layered configs, env overrides win, CLI overrides env
- `crates/sdi-config/tests/threshold_overrides.rs`: missing `expires` → error; expired → ignored; valid → applied
- `crates/sdi-config/tests/sdi_py_compat.rs`: load fixture configs from sdi-py, assert success
- `crates/sdi-cli/tests/init.rs`: `sdi init` writes the expected file; running twice does not clobber existing config

**Watch For:**
- Date parsing: `expires` is a date string (`"2026-09-30"`). Use `toml::value::Datetime` and validate it parses as a date, not datetime — sdi-py accepts date-only
- `core.exclude` and `patterns.scope_exclude` are **replaced** on override, not merged — easy to get wrong with a default `extend` reducer
- `.sdi/config.toml` must not be overwritten if it already exists (`sdi init` is idempotent in that direction)
- YAML parser cannot preserve comments — explicitly accepted per KDD-6, but test-cover the read path against a sdi-py boundaries.yaml fixture

**Seeds Forward:**
- The `Config` struct is now real and consumed by `Pipeline::new` in Milestone 6
- `BoundarySpec` reader becomes input to snapshot assembly in Milestone 7
- `ConfigError` variants are stable from here; new variants are non-breaking via `#[non_exhaustive]`
- Milestone 9 (`sdi boundaries ratify`) depends on this read path; the comment-loss-on-write decision lives there

---

---

## Archived: 2026-04-29 — Unknown Initiative

#### Milestone 3: Parsing Stage with One Language Adapter (Rust)
<!-- milestone-meta
id: "3"
status: "done"
-->


**Scope:** Stand up the parsing pipeline end-to-end with a single language: Rust itself (dogfood). File walker, `LanguageAdapter` trait, `FeatureRecord` struct, parallel parsing via `rayon`. Enforce the CST-drop ownership invariant. The other five adapters land in Milestone 4.

**Deliverables:**
- `LanguageAdapter` trait in `sdi-parsing::adapter` with methods to parse a file and emit a `FeatureRecord`
- `FeatureRecord` struct: path, imports (Vec<String>), exports, function/class/method signatures, pattern instance handles. `serde::Serialize + Deserialize`
- `parse_repository(&Config, &Path) -> impl Iterator<Item = FeatureRecord>` doing breadth-first stable-sorted walk
- `walkdir` + `ignore` + `globset` honoring `.gitignore` and `core.exclude`
- `rayon` parallel parsing; per-worker grammar instance
- `sdi-lang-rust` crate implementing `LanguageAdapter` with `tree-sitter-rust` linked at compile time behind feature `lang-rust`

**Files to create or modify:**
- `crates/sdi-parsing/src/{adapter.rs,feature_record.rs,walker.rs,parse.rs}`
- `crates/sdi-lang-rust/{Cargo.toml,build.rs,src/lib.rs}`
- `tests/fixtures/simple-rust/` with 5–10 known files (cargo crate skeleton, lib.rs with declared modules, mod files)

**Acceptance criteria:**
- `parse_repository` on `tests/fixtures/simple-rust/` returns the same `Vec<FeatureRecord>` (after sorting) on every run
- The fixture has known import counts; assertion in test
- Memory invariant: a test that parses a 1MB Rust file and asserts peak heap stays bounded (use a `tracking-allocator` or count `Tree` allocations via a feature-gated counter)
- Parsing on an empty directory returns zero records, no error
- `core.exclude` glob suppresses files; `.gitignore` is honored

**Tests:**
- `crates/sdi-parsing/tests/walk_ordering.rs`: walk twice, assert identical paths
- `crates/sdi-parsing/tests/memory_invariant.rs`: parse 100 large files, assert no `Tree` survives across files (use a feature-gated `Drop` counter on a wrapper type around `tree_sitter::Tree`)
- `tests/full_pipeline.rs` (top-level): parse fixture, assert `FeatureRecord` count matches a hand-counted constant
- Property test in `crates/sdi-parsing/tests/proptest.rs`: random file content → parse never panics

**Watch For:**
- The parsing API must consume `String` (or `Vec<u8>`) by value and the returned `FeatureRecord` must own no reference into the input — otherwise the CST-drop invariant becomes a lifetime puzzle
- `tree-sitter` grammar instances are not `Send` in some grammar versions; verify before using `rayon::par_iter`. Fall back to per-worker `thread_local!` grammars if needed
- Stable-sort the file list **before** parallelizing; otherwise rayon's internal scheduling can leak ordering nondeterminism into downstream stages
- `walkdir` + `ignore` interaction: use the `ignore` crate's `WalkBuilder` rather than composing manually — `.gitignore` semantics are subtle

**Seeds Forward:**
- The `LanguageAdapter` trait is stable from here. Milestone 4 adds five adapters that implement it without changing the trait
- `FeatureRecord` is the input to Milestone 5 (graph) and Milestone 6 (patterns) — its shape must accommodate both. Pattern instance handles must include enough metadata for the patterns stage without reparsing
- The deterministic walk order is a load-bearing assumption for snapshot bit-stability

---

---

## Archived: 2026-04-29 — Unknown Initiative

#### Milestone 4: Remaining Language Adapters (Python, TS, JS, Go, Java)
<!-- milestone-meta
id: "04"
status: "done"
-->


**Scope:** Implement `LanguageAdapter` for the five remaining default languages. Each in its own crate behind a Cargo feature flag. Compile-time grammar linking. Per-language test fixture.

**Deliverables:**
- `sdi-lang-python`, `sdi-lang-typescript`, `sdi-lang-javascript`, `sdi-lang-go`, `sdi-lang-java` crates each with feature gate and `tree-sitter-<lang>` build dep
- Default workspace feature set enables all six languages (matching sdi-py)
- Per-language minimal fixture under `tests/fixtures/simple-<lang>/`
- Multi-language fixture `tests/fixtures/multi-language/` with Python + TypeScript files
- Language detection by extension wired in the file walker

**Files to create or modify:**
- `crates/sdi-lang-{python,typescript,javascript,go,java}/{Cargo.toml,build.rs,src/lib.rs}`
- `crates/sdi-parsing/src/walker.rs` (extension → adapter dispatch table)
- `tests/fixtures/simple-{python,typescript,javascript,go,java}/`
- `tests/fixtures/multi-language/`

**Acceptance criteria:**
- `cargo build --workspace --no-default-features --features lang-python` produces a binary supporting only Python
- `cargo build --workspace` (default features) produces a binary supporting all six
- Each fixture parses to a known `FeatureRecord` count
- Multi-language fixture produces records from both Python and TypeScript files in a single run
- File with extension matching no enabled grammar is skipped with a stderr DEBUG log

**Tests:**
- `tests/full_pipeline.rs` extended: parse each `simple-<lang>/` fixture
- `tests/multi_language.rs`: parse multi-language fixture, assert per-language record counts
- `crates/sdi-parsing/tests/grammar_missing.rs`: build with only `lang-rust`, parse a Python file, assert skip-with-warning behavior

**Watch For:**
- `tree-sitter-typescript` ships two grammars (TSX and TS) — pick one per `.ts` vs `.tsx` extension. Document the choice
- `tree-sitter-go` and `tree-sitter-java` may have outdated crates.io versions; if so, vendor via `[patch.crates-io]` and add a `DRIFT_LOG.md` entry per dependency strategy in DESIGN
- Build times balloon with all six grammars enabled — keep MSRV CI matrix from doubling by caching `~/.cargo` between jobs
- Each adapter's `FeatureRecord` output must be equivalent (under sorted-by-path normalization) to sdi-py's parsing of the same files. **The TS/JS adapter parity is load-bearing** because Milestone 5's `verify-leiden` suite parses bifl-tracker (TypeScript-heavy) through both implementations; an upstream parsing divergence would alias as a Leiden-quality regression. For the other adapters, parity is verified at fixture level only (hand-counted totals in `tests/fixtures/simple-<lang>/`)

**Seeds Forward:**
- The six adapters are the public-facing language support of MVP. Adding a seventh language post-MVP must use this same trait without modification
- The fixture set established here is reused by every subsequent milestone (graph, detection, patterns, snapshot)
- Multi-language fixture is the basis for the verification suite in Milestone 5

---

---

## Archived: 2026-04-29 — Unknown Initiative

#### Milestone 5: Dependency Graph + Native Leiden Port
<!-- milestone-meta
id: "5"
status: "done"
-->


**Scope:** Build the dependency graph from `FeatureRecord` outputs (`sdi-graph`) and implement the native Leiden community detection (`sdi-detection`). This is the KD11 hot zone — the largest single milestone. Verification suite against `leidenalg` is set up here, gated behind feature `verify-leiden`.

**Deliverables:**
- `DependencyGraph` built on `petgraph::Graph<NodeId, EdgeWeight>`; node-per-file (or per-top-level-module per adapter); directed edges from resolved imports
- Graph metrics: density, cycle count via DFS (excluding self-loops), top-degree hubs, connected component count
- Native Leiden implementation in `sdi-detection::leiden`: Modularity and CPM quality functions; ~1500–2500 LOC total across `mod.rs`, `modularity.rs`, `cpm.rs`, `refine.rs`, `aggregate.rs`
- `LeidenPartition` struct: cluster assignments + per-cluster stability score
- Warm-start path reading `.sdi/cache/partition.json`; cold-start uses `Config::random_seed`
- `verify-leiden` feature: cross-check fixture suite (50, 500, 5000 nodes, including parsed bifl-tracker) against `leidenalg` via a Python harness; assert modularity within 1% and community count within ±10%
- `tools/generate-leiden-fixtures.py` — one-time Python harness that takes a parsed sdi-py snapshot, extracts the dependency graph, runs `leidenalg` with a fixed seed, and emits the `tests/fixtures/leiden-graphs/{small,medium,large}/` adjacency lists and reference modularity values. Re-runnable when fixtures need refresh
- New CI job `verify-leiden.yml` running the gated suite (skipped on PRs without the feature)
- **CSR view decision** — profile the native Leiden against `petgraph::Graph<NodeId, EdgeWeight>` as the adjacency layer. Decide explicitly: keep `petgraph` everywhere, or build a CSR-view module in `sdi-graph::csr` for the detection hot path. Document the decision (whichever way) in `DRIFT_LOG.md` and update KDD-5 in `CLAUDE.md` to "ratified yes/no" before closing the milestone

**Files to create or modify:**
- `crates/sdi-graph/src/{dependency_graph.rs,metrics.rs}` (real implementation)
- `crates/sdi-detection/src/leiden/{mod.rs,modularity.rs,cpm.rs,refine.rs,aggregate.rs}`
- `crates/sdi-detection/src/{partition.rs,warm_start.rs}`
- `crates/sdi-detection/Cargo.toml` (feature `verify-leiden`)
- `crates/sdi-detection/tests/leiden_quality.rs` (gated)
- `tests/fixtures/leiden-graphs/{small,medium,large}/` (parsed adjacency lists + reference modularities from leidenalg, generated by `tools/generate-leiden-fixtures.py`)
- `tools/generate-leiden-fixtures.py`
- `.github/workflows/verify-leiden.yml`
- `DRIFT_LOG.md` entry for the CSR view decision (yes or no — both outcomes are recorded)

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
- The CSR-view decision is now an explicit deliverable (see Deliverables). Profile under realistic graph sizes (5000+ nodes from bifl-tracker) before deciding; "no" is a valid outcome if `petgraph` is fast enough
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

---

## Archived: 2026-04-29 — Unknown Initiative

#### Milestone 6: Pattern Fingerprinting and Catalog
<!-- milestone-meta
id: "06"
status: "done"
-->


**Scope:** Implement `sdi-patterns` — extract per-category subtree shapes from `FeatureRecord` pattern handles, hash with `blake3`, build a `PatternCatalog`, compute pattern entropy. This is the Stage 4 of the pipeline. `sdi-patterns` does **NOT** depend on `sdi-graph` or `sdi-detection` — DESIGN dependency rule.

**Deliverables:**
- `PatternFingerprint` newtype around a `[u8; 32]` blake3 digest
- `PatternCatalog` keyed by `BTreeMap<CategoryName, BTreeMap<PatternFingerprint, PatternStats>>` with instance counts and per-fingerprint file-location lists
- Per-category tree-sitter query strings in `sdi-patterns::queries::<category>` for the default categories (`error_handling`, `async_patterns`, `state_management`, …)
- Pattern entropy calculator (distinct-shape count adjusted for instance distribution)
- `Config::patterns.min_pattern_nodes` filter and `Config::patterns.scope_exclude` excluding files from the catalog only — files remain in graph and partition
- `sdi catalog` command printing the catalog as JSON or text

**Files to create or modify:**
- `crates/sdi-patterns/src/{lib.rs,catalog.rs,fingerprint.rs,entropy.rs}`
- `crates/sdi-patterns/src/queries/{mod.rs,error_handling.rs,async_patterns.rs,...}`
- `crates/sdi-cli/src/commands/catalog.rs`
- `tests/fixtures/high-entropy/` (deliberate variance)

**Acceptance criteria:**
- Same fixture + same config → bit-identical `PatternCatalog` JSON across 100 runs
- `scope_exclude` removes files from the catalog but does not change graph/partition output
- `min_pattern_nodes = 5` filters subtrees with fewer than 5 nodes
- `high-entropy/` fixture produces a higher entropy score than `simple-rust/`
- `sdi catalog --format json` outputs valid JSON to stdout; logs go to stderr
- `blake3` is keyed with the fixed key constant defined exactly once

**Tests:**
- `crates/sdi-patterns/tests/determinism.rs`: 100-run identical-output proptest
- `crates/sdi-patterns/tests/scope_exclude.rs`: file in `scope_exclude` absent from catalog, present in `FeatureRecord` stream
- `crates/sdi-patterns/tests/entropy_ordering.rs`: `entropy(high) > entropy(simple)`
- `crates/sdi-cli/tests/catalog_format.rs`: JSON and text formats both succeed

**Watch For:**
- Tree-sitter queries must be parsed once per category, not per file — cache them in a `OnceCell` keyed by `(language, category)`
- The pattern instance handles in `FeatureRecord` must carry enough info to re-extract the subtree shape without re-walking the CST (the CST has been dropped per Rule 4). If they don't, this milestone has to push some work back into Milestone 3 — flag early
- `BTreeMap` ordering is critical for determinism; `IndexMap` would also work but is forbidden by KDD-10 unless profiling demands
- `categories = "auto"` resolution depends on which languages are present — implement detection from `FeatureRecord` languages

**Seeds Forward:**
- `PatternCatalog` is an input to snapshot assembly in Milestone 7
- The category-name set is publicly stable from here. Adding a category is non-breaking; renaming is breaking
- `sdi catalog` command shape sets the precedent for `sdi show` formatting in Milestone 8

---

---

## Archived: 2026-04-29 — Unknown Initiative

#### Milestone 7: Snapshot Assembly, Delta, and Persistence
<!-- milestone-meta
id: "7"
status: "done"
-->


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
