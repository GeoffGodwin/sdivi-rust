# Milestone Archive

Completed milestone definitions archived from CLAUDE.md.
See git history for the commit that completed each milestone.

---

## Archived: 2026-04-28 — Unknown Initiative

#### Milestone 1: Workspace Scaffold and `sdivi-core` Skeleton
<!-- milestone-meta
id: "01"
status: "done"
-->


**Scope:** Create the Cargo workspace with all crates as empty shells, wire up CI, finalize MSRV, reserve crate names on crates.io with `0.0.0` placeholders, and stand up the `Config` struct and `ExitCode` enum — the two types every other crate depends on. License (Apache 2.0) and `sdivi`-name workaround are already ratified during planning; this milestone just executes them. No real analysis logic yet.

**Deliverables:**
- Cargo workspace with `crates/sdivi-core`, `crates/sdivi-cli`, `crates/sdivi-parsing`, `crates/sdivi-graph`, `crates/sdivi-detection`, `crates/sdivi-patterns`, `crates/sdivi-snapshot`, `crates/sdivi-config`, and the six `sdivi-lang-*` adapter crates as compile-but-empty libraries
- `Config` struct in `sdivi-config` with `Default`, full schema mirroring DESIGN, and 5-level precedence loader stub returning defaults
- `ExitCode` closed enum in `sdivi-core::exit_code` with explicit `i32` discriminants (`Success=0, RuntimeError=1, ConfigError=2, AnalysisError=3, ThresholdExceeded=10`)
- `sdivi-cli` builds a `sdivi --version` binary
- `LICENSE` (Apache 2.0) and `NOTICE` already in place from planning; verify contents match upstream; every crate's `Cargo.toml` sets `license = "Apache-2.0"`
- `rust-toolchain.toml` pinning MSRV to "stable latest minus 2"
- GitHub Actions: `ci.yml` (clippy, fmt, test on Linux/macOS/Windows × stable/MSRV); `release.yml` skeleton (no publish yet); `audit.yml` weekly
- Crate names reserved on crates.io with empty `0.0.0` placeholders. Names to publish: `sdivi-rust` (the install-discovery meta-crate; users `cargo install sdivi-rust`), `sdivi-core`, `sdivi-cli`, `sdivi-config`, `sdivi-parsing`, `sdivi-graph`, `sdivi-detection`, `sdivi-patterns`, `sdivi-snapshot`, `sdivi-lang-rust`, `sdivi-lang-python`, `sdivi-lang-typescript`, `sdivi-lang-javascript`, `sdivi-lang-go`, `sdivi-lang-java`, `sdi-py`, `sdivi-node`. **The bare `sdivi` is unavailable** (taken by an unrelated DI library); the binary stays `sdivi` via `[[bin]] name = "sdivi"` in `sdivi-cli`'s `Cargo.toml`

**Files to create or modify:**
- `Cargo.toml` (workspace, pinned dep versions with `.workspace = true`)
- `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`
- `LICENSE`, `NOTICE`, `README.md`, `CHANGELOG.md` (all already exist; `Cargo.toml` workspace metadata wires them in)
- `crates/sdivi-core/{Cargo.toml,src/lib.rs,src/exit_code.rs,src/error.rs}`
- `crates/sdivi-cli/{Cargo.toml,src/main.rs}`
- `crates/sdivi-config/{Cargo.toml,src/lib.rs,src/config.rs,src/load.rs,src/error.rs}`
- Empty `crates/sdivi-{parsing,graph,detection,patterns,snapshot}/{Cargo.toml,src/lib.rs}`
- Empty `crates/sdivi-lang-{python,typescript,javascript,go,java,rust}/{Cargo.toml,src/lib.rs}`
- `.github/workflows/{ci.yml,release.yml,audit.yml}`

**Acceptance criteria:**
- `cargo build --workspace` succeeds on Linux, macOS, Windows
- `cargo build -p sdivi-cli --release` produces an `sdivi` binary
- `sdivi --version` prints the version from `Cargo.toml`
- `cargo clippy --workspace -- -D warnings` is clean
- `cargo fmt --all --check` is clean
- `Config::default()` returns a struct matching every default in DESIGN's complete config block
- `ExitCode::Success as i32 == 0`, etc., for all five variants
- CI runs green on push and PR

**Tests:**
- `crates/sdivi-config/tests/defaults.rs`: assert every field of `Config::default()` matches DESIGN
- `crates/sdivi-core/tests/exit_code_contract.rs`: assert each variant casts to its documented `i32`
- `crates/sdivi-cli/tests/version.rs`: `assert_cmd::Command::cargo_bin("sdivi").arg("--version")` succeeds

**Watch For:**
- Crate name re-check before publishing — availabilities were verified 2026-04-28 but crates.io is first-come; re-run `cargo search` against each name immediately before `cargo publish`
- Publish order matters: leaf crates first (`sdivi-config`, `sdivi-lang-*`), then `sdivi-parsing`/`sdivi-graph`/`sdivi-detection`/`sdivi-patterns`/`sdivi-snapshot`, then `sdivi-core`, then `sdivi-cli`, then `sdivi-rust` (meta). For empty `0.0.0` placeholders this ordering is cosmetic but for real publishes in m11 it's load-bearing
- crates.io is **append-only** — once `0.0.0` is published it stays; do not panic about needing version bumps later
- MSRV drift: pin a concrete version in `rust-toolchain.toml` and add an MSRV row to the CI matrix
- Every published crate's `Cargo.toml` needs `license = "Apache-2.0"` and a `description` (crates.io rejects publishes without one)
- Workspace `[workspace.dependencies]` block must list every external dep with a pinned version; member crates use `dep.workspace = true`

**Seeds Forward:**
- Every later milestone consumes `Config` and `ExitCode` — their public shape is now load-bearing
- The empty `LanguageAdapter` trait location (`sdivi-parsing::adapter`) is the extension point that all adapter crates will implement
- `crates/sdivi-cli/src/commands/` is created in Milestone 8; its skeleton lives here as a directory but is not populated
- The CI matrix established here is extended in later milestones (verify-leiden gate added in Milestone 5, release publish in Milestone 11)

---

---

## Archived: 2026-04-28 — Unknown Initiative

#### Milestone 2: Config Loading + Boundary Spec Reader
<!-- milestone-meta
id: "2"
status: "done"
-->


**Scope:** Make `Config::load_or_default` actually walk the 5-level precedence chain and parse TOML. Implement `BoundarySpec` reader from YAML (read-only — write is Milestone 9). Threshold overrides with `expires` validation. Wire `sdivi init` so we have a usable command.

**Deliverables:**
- Working `Config::load_or_default(path)` resolving CLI flags > env > project > global > defaults
- TOML parser with structured `ConfigError` variants (`Parse`, `InvalidValue { key, message }`, `MissingExpiresOnOverride { category }`)
- Per-category threshold overrides parsed; missing `expires` errors; expired overrides silently ignored
- Unknown-key deprecation warnings to stderr (never error)
- `BoundarySpec` reader from `.sdivi/boundaries.yaml` via `serde_yaml`
- `sdivi init` command writes a default `.sdivi/config.toml` and detects languages from file extensions
- Env vars wired: `SDIVI_LOG_LEVEL`, `SDIVI_WORKERS`, `SDIVI_CONFIG_PATH`, `SDIVI_SNAPSHOT_DIR`, `NO_COLOR`

**Files to create or modify:**
- `crates/sdivi-config/src/load.rs` (real implementation)
- `crates/sdivi-config/src/thresholds.rs`
- `crates/sdivi-config/src/boundary.rs`
- `crates/sdivi-config/src/error.rs` (extend variants)
- `crates/sdivi-cli/src/commands/init.rs`
- `crates/sdivi-cli/src/commands/mod.rs` registers `init`

**Acceptance criteria:**
- `sdivi init` in an empty repo writes `.sdivi/config.toml` matching the DESIGN default block byte-for-byte
- Config with `[thresholds.overrides.foo]` missing `expires` exits with code 2 and a clear error message naming the category
- An expired override (date in past) is loaded without error and behaves as if absent
- `SDIVI_CONFIG_PATH=/tmp/x.toml sdivi init` reads from that path
- Unknown key like `[unknown_section]` produces a stderr deprecation warning and otherwise succeeds
- `sdi-py`'s real `.sdivi/config.toml` (taken from the bifl-tracker fixtures) loads cleanly

**Tests:**
- `crates/sdivi-config/tests/precedence.rs`: layered configs, env overrides win, CLI overrides env
- `crates/sdivi-config/tests/threshold_overrides.rs`: missing `expires` → error; expired → ignored; valid → applied
- `crates/sdivi-config/tests/sdi_py_compat.rs`: load fixture configs from sdi-py, assert success
- `crates/sdivi-cli/tests/init.rs`: `sdivi init` writes the expected file; running twice does not clobber existing config

**Watch For:**
- Date parsing: `expires` is a date string (`"2026-09-30"`). Use `toml::value::Datetime` and validate it parses as a date, not datetime — sdi-py accepts date-only
- `core.exclude` and `patterns.scope_exclude` are **replaced** on override, not merged — easy to get wrong with a default `extend` reducer
- `.sdivi/config.toml` must not be overwritten if it already exists (`sdivi init` is idempotent in that direction)
- YAML parser cannot preserve comments — explicitly accepted per KDD-6, but test-cover the read path against a sdi-py boundaries.yaml fixture

**Seeds Forward:**
- The `Config` struct is now real and consumed by `Pipeline::new` in Milestone 6
- `BoundarySpec` reader becomes input to snapshot assembly in Milestone 7
- `ConfigError` variants are stable from here; new variants are non-breaking via `#[non_exhaustive]`
- Milestone 9 (`sdivi boundaries ratify`) depends on this read path; the comment-loss-on-write decision lives there

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
- `LanguageAdapter` trait in `sdivi-parsing::adapter` with methods to parse a file and emit a `FeatureRecord`
- `FeatureRecord` struct: path, imports (Vec<String>), exports, function/class/method signatures, pattern instance handles. `serde::Serialize + Deserialize`
- `parse_repository(&Config, &Path) -> impl Iterator<Item = FeatureRecord>` doing breadth-first stable-sorted walk
- `walkdir` + `ignore` + `globset` honoring `.gitignore` and `core.exclude`
- `rayon` parallel parsing; per-worker grammar instance
- `sdivi-lang-rust` crate implementing `LanguageAdapter` with `tree-sitter-rust` linked at compile time behind feature `lang-rust`

**Files to create or modify:**
- `crates/sdivi-parsing/src/{adapter.rs,feature_record.rs,walker.rs,parse.rs}`
- `crates/sdivi-lang-rust/{Cargo.toml,build.rs,src/lib.rs}`
- `tests/fixtures/simple-rust/` with 5–10 known files (cargo crate skeleton, lib.rs with declared modules, mod files)

**Acceptance criteria:**
- `parse_repository` on `tests/fixtures/simple-rust/` returns the same `Vec<FeatureRecord>` (after sorting) on every run
- The fixture has known import counts; assertion in test
- Memory invariant: a test that parses a 1MB Rust file and asserts peak heap stays bounded (use a `tracking-allocator` or count `Tree` allocations via a feature-gated counter)
- Parsing on an empty directory returns zero records, no error
- `core.exclude` glob suppresses files; `.gitignore` is honored

**Tests:**
- `crates/sdivi-parsing/tests/walk_ordering.rs`: walk twice, assert identical paths
- `crates/sdivi-parsing/tests/memory_invariant.rs`: parse 100 large files, assert no `Tree` survives across files (use a feature-gated `Drop` counter on a wrapper type around `tree_sitter::Tree`)
- `tests/full_pipeline.rs` (top-level): parse fixture, assert `FeatureRecord` count matches a hand-counted constant
- Property test in `crates/sdivi-parsing/tests/proptest.rs`: random file content → parse never panics

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
- `sdivi-lang-python`, `sdivi-lang-typescript`, `sdivi-lang-javascript`, `sdivi-lang-go`, `sdivi-lang-java` crates each with feature gate and `tree-sitter-<lang>` build dep
- Default workspace feature set enables all six languages (matching sdi-py)
- Per-language minimal fixture under `tests/fixtures/simple-<lang>/`
- Multi-language fixture `tests/fixtures/multi-language/` with Python + TypeScript files
- Language detection by extension wired in the file walker

**Files to create or modify:**
- `crates/sdivi-lang-{python,typescript,javascript,go,java}/{Cargo.toml,build.rs,src/lib.rs}`
- `crates/sdivi-parsing/src/walker.rs` (extension → adapter dispatch table)
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
- `crates/sdivi-parsing/tests/grammar_missing.rs`: build with only `lang-rust`, parse a Python file, assert skip-with-warning behavior

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


**Scope:** Build the dependency graph from `FeatureRecord` outputs (`sdivi-graph`) and implement the native Leiden community detection (`sdivi-detection`). This is the KD11 hot zone — the largest single milestone. Verification suite against `leidenalg` is set up here, gated behind feature `verify-leiden`.

**Deliverables:**
- `DependencyGraph` built on `petgraph::Graph<NodeId, EdgeWeight>`; node-per-file (or per-top-level-module per adapter); directed edges from resolved imports
- Graph metrics: density, cycle count via DFS (excluding self-loops), top-degree hubs, connected component count
- Native Leiden implementation in `sdivi-detection::leiden`: Modularity and CPM quality functions; ~1500–2500 LOC total across `mod.rs`, `modularity.rs`, `cpm.rs`, `refine.rs`, `aggregate.rs`
- `LeidenPartition` struct: cluster assignments + per-cluster stability score
- Warm-start path reading `.sdivi/cache/partition.json`; cold-start uses `Config::random_seed`
- `verify-leiden` feature: cross-check fixture suite (50, 500, 5000 nodes, including parsed bifl-tracker) against `leidenalg` via a Python harness; assert modularity within 1% and community count within ±10%
- `tools/generate-leiden-fixtures.py` — one-time Python harness that takes a parsed sdi-py snapshot, extracts the dependency graph, runs `leidenalg` with a fixed seed, and emits the `tests/fixtures/leiden-graphs/{small,medium,large}/` adjacency lists and reference modularity values. Re-runnable when fixtures need refresh
- New CI job `verify-leiden.yml` running the gated suite (skipped on PRs without the feature)
- **CSR view decision** — profile the native Leiden against `petgraph::Graph<NodeId, EdgeWeight>` as the adjacency layer. Decide explicitly: keep `petgraph` everywhere, or build a CSR-view module in `sdivi-graph::csr` for the detection hot path. Document the decision (whichever way) in `DRIFT_LOG.md` and update KDD-5 in `CLAUDE.md` to "ratified yes/no" before closing the milestone

**Files to create or modify:**
- `crates/sdivi-graph/src/{dependency_graph.rs,metrics.rs}` (real implementation)
- `crates/sdivi-detection/src/leiden/{mod.rs,modularity.rs,cpm.rs,refine.rs,aggregate.rs}`
- `crates/sdivi-detection/src/{partition.rs,warm_start.rs}`
- `crates/sdivi-detection/Cargo.toml` (feature `verify-leiden`)
- `crates/sdivi-detection/tests/leiden_quality.rs` (gated)
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
- `crates/sdivi-graph/tests/metrics.rs`: hand-built graphs with known density, cycles, hubs
- `crates/sdivi-detection/tests/proptest_seeded.rs`: same seed → same partition (`prop_test_leiden_seeded`)
- `crates/sdivi-detection/tests/leiden_quality.rs` (gated `verify-leiden`): the cross-check suite
- `crates/sdivi-detection/tests/warm_start.rs`: stale cache path → first-iteration honors prior assignments

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
- `.sdivi/cache/partition.json` schema is fixed here; bumping it requires care to keep warm-start beneficial across snapshots

---

---

## Archived: 2026-04-29 — Unknown Initiative

#### Milestone 6: Pattern Fingerprinting and Catalog
<!-- milestone-meta
id: "06"
status: "done"
-->


**Scope:** Implement `sdivi-patterns` — extract per-category subtree shapes from `FeatureRecord` pattern handles, hash with `blake3`, build a `PatternCatalog`, compute pattern entropy. This is the Stage 4 of the pipeline. `sdivi-patterns` does **NOT** depend on `sdivi-graph` or `sdivi-detection` — DESIGN dependency rule.

**Deliverables:**
- `PatternFingerprint` newtype around a `[u8; 32]` blake3 digest
- `PatternCatalog` keyed by `BTreeMap<CategoryName, BTreeMap<PatternFingerprint, PatternStats>>` with instance counts and per-fingerprint file-location lists
- Per-category tree-sitter query strings in `sdivi-patterns::queries::<category>` for the default categories (`error_handling`, `async_patterns`, `state_management`, …)
- Pattern entropy calculator (distinct-shape count adjusted for instance distribution)
- `Config::patterns.min_pattern_nodes` filter and `Config::patterns.scope_exclude` excluding files from the catalog only — files remain in graph and partition
- `sdivi catalog` command printing the catalog as JSON or text

