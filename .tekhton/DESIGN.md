# Design Document — sdi-rust

## Developer Philosophy & Constraints

sdi-rust inherits the core principles of sdi-py unchanged. They are language-agnostic and have already been ratified through the Python POC; the rewrite is a tech-stack pivot, not a redesign. Every contributor — human or agent — is expected to read this section before opening a PR.

### Non-Negotiable Principles

1. **Measurement over opinion.** Every claim sdi-rust makes about a codebase is backed by a concrete, reproducible measurement derived from AST analysis or dependency graph structure. No heuristics that cannot be explained. No scores without traceable inputs. If a metric cannot be decomposed into its constituent measurements, it does not ship.
2. **Fever chart, not thermometer.** Every metric must be trackable over time. The primary output is always the trend — the rate of change of structural coherence, not the absolute state. Alerts fire on rate-of-change thresholds, not absolute values.
3. **Automated inference, human ratification.** sdi-rust infers structural boundaries via Leiden community detection, proposes them, and waits for a human to ratify. The tool measures divergence from declared intent, not from its own opinions. Pattern categories detect structural shapes and count them but never classify code as "good" or "bad."
4. **Safe defaults, zero mandatory config.** Running `sdi snapshot` on an un-initialized repository produces useful output using purely inferred boundaries. Configuration refines and ratifies — it is never required for first use.
5. **Composable Unix tooling.** Reads from filesystem and git history, writes JSON/text to stdout/files, exits with meaningful codes. No daemon, no server, no interactive TUI. Composes with `jq`, `diff`, CI pipelines, and git hooks.
6. **Language-agnostic core, language-specific adapters.** Dependency graph, community detection, pattern fingerprinting, and snapshot diffing live in `sdi-core`. Per-language specifics live in adapter crates (`sdi-lang-python`, `sdi-lang-typescript`, etc.). Tree-sitter provides consistent AST representation across all supported languages.
7. **Deterministic and reproducible — stronger than sdi-py.** Same commit + same config + same boundaries = same snapshot, bit-identical JSON output across runs on the same platform, byte-stable across platforms modulo documented float edge cases. Rust ownership and explicit RNG seeding upgrade sdi-py's discipline-level determinism to a near-guarantee. `BTreeMap` over `HashMap` wherever output ordering matters. RNG is explicit (`rand` with a chosen algorithm + seed). Floating-point is controllable via `f64` ordering rules; deterministic across platforms with care on FMA.
8. **Memory safety enforcement is structural, not aspirational.** sdi-py rule 15 ("tree-sitter CSTs not held in memory simultaneously") was enforced by convention in Python. In Rust it is enforced by ownership: the parsing crate's API consumes file content and yields a `FeatureRecord`; the CST is dropped before the function returns. Memory usage is proportional to the largest single file, not the total codebase size.

### Banned Anti-Patterns

These carry over from sdi-py unchanged. Any PR that introduces one is rejected on principle, not on judgment.

- **No ML/LLM calls in the analysis pipeline.** sdi-rust is a measurement instrument; determinism and reproducibility are non-negotiable.
- **No network calls during analysis.** No telemetry, no update checks, no remote lookups. A snapshot must be producible on an airgapped machine.
- **No opinions about code quality.** sdi-rust never classifies code as "good" or "bad." Pattern entropy is a measurement, not a judgment. Threshold breaches are reported as "exceeded," not as "violations" or "problems."
- **No automatic alert suppression.** Elevated metrics are never silently accepted. Teams must declare migration intent via per-category threshold overrides with explicit expiry dates. After expiry, default thresholds resume without manual intervention.
- **No interactive TUI or daemon mode.** CLI invocation only. Run, produce output, exit.

### Rust-Specific Contributor Rules

- `unsafe` is forbidden in `sdi-core` and the language adapter crates. If a future hot path requires `unsafe`, it lives in a dedicated crate behind a feature flag with a `SAFETY` comment per block. Bindings crates (PyO3, napi-rs) may use `unsafe` only as required by the binding macro.
- The public API of `sdi-core` is reviewed through a stability lens (see KD12). Adding a `pub` symbol is a deliberate decision; removing one is a breaking change.
- All public items have `#[doc]`. `#![deny(missing_docs)]` is enabled on `sdi-core`.
- `cargo clippy -- -D warnings` is part of CI. No allow-listing without inline justification.
- `cargo fmt --check` is part of CI; default rustfmt config, no project-specific overrides.

## Project Overview

sdi-rust is the Rust reimplementation of the Structural Divergence Indexer (SDI), delivered as a Cargo workspace whose primary product is the `sdi-core` library crate. A thin CLI shell crate (`sdi-cli`) re-exposes the library through the familiar `sdi` command surface (`init`, `snapshot`, `diff`, `trend`, `check`, `show`, `boundaries`, `catalog`). Bindings crates (`sdi-py` via PyO3, `sdi-node` via napi-rs) ship incrementally so non-Rust agent runtimes can embed the analysis pipeline without shelling out.

### What sdi-rust Measures

Carries over from sdi-py: the **Structural Divergence Index**, a composite metric tracking the rate of structural drift in a codebase across four dimensions — pattern entropy, convention drift rate, coupling topology delta, and boundary violation velocity. Periodic snapshots are captured via tree-sitter AST parsing and Leiden community detection, diffed over time, and surfaced as trend data and CI gate checks.

### Audience

Software engineers, tech leads, and engineering managers responsible for the structural health of codebases — particularly teams using AI-assisted development at scale where multiple contributors generate code concurrently without shared structural awareness. New audience added in the sdi-rust era: **tooling authors and gardener-LLM developers** who want to embed the pipeline directly via Rust, Python, or Node bindings rather than shelling out to a binary.

### Relationship to sdi-py

sdi-py (the existing `structural-divergence-indexer` repo) is reframed as the POC. It freezes at the v0.x milestone that completes bifl-tracker validation. Bug-fix-only afterward. No v1 work happens on the Python side. sdi-rust starts at `0.1.0` with its own MAJOR.MILESTONE.PATCH counter — it does **not** inherit sdi-py's v0/v1 era system.

### Why a Rewrite, Not a Refactor

The rewrite is justified by deliverables that are structurally awkward in Python:

- A stable embeddable library API
- Single-binary distribution
- Near-deterministic output across platforms
- Ownership-enforced memory discipline for tree-sitter CSTs

A Python-side rework would not have yielded those properties; a Rust port does, by construction. The five-stage pipeline shape (parsing → graph → detection → patterns → snapshot/delta) and the banned anti-patterns transfer unchanged.

### Distribution Model

Open source. License is an open question (see Open Design Questions); the GitHub repo was initialized with MIT, and the recommendation is **Apache 2.0** for the patent grant given corporate adopters in the gardener-LLM use case.

Distribution shapes:

- **crates.io** for `sdi-core` and `sdi-cli`
- **GitHub Releases** for single-binary builds on Linux/macOS/Windows (the "no Python install" requirement)
- **PyPI** (PyO3 wheel) and **npm** (napi-rs prebuilt) for bindings, post-MVP

