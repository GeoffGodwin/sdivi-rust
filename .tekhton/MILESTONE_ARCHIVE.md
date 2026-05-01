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

---

## Archived: 2026-04-29 — Unknown Initiative

#### Milestone 8: `sdi-core` Pure-Compute Reshape and WASM-readiness
<!-- milestone-meta
id: "8"
status: "done"
-->


**Scope:** Restructure the workspace so `sdi-core` is a pure-compute, WASM-compatible facade exposing concrete `compute_*` functions over plain serde input structs. Move today's `sdi-core` (Pipeline + I/O orchestration) into a new `sdi-pipeline` crate. Extract I/O sites from `sdi-detection::warm_start` and `sdi-snapshot::store`. Feature-gate `sdi-graph`/`sdi-detection`/`sdi-patterns`/`sdi-snapshot` so their `FeatureRecord`-taking paths are opt-in (off for WASM). This is the single largest architectural milestone post-MVP-shift; it precedes the CLI work because the public `sdi-core` API surface freezes at 0.1.0 and must be right *before* CLI/docs/release lock it in.

**Why this milestone exists:** A strict-mode TypeScript consumer app at the user's workplace is the first concrete consumer and needs to import sdi-rust as a WASM library. Its mid-June review deadline is the calendar driver. Today's `sdi-core` transitively pulls `tree-sitter`, `walkdir`, `ignore`, `rayon`, and `std::fs::*` — none WASM-compatible. WASM was originally KD14-deferred ("when a concrete consumer exists"); that condition now holds, and KDD-13 ratifies the v0 inclusion. Rule 18 says public API stability begins at 0.1.0, so the reshape must land before the M13 release tag.

**Deliverables:**

- **New crate `sdi-pipeline`** at `crates/sdi-pipeline/` containing the current `Pipeline::{new, snapshot, delta}` shape. Depends on `sdi-parsing`, `sdi-graph`, `sdi-detection`, `sdi-patterns`, `sdi-snapshot`, `sdi-config`, `sdi-core` (for shared types and pure compute calls). All FS- and clock-touching code lives here.
- **Reshaped `sdi-core`** (same crate name, repurposed): pure-compute facade. Depends only on `sdi-graph`, `sdi-detection`, `sdi-patterns`, `sdi-snapshot`, `sdi-config` — each with `default-features = false` to disable the `pipeline-records` feature. No `sdi-parsing` in the dep tree. No `tree-sitter`, no `walkdir`, no `std::fs::*`, no `std::time::SystemTime`.
- **Public input structs in `sdi-core::input`** (all `#[derive(Serialize, Deserialize, Tsify)]`-ready):
  - `DependencyGraphInput { nodes: Vec<NodeInput>, edges: Vec<EdgeInput> }`
  - `NodeInput { id: String, path: String, language: String }` — `id` is the **canonical NodeId**: repo-relative path with forward slashes, no leading `./`, no trailing slash. Validated by `validate_node_id` (see below).
  - `EdgeInput { source: String, target: String }` — both reference `NodeInput.id`.
  - `PatternInstanceInput { fingerprint: String, category: String, node_id: String, location: Option<PatternLocationInput> }` — `location` is optional so foreign extractors that don't track positions can skip it. When `None`, `build_catalog_from_inputs` produces a `PatternStats` with an empty `locations: Vec<_>`. CLI/sdi-pipeline always populates it.
  - `PatternLocationInput { file: String, start_row: u32, start_col: u32 }` — sibling shape to today's `sdi_patterns::PatternLocation`, but with a string `file` (no `PathBuf`, since `PathBuf` doesn't round-trip cleanly through `tsify`).
  - `LeidenConfigInput { seed: u64, gamma: f64, iterations: usize, quality: QualityFunction }`
  - `BoundarySpecInput` (mirrors `BoundarySpec` shape, no FS path field)
  - `ThresholdsInput` (per-category thresholds + `today: NaiveDate` for expiry checks; see "Override expiry" below)
  - `PriorPartition { cluster_assignments: BTreeMap<String, u32> }` — caller supplies prior-snapshot history for consecutive-snapshot stability scoring (string keys are NodeIds; `detect_boundaries` translates internally to/from numeric indices via `NodeInput.id` order)
  - `NormalizeNode { kind: String, children: Vec<NormalizeNode> }` — input shape for `normalize_and_hash`
- **Public input validators in `sdi-core::input`:**
  - `validate_node_id(s: &str) -> Result<(), AnalysisError>` — non-empty; no leading `./`; no trailing `/`; uses forward slashes (no backslashes); no `..` or absolute path components. Called from every `compute_*` entry point that takes a `DependencyGraphInput` or `PriorPartition`. Foreign extractors that ship malformed paths get a structured error instead of silently mismatching hashes downstream.