**Files to create or modify:**
- `crates/sdivi-patterns/src/{lib.rs,catalog.rs,fingerprint.rs,entropy.rs}`
- `crates/sdivi-patterns/src/queries/{mod.rs,error_handling.rs,async_patterns.rs,...}`
- `crates/sdivi-cli/src/commands/catalog.rs`
- `tests/fixtures/high-entropy/` (deliberate variance)

**Acceptance criteria:**
- Same fixture + same config → bit-identical `PatternCatalog` JSON across 100 runs
- `scope_exclude` removes files from the catalog but does not change graph/partition output
- `min_pattern_nodes = 5` filters subtrees with fewer than 5 nodes
- `high-entropy/` fixture produces a higher entropy score than `simple-rust/`
- `sdivi catalog --format json` outputs valid JSON to stdout; logs go to stderr
- `blake3` is keyed with the fixed key constant defined exactly once

**Tests:**
- `crates/sdivi-patterns/tests/determinism.rs`: 100-run identical-output proptest
- `crates/sdivi-patterns/tests/scope_exclude.rs`: file in `scope_exclude` absent from catalog, present in `FeatureRecord` stream
- `crates/sdivi-patterns/tests/entropy_ordering.rs`: `entropy(high) > entropy(simple)`
- `crates/sdivi-cli/tests/catalog_format.rs`: JSON and text formats both succeed

**Watch For:**
- Tree-sitter queries must be parsed once per category, not per file — cache them in a `OnceCell` keyed by `(language, category)`
- The pattern instance handles in `FeatureRecord` must carry enough info to re-extract the subtree shape without re-walking the CST (the CST has been dropped per Rule 4). If they don't, this milestone has to push some work back into Milestone 3 — flag early
- `BTreeMap` ordering is critical for determinism; `IndexMap` would also work but is forbidden by KDD-10 unless profiling demands
- `categories = "auto"` resolution depends on which languages are present — implement detection from `FeatureRecord` languages

**Seeds Forward:**
- `PatternCatalog` is an input to snapshot assembly in Milestone 7
- The category-name set is publicly stable from here. Adding a category is non-breaking; renaming is breaking
- `sdivi catalog` command shape sets the precedent for `sdivi show` formatting in Milestone 8

---

---

## Archived: 2026-04-29 — Unknown Initiative

#### Milestone 7: Snapshot Assembly, Delta, and Persistence
<!-- milestone-meta
id: "7"
status: "done"
-->


**Scope:** Assemble the `Snapshot` from graph + partition + catalog + boundary spec. Implement `compute_delta` as a pure function. Atomic snapshot write to `.sdivi/snapshots/`. Retention enforcement. Wire `sdivi snapshot` and `sdivi diff` end-to-end. The pipeline is now usable.

**Deliverables:**
- `Snapshot` struct in `sdivi-snapshot::snapshot` with `snapshot_version: "1.0"` and all fields from DESIGN's snapshot composition
- `compute_delta(prev: &Snapshot, curr: &Snapshot) -> DivergenceSummary` pure function; first-snapshot returns `null` per-dimension
- `Pipeline::snapshot` and `Pipeline::delta` methods on `sdivi-core::pipeline::Pipeline`
- Atomic write: tempfile in `.sdivi/snapshots/`, then rename. Retention enforced synchronously after write
- `sdivi snapshot [--commit REF] [--format json|text]` command; `sdivi diff <prev> <curr>` command
- Snapshot file naming `snapshot_<timestamp>_<sha>.json` (KDD per Open Q #8)

**Files to create or modify:**
- `crates/sdivi-snapshot/src/{snapshot.rs,delta.rs,store.rs,retention.rs}`
- `crates/sdivi-core/src/pipeline.rs` (real implementation; `Pipeline::new` cheap, `snapshot` runs all five stages)
- `crates/sdivi-cli/src/commands/{snapshot.rs,diff.rs}`
- `tests/full_pipeline.rs` (extended end-to-end)

**Acceptance criteria:**
- `sdivi snapshot` on `simple-rust` fixture writes a JSON file matching schema 1.0; running again produces a second file
- Delta on identical consecutive snapshots: all dimensions `0` (not `null`)
- Delta on first snapshot: all dimensions `null` (not `0`)
- Killing the process mid-write (simulated by injecting a panic before rename in a test) leaves the target directory free of half-written `.json` files
- `retention = 3` keeps only the 3 most recent snapshots after the third write
- Same input + config → bit-identical snapshot JSON (proptest `prop_test_pipeline_deterministic`)
- `sdivi diff <prev> <curr>` prints the divergence summary; exits 0
- Missing boundary spec: snapshot still produced, intent divergence fields absent — no warning

**Tests:**
- `crates/sdivi-snapshot/tests/atomic_write.rs`: simulate panic before rename, assert no leftover tempfile in target dir
- `crates/sdivi-snapshot/tests/retention.rs`: write N+1 with retention N, assert oldest deleted
- `crates/sdivi-snapshot/tests/delta_pure.rs`: `prop_test_delta_pure` referential transparency
- `crates/sdivi-snapshot/tests/null_vs_zero.rs`: first snapshot null, second-identical zero
- `tests/full_pipeline.rs`: end-to-end on every fixture
- `crates/sdivi-cli/tests/snapshot_diff.rs`: `assert_cmd` integration

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

---

## Archived: 2026-04-29 — Unknown Initiative

#### Milestone 8: `sdivi-core` Pure-Compute Reshape and WASM-readiness
<!-- milestone-meta
id: "8"
status: "done"
-->


**Scope:** Restructure the workspace so `sdivi-core` is a pure-compute, WASM-compatible facade exposing concrete `compute_*` functions over plain serde input structs. Move today's `sdivi-core` (Pipeline + I/O orchestration) into a new `sdivi-pipeline` crate. Extract I/O sites from `sdivi-detection::warm_start` and `sdivi-snapshot::store`. Feature-gate `sdivi-graph`/`sdivi-detection`/`sdivi-patterns`/`sdivi-snapshot` so their `FeatureRecord`-taking paths are opt-in (off for WASM). This is the single largest architectural milestone post-MVP-shift; it precedes the CLI work because the public `sdivi-core` API surface freezes at 0.1.0 and must be right *before* CLI/docs/release lock it in.

**Why this milestone exists:** A strict-mode TypeScript consumer app at the user's workplace is the first concrete consumer and needs to import sdivi-rust as a WASM library. Its mid-June review deadline is the calendar driver. Today's `sdivi-core` transitively pulls `tree-sitter`, `walkdir`, `ignore`, `rayon`, and `std::fs::*` — none WASM-compatible. WASM was originally KD14-deferred ("when a concrete consumer exists"); that condition now holds, and KDD-13 ratifies the v0 inclusion. Rule 18 says public API stability begins at 0.1.0, so the reshape must land before the M13 release tag.

**Deliverables:**

- **New crate `sdivi-pipeline`** at `crates/sdivi-pipeline/` containing the current `Pipeline::{new, snapshot, delta}` shape. Depends on `sdivi-parsing`, `sdivi-graph`, `sdivi-detection`, `sdivi-patterns`, `sdivi-snapshot`, `sdivi-config`, `sdivi-core` (for shared types and pure compute calls). All FS- and clock-touching code lives here.
- **Reshaped `sdivi-core`** (same crate name, repurposed): pure-compute facade. Depends only on `sdivi-graph`, `sdivi-detection`, `sdivi-patterns`, `sdivi-snapshot`, `sdivi-config` — each with `default-features = false` to disable the `pipeline-records` feature. No `sdivi-parsing` in the dep tree. No `tree-sitter`, no `walkdir`, no `std::fs::*`, no `std::time::SystemTime`.
- **Public input structs in `sdivi-core::input`** (all `#[derive(Serialize, Deserialize, Tsify)]`-ready):
  - `DependencyGraphInput { nodes: Vec<NodeInput>, edges: Vec<EdgeInput> }`
  - `NodeInput { id: String, path: String, language: String }` — `id` is the **canonical NodeId**: repo-relative path with forward slashes, no leading `./`, no trailing slash. Validated by `validate_node_id` (see below).
  - `EdgeInput { source: String, target: String }` — both reference `NodeInput.id`.
  - `PatternInstanceInput { fingerprint: String, category: String, node_id: String, location: Option<PatternLocationInput> }` — `location` is optional so foreign extractors that don't track positions can skip it. When `None`, `build_catalog_from_inputs` produces a `PatternStats` with an empty `locations: Vec<_>`. CLI/sdivi-pipeline always populates it.
  - `PatternLocationInput { file: String, start_row: u32, start_col: u32 }` — sibling shape to today's `sdivi_patterns::PatternLocation`, but with a string `file` (no `PathBuf`, since `PathBuf` doesn't round-trip cleanly through `tsify`).
  - `LeidenConfigInput { seed: u64, gamma: f64, iterations: usize, quality: QualityFunction }`
  - `BoundarySpecInput` (mirrors `BoundarySpec` shape, no FS path field)
  - `ThresholdsInput` (per-category thresholds + `today: NaiveDate` for expiry checks; see "Override expiry" below)
  - `PriorPartition { cluster_assignments: BTreeMap<String, u32> }` — caller supplies prior-snapshot history for consecutive-snapshot stability scoring (string keys are NodeIds; `detect_boundaries` translates internally to/from numeric indices via `NodeInput.id` order)
  - `NormalizeNode { kind: String, children: Vec<NormalizeNode> }` — input shape for `normalize_and_hash`
- **Public input validators in `sdivi-core::input`:**
  - `validate_node_id(s: &str) -> Result<(), AnalysisError>` — non-empty; no leading `./`; no trailing `/`; uses forward slashes (no backslashes); no `..` or absolute path components. Called from every `compute_*` entry point that takes a `DependencyGraphInput` or `PriorPartition`. Foreign extractors that ship malformed paths get a structured error instead of silently mismatching hashes downstream.