### Invocation Frequency

Typically **once per merge** to the primary branch (CI gate), plus ad-hoc human invocations for exploration. Library bindings introduce a new mode: continuous in-process invocation from a long-running agent runtime.

### Scope Discipline

This is a **port with deltas**, not a from-scratch design. The scope document `sdi-rust-scope.md` enumerates exactly what carries over (principles, pipeline, module dependency rules, exit codes, config schema, boundary spec schema) and what changes (tech stack, distribution shape, determinism guarantees, native Leiden, snapshot schema clean break).

## Tech Stack

### Language and Toolchain

- **Language:** Rust.
- **MSRV:** "stable latest minus 2" — generous enough for distros, conservative enough to use modern features. Pinned in `rust-toolchain.toml` and verified in CI. (See Open Design Questions #4.)

### Workspace Layout

Cargo workspace organized by responsibility, with crates depending on each other along an acyclic graph rooted at `sdi-cli`:

```
sdi-rust/
├── Cargo.toml                   # workspace manifest
├── crates/
│   ├── sdi-core/                # pipeline library — public, stable
│   ├── sdi-cli/                 # binary crate, thin shell over sdi-core
│   ├── sdi-parsing/             # tree-sitter integration, FeatureRecord
│   ├── sdi-graph/               # dependency graph (petgraph or custom CSR)
│   ├── sdi-detection/           # Leiden + boundary inference
│   ├── sdi-patterns/            # PatternFingerprint + catalog
│   ├── sdi-snapshot/            # snapshot model, delta, trend
│   ├── sdi-config/              # config TOML, boundary YAML loaders
│   ├── sdi-lang-python/         # Python tree-sitter adapter
│   ├── sdi-lang-typescript/     # TypeScript adapter
│   ├── sdi-lang-javascript/     # JavaScript adapter
│   ├── sdi-lang-go/             # Go adapter
│   ├── sdi-lang-java/           # Java adapter
│   └── sdi-lang-rust/           # Rust adapter
└── bindings/
    ├── sdi-py/                  # PyO3 — post-MVP
    └── sdi-node/                # napi-rs — post-MVP
```

### Module Dependency Rules

These carry the same KD-style enforcement as in sdi-py:

- `sdi-cli` is the composition root; depends on everything.
- `sdi-parsing` depends only on tree-sitter and `sdi-config`.
- `sdi-graph` depends on `sdi-parsing` output and `petgraph`.
- `sdi-detection` depends on `sdi-graph` output.
- `sdi-patterns` depends on `sdi-parsing` output — **NOT** on graph or detection.
- `sdi-snapshot` is the assembly point: depends on graph, detection, patterns.
- `sdi-config` is leaf; depended on by all.
- `sdi-core` re-exports the public pipeline API; no module imports from `sdi-cli`.
- No cycles between crates.

### Key Dependencies

| Concern              | Crate / Approach                                          |
|----------------------|-----------------------------------------------------------|
| AST parsing          | `tree-sitter` (Rust crate, native — no Python bindings)   |
| Graph                | `petgraph` (default); custom CSR if Leiden hot path needs |
| Community detection  | **Native Rust port of Leiden** (KD11) — no FFI to C++     |
| CLI framework        | `clap` v4, derive macros                                  |
| Terminal output      | `ratatui` for tables/progress, `owo-colors` / `anstream`  |
| TOML                 | `toml` crate (read+write)                                 |
| YAML (boundaries)    | `serde_yaml` for read; comment-preserving write deferred  |
| JSON (snapshots)     | `serde_json`                                              |
| RNG                  | `rand` with `StdRng` + explicit seed                      |
| Hashing              | `blake3` (fingerprints), `xxh3` (cache keys)              |
| Error handling       | `thiserror` for libraries, `anyhow` only at CLI boundary  |
| Serialization        | `serde` everywhere                                        |
| Workspace deps       | Pinned in workspace `Cargo.toml`, inherited via `.workspace = true` |

### Tree-Sitter Grammar Distribution

Compile-time linking with feature flags per language. Each grammar is a build dependency gated by a Cargo feature (`lang-python`, `lang-typescript`, …). Default feature set enables the same languages sdi-py supports. (See Open Design Questions #3.)

### Build, Test, Lint

- **Build:** `cargo build --release` produces the `sdi` binary; `cargo build -p sdi-core` for library-only consumers.
- **Test:** `cargo test` (built-in). `proptest` for property tests around determinism and partition stability. `criterion` for benchmarks (run on tags, not in normal CI). Integration tests in each crate's `tests/` directory plus a top-level `tests/` directory for cross-crate scenarios.
- **Lint / format:** `cargo clippy -- -D warnings`, `cargo fmt --check`. Default rustfmt config; no project-specific overrides.

### Documentation

`rustdoc` is canonical, published to docs.rs for crates.io releases. The README points to docs.rs for the `sdi-core` API; a separate `docs/cli-integration.md` covers the CLI surface.

### CI and Release

GitHub Actions, matrix on (Linux, macOS, Windows) × (stable, MSRV). Lint + build + test on push/PR. Release workflow on tags pushes to crates.io and attaches binaries to GitHub Releases.

### Distribution Channels

- **crates.io:** `sdi-core`, `sdi-cli`, and language adapter crates as needed. Crate name reservation is open question #5.
- **GitHub Releases:** prebuilt `sdi` binaries for Linux/macOS/Windows.
- **PyPI / npm:** binding wheels via PyO3 / napi-rs (post-MVP).
- **WASM:** post-MVP per KD14 — only when a real consumer exists.

## Public API Surface

sdi-rust has two public surfaces that must both stay stable: the **`sdi-core` library API** (the canonical interface, per KD12) and the **`sdi` CLI command surface**. The exit-code contract (0/1/2/3/10) is the hardest contract — see Error Handling Strategy.

### Library API — `sdi-core` Crate

The library is a five-stage pipeline. The 80% use case is one call:

```rust
use sdi_core::{Pipeline, Config};

let cfg = Config::load_or_default(repo_root)?;
let snapshot = Pipeline::new(&cfg).snapshot(repo_root)?;
println!("{}", serde_json::to_string_pretty(&snapshot)?);
```

#### Top-Level Types

##### `Config`

Loaded configuration. `Config::load_or_default(path)` walks the precedence order (CLI > env > project > global > defaults) and returns a fully-resolved `Config`. `Config::default()` returns the built-in defaults. All keys are `Copy`/`Clone` where cheap; complex sub-configs (boundaries, threshold overrides) are owned. Error type: `ConfigError`.

##### `Pipeline`

`Pipeline::new(&Config) -> Pipeline` is a cheap constructor. It holds references to config; it does not parse anything yet.

- `Pipeline::snapshot(&self, path: &Path) -> Result<Snapshot, AnalysisError>` runs the full five-stage pipeline against a repository tree.
- `Pipeline::delta(&self, prev: &Snapshot, curr: &Snapshot) -> DivergenceSummary` is a pure function over two snapshots.

##### `Snapshot`

`serde::Serialize + Deserialize`. Carries `snapshot_version` (`"1.0"` for the sdi-rust schema, KD13 clean break from sdi-py). Always read and write through this type; do not hand-edit JSON.

##### `BoundarySpec`

`serde::Deserialize`. Read from `.sdi/boundaries.yaml` with the same schema sdi-py uses (read-compatible). The write path may regress comment preservation; recommended posture for MVP is "accept comment loss on programmatic write" (Open Design Questions #1).

##### Pipeline-Stage Outputs

`PatternCatalog`, `PatternFingerprint`, `FeatureRecord`, `DependencyGraph`, `LeidenPartition`, `DivergenceSummary` — all `Serialize`-able and inspectable for embedders that want to drive their own downstream analysis.

##### `ExitCode`

A closed enum, published from `sdi-core` so library consumers can return the same shell semantics if they choose:

| Variant              | i32 | Meaning                              |
|----------------------|-----|--------------------------------------|
| `Success`            | 0   | Success                              |
| `RuntimeError`       | 1   | I/O or unexpected runtime failure    |
| `ConfigError`        | 2   | Config or environment error          |
| `AnalysisError`      | 3   | Parse, graph, or detection failure   |
| `ThresholdExceeded`  | 10  | Used exclusively by `sdi check`      |

#### Lower-Level Entry Points

Used by `sdi-cli`, also part of the stable surface:

- `sdi_core::parsing::parse_repository(&Config, &Path) -> impl Iterator<Item = FeatureRecord>`
- `sdi_core::graph::build_dependency_graph(impl Iterator<Item = FeatureRecord>) -> DependencyGraph`
- `sdi_core::detection::detect_communities(&DependencyGraph, seed: u64, gamma: f64) -> LeidenPartition`
- `sdi_core::patterns::build_pattern_catalog(impl Iterator<Item = FeatureRecord>, &Config) -> PatternCatalog`
- `sdi_core::snapshot::assemble(&DependencyGraph, &LeidenPartition, &PatternCatalog, &BoundarySpec, &Config) -> Snapshot`
- `sdi_core::snapshot::compute_delta(prev: &Snapshot, curr: &Snapshot) -> DivergenceSummary`

#### Stability Commitment

Every `pub` item in `sdi-core` is part of the SemVer contract from `0.1.0` onward. Adding `pub` is deliberate; removing or renaming `pub` is a breaking change. Internal-only items live in `pub(crate)` modules.

### CLI Command Surface — `sdi-cli` Crate

Same commands as sdi-py (read-compatible config and boundaries — see KD13):

| Command                              | Purpose                                              |
|--------------------------------------|------------------------------------------------------|
| `sdi init`                           | Write default `.sdi/config.toml` and detect languages |
| `sdi snapshot [--commit REF] [--format json\|text]` | Capture and store a snapshot          |
| `sdi diff <prev> <curr>`             | Compute delta between two stored snapshots           |
| `sdi trend [--last N]`               | Trend across stored snapshots                        |
| `sdi check`                          | Exit 10 if any threshold exceeded; 0 otherwise       |
| `sdi show [<id>] [--format json\|text]` | Inspect a snapshot                                |
| `sdi boundaries {infer,ratify,show}` | Manage boundary spec                                 |
| `sdi catalog [--format json\|text]`  | Pattern catalog inspection                           |

CLI flags map 1:1 onto config keys with **CLI > env > config** precedence (see Config Architecture). Exit codes are stable across versions, including pre-1.0.

### Bindings (Post-MVP)

Bindings crates re-expose `sdi-core` through PyO3 (`sdi-py`) and napi-rs (`sdi-node`). The Python and Node APIs mirror the Rust surface as closely as idiomatic in each language: `sdi.Pipeline(cfg).snapshot(path)`. Bindings ship after `sdi-core` is feature-stable (typically m04 or later).

## Core Algorithms & Data Structures

The pipeline is five sequential stages. Shape carries over from sdi-py unchanged; implementation details are Rust-idiomatic.

### Stage 1 — Parsing (`sdi-parsing`)

**Input:** repository root path + `Config` (which controls language detection and exclude globs).

**Algorithm:**

1. Walk filesystem from repo root, applying `.gitignore` plus configured exclude globs. File discovery is breadth-first, stable-sorted by path so output is deterministic.
2. For each file, detect language by extension. Skip if no grammar is registered.
3. Parse with the language adapter's tree-sitter grammar. Walk the CST and extract a `FeatureRecord` (path, imports, exports, function/class/method signatures, pattern instance handles).
4. **Drop the CST before the function returns.** Memory invariant enforced by ownership: the parsing API consumes file content + grammar, returns `FeatureRecord`, and no CST escapes. Memory usage is proportional to the largest single file.

**Parallelism:** `rayon` parallel iterator over discovered files. Each worker has its own grammar instance; thread-safe by construction.

**Output:** `Iterator<Item = FeatureRecord>` (or `Vec<FeatureRecord>` materialized for downstream stages).

**Complexity:** O(N · F) where N is file count and F is average file size in tokens. Benchmarked target: 50K-LOC repo in under 5 seconds on commodity hardware (informational, not a hard contract — see Performance Characteristics).

### Stage 2 — Graph Construction (`sdi-graph`)

**Input:** `Vec<FeatureRecord>`.

**Algorithm:**

1. Build a node per source file (or per top-level module — the adapter decides).
2. For each `FeatureRecord`, resolve imports against known files; emit a directed edge for each resolved import.
3. Edges are unweighted by default (KD4 from sdi-py carries over). Optional weighting by import frequency / symbol count via `weighted_edges = true`.
4. Graph metrics are computed once per snapshot: density, cycle count via DFS, hubs (top-degree centrality), connected component count.

**Data structure:** `petgraph::Graph<NodeId, EdgeWeight>` for the default path. A custom CSR (compressed sparse row) representation may be added for the Leiden hot path if profiling shows the petgraph indirection dominates (Open Design Questions #2 — decide after Leiden port spike).

**Edge cases:**

- **Unresolved imports:** dropped silently (logged at DEBUG).
- **Self-loops:** kept but excluded from cycle count.
- **Disconnected components:** each is partitioned independently in stage 3.

### Stage 3 — Community Detection (`sdi-detection`) — KD11 Hot Zone

**Input:** `DependencyGraph`, seed (u64), gamma (f64), optional warm-start partition from prior snapshot.

**Algorithm:** native Rust port of Leiden (Traag et al. 2019), implementing Modularity and CPM quality functions. Targets ~1500–2500 LOC. **No FFI to the C++ implementation.**

#### Verification Approach

This is the spec, not a footnote.

- **Fixture suite:** graphs of varying sizes (50, 500, 5000 nodes) parsed from real codebases including bifl-tracker.
- **Cross-check:** for each fixture, run both `leidenalg` (via sdi-py or a one-off Python harness) and the Rust port with a fixed seed.
- **Pass criteria** are *partition quality*, not bit-identity:
  - Modularity score within 1% of leidenalg's
  - Community count within ±10%
  - No community larger than 50% of node count for graphs that leidenalg partitions sensibly
  - Stable output across re-runs with the same seed
- **CI integration:** the regression suite ships in the `sdi-rust` repo and runs in CI gated behind a feature flag (because it requires a Python + leidenalg installation).

**Warm start:** seed from `.sdi/cache/partition.json` (sdi-rust schema, distinct from sdi-py's cache — see KD13).

**Cold start:** deterministic seed from `Config::random_seed` (default 42, sdi-py compatibility).

**Output:** `LeidenPartition` (cluster assignments per node + per-cluster stability score for boundary inference downstream).

### Stage 4 — Pattern Fingerprinting (`sdi-patterns`)

**Input:** `Vec<FeatureRecord>`, `Config` (categories, `min_pattern_nodes`, `scope_exclude`).

**Algorithm:**

1. For each `FeatureRecord`, run the per-category tree-sitter queries against the cached pattern instance handles to extract subtree shapes.
2. Hash each shape with `blake3` to a `PatternFingerprint`.
3. Group fingerprints into a `PatternCatalog`, tagging instance counts and file locations per fingerprint per category.
4. Compute pattern entropy per category (distinct-shape count adjusted for instance distribution).

`scope_exclude` excludes files from the catalog only — the same files remain in the dependency graph and partition. Carry-over from sdi-py.

**Determinism:** `BTreeMap`-keyed catalog so iteration order is deterministic; `blake3` hash seeded from a fixed key.

### Stage 5 — Snapshot Assembly + Delta (`sdi-snapshot`)

**Input:** outputs of stages 2–4 plus `BoundarySpec` (if present).

**Snapshot composition:**

- `snapshot_version: "1.0"` (always — KD13)
- Pipeline stage outputs serialized via serde
- Per-dimension scalars: pattern entropy, convention drift (vs prior), coupling delta, boundary violation count
- Intent divergence metrics, only present if `BoundarySpec` is loaded
- Velocity vectors (delta-from-prior). On first snapshot, all delta fields are `null` (rule 14 in sdi-py — carries over)

**Delta computation:** pure function `compute_delta(prev: &Snapshot, curr: &Snapshot) -> DivergenceSummary`. Returns `null` per-dimension when there is no `prev`, **NOT zero**.

**Storage:** JSON files in `.sdi/snapshots/`. Atomic writes (tempfile + rename in the target directory). Retention enforced synchronously after each write.

**Edge cases (carry over from sdi-py):**

- Missing boundary spec → all metrics computed except intent divergence; no warning, this is normal operation.
- Snapshot with incompatible `snapshot_version` → warning + baseline treatment (no delta), never a crash.
- Identical consecutive snapshots → `DivergenceSummary` fields all zero (not null — null means "no prior").

## Configuration & Options

Library consumers configure sdi-rust via the `Config` struct. CLI consumers configure via flags + `.sdi/config.toml`. Both surfaces share the same precedence order and the same default values. The `.sdi/config.toml` schema is **read-compatible with sdi-py** (no migration required for users coming from the Python POC — see KD13 compatibility matrix).

### Precedence (Highest to Lowest)

1. Function arguments / CLI flags (`--format json`, etc.)
2. Environment variables (`SDI_LOG_LEVEL`, `SDI_WORKERS`, `SDI_CONFIG_PATH`, `SDI_SNAPSHOT_DIR`, `NO_COLOR`)
3. Project-local config (`.sdi/config.toml`)
4. Global user config (`$XDG_CONFIG_HOME/sdi/config.toml` or `~/.config/sdi/config.toml`)
5. Built-in defaults (compiled into `sdi-core`)

All keys are optional. Missing keys fall through to defaults. Malformed TOML returns a `ConfigError` (CLI maps to exit code 2).

### Config Sections

Schema is identical to sdi-py with additive `sdi-rust`-only sections.

| Section            | Key                          | Default                          | Notes                                       |
|--------------------|------------------------------|----------------------------------|---------------------------------------------|
| `[core]`           | `languages`                  | `"auto"`                         | Detect from file extensions                  |
| `[core]`           | `exclude`                    | gitignore-style globs (see below)| Replaces — does not merge — when overridden |
| `[core]`           | `random_seed`                | `42`                             | Cold-start RNG seed                          |
| `[snapshots]`      | `dir`                        | `".sdi/snapshots"`               |                                             |
| `[snapshots]`      | `retention`                  | `100`                            | `0` = unlimited                             |
| `[boundaries]`     | `spec_file`                  | `".sdi/boundaries.yaml"`         |                                             |
| `[boundaries]`     | `leiden_gamma`               | `1.0`                            | Manual override only (KD5, no auto-tuning)  |
| `[boundaries]`     | `stability_threshold`        | `3`                              |                                             |
| `[boundaries]`     | `weighted_edges`             | `false`                          | KD4                                         |
| `[patterns]`       | `categories`                 | `"auto"`                         |                                             |
| `[patterns]`       | `min_pattern_nodes`          | `5`                              |                                             |
| `[patterns]`       | `scope_exclude`              | `[]`                             | Excludes from catalog only                   |
| `[thresholds]`     | `pattern_entropy_rate`       | `2.0`                            |                                             |
| `[thresholds]`     | `convention_drift_rate`      | `3.0`                            |                                             |
| `[thresholds]`     | `coupling_delta_rate`        | `0.15`                           |                                             |
| `[thresholds]`     | `boundary_violation_rate`    | `2.0`                            |                                             |
| `[change_coupling]`| `min_frequency`              | `0.6`                            |                                             |
| `[change_coupling]`| `history_depth`              | `500`                            |                                             |
| `[output]`         | `format`                     | `"text"`                         |                                             |
| `[output]`         | `color`                      | `"auto"`                         |                                             |
| `[determinism]`    | `enforce_btree_order`        | `true`                           | sdi-rust-only; reserved for FMA toggles      |
| `[bindings]`       | (reserved)                   | —                                | Placeholder for binding-specific knobs       |

### Per-Category Threshold Overrides

The `expires` field is **mandatory**; missing it is a config error (rule 6 from sdi-py, carries over). After expiry, the override is silently ignored and defaults resume.

```toml
[thresholds.overrides.error_handling]
pattern_entropy_rate = 5.0
expires = "2026-09-30"
reason = "Migrating to ? operator from `match Err(_)` chains"
```

### Library Mutability

`Config` is consumed at `Pipeline::new`. The pipeline does not mutate config during a snapshot run. Consumers wanting per-call overrides build a new `Config` instance — no global mutable state in `sdi-core`.

### Invalid Values

- Out-of-range numerics (negative seed, negative retention) return `ConfigError::InvalidValue { key, message }`. CLI maps to exit 2.
- Unknown keys produce a deprecation warning but do not error (rule 12 from sdi-py: config keys are never repurposed; removed keys are reserved forever).

### sdi-rust-Only Keys (Additive)

sdi-py config files still load unchanged. New sections are reserved and unused at MVP:

- `[determinism]` — `enforce_btree_order` and reserved future toggles around float ordering / FMA control.
- `[bindings]` — placeholder section (e.g. `[bindings.python] convert_paths_to_str = true`). Reserved.

## Error Handling Strategy

sdi-rust uses `Result<T, E>` everywhere in the library. `panic!` is reserved for "this should be impossible" invariant violations; recoverable errors always return.

### Error Type Taxonomy (`sdi-core`)

| Error Type        | When It Fires                                                              | Exit Code |
|-------------------|----------------------------------------------------------------------------|-----------|
| `ConfigError`     | TOML parse, invalid value, missing required field (e.g. threshold override without `expires`) | 2         |
| `AnalysisError`   | Parsing failure, graph construction failure, Leiden divergence, snapshot serialization failure | 3         |
| `IoError`         | File system errors; network-prohibited operations would fall here (none expected) | 1         |
| `ThresholdError`  | Used only by `sdi check` to signal exceedance                              | 10        |

### Crate Strategy

- **`sdi-core`** and supporting library crates use `thiserror` to derive named error variants with structured fields. Errors carry context (file path, line number, key name) so callers can decide how to surface them.
- **`sdi-cli`** is allowed to use `anyhow` at the binary boundary for ergonomic error chaining in `main`. Library crates do not.

### Exit-Code Contract

Rule 8 from sdi-py — carries over unchanged, public API:

| Code | Meaning                                       |
|------|-----------------------------------------------|
| 0    | Success                                       |
| 1    | Runtime error (I/O, unexpected)               |
| 2    | Config / environment error                    |
| 3    | Analysis error (parse, graph, detection)      |
| 10   | Threshold exceeded — exclusively `sdi check`  |

### Non-Error Cases

- **Missing tree-sitter grammar for one detected language:** warning to stderr, skip those files, continue. Only when **all** detected languages lack grammars does `sdi snapshot` exit with code 3 (rule 10 from sdi-py).
- **Missing boundary spec:** normal operation — all metrics except intent divergence are computed, no warning (rules 11 and 12-cli from sdi-py).
- **First snapshot:** has `null` deltas, not zero (rule 14 / 7-system from sdi-py).

### Stderr / Stdout Discipline

Rule 9 from sdi-py: logs, progress bars, and warnings → **stderr**. Snapshot JSON, summaries, and table output → **stdout**. `sdi show --format json | jq '.'` must always work without contamination.

## Type System & Generics

sdi-rust leans on Rust's type system for invariants that sdi-py enforced by convention.

- `FeatureRecord`, `Snapshot`, `BoundarySpec`, `PatternCatalog`, `DependencyGraph`, `LeidenPartition`, `DivergenceSummary` are concrete types, not traits. Public, `serde::Serialize + Deserialize`. Embedders get exact-shape data, not opaque handles.
- Pipeline stage functions are free functions or methods on `Pipeline`; no inheritance hierarchy. Generics are kept narrow — most are bounded by `Iterator<Item = FeatureRecord>` or `AsRef<Path>`.
- `Config` is a concrete struct with `Default`. No generics.
- Language adapters implement a `LanguageAdapter` trait (in `sdi-parsing`) that the parsing stage dispatches over. The adapter trait is the one stable extension point — third-party languages can be implemented out-of-tree in MVP+1, though plugin loading is deferred (KD6 from sdi-py: no custom pattern categories at MVP).
- `BoundaryViolation`, `PatternInstance`, and similar enums are non-exhaustive (`#[non_exhaustive]`) so adding variants is non-breaking.
- `ExitCode` is a closed enum with explicit `i32` discriminants matching the contract.

### Strict Mode

The workspace passes `cargo clippy -- -D warnings` and `cargo fmt --check`. `#![deny(missing_docs)]` is enabled on `sdi-core`. No `#[allow(...)]` attributes on public items without an inline justification.

## Dependencies & Peer Dependencies

### Philosophy

Minimal but pragmatic. We use mature ecosystem crates for parsing (tree-sitter), graphs (petgraph), serialization (serde), and CLI (clap). We do not vendor or re-implement these. We **do** implement Leiden ourselves (KD11) because the community has no mature pure-Rust Leiden crate and the FFI alternative loses our determinism story.

### Runtime Dependencies (Workspace-Pinned)

| Crate                                 | Purpose                                       |
|---------------------------------------|-----------------------------------------------|
| `tree-sitter` + per-language grammars | AST parsing (gated by features)               |
| `petgraph`                            | Default graph representation                  |
| `serde`, `serde_json`, `serde_yaml`, `toml` | Serialization across snapshot, boundary, config |
| `clap`                                | CLI parsing (sdi-cli only)                    |
| `ratatui`, `owo-colors`, `anstream`   | Terminal output (sdi-cli only)                |
| `rayon`                               | Parsing parallelism                           |
| `rand` (`StdRng`)                     | Explicit RNG with fixed algorithm             |
| `blake3`, `xxh3`                      | Pattern fingerprints / cache keys             |
| `thiserror`                           | Library-side errors                           |
| `anyhow`                              | Binary-side error chaining (sdi-cli only)     |
| `walkdir`, `globset`, `ignore`        | File discovery                                |
| `tracing`, `tracing-subscriber`       | Structured logs to stderr                     |

### Dev Dependencies

- `proptest` (property tests for determinism)
- `criterion` (benchmarks, gated)
- `assert_cmd`, `predicates` (CLI integration tests)
- `tempfile` (test fixtures)
- Python interop fixtures for the KD11 verification suite — gated behind a feature flag and `#[ignore]` test attribute so default `cargo test` does not try to run them

### Peer Dependencies (Binding Crates Only)

- `pyo3` for `sdi-py` (post-MVP)
- `napi`, `napi-derive` for `sdi-node` (post-MVP)

### Vulnerability Strategy

`cargo audit` runs in CI on a weekly schedule and on every release. Yanked crates trigger an issue. We update aggressively within MSRV constraints; the workspace `Cargo.toml` is the single source of truth for versions.

### Vendoring

None at MVP. If a critical bug needs a fork patch, we vendor via `[patch.crates-io]` and document the reason in `DRIFT_LOG.md`.

## Compatibility & Platform Support

### Supported Platforms

| Platform                  | Tier   | CI                  |
|---------------------------|--------|---------------------|
| Linux x86_64 + aarch64    | Tier 1 | ubuntu-latest       |
| macOS x86_64 + aarch64    | Tier 1 | macos-latest        |
| Windows x86_64            | Tier 2 | windows-latest (release-only build verification) |

**MSRV:** "stable latest minus 2." Pinned in `rust-toolchain.toml` and verified in CI.

### Embedding Environments

- Native Rust consumers via crates.io (`sdi-core` library)
- Python via PyO3 wheel (`sdi-py` binding, post-MVP)
- Node via napi-rs prebuilt (`sdi-node` binding, post-MVP)
- Browser / WASM: **not MVP** (KD14). Lands when a real consumer exists.

### Compatibility With sdi-py Artifacts (KD13)

| Artifact                | Compat            | Notes                                                                                       |
|-------------------------|-------------------|---------------------------------------------------------------------------------------------|
| `.sdi/config.toml`      | Read-compatible   | sdi-rust accepts sdi-py config files; new keys are additive.                                |
| `.sdi/boundaries.yaml`  | Read-compatible   | Schema unchanged. Comment preservation on programmatic write may regress (open question #1).|
| `.sdi/snapshots/*.json` | **Clean break**   | sdi-rust does not read sdi-py snapshots. New schema version (1.0). Trend continuity for migrators is lost — acceptable. |
| `.sdi/cache/*`          | **Clean break**   | Internal; no compat concern.                                                                |
| Exit codes              | **Identical**     | Public API contract.                                                                        |
| CLI flag surface        | **Compatible**    | Same commands, same primary flags. New flags additive.                                      |

### Determinism Across Platforms

Bit-identical JSON output across runs on the same platform. Byte-stable across platforms modulo documented float edge cases (FMA on/off can cause last-bit differences in entropy scores — documented and bounded; aggregate metrics still equal).

## Bundle Size & Tree-Shaking

Not applicable in the JavaScript bundling sense — sdi-rust ships as native crates and prebuilt binaries, not as a browser bundle. The analogous Rust concerns and how we handle them:

- **Binary size:** tracked manually per release in `CHANGELOG.md`. Symbols are stripped on release builds. LTO is enabled for the `sdi` binary. No hard budget at MVP.
- **Optional features:** each language adapter is gated by a Cargo feature (`lang-python`, `lang-typescript`, …). Consumers who want a smaller binary can build with only the languages they need.
- **Compile-time grammar inclusion:** tree-sitter grammars are linked at compile time (Open Design Questions #3 — recommend compile-time for MVP). This increases binary size for unused languages; the feature-flag approach gives consumers a knob.
- **Bindings size (post-MVP):** the PyO3 wheel and napi-rs prebuilt are sized per platform. We will revisit if a binding's size becomes a complaint.

Tree-shaking has no Rust analogue — `cargo` already drops unused code by default, and `#[cfg(feature = "...")]` gates handle conditional compilation.

## Performance Characteristics

### Targets

Inherited from sdi-py and tightened where Rust gives headroom:

| Scenario                         | Target                                       | Notes                                |
|----------------------------------|----------------------------------------------|--------------------------------------|
| 50K-LOC repo, cold cache         | Sub-5-second wall-clock                      | Same as sdi-py; Rust expected to comfortably beat it |
| 500K-LOC repo                    | Sub-30-second wall-clock                     | sdi-py struggles here; Rust ownership makes it tractable |
| Memory ceiling                   | Proportional to largest single source file (parsing CST) plus dependency graph footprint | NOT proportional to total codebase size; enforced by ownership in `sdi-parsing` |

### Hot Paths

- **Stage 1 (parsing):** parallelized via `rayon`, with a per-worker grammar instance. Watch tree-sitter grammar contention; if it shows up under profiling, adopt one-grammar-per-language-per-worker pooling.
- **Stage 3 (Leiden):** the KD11 hot path. Targets ~1500–2500 LOC for the port. Profile early; if `petgraph`'s adjacency representation becomes the bottleneck, build a CSR view for detection (Open Design Questions #2).

### Concurrency

Parsing is parallel; graph construction and detection are single-threaded (matches sdi-py). The pipeline as a whole is `Send + Sync` so embedders can run multiple snapshots concurrently against different repos.

### Trade-Offs

- We pick **determinism over raw throughput** where they conflict — `BTreeMap` over `HashMap`, ordered iteration, explicit RNG seed.
- We pick **library-shape over CLI-shape** where they conflict — the `sdi-cli` crate cannot add code paths that aren't reachable through `sdi-core`.

### Benchmarks

`criterion` benches gated by a feature flag, run on release tags only (matches sdi-py's CI-discipline benchmark approach). Track regressions per release in `CHANGELOG.md`. No automated alerting on benchmark drift at MVP.

## Versioning & Release Strategy

sdi-rust uses **MAJOR.MILESTONE.PATCH** versioning, parallel to sdi-py's scheme but with its own counter — the rewrite is a fresh start.

### Component Definitions

- **MAJOR** = design era. `0` while pre-MVP / iterating; `1` when the first ratified post-MVP `DESIGN_v1.md` ships. Each design lives at `.tekhton/DESIGN.md` (era v0) or `.tekhton/DESIGN_v<N>.md` (era v1+).
- **MILESTONE** = position within the current MAJOR. Increments on each Tekhton milestone. Resets to 0 at every MAJOR bump.
- **PATCH** = bugfix / drift fix between milestones. Resets to 0 at each new MILESTONE.

### Era Boundaries

| Era | Versions      | Status                                                  |
|-----|---------------|---------------------------------------------------------|
| v0  | 0.1.0–0.x.y   | Active (this initiative)                                |
| v1  | 1.0.0+        | Future — opens with first ratified `DESIGN_v1.md`       |

sdi-rust does **not** inherit sdi-py's era counters or milestone numbers. v0/m01 in sdi-rust is a fresh start.

### Crates.io Publishing

Each Tekhton milestone that ships user-visible surface area corresponds to a tag and a `cargo publish` of the affected crates. Internal-only milestones bump PATCH and may not publish if no crate's public API changed.

### SemVer Posture

- **Pre-1.0:** any 0.x → 0.(x+1) bump may include breaking changes; we document them in `CHANGELOG.md` and `MIGRATION_NOTES.md` if non-trivial. 0.x.(y+1) is reserved for non-breaking patches.
- **Post-1.0:** strict. `sdi-core` public surface stability per KD12; breaking changes require a major bump and a `MIGRATION_NOTES.md` entry.

### Deprecation Policy

Deprecated symbols are marked `#[deprecated(since, note)]` and stay for at least one minor cycle before removal.

### Changelog

Hand-maintained `CHANGELOG.md` with conventional sections (Added / Changed / Deprecated / Removed / Fixed / Security). Tekhton milestones write entries; we do not auto-generate from commits.

### Release Process

Tag-driven. `git tag vX.Y.Z` triggers the release workflow which runs the matrix build, publishes affected crates, and attaches binaries to the GitHub Release. **Manual approval gate before crates.io push** — no auto-publish.

## Documentation Strategy

Three documentation surfaces, each with a distinct purpose.

### 1. rustdoc / docs.rs

The canonical API reference for `sdi-core` and every public crate. `#![deny(missing_docs)]` is enforced. Every public item has a doc comment with at least one `# Examples` block. Doc tests run in CI (`cargo test --doc`); broken examples fail the build. This applies to `sdi-core` from m01 onward.

### 2. `README.md` (Root)

Quick start, installation paths (cargo, brew, binary releases), one-paragraph "what is SDI", links to docs.rs and the CLI guide. Stays under ~200 lines.

### 3. `docs/` Directory

Long-form guides:

- **`docs/cli-integration.md`** — manual CI integration recipe (`cargo install sdi && sdi check`), GitHub Actions snippet, exit-code reference. A polished GHA reusable action is post-MVP.
- **`docs/library-embedding.md`** — embedding `sdi-core` in a Rust agent runtime; minimal viable consumer; common pitfalls (e.g., do not hold onto `Snapshot` JSON across pipeline runs without re-parsing config).
- **`docs/migrating-from-sdi-py.md`** — what carries (`config.toml`, `boundaries.yaml`, exit codes, CLI commands), what changes (snapshot schema clean break, command flags that gained `--rust-only` behaviors), explicit non-goals (we don't import sdi-py snapshots).
- **`docs/determinism.md`** — explains the `BTreeMap` discipline, the seed contract, the float / FMA notes. Intended as both user reference and contributor onboarding.

### Examples

`examples/` directory in the workspace (not a published crate) with runnable consumer snippets:

- `examples/embed_pipeline.rs` — minimal embedder
- `examples/custom_config.rs` — programmatic config building
- `examples/binding_python.py` — usage of the PyO3 binding (post-MVP)

### Hosting

docs.rs auto-builds on crate publish. `docs/` lives in-repo and is served as-is on GitHub. We do **not** stand up a marketing site at MVP.

### Doc-Test Discipline

Every code block in rustdoc that uses `///` and isn't annotated `no_run` or `ignore` is a doc test. CI fails on broken doc tests.

## Config Architecture

Config architecture mirrors sdi-py exactly (KD13 read-compatibility): same precedence, same TOML schema, same defaults. Plus additive `sdi-rust`-only sections (`[determinism]`, `[bindings]`).

### Precedence (Highest to Lowest)

1. Function arguments (library) / CLI flags (binary)
2. Environment variables (`SDI_LOG_LEVEL`, `SDI_WORKERS`, `SDI_CONFIG_PATH`, `SDI_SNAPSHOT_DIR`, `NO_COLOR`)
3. Project-local `.sdi/config.toml`
4. Global `$XDG_CONFIG_HOME/sdi/config.toml` (fallback `~/.config/sdi/config.toml`)
5. Built-in defaults (compiled into `sdi-core`)

### Complete Default Configuration

```toml
[core]
languages = "auto"
exclude = [
  "**/vendor/**",
  "**/node_modules/**",
  "**/__pycache__/**",
  "**/dist/**",
  "**/build/**",
  "**/target/**",
  "**/.git/**",
]
random_seed = 42

[snapshots]
dir = ".sdi/snapshots"
retention = 100

[boundaries]
spec_file = ".sdi/boundaries.yaml"
leiden_gamma = 1.0
stability_threshold = 3
weighted_edges = false

[patterns]
categories = "auto"
min_pattern_nodes = 5
scope_exclude = []

[thresholds]
pattern_entropy_rate = 2.0
convention_drift_rate = 3.0
coupling_delta_rate = 0.15
boundary_violation_rate = 2.0

[change_coupling]
min_frequency = 0.6
history_depth = 500

[output]
format = "text"
color = "auto"

[determinism]
enforce_btree_order = true

[bindings]
# Reserved for future binding-specific knobs.
```

### Per-Category Threshold Overrides

Mandatory `expires` field. Same shape as sdi-py:

```toml
[thresholds.overrides.error_handling]
pattern_entropy_rate = 5.0
expires = "2026-09-30"
reason = "Migrating to ? operator from `match Err(_)` chains"
```

Missing `expires` returns `ConfigError` (CLI exit 2). After expiry, the override is silently ignored and defaults resume.

### Merge Strategy

Section-by-section override (later wins). Within a section, key-by-key override. Lists in `core.exclude` and `patterns.scope_exclude` are **replaced**, not merged — same semantics as sdi-py. Threshold override sub-tables are merged per-category (each `[thresholds.overrides.<cat>]` is added; existing categories are replaced wholesale).

### Runtime Mutability

`Config` is consumed at `Pipeline::new`. The pipeline does not mutate config during a run. To run two snapshots with different configs, build two `Pipeline` instances. There is no global mutable config in `sdi-core`.

### CLI ↔ Config Mapping

Flags map 1:1 to config keys:

| Flag             | Config Key / Effect                       |
|------------------|-------------------------------------------|
| `--format json`  | `output.format = "json"`                  |
| `--no-color`     | `output.color = "never"` (also `NO_COLOR=1` env) |
| `--workers N`    | Effective parallelism (no config key; CLI/env only) |
| `--seed N`       | `core.random_seed = N`                    |

## Testing Strategy

Three test tiers, plus a dedicated KD11 verification suite.

### Unit + Integration Tests (`cargo test`)

Default `cargo test` runs:

- **Per-crate unit tests** (`#[cfg(test)] mod tests` or `tests/` files in each crate). Coverage targets:
  - 80%+ for `sdi-core`, `sdi-parsing`, `sdi-graph`, `sdi-detection`, `sdi-patterns`, `sdi-snapshot`
  - 60%+ for `sdi-cli`, with the rest covered by integration tests
- **Cross-crate integration tests** in the workspace-level `tests/` directory. Real tree-sitter parsing, real `petgraph` graphs, real fixture repos, real filesystem temp dirs.
- **Doc tests** (`cargo test --doc`). Every public function with a `# Examples` block has a runnable doc test.

#### Fixture Repos

Parallel to sdi-py's `tests/fixtures/`:

- `tests/fixtures/simple-rust/` — 5–10 file Rust crate with known imports
- `tests/fixtures/simple-python/` — known structure, used for cross-language and verification against sdi-py outputs
- `tests/fixtures/multi-language/` — Python + TypeScript adapter exercise
- `tests/fixtures/high-entropy/` — deliberate pattern variance
- `tests/fixtures/evolving/` — git repo with progressive drift, built by a setup script run before the test, stored under `target/test-fixtures`

### Property Tests (`proptest`)

Determinism is the primary property:

- `prop_test_pipeline_deterministic`: same input → bit-identical Snapshot JSON across N runs
- `prop_test_delta_pure`: `compute_delta(a, b)` is referentially transparent
- `prop_test_leiden_seeded`: native Leiden with the same seed produces the same partition

### KD11 Verification Suite

Native Leiden vs `leidenalg`. Gated behind feature `verify-leiden` which requires Python + leidenalg installed. Pass criteria are partition quality (modularity within 1%, community count within ±10%), **NOT bit-identity**. CI runs this on a dedicated job.

### CLI Integration Tests

`assert_cmd` + `predicates` to assert exit codes, stdout/stderr discipline, and atomic-write behavior. The exit-code contract is exhaustively tested because it is a public API.

### Benchmarks

`criterion` benches behind feature `bench`. Run on release tags only; tracked in `CHANGELOG.md`.

### What We Do NOT Test

- **Real network:** no network access from any test (rule 1 from sdi-py).
- **Cross-version migration of sdi-py snapshot JSON:** KD13 is a clean break; we do not import sdi-py snapshots.
- **Bit-identity of snapshot output across platforms:** we test modularity / aggregate-equality across platforms, not last-bit equality of floats.

## Naming Conventions

Rust standard naming, no project-specific overrides.

### Code Naming

| Concept                              | Convention            | Example                                        |
|--------------------------------------|-----------------------|------------------------------------------------|
| Crates                               | `kebab-case`, prefix `sdi-` | `sdi-core`, `sdi-cli`, `sdi-lang-python`  |
| Modules                              | `snake_case`          | `parsing`, `snapshot`                          |
| Types, traits                        | `PascalCase`          | `PatternFingerprint`, `BoundarySpec`, `LanguageAdapter` |
| Functions, methods, fields, variables| `snake_case`          | `parse_repository`, `random_seed`              |
| Constants                            | `SCREAMING_SNAKE_CASE`| `DEFAULT_GAMMA`, `MAX_RETENTION`, `EXIT_THRESHOLD_EXCEEDED` |
| Generics                             | Single-letter when unambiguous; descriptive when not | `T`, `E`, `Lang`, `Adapter` |
| Lifetime parameters                  | `'a`, `'src`, descriptive when more than one is in scope | `'src`, `'cfg`     |

### File Structure

- One concern per file; group related types and functions in the same module file.
- File ceiling guideline: **500 lines**. Above that, split by sub-concern. (sdi-py guideline carries over.)
- Tests co-located via `#[cfg(test)] mod tests` for unit tests; integration tests in `tests/<crate-or-feature>.rs`.

### Domain Terms

Carry over from sdi-py for cross-tool consistency:

| Term                  | Meaning                                                              |
|-----------------------|----------------------------------------------------------------------|
| Snapshot              | A single point-in-time capture                                       |
| Delta                 | A `prev`/`curr` snapshot pair's computed difference                  |
| Trend                 | Aggregation across multiple snapshots over time                      |
| Boundary              | An inferred or ratified module grouping                              |
| Ratification          | Human acceptance of an inferred boundary                             |
| Pattern fingerprint   | A structural hash of a tree-sitter subtree                           |
| Pattern category      | A named group of patterns (e.g. `error_handling`, `async_patterns`)  |
| Pattern entropy       | Distinct shape count adjusted for instance distribution              |
| Drift                 | Rate-of-change of a metric across snapshots                          |
| Velocity              | Per-pattern instance-count delta across snapshots                    |
| Threshold override    | Per-category, time-boxed exemption from a default threshold rate     |

### Imports / `use` Ordering

rustfmt default. We do not enforce a stricter grouping.

## Open Design Questions

The questions named in `sdi-rust-scope.md` plus a couple of additions surfaced while drafting this DESIGN.

1. **YAML library choice.** `serde_yaml` is the obvious read path. Comment-preserving write is the open question — sdi-py uses `ruamel.yaml` for this. Options: (a) accept comment loss on programmatic write, document it; (b) use a comment-preserving Rust YAML crate (verify maturity); (c) hand-write a minimal YAML emitter that preserves the boundary spec's specific comment patterns. **Recommendation:** (a) for MVP, revisit if users complain. Decide before m01.
2. **Graph library.** `petgraph` is the default. If Leiden's hot path needs a more cache-friendly representation (CSR), we may roll our own minimal graph type for the detection stage. **Decide after the Leiden port spike.**
3. **Tree-sitter grammar distribution.** Compile-time linking (each grammar a build dependency) vs. runtime dynamic loading. Compile-time is simpler and matches Rust ecosystem norms; runtime gives smaller binaries when not all grammars are needed. **Recommend compile-time for MVP with feature flags per language.**
4. **MSRV.** Rust minimum supported version. **Recommend "stable latest minus 2"** — generous enough for distros, conservative enough to use modern features. Decide before m01.
5. **Crate name on crates.io.** Verify `sdi`, `sdi-core`, `sdi-cli` are available before committing. **Reserve early** — pre-work item before m01.
6. **License.** sdi-py says MIT or Apache 2.0. The GitHub-side initial commit used MIT. **Recommend Apache 2.0** for the patent grant; the gardener-LLM use case has corporate adopters in mind. Decide and update LICENSE before the first crates.io publish.
7. **Bindings publish cadence.** PyO3 and napi-rs bindings are post-MVP. Open: do they live in this repo (`bindings/` directory in the workspace) or in their own repos? Recommendation is in-repo until they have non-trivial consumer-side surface area; split out only if cross-repo CI complexity outweighs the workspace benefit.
8. **Snapshot file naming.** sdi-py uses `snapshot_<timestamp>_<sha>.json`. Carry forward unchanged unless we hit a real-world filename-length issue on one of the supported platforms.
9. **`BTreeMap` vs `IndexMap` for catalogs.** `BTreeMap` orders by key (deterministic); `IndexMap` preserves insertion order (also deterministic given deterministic insertion). `BTreeMap` is the safer default. Revisit only if profiling shows the comparison cost on a hot path.
10. **Determinism across FMA.** Float fused-multiply-add can produce different last-bit results across CPU microarchitectures. We document this and assert aggregate equality, not bit equality, in cross-platform tests. If a real adopter needs bit-identical output across platforms, we revisit via a build flag that disables FMA in entropy code paths.

## What Not to Build Yet

### Items Moved Into MVP Scope (vs sdi-py's Defer List)

These are the reason for the rewrite and are in-scope for sdi-rust v0:

- **Embeddable library API.** Was deferred in sdi-py (CLI-only); now MVP scope as `sdi-core`.
- **Standalone binary distribution.** Was deferred (PyInstaller complexity); trivially in-scope here (`cargo build --release`).
- **Native Leiden.** Was deferred / reliant on `leidenalg`; now MVP via the KD11 native port.

### Items Still Deferred

- **GitHub Actions reusable action.** Easier with a single binary, but still post-MVP polish. Document manual `cargo install sdi && sdi check` for m01–m03; revisit after a stable schema.
- **WASM bindings.** KD14: not MVP. Lands when a concrete consumer exists.
- **IDE/editor plugin.** Requires stable API and snapshot schema. Post-1.0.
- **SaaS dashboard / web UI.** sdi-rust is a measurement instrument, not a platform. Output is JSON; existing dashboards (Grafana, Datadog) consume it.
- **Auto-remediation / gardener agent.** sdi-rust detects and measures drift; it never fixes it. A companion tool generating consolidation PRs is a separate project.
- **Plugin system for custom analyzers.** Built-in pattern categories only at MVP (KD6 from sdi-py carries over). Extensibility design after real user feedback.
- **Cross-language dependency inference.** v0 tracks only explicit in-language imports. Modeling cross-language coupling (TypeScript → Python via API) requires API contract parsing — out of scope.
- **Historical backfill UX.** `sdi snapshot --commit REF` works for individual commits. Batch backfill across hundreds of commits (parallelism, progress, storage) is not designed; users script it with a bash loop.
- **Real-time / watch mode.** No file watcher daemon. CLI invocation on merge events is the intended cadence. Watch mode violates the Unix philosophy constraint.
- **Automatic drift-vs-evolution classification.** Explicitly rejected (KD1 from sdi-py). Humans declare migration intent via threshold overrides.
- **Stdin input for `sdi diff`.** Carries forward as deferred from sdi-py.
- **`sdi config` subcommand.** Edit `.sdi/config.toml` directly. Same deferral as sdi-py.