- **Public compute functions in `sdi-core`** (all pure, all referentially transparent, all callable from WASM):
  - `compute_coupling_topology(graph: &DependencyGraphInput) -> CouplingTopologyResult`
  - `detect_boundaries(graph: &DependencyGraphInput, cfg: &LeidenConfigInput, prior: &[PriorPartition]) -> BoundaryDetectionResult` — return type includes `cluster_assignments: BTreeMap<String, u32>` (NodeId → cluster), `modularity: f64`, `community_count: u32`, `internal_edge_density: BTreeMap<u32, f64>` (per-community, mirrors today's `LeidenPartition.stability` field — kept under that long name to disambiguate from the new metric), `historical_stability: f64` (the consecutive-snapshot stability score against `prior`, formerly called `stability_score` in the milestone draft), `disconnected_components: u32`. **NodeId translation:** the function indexes nodes by `NodeInput.id` order on entry and translates back to string keys on exit, so internal `LeidenPartition` (numeric `usize` keys) never escapes the function. `PriorPartition.cluster_assignments` is similarly translated by the function before consumption.
  - `compute_boundary_violations(graph: &DependencyGraphInput, spec: &BoundarySpecInput) -> BoundaryViolationResult`
  - `compute_pattern_metrics(patterns: &[PatternInstanceInput]) -> PatternMetricsResult` — fields: `entropy_per_category: BTreeMap<String, f64>`, `total_entropy: f64`, `convention_drift: f64` (defined as the count of distinct fingerprints per category divided by the total instance count per category, averaged across categories — single scalar, [0, 1])
  - `compute_thresholds_check(snapshot: &Snapshot, summary: &DivergenceSummary, cfg: &ThresholdsInput) -> ThresholdCheckResult` — pure form of the exit-10 logic; CLI, CI, and the consumer app all call this. **Override expiry is checked here** (against `cfg.today: NaiveDate`), not at config load (see "Override expiry" deliverable below). For null `summary` fields (first-snapshot path), the corresponding threshold is treated as "not exceeded" (no comparison possible). `ThresholdCheckResult { exit_code: i32, exceeded: Vec<String>, summary: DivergenceSummary, applied_overrides: BTreeMap<String, ThresholdOverride> }` — the CLI's `--format json` for `sdi check` emits this struct directly.
  - `compute_delta(prev: &Snapshot, curr: &Snapshot) -> DivergenceSummary` — re-export from `sdi-snapshot`
  - `assemble_snapshot(metrics, partition, catalog, pattern_metrics, boundary_spec, timestamp, commit) -> Snapshot` — re-export from `sdi-snapshot`. **Renamed from today's `build_snapshot`** (M07); both `sdi-snapshot::build_snapshot` and the `sdi-core` re-export use the new name. The function gains a `pattern_metrics: PatternMetricsResult` argument so `Snapshot` can carry `convention_drift` (see Snapshot shape change below).
  - `compute_trend(snapshots: &[Snapshot], last_n: Option<usize>) -> TrendResult` — re-export from `sdi-snapshot` (new — created in this milestone, see Files list). `last_n = None` means "use all"; `Some(n)` clamps to `min(n, snapshots.len())` (over-large `n` is silently honored as "use all available").
  - `infer_boundaries(partitions: &[PriorPartition], stability_threshold: u32) -> BoundaryInferenceResult` — re-export from `sdi-snapshot` (new — created in this milestone). `partitions` is ordered oldest→newest; the most recent entry is the proposal source, the earlier entries supply the consecutive-snapshot history for stability gating. Caller (CLI or consumer app) is responsible for ordering.
  - `normalize_and_hash(node_kind: &str, children: &[NormalizeNode]) -> String` — canonical fingerprint algorithm. **REPLACES** today's kind-only `sdi_patterns::fingerprint_node_kind`. Algorithm: depth-first canonical walk. For a node with `kind = K` and ordered children `[c1, c2, …]`, the input to `blake3::keyed_hash(FINGERPRINT_KEY, _)` is the byte concatenation of `K`, the byte `0x00`, then for each child the recursive hash bytes (32 bytes each) prefixed by the byte `0x01`. The `0x00`/`0x01` framing prevents a leaf node `K` from colliding with an internal node `K` whose only child happens to start with `K`. Empty `children` → equivalent to today's `fingerprint_node_kind(K)` byte-for-byte (verified by a property test). Returns the 64-char lowercase hex digest. `sdi-patterns::fingerprint::fingerprint_node_kind(kind)` becomes a thin wrapper: `normalize_and_hash(kind, &[])`. The CST walker in `sdi-patterns::catalog::build_catalog` is updated to pass children-derived `NormalizeNode`s when the language adapter populates them; for v0 the adapters still emit kind-only hints, so behavior is unchanged in M07's catalog output (verified by an M07-output-equivalence regression test). The fixed key constant `FINGERPRINT_KEY` in `sdi-patterns::fingerprint` is unchanged and re-exposed via `sdi_core::FINGERPRINT_KEY`.
- **Snapshot shape change (pre-1.0, breaking but uncommitted):** `Snapshot` gains a `pattern_metrics: PatternMetricsResult` field carrying `convention_drift: f64` and `entropy_per_category: BTreeMap<String, f64>`. `DivergenceSummary` gains `convention_drift_delta: Option<f64>`. The `convention_drift_rate` threshold (already in `ThresholdsConfig`) now has something to evaluate against. Update `compute_delta` to populate `convention_drift_delta`. `snapshot_version` stays `"1.0"` (we're pre-release; M07 snapshots are regenerable). M07 `build_snapshot` callers updated for the new argument; CHANGELOG entry calls this out.
- **Override expiry — single source of truth (replaces the dual prune-at-load + check-at-compute path).** Today `sdi-config::thresholds::validate_and_prune_overrides` both validates `expires` format AND removes expired overrides during config load using `SystemTime::now()`. M08 splits these:
  - `validate_overrides_format` (renamed) — validates `expires` is present and well-formed (`YYYY-MM-DD`); errors as `ConfigError::MissingExpiresOnOverride` / `InvalidValue`. **Pure** (no clock). Lives outside the `loader` feature gate.
  - `compute_thresholds_check` — applies expiry against `cfg.today: NaiveDate`. Expired overrides are silently ignored; defaults resume. WASM consumers and CLI use the same logic.
  - `today_iso8601`, `is_expired`, and any remaining `SystemTime::now()` call are deleted from `sdi-config`. The CLI populates `ThresholdsInput::today` from `chrono::Local::now().date_naive()` before calling `compute_thresholds_check`.
  - Net effect: `sdi-config` with `--no-default-features` is fully WASM-clean (no `std::fs::*` AND no `std::time::*`).
- **I/O extraction from compute crates:**
  - `sdi-detection::warm_start.rs` is split: pure `LeidenPartition::{to_json, from_json}` helpers stay on the existing `partition.rs`. The FS calls (`std::fs::read_to_string`, `create_dir_all`, atomic write) move to `sdi-pipeline::cache`. `warm_start.rs` is deleted; `CACHE_FILENAME` moves with the FS calls. `tempfile` is removed from `sdi-detection`'s runtime deps.
  - `sdi-snapshot::store::{write_snapshot, enforce_retention}` → moved to `sdi-pipeline::store`. `assemble_snapshot`, `compute_delta`, `null_summary` stay pure in `sdi-snapshot`. New pure modules `sdi-snapshot::trend` (`compute_trend`) and `sdi-snapshot::boundary_inference` (`infer_boundaries`) are created in this milestone.
- **Cargo feature gating** on `sdi-graph`, `sdi-detection`, `sdi-patterns`, `sdi-snapshot`:
  - Default feature `pipeline-records` enables `sdi-parsing` dep + the `*_from_records` functions taking `&[FeatureRecord]`.
  - When disabled (sdi-core's WASM build), only the `*_from_input` paths taking `&DependencyGraphInput` / `&[PatternInstanceInput]` compile.
  - `sdi-core`'s `Cargo.toml` declares each compute crate with `default-features = false`.
- **`sdi-config` loader feature-gate.** Add a default feature `loader` (default ON) that gates all FS-touching code paths (`Config::load_or_default`, `load_with_paths`, `BoundarySpec::load`, the env-var resolver, every `std::fs::*` call site). With `default-features = false`, only data structs and pure validators (`validate_overrides_format`, `validate_date_format`) compile. `sdi-core` declares `sdi-config = { workspace = true, default-features = false }`. `sdi-pipeline` and `sdi-cli` keep `loader` on. Verify via `cargo tree -p sdi-config --target wasm32-unknown-unknown --no-default-features` showing no `std::fs` / `std::time` consumers and via a `wasm32-unknown-unknown` build of `sdi-config`.
- **Leiden API enhancements (consumer-app requirements 4a–4e):**
  - 4a: `seed: u64` already configurable — verified.
  - 4b: `cluster_assignments: BTreeMap<String, u32>` exposed in `BoundaryDetectionResult`.
  - 4c: `stability_score: f64` computed against `prior: &[PriorPartition]` passed in by caller (no internal history; caller owns the snapshot timeline).
  - 4d: `gamma`, `iterations`, `QualityFunction { Modularity, CPM }` all in `LeidenConfigInput`.
  - 4e: disconnected-graph handling — each connected component becomes its own community-set; explicit fixture test.
- **`sdi-cli` rewired** to call `sdi-pipeline::Pipeline` for the orchestration path. CLI `check` calls `sdi_core::compute_thresholds_check` for the exit-logic. Existing M01–M07 tests pass unchanged.

**Files to create or modify:**

- **New crate:** `crates/sdi-pipeline/{Cargo.toml, src/lib.rs, src/pipeline.rs, src/cache.rs, src/store.rs, src/error.rs}`
- **New:** `crates/sdi-core/src/input.rs` (all input structs + `validate_node_id`)
- **New:** `crates/sdi-core/src/compute/{mod.rs, coupling.rs, boundaries.rs, patterns.rs, thresholds.rs, normalize.rs}`
- **New:** `crates/sdi-core/src/facade.rs` (re-exports of `compute_delta`, `assemble_snapshot`, `compute_trend`, `infer_boundaries` from `sdi-snapshot`)
- **New in `sdi-snapshot`:** `crates/sdi-snapshot/src/trend.rs` (`compute_trend`, `TrendResult`); `crates/sdi-snapshot/src/boundary_inference.rs` (`infer_boundaries`, `BoundaryInferenceResult`). Both pure; both added to `crates/sdi-snapshot/src/lib.rs`'s `pub use` re-export block.
- **Modify:** `crates/sdi-core/{Cargo.toml, src/lib.rs}` — drop `sdi-parsing`; declare `sdi-graph`/`sdi-detection`/`sdi-patterns`/`sdi-snapshot`/`sdi-config` each with `default-features = false`; add `chrono = { workspace = true, default-features = false, features = ["serde"] }`; `pipeline.rs` deleted; existing M07 re-exports replaced by the M08 surface (`compute_*` functions + the `*Input` family + `FINGERPRINT_KEY` + `assemble_snapshot` etc.).
- **Rename:** `sdi-snapshot::build_snapshot` → `sdi-snapshot::assemble_snapshot` (signature gains a `pattern_metrics: PatternMetricsResult` argument). Update every call site in the workspace. CHANGELOG entry notes the rename.
- **Modify Snapshot shape:** add `pattern_metrics: PatternMetricsResult` field to `Snapshot`; `convention_drift_delta: Option<f64>` field to `DivergenceSummary`. Update `compute_delta` to populate the new delta. `snapshot_version` stays `"1.0"` (pre-release).
- **Split + delete:** `crates/sdi-detection/src/warm_start.rs` — `LeidenPartition::{to_json, from_json}` already on `partition.rs` (verified); the FS calls (`std::fs::read_to_string`, `create_dir_all`, atomic tempfile write) and `CACHE_FILENAME` move to `crates/sdi-pipeline/src/cache.rs`. `warm_start.rs` deleted. `tempfile` removed from `crates/sdi-detection/Cargo.toml`'s runtime deps (it's only used by `save_cached_partition`).
- **Move:** `crates/sdi-snapshot/src/store.rs` + `crates/sdi-snapshot/src/retention.rs` → `crates/sdi-pipeline/src/store.rs` (consolidated). `iso_to_filename_safe` (currently in store.rs) moves with it.
- **Modify:** `crates/sdi-graph/Cargo.toml` — add `[features] pipeline-records = ["dep:sdi-parsing"]` and `default = ["pipeline-records"]`. Cfg-gate `build_dependency_graph(records: &[FeatureRecord])` behind `#[cfg(feature = "pipeline-records")]`. Add `build_dependency_graph_from_input(input: &DependencyGraphInput) -> DependencyGraph` (uses `validate_node_id` on every `NodeInput.id` and `EdgeInput.{source,target}` before construction).
- **Modify:** `crates/sdi-detection/Cargo.toml` — same feature pattern. Cfg-gate `LeidenConfig::from_sdi_config` only if needed (no `sdi-parsing` types touched today). Remove `tempfile` runtime dep.
- **Modify:** `crates/sdi-patterns/Cargo.toml` — same feature pattern. Add `crates/sdi-patterns/src/from_inputs.rs` with `build_catalog_from_inputs(patterns: &[PatternInstanceInput], cfg: &PatternsConfig) -> PatternCatalog` (when `PatternInstanceInput.location` is `None`, the resulting `PatternStats.locations` is an empty `Vec<_>`).
- **Modify:** `crates/sdi-patterns/src/fingerprint.rs` and **new:** `crates/sdi-patterns/src/normalize.rs` — implement the new tree-aware `normalize_and_hash(kind, children)` (algorithm spec in the compute-functions section above). `fingerprint_node_kind(kind)` becomes a thin wrapper calling `normalize_and_hash(kind, &[])`. `FINGERPRINT_KEY` is unchanged. The `sdi-core` re-export `sdi_core::FINGERPRINT_KEY` and `sdi_core::normalize_and_hash` are added.
- **Modify:** `crates/sdi-snapshot/Cargo.toml` — same feature pattern. Add `chrono = { workspace = true, default-features = false }` if any snapshot code needs `NaiveDate` (likely yes via `DivergenceSummary` if we attach trend dates).
- **Modify:** `crates/sdi-config/Cargo.toml` — add `[features] loader = []` (default ON). Cfg-gate `Config::load_or_default`, `load_with_paths`, `BoundarySpec::load`, env-var resolver, every `std::fs::*` call site, and `validate_and_prune_overrides`'s pruning step behind `#[cfg(feature = "loader")]`. Keep data structs and pure validators (`validate_overrides_format`, `validate_date_format`) outside the gate. **Delete** `today_iso8601` and `is_expired` from `crates/sdi-config/src/thresholds.rs` — the clock-touching expiry check moves into `sdi_core::compute_thresholds_check`.
- **Modify:** `crates/sdi-cli/Cargo.toml` — depend on `sdi-pipeline` (orchestration) and `sdi-core` (shared types + thresholds-check function); add `chrono = { workspace = true, features = ["clock"] }` (clock is OK in CLI; only sdi-core forbids it).
- **Modify:** `crates/sdi-cli/src/**` — `s/sdi_core::Pipeline/sdi_pipeline::Pipeline/`; route `sdi check` exit logic through `sdi_core::compute_thresholds_check`. CLI populates `ThresholdsInput::today` from `chrono::Local::now().date_naive()` before the call. CLI keeps anyhow→exit-code mapping; the new exit-10 path calls `compute_thresholds_check` and uses the returned `ThresholdCheckResult.exit_code` directly (so `error_exit_code` doesn't need to change).
- **Modify:** `Cargo.toml` (workspace) — add `crates/sdi-pipeline` to `members`; add `sdi-pipeline = { path = "crates/sdi-pipeline" }` to `[workspace.dependencies]`; add `chrono = { version = "0.4", default-features = false, features = ["serde"] }` to `[workspace.dependencies]` (the `clock` feature stays opt-in per consumer).
- **Add:** `crates/sdi-core/tests/wasm_compat.rs` — `#[cfg(target_arch = "wasm32")]` smoke test that imports + calls each `compute_*` function. CI builds with `cargo build -p sdi-core --target wasm32-unknown-unknown --no-default-features`.
- **Add:** `crates/sdi-core/tests/normalize_and_hash.rs` — includes the M07-equivalence regression test (`normalize_and_hash(kind, &[])` matches `fingerprint_node_kind(kind)` byte-for-byte for the M07 fixture set).
- **Add:** `crates/sdi-core/tests/validate_node_id.rs` — exercises the canonicalization rule; structured error on `./foo`, `foo/`, backslashes, `..`, absolute paths.
- **Update:** `CHANGELOG.md` — entries under "Unreleased": the `sdi-core` reshape, consumer-app-driven scope shift, the `build_snapshot` → `assemble_snapshot` rename, the `Snapshot.pattern_metrics` field addition, the `DivergenceSummary.convention_drift_delta` field addition, the `fingerprint_node_kind` → `normalize_and_hash` algorithm extension (M07 fixture catalogs unchanged), and the override-expiry single-source-of-truth move.

**Acceptance criteria:**

- `cargo build -p sdi-core --target wasm32-unknown-unknown --no-default-features` succeeds. `cargo tree -p sdi-core --target wasm32-unknown-unknown --no-default-features` shows zero entries for `tree-sitter*`, `walkdir`, `ignore`, `rayon`, `tempfile`, or any crate transitively pulling `std::time::SystemTime`. Same for `cargo tree -p sdi-config --target wasm32-unknown-unknown --no-default-features` (loader-feature gate exercised, including the deletion of `today_iso8601`).
- `cargo build --workspace` (native targets, default features) succeeds. All M01–M07 tests pass; the only intentional behavior changes are: (a) `Snapshot` carries `pattern_metrics`; (b) `DivergenceSummary` carries `convention_drift_delta`; (c) `build_snapshot` is renamed to `assemble_snapshot`. Tests touching those points are updated in this milestone, not later.
- `compute_*` functions return identical results to today's pipeline when fed equivalent input. Verified by a fixture-based parity test: run `sdi-pipeline::Pipeline::snapshot` on `tests/fixtures/simple-rust/`; extract a `DependencyGraphInput` from the resulting `Snapshot`; feed through each `sdi-core::compute_*`; assert per-dimension equality.
- `normalize_and_hash(kind, &[])` produces the **same** `blake3` digest as today's `fingerprint_node_kind(kind)` for the full M07 fixture set (M07-equivalence regression test). The M07 catalog output for `tests/fixtures/simple-rust/` is byte-identical between M07 and M08 (the algorithm extension is a superset; today's adapters emit no children).
- `validate_node_id` rejects `./foo`, `foo/`, `foo\bar`, `../foo`, `/foo`, the empty string. Accepts `src/lib.rs`, `crates/sdi-core/src/lib.rs`, single-segment names like `Cargo.toml`.
- Override expiry: a `[thresholds.overrides.<cat>]` block with `expires = "2020-01-01"` is **not pruned at config load** (the config retains it as data); `compute_thresholds_check` with `cfg.today = 2026-04-29` ignores it (defaults resume); the same call with `cfg.today = "2019-12-31"` honors it. CLI parity test: `sdi check` produces the same exit code as a programmatic `compute_thresholds_check` call for the same inputs.
- First-snapshot `compute_thresholds_check` with a null `DivergenceSummary` returns `exit_code = 0` and `exceeded = []` (per Critical System Rule 5). Asserted in `compute_thresholds_check.rs` test.
- New tests pass:
  - `crates/sdi-core/tests/wasm_compat.rs`
  - `crates/sdi-core/tests/compute_topology.rs`
  - `crates/sdi-core/tests/compute_pattern_metrics.rs` (entropy + convention_drift)
  - `crates/sdi-core/tests/compute_thresholds_check.rs` (exit-10 logic parity with `sdi check`; first-snapshot null-summary case; expired-override case)
  - `crates/sdi-core/tests/leiden_disconnected.rs`
  - `crates/sdi-core/tests/leiden_historical_stability.rs` (renamed from `leiden_stability_score.rs` to match the field name)
  - `crates/sdi-core/tests/normalize_and_hash.rs` (includes the M07-equivalence regression)
  - `crates/sdi-core/tests/validate_node_id.rs`
  - `crates/sdi-snapshot/tests/compute_trend.rs` (last_n clamping; ordering; empty input)
  - `crates/sdi-snapshot/tests/infer_boundaries.rs` (stability_threshold gating)
  - `crates/sdi-pipeline/tests/pipeline_smoke.rs` (moved from `crates/sdi-core/tests/`)
- `cargo clippy --workspace -- -D warnings` clean. `cargo fmt --check` clean. Doc tests pass.

**Tests:**

- Pure-compute parity: round-trip pipeline output through `compute_*` and assert identical `Snapshot` JSON (modulo the new `pattern_metrics` field, which is populated identically by both paths).
- Historical stability: feed an empty `prior: &[]` (or a single-element list — no prior to compare against) → `historical_stability = 0.0`. Feed a synthetic 3-snapshot history with stable clusters → score approaches 1.0. Feed unstable history → score < 0.3.
- Disconnected graph: 3 components of 4 nodes each, no edges between → exactly 3 communities; `disconnected_components = 3`; modularity defined per-component or explicitly reported as N/A across the whole.
- Override expiry: parametric over `cfg.today`, asserts the same `compute_thresholds_check` call returns different `exit_code` / `applied_overrides` based purely on the supplied date — no clock side effect.
- WASM compile: CI job builds `sdi-core` for `wasm32-unknown-unknown` with `--no-default-features`.
- Feature-flag-flip CI matrix: `cargo test -p <crate> --no-default-features` for each of `sdi-graph`, `sdi-detection`, `sdi-patterns`, `sdi-snapshot`, `sdi-config`. Verifies loader-gated and pipeline-records-gated code paths are properly cfg'd.

**Watch For:**

- The `blake3` fixed-key constant becomes part of `sdi-core`'s public surface (Rule 19). Changing it post-0.1.0 invalidates every snapshot fingerprint. Lock it down here.
- `NodeId` canonicalization rule (`validate_node_id`): repo-relative path; forward slashes; no leading `./`; no trailing `/`; no backslashes; no `..`; not absolute; not empty. Documented on `NodeInput::id` and called from every `compute_*` entry that takes a graph or partition.
- WASM target's lack of `std::time::SystemTime`: every code path calling clock APIs lives in `sdi-pipeline` or `sdi-cli`, not `sdi-core` and not in `sdi-config` with `loader` off. `today_iso8601`/`is_expired` are deleted; expiry comparison happens in `compute_thresholds_check` against `cfg.today`.
- `chrono`'s default features include `clock` (`SystemTime`) and `std`. `sdi-core` uses `chrono = { default-features = false, features = ["serde"] }`; `sdi-cli` adds `features = ["clock"]`.
- Adding `pub` items here is the public API surface. SemVer commitment for `sdi-core` begins at this milestone. Be deliberate.
- `sdi-pipeline` is a *public* crate (CLI and Rust embedders use it for the orchestration path), but it's not the WASM target. Document this distinction in its rustdoc.
- Rayon is not WASM-compatible. Today it's only in `sdi-parsing`; verify it doesn't leak into `sdi-core` via any compute crate. The feature-gate keeps it confined.
- Convention drift's exact formula (count of distinct fingerprints per category / total instances per category, averaged across categories) is recorded in the rustdoc on `compute_pattern_metrics` and asserted by a fixture test. Pre-1.0 we can refine; post-1.0 changes are breaking.
- `assemble_snapshot` rename: every M07 test, doc-test, and example references `build_snapshot` today. Sweep with `rg "build_snapshot"` and update in one commit so the rename lands atomically.

**Seeds Forward:**

- M09 wires `sdi-cli` to call `sdi-pipeline` (largely a search/replace on `sdi_core::Pipeline` → `sdi_pipeline::Pipeline`). `sdi check` uses `sdi_core::compute_thresholds_check` for exit-logic and populates `cfg.today` from `chrono::Local::now().date_naive()`.
- M10 (boundaries CLI) calls the pure `sdi_core::infer_boundaries` for the proposal logic; only the YAML write path is in `sdi-pipeline` / CLI.
- M11 docs cover both the `sdi-pipeline::Pipeline` orchestration API and the pure-compute `sdi-core::compute_*` API. The bifl-tracker validation runs both paths and asserts equivalence (including the new `pattern_metrics.convention_drift` field).
- M12 (WASM crate) wraps `sdi-core` directly with `wasm-bindgen` + `tsify`. The compute functions defined here are the WASM exports. The `BoundaryDetectionResult` field renames (`internal_edge_density`, `historical_stability`) flow straight to the generated `.d.ts`.
- M13 (release) publishes `sdi-pipeline` to crates.io alongside the existing crates and `@geoffgodwin/sdi-wasm` to npm.
- The `normalize_and_hash` algorithm (depth-first, `0x00`/`0x01` framing) is the canonical fingerprint the consumer app's TS extractors will mirror. Empty-children equivalence with `fingerprint_node_kind` keeps M07 catalogs intact.

---

---

## Archived: 2026-04-30 — Unknown Initiative

#### Milestone 9: Trend, Check, Show — Remaining CLI Commands
<!-- milestone-meta
id: "9"
status: "done"
-->


**Scope:** The four remaining CLI commands — `trend`, `check`, `show`, plus the `boundaries` parent (subcommands in Milestone 10). Wire the threshold-exceeded exit-10 contract through the pure `sdi-core::compute_thresholds_check` function delivered in M08. Polish stdout/stderr discipline and JSON output shape.

**Deliverables:**
- `sdi trend [--last N]` aggregating across stored snapshots — calls `sdi_core::compute_trend` (pure) and `sdi_pipeline` for the snapshot-store read. With fewer than 2 snapshots, prints `"sdi trend: not enough snapshots (need ≥2)"` to stderr and exits 0. When `N > stored_count`, silently uses what's available (no error).
- `sdi check` — captures a fresh snapshot, compares it to the most recent stored prior, and routes through `sdi_core::compute_thresholds_check` for the exit decision. Exits `10` if any threshold exceeded, `0` otherwise. **First-run case** (no prior snapshot): `compute_thresholds_check` receives a null `DivergenceSummary` and returns `exit_code = 0` and `exceeded = []` — first-run check is always green, by design (Critical System Rule 5). Today's date is populated into `cfg.today: NaiveDate` by the CLI from `chrono::Local::now().date_naive()` before the call (clock read happens in CLI, not core). **Flag:** `--no-write` skips writing the freshly-captured snapshot to `.sdi/snapshots/` — useful for CI gates that don't want to pollute history. With `--no-write`, retention is also not enforced. Default behavior writes the snapshot (matching `sdi snapshot` semantics).
- `sdi show [<id>] [--format json|text]` inspects a snapshot. `<id>` is optional; with no argument, the **latest snapshot** by lexicographic filename order (= chronological, per the M07 file-naming scheme `snapshot_<YYYYMMDDTHHMMSS>_<hash>.json`) is shown. With `--format json`, output is the raw `Snapshot` JSON on stdout (so `sdi show --format json | jq '.snapshot_version'` returns `"1.0"` per the existing acceptance test).
- `sdi boundaries` parent command with subcommand stubs (`infer`, `ratify`, `show`). Each stub prints `"sdi boundaries <subcmd>: not implemented until M10"` to stderr and exits **0** — keeps CI scripts that survey command help working. M10 fills them in.
- Text formatter using `ratatui` for tables and `owo-colors`/`anstream` for color (auto-detected via `Config::output.color` and `NO_COLOR`)
- JSON formatter producing schema-stable output:
  - `sdi show --format json` → raw `Snapshot` JSON
  - `sdi check --format json` → `ThresholdCheckResult` from `sdi-core` (`{ exit_code, exceeded: [string], summary: DivergenceSummary, applied_overrides: { ... } }`); the CLI process exit code still tracks `result.exit_code`, JSON is informational
  - `sdi trend --format json` → `TrendResult` from `sdi-snapshot`
  - `sdi diff --format json` → `DivergenceSummary` (already in M07's shape)

**Files to create or modify:**
- **New:** `crates/sdi-cli/src/commands/{trend.rs,check.rs,show.rs,boundaries.rs}`
- **Modify:** `crates/sdi-cli/src/main.rs` — register the four new subcommands in the `Commands` enum and dispatch in `main`. `Check` carries `--no-write: bool` and `--format: String`. `Trend` carries `--last: Option<usize>` and `--format: String`. `Show` carries `<id>: Option<String>` and `--format: String`. `Boundaries` carries a nested subcommand enum with `Infer`, `Ratify`, `Show` variants (stubs for M09).
- **Modify:** `crates/sdi-cli/src/output/{json.rs,text.rs}` — extend with formatters for `TrendResult`, `ThresholdCheckResult`, and the standalone `Snapshot` show path. JSON path is a thin `serde_json::to_string_pretty` per result struct; text path uses `ratatui` rendering to a `Vec<u8>` buffer (no alternate-screen / no raw-mode TUI).
- **No-op for `crates/sdi-cli/src/logging.rs`** — already exists from M01 (tracing-subscriber → stderr at `warn` default). Touch only if `--verbose` flag is added to `sdi check`; not needed for the M09 acceptance criteria.
- **Extend:** `crates/sdi-pipeline/src/store.rs` — add `read_snapshots(dir: &Path) -> std::io::Result<Vec<Snapshot>>` returning chronologically-ordered (oldest→newest) snapshots and `read_snapshot_by_id(dir: &Path, id: &str) -> std::io::Result<Snapshot>` for `sdi show <id>`. Add `latest_snapshot(dir: &Path) -> std::io::Result<Option<Snapshot>>` for `sdi check` and `sdi show` with no id.
- **Extend:** `crates/sdi-pipeline/src/pipeline.rs` — `Pipeline::snapshot` gains an internal `WriteMode::{Persist, EphemeralForCheck}` enum so `sdi check --no-write` can capture without touching `.sdi/snapshots/`. Default callers stay on `Persist`.

**Acceptance criteria:**
- `sdi check` exits `0` on a fresh snapshot below thresholds; `10` when any threshold is exceeded. Exit logic is a thin wrapper around `compute_thresholds_check`'s `ThresholdCheckResult.exit_code`.
- **First-run `sdi check` exits `0`** (no prior snapshot → null `DivergenceSummary` → `compute_thresholds_check` returns `exit_code = 0` with `exceeded = []`). Asserted in `crates/sdi-cli/tests/exit_codes.rs`.
- `sdi check --no-write` does not create a file in `.sdi/snapshots/`. Asserted by counting the snapshot directory before/after.
- An expired threshold override is silently ignored — `sdi check` uses defaults after expiry. Expiry comparison happens inside `compute_thresholds_check` against the `today` argument supplied by the CLI; the config retains the override block as data (no load-time pruning).
- `sdi show` with no `<id>` prints the latest snapshot (lexicographically last `snapshot_*.json` in `.sdi/snapshots/`).
- `sdi show --format json | jq '.snapshot_version'` returns `"1.0"` (no stderr contamination on stdout).
- `sdi check --format json | jq '.exit_code'` returns the integer exit code (and the process itself exits with the same value).
- `NO_COLOR=1 sdi show` produces no ANSI escape codes. `--no-color` is equivalent.
- `sdi trend --last 5` aggregates across the 5 most recent snapshots. `--last 9999` against a directory with 3 snapshots silently uses 3 (no error).
- `sdi trend` with 0 or 1 snapshot prints the friendly "not enough snapshots" message to stderr and exits `0`.
- `sdi boundaries infer|ratify|show` each prints "not implemented until M10" to stderr and exits `0`.
- Logs from `tracing` go to stderr regardless of format.

**Tests:**
- `crates/sdi-cli/tests/exit_codes.rs`: full matrix of exit codes 0/1/2/3/10. Includes first-run `sdi check` (exit 0), expired-override `sdi check` (exit 0), threshold-exceeded `sdi check` (exit 10), bad-config `sdi check` (exit 2).
- `crates/sdi-cli/tests/stdout_stderr_split.rs`: redirect each stream to a file; assert JSON validity on stdout for every `--format json` command and zero JSON contamination on stderr.
- `crates/sdi-cli/tests/check_thresholds.rs`: synthetic snapshots driving every threshold variant (pattern_entropy, convention_drift, coupling_delta, boundary_violation); verify CLI exit code matches `compute_thresholds_check` programmatically. Includes a `--no-write` assertion (snapshot dir count unchanged).
- `crates/sdi-cli/tests/show_format.rs`: `sdi show --format json` parses as `Snapshot`; `sdi show` with no id selects latest; `sdi show <id>` selects specifically.
- `crates/sdi-cli/tests/trend_format.rs`: `sdi trend --last N` clamps to available; <2 snapshots → friendly stderr message + exit 0; `--format json` parses as `TrendResult`.
- `crates/sdi-cli/tests/boundaries_stub.rs`: each of `sdi boundaries {infer, ratify, show}` exits 0 and writes to stderr only.
- `crates/sdi-cli/tests/no_color.rs`: `NO_COLOR=1` and `--no-color` both suppress color across show / check / trend.

**Watch For:**
- `sdi check` is the **only** command that may exit `10` — any other command emitting `10` is a bug.
- The text formatter must not block JSON consumers: use `tokio`-free synchronous `ratatui` rendering directly to a `Vec<u8>` buffer, then write to stdout — do **not** initialize a TUI mode (no alternate screen, no raw mode).
- `compute_thresholds_check` is the source of truth for exit logic. Any new threshold/override semantics land in `sdi-core`, not in CLI flags. CLI is presentation only.
- Threshold check consults overrides per-category and checks expiry against `cfg.today`. CLI pulls the date from `chrono::Local::now().date_naive()` and writes it into `ThresholdsInput::today` before calling `compute_thresholds_check`. The pure function never reads the clock.
- `sdi check` writes a snapshot by default; `--no-write` skips the write **and** the retention enforcement. CI gates that don't want history pollution use `--no-write`.
- `sdi show` with no id picks lexicographic-last from `.sdi/snapshots/snapshot_*.json` — relies on the M07 file-naming scheme. If the scheme ever changes, this default-selection logic must change with it.
- `sdi trend` on fewer than 2 snapshots: friendly stderr message + exit 0. `--last N` larger than the stored count is silently clamped (no error).
- The `sdi boundaries` subcommands are M09 stubs that exit 0 — do **not** wire to `sdi_core::infer_boundaries` here; that's M10. Premature wiring couples this milestone to M10's YAML-write path and breaks the parallel-development plan.

**Seeds Forward:**
- The `sdi check` exit-10 path is the public CI gate contract. Any future threshold (e.g., per-category) must continue exiting 10 and route through `compute_thresholds_check`.
- The text formatter shape is reused by `sdi boundaries show` in M10.
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