- **Public compute functions in `sdivi-core`** (all pure, all referentially transparent, all callable from WASM):
  - `compute_coupling_topology(graph: &DependencyGraphInput) -> CouplingTopologyResult`
  - `detect_boundaries(graph: &DependencyGraphInput, cfg: &LeidenConfigInput, prior: &[PriorPartition]) -> BoundaryDetectionResult` — return type includes `cluster_assignments: BTreeMap<String, u32>` (NodeId → cluster), `modularity: f64`, `community_count: u32`, `internal_edge_density: BTreeMap<u32, f64>` (per-community, mirrors today's `LeidenPartition.stability` field — kept under that long name to disambiguate from the new metric), `historical_stability: f64` (the consecutive-snapshot stability score against `prior`, formerly called `stability_score` in the milestone draft), `disconnected_components: u32`. **NodeId translation:** the function indexes nodes by `NodeInput.id` order on entry and translates back to string keys on exit, so internal `LeidenPartition` (numeric `usize` keys) never escapes the function. `PriorPartition.cluster_assignments` is similarly translated by the function before consumption.
  - `compute_boundary_violations(graph: &DependencyGraphInput, spec: &BoundarySpecInput) -> BoundaryViolationResult`
  - `compute_pattern_metrics(patterns: &[PatternInstanceInput]) -> PatternMetricsResult` — fields: `entropy_per_category: BTreeMap<String, f64>`, `total_entropy: f64`, `convention_drift: f64` (defined as the count of distinct fingerprints per category divided by the total instance count per category, averaged across categories — single scalar, [0, 1])
  - `compute_thresholds_check(snapshot: &Snapshot, summary: &DivergenceSummary, cfg: &ThresholdsInput) -> ThresholdCheckResult` — pure form of the exit-10 logic; CLI, CI, and the consumer app all call this. **Override expiry is checked here** (against `cfg.today: NaiveDate`), not at config load (see "Override expiry" deliverable below). For null `summary` fields (first-snapshot path), the corresponding threshold is treated as "not exceeded" (no comparison possible). `ThresholdCheckResult { exit_code: i32, exceeded: Vec<String>, summary: DivergenceSummary, applied_overrides: BTreeMap<String, ThresholdOverride> }` — the CLI's `--format json` for `sdivi check` emits this struct directly.
  - `compute_delta(prev: &Snapshot, curr: &Snapshot) -> DivergenceSummary` — re-export from `sdivi-snapshot`
  - `assemble_snapshot(metrics, partition, catalog, pattern_metrics, boundary_spec, timestamp, commit) -> Snapshot` — re-export from `sdivi-snapshot`. **Renamed from today's `build_snapshot`** (M07); both `sdivi-snapshot::build_snapshot` and the `sdivi-core` re-export use the new name. The function gains a `pattern_metrics: PatternMetricsResult` argument so `Snapshot` can carry `convention_drift` (see Snapshot shape change below).
  - `compute_trend(snapshots: &[Snapshot], last_n: Option<usize>) -> TrendResult` — re-export from `sdivi-snapshot` (new — created in this milestone, see Files list). `last_n = None` means "use all"; `Some(n)` clamps to `min(n, snapshots.len())` (over-large `n` is silently honored as "use all available").
  - `infer_boundaries(partitions: &[PriorPartition], stability_threshold: u32) -> BoundaryInferenceResult` — re-export from `sdivi-snapshot` (new — created in this milestone). `partitions` is ordered oldest→newest; the most recent entry is the proposal source, the earlier entries supply the consecutive-snapshot history for stability gating. Caller (CLI or consumer app) is responsible for ordering.
  - `normalize_and_hash(node_kind: &str, children: &[NormalizeNode]) -> String` — canonical fingerprint algorithm. **REPLACES** today's kind-only `sdivi_patterns::fingerprint_node_kind`. Algorithm: depth-first canonical walk. For a node with `kind = K` and ordered children `[c1, c2, …]`, the input to `blake3::keyed_hash(FINGERPRINT_KEY, _)` is the byte concatenation of `K`, the byte `0x00`, then for each child the recursive hash bytes (32 bytes each) prefixed by the byte `0x01`. The `0x00`/`0x01` framing prevents a leaf node `K` from colliding with an internal node `K` whose only child happens to start with `K`. Empty `children` → equivalent to today's `fingerprint_node_kind(K)` byte-for-byte (verified by a property test). Returns the 64-char lowercase hex digest. `sdivi-patterns::fingerprint::fingerprint_node_kind(kind)` becomes a thin wrapper: `normalize_and_hash(kind, &[])`. The CST walker in `sdivi-patterns::catalog::build_catalog` is updated to pass children-derived `NormalizeNode`s when the language adapter populates them; for v0 the adapters still emit kind-only hints, so behavior is unchanged in M07's catalog output (verified by an M07-output-equivalence regression test). The fixed key constant `FINGERPRINT_KEY` in `sdivi-patterns::fingerprint` is unchanged and re-exposed via `sdivi_core::FINGERPRINT_KEY`.
- **Snapshot shape change (pre-1.0, breaking but uncommitted):** `Snapshot` gains a `pattern_metrics: PatternMetricsResult` field carrying `convention_drift: f64` and `entropy_per_category: BTreeMap<String, f64>`. `DivergenceSummary` gains `convention_drift_delta: Option<f64>`. The `convention_drift_rate` threshold (already in `ThresholdsConfig`) now has something to evaluate against. Update `compute_delta` to populate `convention_drift_delta`. `snapshot_version` stays `"1.0"` (we're pre-release; M07 snapshots are regenerable). M07 `build_snapshot` callers updated for the new argument; CHANGELOG entry calls this out.
- **Override expiry — single source of truth (replaces the dual prune-at-load + check-at-compute path).** Today `sdivi-config::thresholds::validate_and_prune_overrides` both validates `expires` format AND removes expired overrides during config load using `SystemTime::now()`. M08 splits these:
  - `validate_overrides_format` (renamed) — validates `expires` is present and well-formed (`YYYY-MM-DD`); errors as `ConfigError::MissingExpiresOnOverride` / `InvalidValue`. **Pure** (no clock). Lives outside the `loader` feature gate.
  - `compute_thresholds_check` — applies expiry against `cfg.today: NaiveDate`. Expired overrides are silently ignored; defaults resume. WASM consumers and CLI use the same logic.
  - `today_iso8601`, `is_expired`, and any remaining `SystemTime::now()` call are deleted from `sdivi-config`. The CLI populates `ThresholdsInput::today` from `chrono::Local::now().date_naive()` before calling `compute_thresholds_check`.
  - Net effect: `sdivi-config` with `--no-default-features` is fully WASM-clean (no `std::fs::*` AND no `std::time::*`).
- **I/O extraction from compute crates:**
  - `sdivi-detection::warm_start.rs` is split: pure `LeidenPartition::{to_json, from_json}` helpers stay on the existing `partition.rs`. The FS calls (`std::fs::read_to_string`, `create_dir_all`, atomic write) move to `sdivi-pipeline::cache`. `warm_start.rs` is deleted; `CACHE_FILENAME` moves with the FS calls. `tempfile` is removed from `sdivi-detection`'s runtime deps.
  - `sdivi-snapshot::store::{write_snapshot, enforce_retention}` → moved to `sdivi-pipeline::store`. `assemble_snapshot`, `compute_delta`, `null_summary` stay pure in `sdivi-snapshot`. New pure modules `sdivi-snapshot::trend` (`compute_trend`) and `sdivi-snapshot::boundary_inference` (`infer_boundaries`) are created in this milestone.
- **Cargo feature gating** on `sdivi-graph`, `sdivi-detection`, `sdivi-patterns`, `sdivi-snapshot`:
  - Default feature `pipeline-records` enables `sdivi-parsing` dep + the `*_from_records` functions taking `&[FeatureRecord]`.
  - When disabled (sdivi-core's WASM build), only the `*_from_input` paths taking `&DependencyGraphInput` / `&[PatternInstanceInput]` compile.
  - `sdivi-core`'s `Cargo.toml` declares each compute crate with `default-features = false`.
- **`sdivi-config` loader feature-gate.** Add a default feature `loader` (default ON) that gates all FS-touching code paths (`Config::load_or_default`, `load_with_paths`, `BoundarySpec::load`, the env-var resolver, every `std::fs::*` call site). With `default-features = false`, only data structs and pure validators (`validate_overrides_format`, `validate_date_format`) compile. `sdivi-core` declares `sdivi-config = { workspace = true, default-features = false }`. `sdivi-pipeline` and `sdivi-cli` keep `loader` on. Verify via `cargo tree -p sdivi-config --target wasm32-unknown-unknown --no-default-features` showing no `std::fs` / `std::time` consumers and via a `wasm32-unknown-unknown` build of `sdivi-config`.
- **Leiden API enhancements (consumer-app requirements 4a–4e):**
  - 4a: `seed: u64` already configurable — verified.
  - 4b: `cluster_assignments: BTreeMap<String, u32>` exposed in `BoundaryDetectionResult`.
  - 4c: `stability_score: f64` computed against `prior: &[PriorPartition]` passed in by caller (no internal history; caller owns the snapshot timeline).
  - 4d: `gamma`, `iterations`, `QualityFunction { Modularity, CPM }` all in `LeidenConfigInput`.
  - 4e: disconnected-graph handling — each connected component becomes its own community-set; explicit fixture test.
- **`sdivi-cli` rewired** to call `sdivi-pipeline::Pipeline` for the orchestration path. CLI `check` calls `sdivi_core::compute_thresholds_check` for the exit-logic. Existing M01–M07 tests pass unchanged.

**Files to create or modify:**

- **New crate:** `crates/sdivi-pipeline/{Cargo.toml, src/lib.rs, src/pipeline.rs, src/cache.rs, src/store.rs, src/error.rs}`
- **New:** `crates/sdivi-core/src/input.rs` (all input structs + `validate_node_id`)
- **New:** `crates/sdivi-core/src/compute/{mod.rs, coupling.rs, boundaries.rs, patterns.rs, thresholds.rs, normalize.rs}`
- **New:** `crates/sdivi-core/src/facade.rs` (re-exports of `compute_delta`, `assemble_snapshot`, `compute_trend`, `infer_boundaries` from `sdivi-snapshot`)
- **New in `sdivi-snapshot`:** `crates/sdivi-snapshot/src/trend.rs` (`compute_trend`, `TrendResult`); `crates/sdivi-snapshot/src/boundary_inference.rs` (`infer_boundaries`, `BoundaryInferenceResult`). Both pure; both added to `crates/sdivi-snapshot/src/lib.rs`'s `pub use` re-export block.
- **Modify:** `crates/sdivi-core/{Cargo.toml, src/lib.rs}` — drop `sdivi-parsing`; declare `sdivi-graph`/`sdivi-detection`/`sdivi-patterns`/`sdivi-snapshot`/`sdivi-config` each with `default-features = false`; add `chrono = { workspace = true, default-features = false, features = ["serde"] }`; `pipeline.rs` deleted; existing M07 re-exports replaced by the M08 surface (`compute_*` functions + the `*Input` family + `FINGERPRINT_KEY` + `assemble_snapshot` etc.).
- **Rename:** `sdivi-snapshot::build_snapshot` → `sdivi-snapshot::assemble_snapshot` (signature gains a `pattern_metrics: PatternMetricsResult` argument). Update every call site in the workspace. CHANGELOG entry notes the rename.
- **Modify Snapshot shape:** add `pattern_metrics: PatternMetricsResult` field to `Snapshot`; `convention_drift_delta: Option<f64>` field to `DivergenceSummary`. Update `compute_delta` to populate the new delta. `snapshot_version` stays `"1.0"` (pre-release).
- **Split + delete:** `crates/sdivi-detection/src/warm_start.rs` — `LeidenPartition::{to_json, from_json}` already on `partition.rs` (verified); the FS calls (`std::fs::read_to_string`, `create_dir_all`, atomic tempfile write) and `CACHE_FILENAME` move to `crates/sdivi-pipeline/src/cache.rs`. `warm_start.rs` deleted. `tempfile` removed from `crates/sdivi-detection/Cargo.toml`'s runtime deps (it's only used by `save_cached_partition`).
- **Move:** `crates/sdivi-snapshot/src/store.rs` + `crates/sdivi-snapshot/src/retention.rs` → `crates/sdivi-pipeline/src/store.rs` (consolidated). `iso_to_filename_safe` (currently in store.rs) moves with it.
- **Modify:** `crates/sdivi-graph/Cargo.toml` — add `[features] pipeline-records = ["dep:sdivi-parsing"]` and `default = ["pipeline-records"]`. Cfg-gate `build_dependency_graph(records: &[FeatureRecord])` behind `#[cfg(feature = "pipeline-records")]`. Add `build_dependency_graph_from_input(input: &DependencyGraphInput) -> DependencyGraph` (uses `validate_node_id` on every `NodeInput.id` and `EdgeInput.{source,target}` before construction).
- **Modify:** `crates/sdivi-detection/Cargo.toml` — same feature pattern. Cfg-gate `LeidenConfig::from_sdivi_config` only if needed (no `sdivi-parsing` types touched today). Remove `tempfile` runtime dep.
- **Modify:** `crates/sdivi-patterns/Cargo.toml` — same feature pattern. Add `crates/sdivi-patterns/src/from_inputs.rs` with `build_catalog_from_inputs(patterns: &[PatternInstanceInput], cfg: &PatternsConfig) -> PatternCatalog` (when `PatternInstanceInput.location` is `None`, the resulting `PatternStats.locations` is an empty `Vec<_>`).
- **Modify:** `crates/sdivi-patterns/src/fingerprint.rs` and **new:** `crates/sdivi-patterns/src/normalize.rs` — implement the new tree-aware `normalize_and_hash(kind, children)` (algorithm spec in the compute-functions section above). `fingerprint_node_kind(kind)` becomes a thin wrapper calling `normalize_and_hash(kind, &[])`. `FINGERPRINT_KEY` is unchanged. The `sdivi-core` re-export `sdivi_core::FINGERPRINT_KEY` and `sdivi_core::normalize_and_hash` are added.
- **Modify:** `crates/sdivi-snapshot/Cargo.toml` — same feature pattern. Add `chrono = { workspace = true, default-features = false }` if any snapshot code needs `NaiveDate` (likely yes via `DivergenceSummary` if we attach trend dates).
- **Modify:** `crates/sdivi-config/Cargo.toml` — add `[features] loader = []` (default ON). Cfg-gate `Config::load_or_default`, `load_with_paths`, `BoundarySpec::load`, env-var resolver, every `std::fs::*` call site, and `validate_and_prune_overrides`'s pruning step behind `#[cfg(feature = "loader")]`. Keep data structs and pure validators (`validate_overrides_format`, `validate_date_format`) outside the gate. **Delete** `today_iso8601` and `is_expired` from `crates/sdivi-config/src/thresholds.rs` — the clock-touching expiry check moves into `sdivi_core::compute_thresholds_check`.
- **Modify:** `crates/sdivi-cli/Cargo.toml` — depend on `sdivi-pipeline` (orchestration) and `sdivi-core` (shared types + thresholds-check function); add `chrono = { workspace = true, features = ["clock"] }` (clock is OK in CLI; only sdivi-core forbids it).
- **Modify:** `crates/sdivi-cli/src/**` — `s/sdivi_core::Pipeline/sdivi_pipeline::Pipeline/`; route `sdivi check` exit logic through `sdivi_core::compute_thresholds_check`. CLI populates `ThresholdsInput::today` from `chrono::Local::now().date_naive()` before the call. CLI keeps anyhow→exit-code mapping; the new exit-10 path calls `compute_thresholds_check` and uses the returned `ThresholdCheckResult.exit_code` directly (so `error_exit_code` doesn't need to change).
- **Modify:** `Cargo.toml` (workspace) — add `crates/sdivi-pipeline` to `members`; add `sdivi-pipeline = { path = "crates/sdivi-pipeline" }` to `[workspace.dependencies]`; add `chrono = { version = "0.4", default-features = false, features = ["serde"] }` to `[workspace.dependencies]` (the `clock` feature stays opt-in per consumer).
- **Add:** `crates/sdivi-core/tests/wasm_compat.rs` — `#[cfg(target_arch = "wasm32")]` smoke test that imports + calls each `compute_*` function. CI builds with `cargo build -p sdivi-core --target wasm32-unknown-unknown --no-default-features`.
- **Add:** `crates/sdivi-core/tests/normalize_and_hash.rs` — includes the M07-equivalence regression test (`normalize_and_hash(kind, &[])` matches `fingerprint_node_kind(kind)` byte-for-byte for the M07 fixture set).
- **Add:** `crates/sdivi-core/tests/validate_node_id.rs` — exercises the canonicalization rule; structured error on `./foo`, `foo/`, backslashes, `..`, absolute paths.
- **Update:** `CHANGELOG.md` — entries under "Unreleased": the `sdivi-core` reshape, consumer-app-driven scope shift, the `build_snapshot` → `assemble_snapshot` rename, the `Snapshot.pattern_metrics` field addition, the `DivergenceSummary.convention_drift_delta` field addition, the `fingerprint_node_kind` → `normalize_and_hash` algorithm extension (M07 fixture catalogs unchanged), and the override-expiry single-source-of-truth move.

**Acceptance criteria:**

- `cargo build -p sdivi-core --target wasm32-unknown-unknown --no-default-features` succeeds. `cargo tree -p sdivi-core --target wasm32-unknown-unknown --no-default-features` shows zero entries for `tree-sitter*`, `walkdir`, `ignore`, `rayon`, `tempfile`, or any crate transitively pulling `std::time::SystemTime`. Same for `cargo tree -p sdivi-config --target wasm32-unknown-unknown --no-default-features` (loader-feature gate exercised, including the deletion of `today_iso8601`).
- `cargo build --workspace` (native targets, default features) succeeds. All M01–M07 tests pass; the only intentional behavior changes are: (a) `Snapshot` carries `pattern_metrics`; (b) `DivergenceSummary` carries `convention_drift_delta`; (c) `build_snapshot` is renamed to `assemble_snapshot`. Tests touching those points are updated in this milestone, not later.
- `compute_*` functions return identical results to today's pipeline when fed equivalent input. Verified by a fixture-based parity test: run `sdivi-pipeline::Pipeline::snapshot` on `tests/fixtures/simple-rust/`; extract a `DependencyGraphInput` from the resulting `Snapshot`; feed through each `sdivi-core::compute_*`; assert per-dimension equality.
- `normalize_and_hash(kind, &[])` produces the **same** `blake3` digest as today's `fingerprint_node_kind(kind)` for the full M07 fixture set (M07-equivalence regression test). The M07 catalog output for `tests/fixtures/simple-rust/` is byte-identical between M07 and M08 (the algorithm extension is a superset; today's adapters emit no children).
- `validate_node_id` rejects `./foo`, `foo/`, `foo\bar`, `../foo`, `/foo`, the empty string. Accepts `src/lib.rs`, `crates/sdivi-core/src/lib.rs`, single-segment names like `Cargo.toml`.
- Override expiry: a `[thresholds.overrides.<cat>]` block with `expires = "2020-01-01"` is **not pruned at config load** (the config retains it as data); `compute_thresholds_check` with `cfg.today = 2026-04-29` ignores it (defaults resume); the same call with `cfg.today = "2019-12-31"` honors it. CLI parity test: `sdivi check` produces the same exit code as a programmatic `compute_thresholds_check` call for the same inputs.
- First-snapshot `compute_thresholds_check` with a null `DivergenceSummary` returns `exit_code = 0` and `exceeded = []` (per Critical System Rule 5). Asserted in `compute_thresholds_check.rs` test.
- New tests pass:
  - `crates/sdivi-core/tests/wasm_compat.rs`
  - `crates/sdivi-core/tests/compute_topology.rs`
  - `crates/sdivi-core/tests/compute_pattern_metrics.rs` (entropy + convention_drift)
  - `crates/sdivi-core/tests/compute_thresholds_check.rs` (exit-10 logic parity with `sdivi check`; first-snapshot null-summary case; expired-override case)
  - `crates/sdivi-core/tests/leiden_disconnected.rs`
  - `crates/sdivi-core/tests/leiden_historical_stability.rs` (renamed from `leiden_stability_score.rs` to match the field name)
  - `crates/sdivi-core/tests/normalize_and_hash.rs` (includes the M07-equivalence regression)
  - `crates/sdivi-core/tests/validate_node_id.rs`
  - `crates/sdivi-snapshot/tests/compute_trend.rs` (last_n clamping; ordering; empty input)
  - `crates/sdivi-snapshot/tests/infer_boundaries.rs` (stability_threshold gating)
  - `crates/sdivi-pipeline/tests/pipeline_smoke.rs` (moved from `crates/sdivi-core/tests/`)
- `cargo clippy --workspace -- -D warnings` clean. `cargo fmt --check` clean. Doc tests pass.

**Tests:**

- Pure-compute parity: round-trip pipeline output through `compute_*` and assert identical `Snapshot` JSON (modulo the new `pattern_metrics` field, which is populated identically by both paths).
- Historical stability: feed an empty `prior: &[]` (or a single-element list — no prior to compare against) → `historical_stability = 0.0`. Feed a synthetic 3-snapshot history with stable clusters → score approaches 1.0. Feed unstable history → score < 0.3.
- Disconnected graph: 3 components of 4 nodes each, no edges between → exactly 3 communities; `disconnected_components = 3`; modularity defined per-component or explicitly reported as N/A across the whole.
- Override expiry: parametric over `cfg.today`, asserts the same `compute_thresholds_check` call returns different `exit_code` / `applied_overrides` based purely on the supplied date — no clock side effect.
- WASM compile: CI job builds `sdivi-core` for `wasm32-unknown-unknown` with `--no-default-features`.
- Feature-flag-flip CI matrix: `cargo test -p <crate> --no-default-features` for each of `sdivi-graph`, `sdivi-detection`, `sdivi-patterns`, `sdivi-snapshot`, `sdivi-config`. Verifies loader-gated and pipeline-records-gated code paths are properly cfg'd.

**Watch For:**

- The `blake3` fixed-key constant becomes part of `sdivi-core`'s public surface (Rule 19). Changing it post-0.1.0 invalidates every snapshot fingerprint. Lock it down here.
- `NodeId` canonicalization rule (`validate_node_id`): repo-relative path; forward slashes; no leading `./`; no trailing `/`; no backslashes; no `..`; not absolute; not empty. Documented on `NodeInput::id` and called from every `compute_*` entry that takes a graph or partition.
- WASM target's lack of `std::time::SystemTime`: every code path calling clock APIs lives in `sdivi-pipeline` or `sdivi-cli`, not `sdivi-core` and not in `sdivi-config` with `loader` off. `today_iso8601`/`is_expired` are deleted; expiry comparison happens in `compute_thresholds_check` against `cfg.today`.
- `chrono`'s default features include `clock` (`SystemTime`) and `std`. `sdivi-core` uses `chrono = { default-features = false, features = ["serde"] }`; `sdivi-cli` adds `features = ["clock"]`.
- Adding `pub` items here is the public API surface. SemVer commitment for `sdivi-core` begins at this milestone. Be deliberate.
- `sdivi-pipeline` is a *public* crate (CLI and Rust embedders use it for the orchestration path), but it's not the WASM target. Document this distinction in its rustdoc.
- Rayon is not WASM-compatible. Today it's only in `sdivi-parsing`; verify it doesn't leak into `sdivi-core` via any compute crate. The feature-gate keeps it confined.
- Convention drift's exact formula (count of distinct fingerprints per category / total instances per category, averaged across categories) is recorded in the rustdoc on `compute_pattern_metrics` and asserted by a fixture test. Pre-1.0 we can refine; post-1.0 changes are breaking.
- `assemble_snapshot` rename: every M07 test, doc-test, and example references `build_snapshot` today. Sweep with `rg "build_snapshot"` and update in one commit so the rename lands atomically.

**Seeds Forward:**

- M09 wires `sdivi-cli` to call `sdivi-pipeline` (largely a search/replace on `sdivi_core::Pipeline` → `sdivi_pipeline::Pipeline`). `sdivi check` uses `sdivi_core::compute_thresholds_check` for exit-logic and populates `cfg.today` from `chrono::Local::now().date_naive()`.
- M10 (boundaries CLI) calls the pure `sdivi_core::infer_boundaries` for the proposal logic; only the YAML write path is in `sdivi-pipeline` / CLI.
- M11 docs cover both the `sdivi-pipeline::Pipeline` orchestration API and the pure-compute `sdivi-core::compute_*` API. The bifl-tracker validation runs both paths and asserts equivalence (including the new `pattern_metrics.convention_drift` field).
- M12 (WASM crate) wraps `sdivi-core` directly with `wasm-bindgen` + `tsify`. The compute functions defined here are the WASM exports. The `BoundaryDetectionResult` field renames (`internal_edge_density`, `historical_stability`) flow straight to the generated `.d.ts`.
- M13 (release) publishes `sdivi-pipeline` to crates.io alongside the existing crates and `@geoffgodwin/sdivi-wasm` to npm.
- The `normalize_and_hash` algorithm (depth-first, `0x00`/`0x01` framing) is the canonical fingerprint the consumer app's TS extractors will mirror. Empty-children equivalence with `fingerprint_node_kind` keeps M07 catalogs intact.

---

---

## Archived: 2026-04-30 — Unknown Initiative

#### Milestone 9: Trend, Check, Show — Remaining CLI Commands
<!-- milestone-meta
id: "9"
status: "done"
-->


**Scope:** The four remaining CLI commands — `trend`, `check`, `show`, plus the `boundaries` parent (subcommands in Milestone 10). Wire the threshold-exceeded exit-10 contract through the pure `sdivi-core::compute_thresholds_check` function delivered in M08. Polish stdout/stderr discipline and JSON output shape.

**Deliverables:**
- `sdivi trend [--last N]` aggregating across stored snapshots — calls `sdivi_core::compute_trend` (pure) and `sdivi_pipeline` for the snapshot-store read. With fewer than 2 snapshots, prints `"sdivi trend: not enough snapshots (need ≥2)"` to stderr and exits 0. When `N > stored_count`, silently uses what's available (no error).
- `sdivi check` — captures a fresh snapshot, compares it to the most recent stored prior, and routes through `sdivi_core::compute_thresholds_check` for the exit decision. Exits `10` if any threshold exceeded, `0` otherwise. **First-run case** (no prior snapshot): `compute_thresholds_check` receives a null `DivergenceSummary` and returns `exit_code = 0` and `exceeded = []` — first-run check is always green, by design (Critical System Rule 5). Today's date is populated into `cfg.today: NaiveDate` by the CLI from `chrono::Local::now().date_naive()` before the call (clock read happens in CLI, not core). **Flag:** `--no-write` skips writing the freshly-captured snapshot to `.sdivi/snapshots/` — useful for CI gates that don't want to pollute history. With `--no-write`, retention is also not enforced. Default behavior writes the snapshot (matching `sdivi snapshot` semantics).
- `sdivi show [<id>] [--format json|text]` inspects a snapshot. `<id>` is optional; with no argument, the **latest snapshot** by lexicographic filename order (= chronological, per the M07 file-naming scheme `snapshot_<YYYYMMDDTHHMMSS>_<hash>.json`) is shown. With `--format json`, output is the raw `Snapshot` JSON on stdout (so `sdivi show --format json | jq '.snapshot_version'` returns `"1.0"` per the existing acceptance test).
- `sdivi boundaries` parent command with subcommand stubs (`infer`, `ratify`, `show`). Each stub prints `"sdivi boundaries <subcmd>: not implemented until M10"` to stderr and exits **0** — keeps CI scripts that survey command help working. M10 fills them in.
- Text formatter using `ratatui` for tables and `owo-colors`/`anstream` for color (auto-detected via `Config::output.color` and `NO_COLOR`)
- JSON formatter producing schema-stable output:
  - `sdivi show --format json` → raw `Snapshot` JSON
  - `sdivi check --format json` → `ThresholdCheckResult` from `sdivi-core` (`{ exit_code, exceeded: [string], summary: DivergenceSummary, applied_overrides: { ... } }`); the CLI process exit code still tracks `result.exit_code`, JSON is informational
  - `sdivi trend --format json` → `TrendResult` from `sdivi-snapshot`
  - `sdivi diff --format json` → `DivergenceSummary` (already in M07's shape)

**Files to create or modify:**
- **New:** `crates/sdivi-cli/src/commands/{trend.rs,check.rs,show.rs,boundaries.rs}`
- **Modify:** `crates/sdivi-cli/src/main.rs` — register the four new subcommands in the `Commands` enum and dispatch in `main`. `Check` carries `--no-write: bool` and `--format: String`. `Trend` carries `--last: Option<usize>` and `--format: String`. `Show` carries `<id>: Option<String>` and `--format: String`. `Boundaries` carries a nested subcommand enum with `Infer`, `Ratify`, `Show` variants (stubs for M09).
- **Modify:** `crates/sdivi-cli/src/output/{json.rs,text.rs}` — extend with formatters for `TrendResult`, `ThresholdCheckResult`, and the standalone `Snapshot` show path. JSON path is a thin `serde_json::to_string_pretty` per result struct; text path uses `ratatui` rendering to a `Vec<u8>` buffer (no alternate-screen / no raw-mode TUI).
- **No-op for `crates/sdivi-cli/src/logging.rs`** — already exists from M01 (tracing-subscriber → stderr at `warn` default). Touch only if `--verbose` flag is added to `sdivi check`; not needed for the M09 acceptance criteria.
- **Extend:** `crates/sdivi-pipeline/src/store.rs` — add `read_snapshots(dir: &Path) -> std::io::Result<Vec<Snapshot>>` returning chronologically-ordered (oldest→newest) snapshots and `read_snapshot_by_id(dir: &Path, id: &str) -> std::io::Result<Snapshot>` for `sdivi show <id>`. Add `latest_snapshot(dir: &Path) -> std::io::Result<Option<Snapshot>>` for `sdivi check` and `sdivi show` with no id.
- **Extend:** `crates/sdivi-pipeline/src/pipeline.rs` — `Pipeline::snapshot` gains an internal `WriteMode::{Persist, EphemeralForCheck}` enum so `sdivi check --no-write` can capture without touching `.sdivi/snapshots/`. Default callers stay on `Persist`.

**Acceptance criteria:**
- `sdivi check` exits `0` on a fresh snapshot below thresholds; `10` when any threshold is exceeded. Exit logic is a thin wrapper around `compute_thresholds_check`'s `ThresholdCheckResult.exit_code`.
- **First-run `sdivi check` exits `0`** (no prior snapshot → null `DivergenceSummary` → `compute_thresholds_check` returns `exit_code = 0` with `exceeded = []`). Asserted in `crates/sdivi-cli/tests/exit_codes.rs`.
- `sdivi check --no-write` does not create a file in `.sdivi/snapshots/`. Asserted by counting the snapshot directory before/after.
- An expired threshold override is silently ignored — `sdivi check` uses defaults after expiry. Expiry comparison happens inside `compute_thresholds_check` against the `today` argument supplied by the CLI; the config retains the override block as data (no load-time pruning).
- `sdivi show` with no `<id>` prints the latest snapshot (lexicographically last `snapshot_*.json` in `.sdivi/snapshots/`).
- `sdivi show --format json | jq '.snapshot_version'` returns `"1.0"` (no stderr contamination on stdout).
- `sdivi check --format json | jq '.exit_code'` returns the integer exit code (and the process itself exits with the same value).
- `NO_COLOR=1 sdivi show` produces no ANSI escape codes. `--no-color` is equivalent.
- `sdivi trend --last 5` aggregates across the 5 most recent snapshots. `--last 9999` against a directory with 3 snapshots silently uses 3 (no error).
- `sdivi trend` with 0 or 1 snapshot prints the friendly "not enough snapshots" message to stderr and exits `0`.
- `sdivi boundaries infer|ratify|show` each prints "not implemented until M10" to stderr and exits `0`.
- Logs from `tracing` go to stderr regardless of format.

**Tests:**
- `crates/sdivi-cli/tests/exit_codes.rs`: full matrix of exit codes 0/1/2/3/10. Includes first-run `sdivi check` (exit 0), expired-override `sdivi check` (exit 0), threshold-exceeded `sdivi check` (exit 10), bad-config `sdivi check` (exit 2).
- `crates/sdivi-cli/tests/stdout_stderr_split.rs`: redirect each stream to a file; assert JSON validity on stdout for every `--format json` command and zero JSON contamination on stderr.
- `crates/sdivi-cli/tests/check_thresholds.rs`: synthetic snapshots driving every threshold variant (pattern_entropy, convention_drift, coupling_delta, boundary_violation); verify CLI exit code matches `compute_thresholds_check` programmatically. Includes a `--no-write` assertion (snapshot dir count unchanged).
- `crates/sdivi-cli/tests/show_format.rs`: `sdivi show --format json` parses as `Snapshot`; `sdivi show` with no id selects latest; `sdivi show <id>` selects specifically.
- `crates/sdivi-cli/tests/trend_format.rs`: `sdivi trend --last N` clamps to available; <2 snapshots → friendly stderr message + exit 0; `--format json` parses as `TrendResult`.
- `crates/sdivi-cli/tests/boundaries_stub.rs`: each of `sdivi boundaries {infer, ratify, show}` exits 0 and writes to stderr only.
- `crates/sdivi-cli/tests/no_color.rs`: `NO_COLOR=1` and `--no-color` both suppress color across show / check / trend.

**Watch For:**
- `sdivi check` is the **only** command that may exit `10` — any other command emitting `10` is a bug.
- The text formatter must not block JSON consumers: use `tokio`-free synchronous `ratatui` rendering directly to a `Vec<u8>` buffer, then write to stdout — do **not** initialize a TUI mode (no alternate screen, no raw mode).
- `compute_thresholds_check` is the source of truth for exit logic. Any new threshold/override semantics land in `sdivi-core`, not in CLI flags. CLI is presentation only.
- Threshold check consults overrides per-category and checks expiry against `cfg.today`. CLI pulls the date from `chrono::Local::now().date_naive()` and writes it into `ThresholdsInput::today` before calling `compute_thresholds_check`. The pure function never reads the clock.
- `sdivi check` writes a snapshot by default; `--no-write` skips the write **and** the retention enforcement. CI gates that don't want history pollution use `--no-write`.
- `sdivi show` with no id picks lexicographic-last from `.sdivi/snapshots/snapshot_*.json` — relies on the M07 file-naming scheme. If the scheme ever changes, this default-selection logic must change with it.
- `sdivi trend` on fewer than 2 snapshots: friendly stderr message + exit 0. `--last N` larger than the stored count is silently clamped (no error).
- The `sdivi boundaries` subcommands are M09 stubs that exit 0 — do **not** wire to `sdivi_core::infer_boundaries` here; that's M10. Premature wiring couples this milestone to M10's YAML-write path and breaks the parallel-development plan.

**Seeds Forward:**
- The `sdivi check` exit-10 path is the public CI gate contract. Any future threshold (e.g., per-category) must continue exiting 10 and route through `compute_thresholds_check`.
- The text formatter shape is reused by `sdivi boundaries show` in M10.
- JSON output shape is the contract embedders rely on; future milestones cannot break it without a snapshot-version bump. The four JSON shapes (`Snapshot`, `ThresholdCheckResult`, `TrendResult`, `DivergenceSummary`) are documented in `docs/cli-integration.md` (M11).
- The consumer app invokes `compute_thresholds_check` directly via WASM (M12); the CLI is one of three callers (CLI / Rust embedders / consumer app via WASM) — keep the function's input shape stable.
- `WriteMode::EphemeralForCheck` introduced for `--no-write` is the seam any future "dry-run snapshot" feature reuses. Don't add a second seam.

---

---

## Archived: 2026-04-30 — Unknown Initiative

#### Milestone 10: Boundaries — Infer, Ratify, Show
<!-- milestone-meta
id: "10"
status: "done"
-->


**Scope:** Implement the boundary lifecycle: infer modules from a `LeidenPartition` (using the pure `sdivi_core::infer_boundaries` function delivered in M08), ratify them into `.sdivi/boundaries.yaml`, and inspect via `show`. Comment loss on programmatic write is accepted per KDD-6.

**Deliverables:**
- `sdivi boundaries infer` proposes module groupings from the most recent partition, using `sdivi_core::infer_boundaries` for the proposal logic
- `sdivi boundaries ratify` writes (or merges) accepted groupings into `.sdivi/boundaries.yaml`
- `sdivi boundaries show` prints the current spec
- YAML write path via `serde_yaml` with documented comment-loss behavior — lives in `sdivi-pipeline::store` (I/O), not in `sdivi-config` (pure)
- Stability tracking: `Config::boundaries.stability_threshold` (default 3) gates which clusters are mature enough to propose. Stability is computed inside `sdivi_core::infer_boundaries` against the partition history (oldest→newest, latest is the proposal source) that the caller assembles. CLI loads it from `.sdivi/cache/`; the consumer app supplies its own list.

**Files to create or modify:**
- `crates/sdivi-pipeline/src/boundaries.rs` — read the last N stored `Snapshot`s from `.sdivi/snapshots/` (each carries its `LeidenPartition`), assemble a `Vec<PriorPartition>` ordered oldest→newest, call `sdivi_core::infer_boundaries`, atomic-write YAML to `.sdivi/boundaries.yaml`. `N = stability_threshold + 1` is sufficient (need the proposal source plus enough history to gate it). No separate partition-history cache is introduced — the snapshot store already retains partitions per `Config::snapshots.retention`.
- `crates/sdivi-config/src/boundary.rs` — add pure YAML serialization helper `BoundarySpec::to_yaml(&self) -> String` (no FS). Lives outside the M08 `loader` feature gate so it's available in WASM. The FS-touching `BoundarySpec::write(&self, path)` lives in `sdivi-pipeline::store` (gated by `loader` if any sdivi-config helper is needed there).
- `crates/sdivi-cli/src/commands/boundaries.rs` — full subcommand impl (parent stub from M09)
- `docs/migrating-from-sdi-py.md` — **create** this file with at minimum the YAML comment-loss section (full migration guide is finished in Milestone 11)

**Acceptance criteria:**
- `sdivi boundaries infer` on a multi-snapshot history proposes groupings only for clusters present in `stability_threshold` consecutive snapshots
- `sdivi boundaries ratify` writes a valid YAML file; reading it back produces an equivalent `BoundarySpec`
- A user-written `boundaries.yaml` with comments loses comments on the next ratify; behavior documented in `docs/migrating-from-sdi-py.md` (file is created here as a stub with the comment-loss section; Milestone 11 fills out the rest of the migration guide)
- `sdivi boundaries show` prints the spec in either YAML or JSON format
- `sdivi_core::infer_boundaries` is callable directly by the consumer app (via WASM in M12) with a caller-supplied prior-partition history

**Tests:**
- `tests/boundary_lifecycle.rs`: build evolving fixture, run `infer`/`ratify`/`show` end-to-end
- `crates/sdivi-config/tests/boundary_roundtrip.rs`: write then read; equivalent spec
- `crates/sdivi-cli/tests/boundaries_show.rs`: format flags work
- `crates/sdivi-core/tests/infer_boundaries.rs`: pure-compute infer test (no FS); same proposal as the CLI for an equivalent prior-history

**Watch For:**
- The `sdi-py` `boundaries.yaml` schema is read-compatible — DO NOT introduce sdivi-rust-only fields here without an explicit `tekhton` design discussion
- Comment loss surprises users; `sdivi boundaries ratify` should print a stderr warning the first time it overwrites a file with comments
- Atomic write applies here too — same tempfile + rename pattern as snapshots, in `sdivi-pipeline::store`
- Inference must respect the `stability_threshold` over historical snapshots, not just propose every cluster from the latest one. The history is supplied to `sdivi_core::infer_boundaries` as `&[PriorPartition]` — the caller (CLI or the consumer app) loads it.

**Seeds Forward:**
- `BoundarySpec` write path established here is the only programmatic write point. Future write-back features (e.g., editor plugin, post-1.0) reuse it.
- The decision on comment preservation can be revisited post-MVP without breaking the schema.
- The consumer app gets boundary inference + violation detection via `sdivi_core::{infer_boundaries, compute_boundary_violations}` — same surface, different consumer.

---

---

## Archived: 2026-05-01 — Unknown Initiative

#### Milestone 11: Documentation, Examples, Determinism Polish, bifl-tracker Validation
<!-- milestone-meta
id: "11"
status: "done"
-->


**Scope:** Stand up the documentation surfaces (`README.md`, `docs/*.md`, rustdoc with `#![deny(missing_docs)]` on `sdivi-core`). Doc tests in CI. Runnable examples covering both the orchestration path (`sdivi-pipeline`) and the pure-compute path (`sdivi-core`). Tighten determinism with `proptest` regression suite and FMA documentation. **Run sdivi-rust end-to-end against bifl-tracker as the v0 validation gate** — this is the user-facing acceptance for "the rewrite produces the same answers." Failures here block 0.1.0 release.

**Deliverables:**
- `README.md` quick start, install paths (cargo, brew, binary, npm for the WASM bundle), one-paragraph SDIVI overview, links — under 200 lines
- `docs/cli-integration.md` with `cargo install sdivi && sdivi check` GHA snippet and exit-code reference
- `docs/library-embedding.md` with **two** sections: Rust embedders using `sdivi-pipeline::Pipeline` for FS-based runs, and pure-compute embedders (Rust + JS via WASM) using `sdivi-core::compute_*` functions. Includes a consumer-app-style example: caller has its own AST extractors, computes `normalize_and_hash` per node, ships hashes + graph to `sdivi-core`.
- `docs/migrating-from-sdi-py.md` with full carry/change matrix (file stub from M10)
- `docs/determinism.md` with `BTreeMap` discipline, seed contract, FMA notes, and a section on "feeding sdivi-core canonical input from a foreign extractor" (the `NodeId` canonicalization rule, `normalize_and_hash` invariant)
- `examples/embed_pipeline.rs` — Rust orchestration via `sdivi-pipeline`
- `examples/embed_compute.rs` — Rust pure-compute via `sdivi-core` (mirrors what the consumer app does, minus the WASM boundary)
- `examples/custom_config.rs` — programmatic Config building
- `#![deny(missing_docs)]` enabled on `sdivi-core` with docs for every public item; rustdoc on `sdivi-pipeline` highly recommended but not enforced
- `cargo test --doc` runs in CI; no broken doc tests
- `proptest` regression directory checked in; `prop_test_pipeline_deterministic`, `prop_test_delta_pure`, `prop_test_leiden_seeded`, `prop_test_normalize_and_hash_stable`, `prop_test_compute_thresholds_check_pure` all permanent
- **bifl-tracker validation harness** at `tools/validate-against-bifl-tracker.sh` — uses local checkout of `~/workspace/geoffgodwin/bifl-tracker`, runs `sdivi snapshot` at a fixed set of commits across its history (the same commits sdi-py validated against), and compares snapshot output to sdi-py's stored snapshots. Acceptable variance per KD11: modularity within 1%, community count within ±10%, pattern entropy within 5%. The compared sdi-py snapshots are pinned in `tests/fixtures/bifl-tracker-baselines/`.
- **Pure-compute parity check** in the same harness: for each fixture commit, run `sdivi-pipeline::Pipeline::snapshot` (Snapshot A); separately construct `DependencyGraphInput` + `Vec<PatternInstanceInput>` from the same parsed `FeatureRecord`s and call `sdivi-core::compute_*` (results B); assert A's per-dimension metrics match B's within FMA tolerance. This validates that the consumer app's WASM-mediated path (M12) produces the same answers as the native CLI given equivalent input.

**Files to create or modify:**
- `README.md`, `docs/{cli-integration,library-embedding,migrating-from-sdi-py,determinism}.md` — `migrating-from-sdi-py.md` already exists from Milestone 10 with the comment-loss section; expand with the full carry/change matrix
- `examples/{embed_pipeline,embed_compute,custom_config}.rs`
- Doc comments throughout `crates/sdivi-core/src/**/*.rs`
- `crates/sdivi-core/src/lib.rs` enables `#![deny(missing_docs)]`
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
- Doc tests on every public function in `sdivi-core` with an `# Examples` block
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
- The doc structure is the canonical reference for embedders. M12 (WASM) cross-links `docs/library-embedding.md` from the `sdivi-wasm` README.
- `proptest` regressions stay in CI permanently — a regression file commit is mandatory after a flaky test surfaces a real shrinkage.
- The bifl-tracker baselines are the v0 acceptance gate; updating them after 0.1.0 is a deliberate decision (any tolerance breach during 0.x patch work blocks the patch).

---

---

## Archived: 2026-05-01 — Unknown Initiative

#### Milestone 12: WASM Crate, npm Package, Consumer App Integration
<!-- milestone-meta
id: "12"
status: "done"
-->


**Scope:** Build the `sdivi-wasm` crate that wraps `sdivi-core` with `wasm-bindgen` + `tsify`-derived `.d.ts`. Produce npm package `@geoffgodwin/sdivi-wasm` matching the shape the consumer app needs (sql.js-style async `init()`, then synchronous calls). Validate end-to-end against the consumer app as the first concrete consumer. This milestone closes the loop on KDD-13 (WASM in v0).

**Deliverables:**
- `bindings/sdivi-wasm/{Cargo.toml, src/lib.rs, src/exports.rs}` — `crate-type = ["cdylib"]`. Depends on `sdivi-core` only (and `wasm-bindgen`, `tsify`, `serde-wasm-bindgen`, `js-sys` as needed). No `sdivi-pipeline`, no `sdivi-parsing`, no FS dep.
- `wasm-bindgen` exports for each pure-compute function delivered in M08:
  - `compute_coupling_topology(graph: DependencyGraphInput) -> CouplingTopologyResult`
  - `detect_boundaries(graph: DependencyGraphInput, cfg: LeidenConfigInput, prior: Vec<PriorPartition>) -> BoundaryDetectionResult`
  - `compute_boundary_violations(graph: DependencyGraphInput, spec: BoundarySpecInput) -> BoundaryViolationResult`
  - `compute_pattern_metrics(patterns: Vec<PatternInstanceInput>) -> PatternMetricsResult`
  - `compute_thresholds_check(snapshot: Snapshot, summary: DivergenceSummary, cfg: ThresholdsInput) -> ThresholdCheckResult`
  - `compute_delta(prev: Snapshot, curr: Snapshot) -> DivergenceSummary`
  - `assemble_snapshot(...) -> Snapshot`
  - `compute_trend(snapshots: Vec<Snapshot>, last_n: Option<u32>) -> TrendResult`
  - `infer_boundaries(prior_partitions: Vec<PriorPartition>, stability_threshold: u32) -> BoundaryInferenceResult`
  - `normalize_and_hash(node_kind: String, children: Vec<NormalizeNode>) -> String`
- All input/output types derive `tsify::Tsify` so `.d.ts` is generated automatically — the consumer app gets accurate types without manual sync. Strict-TS-compatible: every optional field is explicitly `T | undefined`, no implicit `any`.
- `package.json` shape (consumer-app compatible):
  ```json
  {
    "name": "@geoffgodwin/sdivi-wasm",
    "version": "0.1.0",
    "main": "sdivi-wasm.js",
    "types": "sdivi_wasm.d.ts",
    "files": ["sdivi_wasm_bg.wasm", "sdivi_wasm.js", "sdivi_wasm.d.ts"],
    "license": "Apache-2.0",
    "repository": "..."
  }
  ```
- Build pipeline: `wasm-pack build --target bundler --release` (the consumer app uses webpack/vite-style bundlers; switch to `--target web` only if the consumer explicitly needs raw `.wasm` URL loading).
- `wasm-opt -Os` post-processing to keep bundle size down. Target: under 1 MB compressed `.wasm`.
- Async `init()` pattern matching the sql.js / wasm-bindgen idiom: caller `await init()`, then synchronous calls thereafter.
- `examples/binding_node.ts` — consumer-app-shaped usage:
  ```ts
  import init, { detect_boundaries, normalize_and_hash } from '@geoffgodwin/sdivi-wasm';
  await init();
  const hash = normalize_and_hash('try_expression', [...]);
  const result = detect_boundaries(graph, cfg, []);
  ```
- `bindings/sdivi-wasm/README.md` covering install, init pattern, every export, and the strict-TS guarantees.
- `bindings/sdivi-wasm/build.sh` — single script for the local-dev WASM build.
- `.github/workflows/wasm.yml` — builds the WASM bundle on push to any branch, dry-runs `npm publish` on tagged releases. Asserts bundle size budget (fails CI above 1.2 MB).

**Files to create or modify:**
- New: `bindings/sdivi-wasm/{Cargo.toml, src/lib.rs, src/exports.rs, package.json, README.md, build.sh}`
- New: `bindings/sdivi-wasm/tests/wasm_smoke.rs` (using `wasm-bindgen-test` against headless Node)
- New: `examples/binding_node.ts`
- New: `.github/workflows/wasm.yml`
- Modify: workspace `Cargo.toml` — add `bindings/sdivi-wasm` to `members`. Pin `wasm-bindgen`, `tsify`, `serde-wasm-bindgen` versions in `[workspace.dependencies]`.

**Acceptance criteria:**
- `wasm-pack build --target bundler --release` produces a working bundle. Output `.wasm` < 1.2 MB.
- `npm pack` produces a tarball the consumer app can `npm install` from a file path; `import init, { ... } from '@geoffgodwin/sdivi-wasm'` resolves all named exports.
- All `compute_*` functions callable from TS with correct types. The generated `.d.ts` passes `tsc --strict --noUncheckedIndexedAccess --exactOptionalPropertyTypes` against a sample consumer.
- Smoke test: feed a known fixture's nodes/edges/patterns through both `sdivi-wasm` (in Node) and native `sdivi-core` (in Rust); assert per-dimension equality within the FMA tolerance documented in `docs/determinism.md` (Open Q #10). `normalize_and_hash` outputs must be **bit-identical** across the two — they're hash-only, no float math.
- `normalize_and_hash` produces identical `blake3` digests in WASM vs native sdivi-core for the same input. The consumer app's TS extractors can call this and trust the hash matches what Rust would produce.
- `@geoffgodwin/sdivi-wasm@0.1.0-rc.0` dry-run publish via `npm publish --dry-run` succeeds.
- A real consumer-app integration smoke: invoke from a consumer-app dev branch, run a full divergence-summary cycle on a sample repo, confirm reasonable output. (This is a manual gate — checked off by the M12 author after coordinating with the consumer-app repo.)

**Tests:**
- `bindings/sdivi-wasm/tests/wasm_smoke.rs` — `wasm-bindgen-test` runs each export and asserts non-trivial output
- Cross-platform hash determinism: a Linux CI job builds + runs the WASM smoke; a macOS CI job does the same; both must produce the same `normalize_and_hash` output for a fixture input.
- Bundle-size regression: CI job compares `.wasm` size against a checked-in budget file; fails if over.

**Watch For:**
- `wasm-bindgen` doesn't auto-serialize `BTreeMap<String, T>` — must use `tsify`'s serde adapter or convert to `Vec<(String, T)>` at the boundary. Pick one approach and document.
- `tsify` is still pre-1.0; pin a specific version in workspace deps and add a watch entry to `DRIFT_LOG.md` if a breaking version bump appears.
- The consumer app uses strict TS with `noUncheckedIndexedAccess` and `exactOptionalPropertyTypes`. Verify the generated `.d.ts` passes these flags before claiming compatibility.
- WASM has no FS — by construction since `sdivi-core` has none after M08. But `cargo tree -p sdivi-wasm --target wasm32-unknown-unknown` is a CI assertion: zero entries for `tree-sitter*`, `walkdir`, `ignore`, `rayon`, `tempfile`, `std::time::SystemTime`-touching crates.
- The `blake3` fingerprint must produce identical bytes between native sdivi-core and sdivi-wasm. If `blake3`'s SIMD intrinsics are platform-specific, force the portable backend or assert FMA-tolerance equality only.
- npm scope `@geoffgodwin/` requires an org or user-scope on npmjs.com. Verify the scope is registered and `geoff.godwin@gmail.com` has publish rights before M13.
- Bundle size: `wasm-opt -Os` is mandatory. `lto = "fat"` for the WASM build profile. Strip debug symbols.
- WASM error handling: any `panic!` in sdivi-core becomes an unhelpful `unreachable executed` in JS. Use `console_error_panic_hook` so panics produce stack traces in dev builds.

**Seeds Forward:**
- M13 publishes `@geoffgodwin/sdivi-wasm` on the same tag-driven workflow as crates.io, behind the same manual approval gate.
- The consumer-app integration is the canonical embedder usage pattern; future bindings (PyO3, napi-rs — currently post-MVP / v1 era) follow the same wrap-`sdivi-core` shape.
- If WASM integration surfaces missing pure-compute capabilities (e.g., the consumer app needs a function not exposed in M08), they're added to `sdivi-core` with the same input-struct + `compute_*` pattern. The pattern, not the specific function list, is the load-bearing API decision.

---

---

## Archived: 2026-05-01 — Unknown Initiative

#### Milestone 13: Release Pipeline and Distribution
<!-- milestone-meta
id: "13"
status: "done"
-->


**Scope:** Ship the v0 release. Tag-driven workflow publishes affected crates to crates.io and the WASM bundle to npm, both behind a manual approval gate. Matrix-built binaries attached to GitHub Releases. `cargo audit` weekly. Cut `0.1.0`.

**Deliverables:**
- `.github/workflows/release.yml` with manual approval gate before crates.io and npm pushes
- `cargo dist` or hand-rolled matrix build (Linux x86_64+aarch64, macOS x86_64+aarch64, Windows x86_64) attaching stripped binaries with LTO to GitHub Release
- `CHANGELOG.md` with `0.1.0` entry covering all milestones
- `cargo audit` weekly cron in `.github/workflows/audit.yml`
- Binary size and `.wasm` bundle size tracked in `CHANGELOG.md` per release
- crates.io publish (in dependency order):
  1. `sdivi-config`
  2. six `sdivi-lang-*`
  3. `sdivi-parsing`
  4. `sdivi-graph`
  5. `sdivi-detection`
  6. `sdivi-patterns`
  7. `sdivi-snapshot`
  8. `sdivi-core` (depends on 4–7 with `default-features = false`)
  9. `sdivi-pipeline` (new in M08; depends on 3–8)
  10. `sdivi-cli` (depends on 9 + `sdivi-core` for shared types)
  11. `sdivi-rust` (install-discovery meta-crate)
- npm publish: `@geoffgodwin/sdivi-wasm@0.1.0` on the same tag, behind the same manual approval
- PyO3/napi-rs bindings remain post-MVP / v1 era (see deferred `m12-bindings-pyo3-and-napi-rs-post-mvp.md`)

**Files to create or modify:**
- `.github/workflows/release.yml` (full)
- `CHANGELOG.md` (0.1.0 entry)
- `Cargo.toml` (workspace version bump to `0.1.0`)
- Each crate's `Cargo.toml` populated with `description`, `repository`, `license = "Apache-2.0"`, `readme`, `keywords`, `categories`
- `bindings/sdivi-wasm/package.json` version pinned to `0.1.0`

**Acceptance criteria:**
- Tagging `v0.1.0` triggers the release workflow; crates.io and npm pushes wait on manual approval
- After approval, `cargo install sdivi-rust` from crates.io succeeds and produces a working `sdivi` binary on PATH (binary name comes from `[[bin]] name = "sdivi"` in `sdivi-cli`)
- After approval, `npm install @geoffgodwin/sdivi-wasm` works and the consumer app can `import init, { ... }` successfully against a non-local registry
- GitHub Release page has binaries for all five Tier-1 + Tier-2 platforms
- Binary size and `.wasm` bundle size noted in CHANGELOG
- `cargo audit` cron green
- bifl-tracker validation harness from M11 passes against the tagged commit

**Tests:**
- Dry-run the release workflow on a `v0.1.0-rc.N` pre-tag
- Smoke test: `cargo install --version 0.1.0 sdivi-rust` on each platform; `sdivi --version` reports `0.1.0`
- Smoke test: `npm install @geoffgodwin/sdivi-wasm@0.1.0-rc.N` from a clean Node project; `await init()` and call one export
- `cargo audit` clean

**Watch For:**
- crates.io is append-only — no yanking-as-rollback; once `0.1.0` is published it stays. Validate carefully via the dry run.
- npm is also effectively append-only at the version level (unpublishing is restricted after 72h). Same care applies.
- Manual approval gate must be enforced for both registries — auto-publish on tag is explicitly rejected by DESIGN
- LICENSE in each crate's metadata must say `Apache-2.0`, matching the repo LICENSE; npm `license` field also `Apache-2.0`
- Strip + LTO bloat fix: `[profile.release] lto = "thin"`, `strip = true`, `panic = "abort"` (the last only if no test code unwinds)
- WASM profile is separate: `[profile.release-wasm] inherits = "release", lto = "fat", opt-level = "s"`
- npm scope `@geoffgodwin/` must be claimed and the publish token configured in GitHub Actions secrets before this milestone runs
- Publish ordering: `sdivi-core` before `sdivi-pipeline` (the new dep order); `sdivi-wasm` last (depends only on `sdivi-core` but published to npm, not crates.io, so it doesn't block other crates)

**Seeds Forward:**
- `0.1.0` is the SemVer commitment baseline. Adding `pub` items to `sdivi-core` is now deliberate; removing them requires a major bump to `1.0.0`.
- The release workflow is reused for every subsequent tag, including npm-only patches if WASM ever needs a fast-track fix
- Distribution channels (crates.io + GitHub Releases + npm) are the public commitments. Adding PyPI in a v1 era must not regress these.
- The consumer app becomes a real-world post-release validation source — track its issue intake against `sdivi-wasm` as the first signal of API churn pressure.

---

---

## Archived: 2026-05-01 — Unknown Initiative

#### Milestone 14: Per-Category Threshold Override Wiring
<!-- milestone-meta
id: "14"
status: "done"
-->


**Scope:** Make `ThresholdsInput::overrides` and `ThresholdsInput::today` actually load-bearing. Surface per-category breakouts in `PatternMetricsResult` and `DivergenceSummary` so `compute_thresholds_check` has something to filter against. Wire active (non-expired) overrides as the per-category limit. Retire the orphaned `TODO(M09)` markers and the `M08`/`M09`-named stub tests. Schema stays `"1.0"` — every change is additive.

**Why this milestone exists:** `ThresholdsInput::overrides` and `ThresholdsInput::today` are committed `pub` API as of M08. Per CLAUDE.md Rule 18 these become permanent SemVer commitments at the v0.1.0 tag — and right now they are silent no-ops (`crates/sdivi-core/src/compute/thresholds.rs:92-94`). CLAUDE.md Rule 12 documents per-category overrides as a working feature; init writes `[thresholds.overrides.<cat>]` examples into every new repo's config. Shipping v0 with a documented-but-vacuous knob locks the vacuous behavior. The remaining open `HUMAN_ACTION_REQUIRED` item explicitly demands a decision before tag time; this milestone is that decision.

**Deliverables:**

- **Per-category fields on `PatternMetricsResult`** (additive):
  - `convention_drift_per_category: BTreeMap<String, f64>` — same formula as the existing scalar `convention_drift` (distinct fingerprints divided by total instance count) but kept per-category instead of averaged. The existing `convention_drift` scalar stays as the average for backward-compat consumers; the new field is the source of truth for per-category override filtering.
  - `entropy_per_category` already exists — no change.
  - Computed in `sdivi_core::compute::patterns::compute_pattern_metrics` and in `sdivi-pipeline::pipeline::compute_pattern_metrics_from_catalog` (both call sites must produce the same map for the same input).
- **Per-category delta fields on `DivergenceSummary`** (additive):
  - `pattern_entropy_per_category_delta: Option<BTreeMap<String, f64>>` — `None` on the first-snapshot path (KDD-9 semantics preserved); `Some(map)` otherwise. Map keys are the union of categories present in either `prev` or `curr`; missing-side values are treated as `0.0` so a newly-introduced category surfaces as a positive delta.
  - `convention_drift_per_category_delta: Option<BTreeMap<String, f64>>` — same shape, same null semantics.
  - `null_summary()` updated to set both new fields to `None`.
  - `compute_delta` updated to populate both from `prev.pattern_metrics.{entropy_per_category, convention_drift_per_category}` vs. the same on `curr`.
- **Override wiring in `compute_thresholds_check`:**
  - Build a per-category effective rate map: for each `(category, override)` in `cfg.overrides`, parse `override.expires` as `NaiveDate` (delegated to the existing `validate_date_format` from `sdivi-config`'s pure validators); skip if `cfg.today > expires` (silent ignore, per Rule 12); otherwise the override's `pattern_entropy_rate` / `convention_drift_rate` / `coupling_delta_rate` / `boundary_violation_rate` (each `Option<f64>`) replaces the global rate **only for that category**.
  - Aggregate dimensions (`summary.pattern_entropy_delta`, `summary.convention_drift_delta`) continue to use the global rate. Per-category breaches use the per-category effective rate.
  - The existing aggregate breach evaluation stays exactly as-is. The new evaluation iterates over the per-category delta maps when present and emits one `ThresholdBreachInfo` per breaching category.
- **`ThresholdBreachInfo` gains `category: Option<String>`** (additive). `None` for the existing aggregate breaches; `Some("error_handling")` for per-category breaches. Existing `breaches[0].dimension == "pattern_entropy"` shape is unchanged.
- **`ThresholdCheckResult` gains `applied_overrides: BTreeMap<String, AppliedOverrideInfo>`** (additive) — diagnostic surface for `sdivi check --format json` consumers and for the consumer app: which overrides were considered, which were active, which were expired. `AppliedOverrideInfo { active: bool, expires: NaiveDate, expired_reason: Option<String> }`. CLI text output may render this as a small table under the breach list when non-empty.
- **Retirement of stale milestone markers:**
  - `crates/sdivi-core/src/compute/thresholds.rs:92-94` — delete the `TODO(M09)` block; replace with a one-line note describing the per-category dispatch only if the implementation isn't self-evident.
  - `crates/sdivi-core/tests/compute_thresholds_check.rs:107` — rename `override_not_wired_in_m08_base_rate_applies` to `active_override_raises_per_category_limit` and rewrite to assert the new behavior (an unexpired entropy override at 50.0 prevents a per-category entropy=3.0 from breaching while the global rate of 2.0 is still applied to the aggregate dimension).
  - `crates/sdivi-core/tests/compute_thresholds_check.rs:125` — rename `base_rate_applies_regardless_of_override_state_m08` to `expired_override_falls_back_to_global_rate` and update to assert that an expired override is silently ignored.
  - All `M08:` / `M09:` / `(wired up in M09)` annotation comments in test bodies removed.
- **`compute_thresholds_check` doc test in rustdoc** updated to demonstrate an active override; existing first-snapshot doc test preserved.
- **CHANGELOG.md** entry under the next-release section: "Threshold overrides are now active. `ThresholdsInput.overrides` and `ThresholdsInput.today` filter per-category breaches against expiry. Snapshot schema stays `1.0`; new `pattern_metrics.convention_drift_per_category`, `delta.pattern_entropy_per_category_delta`, `delta.convention_drift_per_category_delta`, and `ThresholdBreachInfo.category` are additive."

**Files to create or modify:**

- `crates/sdivi-snapshot/src/snapshot.rs` — extend `PatternMetricsResult` with `convention_drift_per_category`. Update `Default` impl to empty `BTreeMap`.
- `crates/sdivi-snapshot/src/delta.rs` — extend `DivergenceSummary` with the two new `Option<BTreeMap<String, f64>>` fields. Update `null_summary()` and `compute_delta`.
- `crates/sdivi-snapshot/tests/delta_proptest.rs` (or equivalent existing proptest module) — extend the `prop_test_delta_pure` regression to cover the new fields' purity.
- `crates/sdivi-core/src/compute/patterns.rs` — populate `convention_drift_per_category` in `compute_pattern_metrics`. Update doc test.
- `crates/sdivi-pipeline/src/pipeline.rs` — populate `convention_drift_per_category` in `compute_pattern_metrics_from_catalog` to match the pure-compute path byte-for-byte.
- `crates/sdivi-pipeline/tests/parity.rs` (extend M11's pure-compute parity check, if present) — assert `convention_drift_per_category` matches between pipeline and pure-compute paths.
- `crates/sdivi-core/src/compute/thresholds.rs` — implement override + expiry logic; populate `applied_overrides`; populate `category` on per-category breaches. Delete the orphaned `TODO(M09)` block.
- `crates/sdivi-core/src/input/types.rs` — add `AppliedOverrideInfo` (or equivalent type) used by `ThresholdCheckResult`. Add `category: Option<String>` to `ThresholdBreachInfo` (currently in `compute/thresholds.rs`; keep it there unless input/types is the cleaner home).
- `crates/sdivi-core/tests/compute_thresholds_check.rs` — rename two stub tests, rewrite their bodies, add: `active_override_blocks_per_category_breach`, `expired_override_falls_back_to_global_rate`, `aggregate_dimension_uses_global_rate_when_only_one_category_is_overridden`, `applied_overrides_reports_active_and_expired_separately`.
- `crates/sdivi-cli/src/commands/check.rs` — pass through the new `applied_overrides` to JSON output; CLI text output gains a "applied overrides" line when non-empty.
- `crates/sdivi-cli/tests/check_format.rs` — assert `applied_overrides` round-trips in JSON output; add a fixture exercising an active override.
- `bindings/sdivi-wasm/src/lib.rs` — re-export the extended `ThresholdsInput` / `ThresholdCheckResult` shape; tsify regenerates `.d.ts`. No new exports needed (the extension is on existing types).
- `bindings/sdivi-wasm/tests/wasm_bindgen_thresholds.rs` (or equivalent) — smoke-test the new fields are visible from WASM.
- `CHANGELOG.md` — entry as above.
- `docs/library-embedding.md` — short addendum showing a Meridian-style caller supplying `today` and `overrides`.

**Acceptance criteria:**

- `compute_thresholds_check` returns `breached = false` when an active override raises the per-category limit above the observed per-category delta, even though the global rate would have flagged the same value. (New test `active_override_blocks_per_category_breach` covers this.)
- `compute_thresholds_check` returns `breached = true` when the override is expired (`cfg.today > expires`), with the breach using the global rate. (New test `expired_override_falls_back_to_global_rate` covers this.)
- `ThresholdCheckResult.applied_overrides` enumerates every `cfg.overrides` entry with its `active` flag and parsed `expires`.
- `compute_delta` populates the two new per-category delta maps with the union-of-keys-zero-fill rule.
- `null_summary()` returns `None` for both new fields.
- `snapshot_version` remains `"1.0"`. Reading an M13-era snapshot produces no warning and yields aggregate-only deltas (the per-category fields default to `None`).
- `cargo build -p sdivi-core --target wasm32-unknown-unknown --no-default-features` still succeeds with zero forbidden deps.
- The existing M11 bifl-tracker validation harness still passes within the documented tolerances against the new code (run as a regression gate).
- No `TODO(M09)`, `M08`, `M09`, or `(wired up in M09)` strings remain in `crates/sdivi-core/src/` or `crates/sdivi-core/tests/`. (Historical-context comments referring to "the M08 offset fix" in unrelated code paths are out of scope.)
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- `cargo test --workspace` passes including doctests.

**Tests:**

- Unit: `active_override_blocks_per_category_breach`, `expired_override_falls_back_to_global_rate`, `applied_overrides_reports_active_and_expired_separately`, `aggregate_dimension_uses_global_rate_when_only_one_category_is_overridden`, `category_present_in_curr_only_surfaces_positive_delta`, `category_present_in_prev_only_surfaces_negative_delta`.
- Property: extend the existing `prop_test_compute_thresholds_check_pure` to randomly generate override maps with mixed expiry dates and assert purity (same `cfg`, same `summary` → same `ThresholdCheckResult`).
- Property: a new `prop_test_delta_per_category_pure` over random per-category metric maps.
- Doctests on `ThresholdsInput`, `ThresholdCheckResult`, `compute_thresholds_check`, `compute_delta`, `null_summary` updated to match new shapes.
- Schema-stability test (existing or new): a serialized M13 snapshot deserializes cleanly and yields aggregate-only deltas with the per-category fields `None`.
- Cross-binding parity smoke test: serialize a `ThresholdCheckResult` with `applied_overrides` populated, round-trip through `serde-wasm-bindgen`, assert equality.

**Watch For:**

- **Doc-comment placement.** Per CLAUDE.md Code Conventions, when inserting `category: Option<String>` into `ThresholdBreachInfo`, ensure a blank line separates the new field's `///` block from the next field's `///` block. The file is in `sdivi-core` where `#![deny(missing_docs)]` will catch a missing doc, but won't catch a doc that silently re-attaches to the wrong field.
- **Aggregate-vs-per-category semantics.** It is intentional that the global `pattern_entropy_rate` continues to apply to the aggregate `summary.pattern_entropy_delta` even when per-category overrides exist. An override of `pattern_entropy_rate = 5.0` for category `error_handling` only suppresses breaches *of the error_handling per-category delta*, never of the aggregate. Tests must lock this distinction down — otherwise a future refactor could collapse them.
- **Expiry-boundary date.** `cfg.today > expires` is the silent-ignore condition. `cfg.today == expires` should keep the override active (the override "expires" *after* the named date, not on it). Test both boundaries explicitly.
- **`expires` parsing.** `validate_date_format` exists in `sdivi-config` outside the `loader` feature gate (M08 deliverable). If for any reason `compute_thresholds_check` cannot parse a stored `expires` string, the override is treated as inactive and a structured note is added to `applied_overrides[<cat>].expired_reason`. `compute_thresholds_check` does not return `Err` for malformed overrides — config-load-time validation already rejects those at `ConfigError::InvalidValue`.
- **Schema additivity, not bumping.** Snapshot schema stays `"1.0"`. New fields use `#[serde(default)]` so M13-era snapshots deserialize cleanly. No `MIGRATION_NOTES.md` entry required.
- **Stale `M08`/`M09` strings outside thresholds tests.** `git grep "M0[89]"` in `crates/sdivi-core/` is the verification hammer. Historical-context comments in unrelated tests (e.g., `leiden_id_collision.rs`'s "Fix (M08): ...") are descriptive and stay; the test names and TODO markers in `compute_thresholds_check.rs` are the targets.
- **HAR file.** The unchecked `[ ]` item in `.tekhton/HUMAN_ACTION_REQUIRED.md` (currently misfiled under `## Resolved`) should be checked off and moved to a properly-formatted Resolved entry by the milestone-closing commit.

**Seeds Forward:**

- Per-category metric breakouts in `PatternMetricsResult` and `DivergenceSummary` are now public surface. Future analyzers (e.g., the change-coupling work in M15) can follow the same pattern: aggregate field for backward-compat consumers, per-category map for fine-grained gating.
- The `applied_overrides` diagnostic surface is the precedent for any future "rule was considered but not fired" reporting in `compute_*` functions. Consumer-app dashboards will lean on it.
- v0.x can introduce per-category override of `coupling_delta_rate` and `boundary_violation_rate` against future per-category deltas with no schema bump — the input shape already accepts the rates.

---

---

## Archived: 2026-05-01 — Unknown Initiative


#### Milestone 15: Change-Coupling Analyzer
<!-- milestone-meta
id: "15"
status: "done"
-->
<!-- PM-tweaked: 2026-05-01 -->


**Scope:** Implement the change-coupling computation that turns the existing `[change_coupling]` config block into a real analyzer. Walk the last `history_depth` commits via shell-out to `git log`, compute file-pair co-change frequencies, surface them in the snapshot, and feed them into Leiden as edge weights when `boundaries.weighted_edges = true`. Two-layer split: I/O lives in `sdivi-pipeline::change_coupling`; the math lives in `sdivi-core::compute_change_coupling` (pure, WASM-callable, foreign-extractor-friendly).

**Why this milestone exists:** `ChangeCouplingConfig`, `boundaries.weighted_edges`, and the `[change_coupling]` block in `init`'s default config template are advertised features that no code currently reads. CLAUDE.md's `Config Architecture` table documents `min_frequency = 0.6` and `history_depth = 500` as if they did something. They don't. Shipping v0 with a documented-but-inert analyzer is worse than shipping v0 without it: it actively misleads embedders and adopters. This milestone makes the existing config surface load-bearing and gives Meridian (and any other consumer that may want temporal coupling later) a stable, pure-compute entry point that does not depend on the consumer having its own libgit2 or git binary in scope.

**Deliverables:**

- **Pure-compute entry point in `sdivi-core`:**
  - `compute_change_coupling(events: &[CoChangeEventInput], cfg: &ChangeCouplingConfigInput) -> ChangeCouplingResult` — given an ordered list of commit-events (each carrying the set of paths touched by that commit), produces the set of file-pair frequencies that meet `min_frequency`. Pure, referentially transparent, no I/O, no clock.
  - Algorithm: for each commit, enumerate every unordered pair of paths in the commit's `files`; increment a per-pair counter; divide each pair's count by `commits_analyzed`; emit pairs whose frequency `>= min_frequency` **and whose `cochange_count >= 2`** (a single co-occurrence is not change coupling — it may be a coincidence of the same initial commit creating both files). [PM: Added the `cochange_count >= 2` guard to match the stated unit-test behaviour "single commit → no pairs (a pair requires two distinct commits)." Without this guard, a single commit touching `{a, b}` produces frequency `1.0`, which passes any `min_frequency` filter. The guard makes the test case and the algorithm consistent.]
  - Pairs are ordered by `(source, target)` lexicographically with `source < target`; output is a `Vec<CoChangePair>` sorted by `(source, target)` for byte-identical determinism.
  - `commits_analyzed = min(events.len(), cfg.history_depth as usize)`. The function operates on the trailing window: events are assumed oldest-first; the last `history_depth` are the analyzed window.
- **New input/output types in `sdivi-core::input`:**
  - `CoChangeEventInput { commit_sha: String, commit_date: String, files: Vec<String> }` — `commit_date` is ISO-8601 (`YYYY-MM-DDTHH:MM:SSZ`); `files` are canonical NodeIds (validated via the existing `validate_node_id`).
  - `ChangeCouplingConfigInput { min_frequency: f64, history_depth: u32 }` — mirrors `sdivi-config::ChangeCouplingConfig` with serde-and-tsify derives. `min_frequency` validated `[0.0, 1.0]` at the entry point; out-of-range returns `AnalysisError::InvalidConfig`.
  - `ChangeCouplingResult { pairs: Vec<CoChangePair>, commits_analyzed: u32, distinct_files_touched: u32 }`. [PM: `distinct_files_touched` is the count of unique path strings that appear in at least one of the analyzed commit events (i.e., `HashSet`-collect `files` across the trailing `commits_analyzed` events, then `.len()`). This was not defined in the original.]
  - `CoChangePair { source: String, target: String, frequency: f64, cochange_count: u32 }` — `source < target` lexicographically; `frequency = cochange_count / commits_analyzed`.
- **I/O entry point in `sdivi-pipeline`:**
  - `crates/sdivi-pipeline/src/change_coupling.rs` — `collect_cochange_events(repo_root: &Path, history_depth: u32, ending_at: Option<&str>) -> Result<Vec<CoChangeEventInput>, ChangeCouplingError>`. Shells out to `git log --no-pager -z --name-only --format=%x00COMMIT%x00%H%x00%cI%x00 -n <history_depth> [<ending_at>]`. Parses NUL-separated output into events. Translates paths to canonical NodeId form (forward slashes, repo-relative, no leading `./`).
  - `ending_at = None` defaults to `HEAD`. `M16` will pass a resolved SHA when `--commit REF` is in play.
  - Returns `Ok(vec![])` if the repo has no `.git/` (Rule 16-style: missing input is normal), with a stderr `tracing::info!` line — never an error.
  - Returns a structured `ChangeCouplingError` for malformed `git log` output or non-zero exit (other than the "no git" path).
- **Pipeline integration:**
  - `Pipeline::snapshot` calls `collect_cochange_events` after Stage 2 (graph), passes the events plus `ChangeCouplingConfigInput` to `compute_change_coupling`, stores the result in the snapshot.
  - When `config.boundaries.weighted_edges = true`, the Leiden detection stage (Stage 3) receives the change-coupling pairs as edge weights: for each `(source, target)` pair in `ChangeCouplingResult.pairs` whose endpoints both exist in the dependency graph, the existing import edge's weight is **multiplied** by `(1.0 + frequency)`. (Multiplicative — never additive — so a pair with frequency 1.0 doubles the edge weight; a pair with frequency 0.6 raises it by 60%.) Pairs whose endpoints are not connected by an import edge are **not** added as new edges (KDD-5 keeps the graph import-only; change-coupling is a weight modulation, not a new topology).
  - `compute_change_coupling` is called regardless of `weighted_edges`; the result lands in the snapshot either way.
- **Snapshot shape extension (additive, schema stays `"1.0"`):**
  - `Snapshot.change_coupling: Option<ChangeCouplingResult>` — `None` when the repo has no git history (or `history_depth = 0`); `Some(_)` otherwise. `#[serde(default)]` so M14-era snapshots deserialize as `None`.
  - No new `DivergenceSummary` field for v0 — change-coupling is a descriptive snapshot dimension, not a threshold-gated one. (The existing `coupling_delta_rate` continues to gate graph density, not change-coupling.)
- **Leiden input shape:**
  - `LeidenConfigInput` gains `edge_weights: Option<BTreeMap<(String, String), f64>>` — `None` (default) means "all weights = 1.0" (existing behavior); `Some(map)` means "use the supplied weights, defaulting to 1.0 for unlisted edges". `BTreeMap` keys are `(source, target)` with `source < target`. `#[serde(default)]`.
  - `detect_boundaries` (existing in `sdivi-core`) feeds the optional weights into the petgraph edge weights at graph-construction time. The existing Leiden algorithm already operates on weighted edges (graph density, modularity calculations); this surfaces the existing capability through the input shape.
  - Pipeline path: when `weighted_edges = true`, `Pipeline::snapshot` constructs the `edge_weights` map from `ChangeCouplingResult.pairs` and passes it through `LeidenConfigInput`.
- **WASM exports:**
  - `bindings/sdivi-wasm/src/lib.rs` — export `compute_change_coupling`, `CoChangeEventInput`, `ChangeCouplingConfigInput`, `ChangeCouplingResult`, `CoChangePair`. tsify regenerates `.d.ts`.
  - This is the Meridian-facing surface: a consumer with its own commit history walker (e.g., reading from the IDE's git index) supplies `Vec<CoChangeEventInput>` directly, no shell-out needed.
- **CLI surface:**
  - `sdivi show` text output gains a `change coupling: <N> pairs (top 5: …)` line when `change_coupling` is `Some` and non-empty.
  - `sdivi show --format json` already serializes the full snapshot; the new field appears automatically.
  - No new CLI flags. `boundaries.weighted_edges` is the existing knob.
- **Documentation:**
  - `docs/library-embedding.md` — new section "Computing change-coupling from a foreign extractor" showing a Meridian-style consumer that supplies its own `CoChangeEventInput` slice (e.g., from VSCode's git extension) and calls `compute_change_coupling` directly via WASM.
  - `docs/cli-integration.md` — short note on `boundaries.weighted_edges` and how it interacts with Leiden communities.
  - `docs/determinism.md` — note that `git log` output ordering is deterministic for a fixed `HEAD`, and the canonical NodeId translation ensures cross-platform path equivalence (forward slashes, no `./`).
- **CHANGELOG.md** entry: "Change-coupling analyzer wired up. New snapshot field `change_coupling`. New `boundaries.weighted_edges = true` mode multiplies import-edge weights by `(1.0 + frequency)`. New pure-compute entry point `sdivi_core::compute_change_coupling` exported through WASM. Schema stays `1.0`."

**Migration Impact:** [PM: Added this section. The snapshot schema stays `"1.0"` and the new field is `#[serde(default)]`, so M14-era snapshots deserialize with `change_coupling = None` without any error. No config migration is required. The `boundaries.weighted_edges` key was already present in the schema (previously inert); after this milestone it becomes load-bearing, which changes its observable behaviour but not its type or default value (`false`). Users who had `weighted_edges = true` in their config before M15 will now see Leiden receive modified edge weights on their next snapshot run; they should be aware their community partition may shift.]

**Files to create or modify:**

- **New:** `crates/sdivi-pipeline/src/change_coupling.rs` (`collect_cochange_events`, `ChangeCouplingError`).
- **New:** `crates/sdivi-core/src/compute/change_coupling.rs` (`compute_change_coupling`).
- **New (or extend `crates/sdivi-core/src/input/types.rs`):** add `CoChangeEventInput`, `ChangeCouplingConfigInput`, `ChangeCouplingResult`, `CoChangePair`.
- **Modify:** `crates/sdivi-core/src/input/types.rs` — extend `LeidenConfigInput` with `edge_weights: Option<BTreeMap<(String, String), f64>>`.
- **Modify:** `crates/sdivi-core/src/compute/boundaries.rs` (or wherever `detect_boundaries` lives) — feed `edge_weights` into the petgraph edge weights.
- **Modify:** `crates/sdivi-snapshot/src/snapshot.rs` — add `change_coupling: Option<ChangeCouplingResult>` to `Snapshot`. Update `assemble_snapshot` to take the new field.
- **Modify:** `crates/sdivi-pipeline/src/pipeline.rs` — call `collect_cochange_events`, call `compute_change_coupling`, populate `Snapshot.change_coupling`, wire edge weights into Leiden when `weighted_edges = true`.
- **Modify:** `crates/sdivi-pipeline/src/lib.rs` — re-export `change_coupling` module.
- **Modify:** `crates/sdivi-cli/src/output/text.rs`, `crates/sdivi-cli/src/output/json.rs` — surface change-coupling in `sdivi show`.
- **Modify:** `bindings/sdivi-wasm/src/lib.rs` — export the new compute function and types.
- **Modify:** `bindings/sdivi-wasm/Cargo.toml` — no new deps; tsify handles the `.d.ts`.
- **New tests:**
  - `crates/sdivi-core/tests/compute_change_coupling.rs` — pure unit tests, including determinism, sort order, and `min_frequency` filtering.
  - `crates/sdivi-pipeline/tests/change_coupling_git.rs` — integration test against a tempdir git repo with a known commit history (set up in-test via `std::process::Command::new("git")` calls).
  - `tests/change_coupling_lifecycle.rs` (workspace-level) — end-to-end: snapshot a fixture, assert `Snapshot.change_coupling` is `Some` with expected pairs.
  - `crates/sdivi-detection/tests/leiden_weighted_edges.rs` — assert that `weighted_edges = true` with a non-trivial weight map produces a different community partition than `false` on the same graph (or the same partition with higher modularity — either is acceptance).
- **Test fixture:** `tests/fixtures/change-coupling-history/` — a small git fixture (built by a setup script under `target/test-fixtures` like the existing `evolving/` fixture) with 10 commits exercising known co-change pairs. The setup script lives in `tools/build-test-fixtures.sh`; the integration test invokes it before running. The fixture is **not** checked in as a git repo (no nested `.git/`); the script reconstructs it from a manifest.
- **Modify:** `tools/validate-against-bifl-tracker.sh` (M11 harness) — extend to assert `Snapshot.change_coupling` is `Some` for the bifl-tracker baselines (it has plenty of history). Tolerance: existing per-dimension tolerances unchanged; the new field is asserted to be non-empty but its content is not gated against sdi-py baselines (sdi-py's change-coupling implementation may not have shipped, and KDD-1 says clean break on snapshots anyway).
- **Modify:** `CHANGELOG.md`, `docs/library-embedding.md`, `docs/cli-integration.md`, `docs/determinism.md`.

**Acceptance criteria:**

- `Pipeline::snapshot` against the test fixture with 10 commits produces a snapshot whose `change_coupling.pairs` contains exactly the expected pairs (computed by hand in the test) at exactly the expected frequencies.
- `compute_change_coupling` is pure: same `events` + same `cfg` produces a byte-identical `ChangeCouplingResult` (proptest covers this).
- A repo with no `.git/` directory produces a snapshot with `change_coupling = None` and no warning to stderr beyond a `tracing::info!` line.
- A repo with `history_depth = 0` produces `change_coupling = Some(ChangeCouplingResult { pairs: vec![], commits_analyzed: 0, distinct_files_touched: 0 })` (zero-history is *not* the same as "no git" — the user explicitly disabled the analyzer).
- `cargo build -p sdivi-core --target wasm32-unknown-unknown --no-default-features` succeeds; `cargo tree -p sdivi-core --target wasm32-unknown-unknown --no-default-features` shows zero entries for shell-out crates, `git2`, `gix`, `walkdir`, `ignore`, `rayon`, `tempfile`.
- `bindings/sdivi-wasm` test imports `compute_change_coupling` from the bundled `.wasm` and produces the same result as the native call for the same `CoChangeEventInput` slice.
- `boundaries.weighted_edges = true` against the change-coupling fixture produces a Leiden partition whose modularity is **not less than** the unweighted-baseline modularity (equality is acceptable; degradation is a regression).
- The existing M11 bifl-tracker harness still passes within the documented tolerances on per-dimension metrics.
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- `cargo test --workspace` passes including doctests.
- `snapshot_version` remains `"1.0"`.
- [PM: `sdivi show` (text format) against a snapshot with a non-empty `change_coupling` field prints a line matching the pattern `change coupling: <N> pairs (top 5: …)` to stdout and nothing to stderr for that field. Verified by `assert_cmd` in the CLI exit-code test suite.]

**Tests:**

- Unit (pure-compute): empty events → empty pairs; single commit touching `{a, b}` → no pairs (frequency = 1.0 but `cochange_count = 1 < 2`); [PM: Restated to match the clarified algorithm: a pair requires `cochange_count >= 2`, not just `frequency >= min_frequency`.] two commits each touching `{a, b}` → one pair `(a, b)` at frequency 1.0 with `cochange_count = 2`; out-of-window commits ignored when `events.len() > history_depth`.
- Unit (filter): three pairs with frequencies 0.8, 0.6, 0.4 against `min_frequency = 0.6` → exactly two pairs survive, sorted lexicographically (both also have `cochange_count >= 2` in the fixture).
- Property: `prop_test_change_coupling_deterministic` — randomly generated `Vec<CoChangeEventInput>` × random `cfg` produces stable output.
- Property: `prop_test_change_coupling_pair_ordering` — emitted pairs are always sorted with `source < target`.
- Integration (git): `crates/sdivi-pipeline/tests/change_coupling_git.rs` — tempdir repo, 5 commits with known file touches, `collect_cochange_events` returns the expected `Vec<CoChangeEventInput>`.
- Integration (path canonicalization): a fixture with a path containing a space and a Unicode character round-trips correctly through the `git log -z` parser and through `validate_node_id`.
- Integration (Leiden weighted): synthetic graph with known partition under unweighted Leiden, weighted Leiden produces a different (or higher-modularity) partition with a known weight map.
- Integration (no-git): a tempdir without `.git/` produces `change_coupling = None`.
- Integration (history-depth-0): a repo with `history_depth = 0` produces an empty `ChangeCouplingResult`.
- [PM: Unit (`distinct_files_touched`): two commits, first touching `{a, b}`, second touching `{b, c}` → `distinct_files_touched = 3` (unique paths: a, b, c). Validates the "unique path strings across analyzed events" definition.]
- WASM smoke: `compute_change_coupling` callable from a JS test (mirrors M12 patterns).
- Doctests on `compute_change_coupling`, `CoChangeEventInput`, `ChangeCouplingResult`, `CoChangePair` with concrete `# Examples` blocks.

**Watch For:**

- **Path canonicalization.** `git log -z --name-only` emits paths in repo-relative form with forward slashes on every platform (git's internal representation). Translation to canonical NodeId is essentially a no-op except for stripping any leading `./` and asserting via `validate_node_id`. Test fixture must include a path that would *not* round-trip cleanly under naive `Path::to_str()` (e.g., a path with a space) so the parser is exercised.
- **`-z` framing.** `git log -z --name-only --format=%x00COMMIT%x00%H%x00%cI%x00 -n N` produces NUL-separated output where commits are delimited by a sentinel `COMMIT` token (with NULs around it). A more careful framing using `--format` with multiple unique sentinels may be needed; the milestone author should validate against a fixture with merge commits, empty commits, and commits whose subject contains arbitrary bytes. Do not parse with regex; parse byte-by-byte.
- **Merge commits.** A merge commit's `--name-only` output lists files that conflict-resolved during the merge — typically zero files for a non-conflict merge. This is fine for v0; document the behavior. Do not pass `--first-parent` (it would skip topic-branch commits' co-change signal).
- **Renames.** `git log --follow` is single-file only; for a multi-file co-change view, follow-renames is impractical at the v0 scope. Renames register as one delete + one add in two separate commits, which spuriously inflates the file count. Document in `docs/migrating-from-sdi-py.md` and `docs/cli-integration.md`. Do not silently filter rename pairs — that hides real signal.
- **Submodules.** `git log` does not recurse. Submodule changes do not contribute to co-change. Document.
- **Determinism across machines.** `git log` output ordering is deterministic given a fixed HEAD/ref. The pure-compute side preserves order; the I/O side does not reorder. A cross-platform regression test (run on Linux, macOS, Windows in CI) must produce byte-identical `ChangeCouplingResult` for the same fixture.
- **Windows shell-out.** `std::process::Command::new("git")` works on Windows when `git` is on `PATH` (Git for Windows installs it there by default). The milestone must include a Windows CI matrix entry that verifies the integration test runs on `windows-latest`. If `git` is not installed in the CI runner image, `actions/setup-git` or equivalent is required.
- **Empty-window edge case.** `history_depth` larger than the actual commit count: `commits_analyzed = events.len()`, not `history_depth`. The frequency denominator is the actual analyzed window, never the configured ceiling. Test explicitly.
- **`min_frequency = 0.0`.** Yields every pair that co-changed in at least 2 commits (`cochange_count >= 2`). Allowed; document the implication. [PM: Consistent with the `cochange_count >= 2` guard: `min_frequency = 0.0` does not mean "any single co-occurrence counts."]
- **`min_frequency = 1.0`.** Yields only pairs that co-changed in every analyzed commit. Allowed; document.
- **Weighted Leiden + KDD-2 verification.** The `verify-leiden` harness (M11) compares against `leidenalg`'s unweighted output by default. Weighted-edge verification against `leidenalg`'s weighted mode is *out of scope* for v0 — KDD-2's tolerance is partition quality, not bit-identity, and weighted-mode parity verification is best left for v0.x. Document this in `docs/determinism.md`.
- **CLAUDE.md update.** The "Config Architecture" table currently lists `min_frequency = 0.6` and `history_depth = 500` without indicating the analyzer is wired. The user should update CLAUDE.md after this milestone closes to reflect that `[change_coupling]` is no longer a forward-reference. Do not modify CLAUDE.md from this milestone.
- **Doc-comment placement.** When inserting `change_coupling: Option<ChangeCouplingResult>` into `Snapshot`, ensure a blank line separates the new field's `///` block from the next field's `///` block.

**Seeds Forward:**

- `compute_change_coupling` taking `CoChangeEventInput` (string SHAs, string dates, string paths) is the contract Meridian relies on. Adding fields to `CoChangeEventInput` is additive. Removing fields is breaking and post-1.0.
- `boundaries.weighted_edges = true` is the lever for "use both structural and historical coupling for community detection." A future v0.x can introduce per-pair weight overrides for users who want to suppress noisy co-change signals.
- The `LeidenConfigInput.edge_weights` shape is the precedent for future graph-edge-weighting features (test coupling, runtime trace coupling, etc.) — they all reduce to producing a `BTreeMap<(String, String), f64>` and feeding it through the same input shape.
- M16 (`--commit REF`) reuses the same `git` shell-out pattern (`std::process::Command`, NUL-framing, repo-relative paths) and `change_coupling`'s `ending_at` parameter for "history ending at REF."
- Post-v0 metric: a `change_coupling_delta_rate` threshold that gates "unusual increase in co-change pair density" can be added with no schema bump — the per-snapshot pair count is already a number.

---

## Archived: 2026-05-01 — Unknown Initiative

#### Milestone 16: Snapshot at Historical Commit (`--commit REF`)
<!-- milestone-meta
id: "16"
status: "done"
-->


**Scope:** Make `sdivi snapshot --commit REF` actually run the pipeline against the tree at REF. Today the flag is plumbed from CLI → `Pipeline::snapshot(commit)` → snapshot metadata as a label, but the parsing stage still reads the working tree, which is misleading. This milestone resolves REF to an immutable SHA, materializes the tree at REF in a tempdir via `git archive | tar -x`, runs the full pipeline against the tempdir, and labels the resulting snapshot with the resolved SHA and the commit's commit-date. M15's change-coupling analyzer is rewired to use `ending_at = REF` so historical snapshots get their proper historical context.

**Why this milestone exists:** CLAUDE.md's "What Not to Build Yet" section currently states `sdivi snapshot --commit REF works for individual commits.` That is false today: only the snapshot's `commit` field is populated; the analyzed tree is still the working directory's. Two consequences: (1) historical backfill produces snapshots labeled with one commit but reflecting a different tree, which silently corrupts trend lines; (2) the documented one-shot historical analysis path is unusable, so users have no in-tool way to backfill. With M15 surfacing change-coupling per-snapshot, fixing `--commit REF` is now load-bearing for any consumer that wants accurate temporal data — including Meridian's "show me how this codebase looked at last month's release tag" use case. The change is small, the surface is contained, and shipping v0 with the documented-but-broken flag is an honesty problem more than a scope problem.

**Deliverables:**

- **Ref resolution and tree extraction in `sdivi-pipeline`:**
  - `crates/sdivi-pipeline/src/commit_extract.rs` (new module):
    - `resolve_ref_to_sha(repo_root: &Path, reference: &str) -> Result<String, CommitExtractError>` — shells out to `git -C <repo_root> rev-parse --verify <reference>`. Returns the full 40-char SHA. Errors: non-zero exit, output not a valid SHA.
    - `commit_date_iso(repo_root: &Path, sha: &str) -> Result<String, CommitExtractError>` — `git -C <repo_root> show -s --format=%cI <sha>`. Returns `YYYY-MM-DDTHH:MM:SS+00:00`-style ISO 8601.
    - `extract_commit_tree(repo_root: &Path, sha: &str) -> Result<TempDir, CommitExtractError>` — pipes `git archive --format=tar <sha>` into `tar -xC <tempdir>`. The pipe is set up via two `std::process::Command` invocations connected through `std::process::Stdio`. The returned `TempDir` cleans up on drop.
    - `CommitExtractError` thiserror enum: `{ RefResolutionFailed { reference, stderr }, CommitNotFound { sha }, ArchiveFailed { stderr }, TarFailed { stderr }, IoError(std::io::Error) }`.
- **Pipeline integration:**
  - `Pipeline::snapshot(repo_root, commit, timestamp)` (existing signature, no change) becomes:
    - When `commit = None`: behavior unchanged. Pipeline runs against `repo_root`. Snapshot's `commit` field is `None`.
    - When `commit = Some(reference)`:
      1. `sha = resolve_ref_to_sha(repo_root, reference)?`
      2. `commit_date = commit_date_iso(repo_root, &sha)?`
      3. `tempdir = extract_commit_tree(repo_root, &sha)?`
      4. The provided `timestamp` argument is **overridden** by `commit_date` for snapshot file naming and `Snapshot.timestamp` (so trend ordering tracks chronology, not wall-clock-of-CLI-invocation).
      5. The five-stage pipeline runs against `tempdir.path()` (parsing, graph, detection, patterns, snapshot assembly).
      6. M15's change-coupling collection runs against the **original `repo_root`**, not the tempdir (the tempdir has no `.git/`). `collect_cochange_events` is called with `ending_at = Some(&sha)` so the analyzed window is the `history_depth` commits ending at REF, not at HEAD.
      7. The Leiden warm-start cache continues to read/write `<repo_root>/.sdivi/cache/partition.json` (not the tempdir). Cache reuse across `--commit` invocations is a feature, not a bug.
      8. Snapshot file is written to `<repo_root>/.sdivi/snapshots/` (not the tempdir's). Atomic write + retention enforcement unchanged.
      9. `Snapshot.commit = Some(sha)` (the resolved SHA, not the user-supplied ref name).
      10. `tempdir` drops at end of scope, removing the materialized tree.
  - `Pipeline::snapshot_with_mode` follows the same logic; `WriteMode::EphemeralForCheck` from M09 still applies (no snapshot persisted, no cache write).
- **Documentation:**
  - `docs/cli-integration.md` — new section "Analyzing a historical commit" describing `sdivi snapshot --commit REF`, what the flag does, what gets persisted (the snapshot is a real persisted snapshot named after the commit-date timestamp), and the interaction with change-coupling history.
  - `docs/library-embedding.md` — note that the pure-compute path (`sdivi-core`) does not need any of this — embedders that already have their own tree extraction (the consumer app, Meridian) call `compute_*` directly with whatever tree they have in hand. `--commit REF` is an `sdivi-pipeline` / CLI convenience.
  - `CLAUDE.md` — the line `Historical backfill UX — sdivi snapshot --commit REF works for individual commits. Batch backfill is unsupported; users script it.` in "What Not to Build Yet" remains accurate after this milestone (now genuinely so). The user is responsible for confirming the line still matches reality post-merge; no milestone-time edit.
- **CHANGELOG.md** entry: "`sdivi snapshot --commit REF` now analyzes the actual tree at REF. The snapshot is labeled with the resolved SHA and the commit's commit-date (not wall-clock time). Change-coupling history is computed ending at REF. Pre-v0 callers relying on the prior label-only behavior must adjust."
- **Error UX:**
  - `--commit nonexistent` → `CommitExtractError::RefResolutionFailed`, CLI exit 1 with a stderr message naming the unresolvable reference.
  - `--commit` against a non-git directory → `RefResolutionFailed` (git itself errors). Same exit 1 path.
  - `--commit` against a shallow clone where REF is below the shallow boundary → `git rev-parse` succeeds but `git archive` may fail; surface as `ArchiveFailed`.
  - All error paths include the captured `stderr` from the failing `git` invocation so users see git's actual diagnostic.

**Files to create or modify:**

- **New:** `crates/sdivi-pipeline/src/commit_extract.rs` — `resolve_ref_to_sha`, `commit_date_iso`, `extract_commit_tree`, `CommitExtractError`.
- **Modify:** `crates/sdivi-pipeline/src/error.rs` (or wherever `PipelineError` is defined) — add `CommitExtract(CommitExtractError)` variant with `#[from]`.
- **Modify:** `crates/sdivi-pipeline/src/pipeline.rs` — branch `Pipeline::snapshot_with_mode` on `commit.is_some()`; call the new extraction helpers; pass tempdir as the parsing root; pass `ending_at = Some(&sha)` to `collect_cochange_events`; override `timestamp` with `commit_date_iso` output.
- **Modify:** `crates/sdivi-pipeline/src/lib.rs` — re-export `commit_extract` module (probably `pub(crate)` unless an embedder needs the helpers directly; expose at minimum `CommitExtractError` for downstream error matching).
- **Modify:** `crates/sdivi-cli/src/commands/snapshot.rs` — error formatting for the new `PipelineError::CommitExtract` variant; CLI-level integration test fixture.
- **New:** `crates/sdivi-pipeline/tests/commit_snapshot.rs` — fixture git repo with three commits; snapshot at HEAD, HEAD~1, HEAD~2; assert each is distinct, assert SHA labeling, assert commit-date timestamping, assert tempdir cleanup.
- **New:** `tests/historical_commit_lifecycle.rs` (workspace-level) — full CLI invocation via `assert_cmd`: `sdivi snapshot --commit HEAD~1` against a built fixture, parse snapshot JSON, verify expected fields, verify file naming under `.sdivi/snapshots/`.
- **Modify:** `crates/sdivi-cli/tests/exit_codes.rs` — add cases for `--commit nonexistent` (exit 1) and `--commit` in a non-git directory (exit 1).
- **Modify:** `tools/build-test-fixtures.sh` — extend to build a `historical-commits/` fixture: a tempdir git repo with three known commits each touching a known file set. Built into `target/test-fixtures/historical-commits/` like the existing `evolving/` fixture.
- **Modify:** `crates/sdivi-pipeline/Cargo.toml` — `tempfile` is already a dev-dep; promote to a runtime dep (it's now used in `extract_commit_tree`'s production path). No new external deps.
- **Modify:** `docs/cli-integration.md`, `CHANGELOG.md`.

**Acceptance criteria:**

- `sdivi snapshot --commit HEAD~1` against the fixture (a 3-commit repo where each commit adds a distinct file) produces a snapshot whose graph node count matches the file count *as of HEAD~1*, not HEAD.
- The snapshot's `commit` field is the full 40-char SHA, not the ref name `HEAD~1`.
- The snapshot's `timestamp` field is the commit's commit-date in ISO 8601 form, not the wall-clock time of the CLI invocation.
- The snapshot file's name (under `.sdivi/snapshots/`) reflects the commit-date timestamp, so lexicographic sort matches chronological sort across mixed `--commit` and HEAD invocations.
- The Leiden warm-start cache at `<repo_root>/.sdivi/cache/partition.json` is read and written normally (not isolated to the tempdir).
- The change-coupling section (M15) of the snapshot reflects the `history_depth` commits ending at the resolved SHA, not ending at HEAD.
- `tempdir` is dropped before `Pipeline::snapshot` returns; a test that captures the tempdir path during execution and re-checks after asserts the path no longer exists.
- `sdivi snapshot --commit nonexistent-ref` exits with code 1 and a stderr message including git's actual `rev-parse` error.
- `sdivi snapshot --commit HEAD` (with no other changes) produces a snapshot byte-identical to `sdivi snapshot` with no `--commit` flag, *except* that the `commit` field carries the resolved SHA where the no-flag path leaves it `None`, and the `timestamp` is the commit-date instead of wall-clock. (This asymmetry is the intended UX — users who want labeling can pass `--commit HEAD`.)
- A second invocation of `sdivi snapshot --commit <sha>` with the same SHA produces a byte-identical snapshot file (modulo the file's mtime), verifying determinism across re-runs.
- The M11 bifl-tracker harness still passes within tolerances against HEAD-based snapshots; a new sub-step picks one historical commit from the bifl-tracker baseline set, runs `sdivi snapshot --commit <historical-sha>`, and verifies the per-dimension metrics match the sdi-py-era snapshot for that same commit (within the same KDD-2 tolerances). This is the v0 acceptance gate for "historical backfill is real."
- `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- `cargo test --workspace` passes including doctests.
- `snapshot_version` remains `"1.0"`.

**Tests:**

- Integration: `crates/sdivi-pipeline/tests/commit_snapshot.rs` covers the three-commit fixture across HEAD/HEAD~1/HEAD~2.
- Integration: `tests/historical_commit_lifecycle.rs` exercises the full CLI invocation.
- Negative: `--commit nonexistent` → exit 1.
- Negative: `--commit` in a dir with no `.git/` → exit 1 (git emits a `not a git repository` error; surface it through `RefResolutionFailed`).
- Determinism: two back-to-back invocations of `--commit <sha>` produce byte-identical snapshot JSON.
- Cleanup: tempdir path captured during run is gone after `Pipeline::snapshot` returns. (Test technique: `TempDir::into_path` is *not* called; the test retrieves the path before drop and asserts post-drop non-existence — but since `tempdir` is owned inside the pipeline, the test instead replaces extraction with a hookable variant or asserts via `/proc/self/fd` / open-handle inspection. Pragmatic alternative: count `target/sdivi-extract-*` directories before and after; assert delta = 0.)
- Change-coupling-at-REF: fixture with a known co-change history; snapshot at HEAD~1 produces a `change_coupling` section reflecting commits up to HEAD~1, *not* including the most recent commit.
- Cross-platform: the integration test runs on Linux, macOS, and Windows in CI. Windows requires `git` and `tar` (or the `git archive --format=zip` fallback — see Watch For).

**Watch For:**

- **Windows `tar` availability.** Default Windows 10/11 ships `tar.exe` in `System32` since 2017, but minimal CI runner images may not have it on `PATH`. Mitigation order: (1) check for `tar` via `Command::new("tar").arg("--version")` at the top of `extract_commit_tree`; (2) if absent, fall back to `git archive --format=zip` piped to a Rust-side zip extractor. For v0, prefer mitigation (1) only; if `tar` is missing, return a structured error advising the user. Document the requirement in `docs/cli-integration.md`. The `windows-latest` GitHub runner has `tar` available — verify in CI.
- **Pipe wiring.** `git archive --format=tar <sha> | tar -xC <tempdir>` is two processes. In Rust: spawn `git archive` with `Stdio::piped()` for stdout, spawn `tar` with `Stdio::piped()` for stdin, copy bytes via `std::io::copy` (or pass the file handle directly via `Stdio::from(child.stdout)`). Test: archive a commit with 1MB+ of files, ensure the pipe doesn't deadlock on a small kernel buffer.
- **`.gitattributes export-ignore`.** `git archive` honors `.gitattributes export-ignore` directives. Files marked `export-ignore` are excluded from the archive. This may surprise users who expect "everything tracked at this commit" — document. Acceptance test: a fixture with an `export-ignore`'d file and a non-ignored file; the snapshot's graph excludes the ignored file. (This is *correct* behavior — but it must be documented because it diverges from `git checkout`'s semantics.)
- **Shallow clones.** If `repo_root` is a shallow clone and REF is below the shallow boundary, `git rev-parse` succeeds but `git archive` errors. The error is surfaced cleanly via `ArchiveFailed`. CI runners often produce shallow clones by default; the integration tests should use `actions/checkout@v4` with `fetch-depth: 0` for milestones touching git history.
- **Submodules.** `git archive` does not recurse into submodules. A snapshot at REF with submodule references will not include submodule contents. Document. (Same caveat as M15.)
- **Symlinks.** `git archive | tar -x` preserves symlinks. The parsing stage's `walkdir + ignore` already handles symlinks correctly. No special handling needed.
- **Path canonicalization.** When the tempdir replaces `repo_root`, `walkdir` yields paths under the tempdir. The `FeatureRecord` machinery already produces NodeIds via the existing canonicalizer, which strips the prefix and emits repo-relative paths. Verify: a snapshot at HEAD~1 has the same `NodeInput.id` strings as a snapshot at HEAD for files that exist in both — this is what makes the partition-cache warm-start meaningful across `--commit` invocations.
- **Cache poisoning across `--commit` invocations.** The Leiden warm-start cache is keyed by the partition's structure, not by file content. If commit A and commit B have very different file sets, warming from A's partition into B's run still works (Leiden converges from any starting partition; warm-start is an optimization, not a correctness primitive). Document.
- **Concurrency.** Two simultaneous `sdivi snapshot --commit ...` invocations against the same `repo_root` would race on `.sdivi/snapshots/` and `.sdivi/cache/partition.json`. The atomic write rule (Rule 9) protects the snapshot files; the cache write is non-atomic and the last writer wins. This is acceptable for v0 (the snapshot is the durable artifact; the cache is regenerable). Document.
- **`commit_date_iso` timezone.** `git show -s --format=%cI` emits committer-date in the committer's timezone (e.g., `2026-04-30T14:23:01-07:00`). For deterministic snapshot file naming and trend ordering, normalize to UTC at this layer (call `.with_timezone(&Utc)` after parsing, then re-emit). Cross-platform tests must lock this down.
- **`Pipeline::snapshot`'s `timestamp` parameter is now usually overridden.** Existing callers passing a wall-clock timestamp expect it to land in the snapshot. With M16, when `commit` is `Some`, the supplied `timestamp` is silently replaced by `commit_date_iso`. This is intentional — historical snapshots labeled with wall-clock time poison trend ordering — but it's a behavior change. The `Pipeline::snapshot` rustdoc must document this, and the CHANGELOG entry must call it out.
- **CLAUDE.md.** No edit required from this milestone — the line `sdivi snapshot --commit REF works for individual commits` is now true. The user should confirm post-merge that the broader Rule-set and Critical-System-Rules sections don't need an additional line about commit-date timestamp normalization.
- **Doc-comment placement.** Standard caveat from M14/M15.

**Seeds Forward:**

- `--commit REF` is the foundation for any future batch-backfill UX (`sdivi snapshot --commit-range A..B` or `sdivi backfill`). Such a UX iterates `git rev-list A..B` and calls `Pipeline::snapshot` for each — the per-commit cost is dominated by parsing, which is cached implicitly via the partition warm-start across runs. The "What Not to Build Yet" line on batch backfill stays in force for v0; M16 just makes the per-commit primitive solid.
- The `commit_extract` module is the natural home for any future git-touching helpers (e.g., `current_branch`, `is_dirty_working_tree`). Keep it small in v0.
- A v0.x can introduce a `--ephemeral-commit REF` mode (snapshot computed but not persisted) that pairs with M09's `WriteMode::EphemeralForCheck`. Out of scope for M16; the primitive is already there.
- Bare-repository support: with M16's tempdir extraction, `sdivi snapshot --commit REF` could in principle run against a bare repo (no working tree). Not a v0 promise; document as "may work, not tested."

---
