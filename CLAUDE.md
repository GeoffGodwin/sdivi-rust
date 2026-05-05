# sdivi-rust

## Project Identity

sdivi-rust is the Rust reimplementation of the Structural Divergence Indexer (SDIVI), a measurement instrument that tracks the rate of structural drift in a codebase. It is delivered as a Cargo workspace with a two-layer library shape: `sdivi-core` (the pure-compute facade — no I/O, no clock, no tree-sitter, WASM-compatible; concrete `compute_*` functions over plain serde input structs) and `sdivi-pipeline` (the orchestration crate that owns FS reads, snapshot writes, partition cache, and the five-stage `Pipeline::snapshot(&Path)` API). A thin CLI shell (`sdivi-cli`) re-exposes the `sdivi` command surface (`init`, `snapshot`, `diff`, `trend`, `check`, `show`, `boundaries`, `catalog`). The audience is software engineers, tech leads, engineering managers responsible for structural health of codebases — and now tooling authors, gardener-LLM developers, and TS/JS tools (e.g. the consumer app) that want to embed the analysis pipeline directly via Rust or WASM bindings rather than shelling out to a binary.

**Languages:**
- Rust

**Frameworks and key dependencies:** `tree-sitter` (AST parsing, per-language grammars gated by Cargo features), `petgraph` (graph representation; KDD-5 — no CSR view), `clap` v4 with derive (CLI), `ratatui` + `owo-colors` + `anstream` (terminal output), `rayon` (parsing parallelism), `serde` + `serde_json` + `serde_yml` + `toml` (serialization), `rand` with `StdRng` (seeded RNG), `blake3` (pattern fingerprints) and `xxh3` (cache keys), `thiserror` (library errors) plus `anyhow` (sdivi-cli only), `walkdir` + `globset` + `ignore` (file discovery), `tracing` + `tracing-subscriber` (stderr structured logs), `chrono` with `default-features = false` in sdivi-core to avoid the clock feature. M12 adds `wasm-bindgen` + `tsify` + `serde-wasm-bindgen` for `bindings/sdivi-wasm`. PyO3 (`sdi-py`) and napi-rs (`sdivi-node`) remain post-MVP.

**Target platforms and deployment model:**
- Tier 1: Linux x86_64 + aarch64 (CI on `ubuntu-latest`); macOS x86_64 + aarch64 (CI on `macos-latest`)
- Tier 2: Windows x86_64 (CI on `windows-latest`, release-only build verification)
- MSRV: "stable latest minus 2", pinned in `rust-toolchain.toml`, verified in CI
- Distribution: crates.io (`sdivi-core`, `sdivi-pipeline`, `sdivi-cli`, language adapters), GitHub Releases (prebuilt `sdivi` binaries Linux/macOS/Windows), npm (`@geoffgodwin/sdivi-wasm` — wasm-bindgen + tsify-derived `.d.ts`, ships in v0 per KDD-13). PyPI (PyO3 wheel) and napi-rs Node prebuilt remain post-MVP / v1 era.
- Invocation: typically once per merge to the primary branch (CI gate) plus ad-hoc human invocations; bindings introduce continuous in-process invocation from long-running agent runtimes. No daemon, no server, no interactive TUI.

**License:** **Apache 2.0** (ratified 2026-04-28). Permissive open source with explicit attribution preservation, contributor patent grant, and broad enterprise acceptance. `LICENSE` and `NOTICE` are at the repo root; every published crate sets `license = "Apache-2.0"` in its `Cargo.toml`.

## Architecture Philosophy

sdivi-rust inherits sdi-py's principles unchanged — they are language-agnostic and have already been ratified through the Python POC. Rust upgrades several of them from convention to compiler-enforced invariants.

### Concrete Patterns This Project Follows

- **Five-stage sequential pipeline.** `parsing → graph → detection → patterns → snapshot/delta`. No stage reaches backward; downstream stages consume the previous stage's output as data. The pipeline is owned by `sdivi-pipeline`; the per-stage compute lives in `sdivi-core` and the compute crates.
- **Two-layer library shape (KDD-12).** `sdivi-core` is the pure-compute facade exposing `compute_*` functions over plain serde `*Input` structs. `sdivi-pipeline` is the orchestration crate adding FS, clock, and atomic-write I/O. CLI and FS-based Rust embedders use `sdivi-pipeline`; bindings and embedders with their own extractors (e.g. the consumer app) use `sdivi-core` directly.
- **Composition root in `sdivi-cli`.** All wiring of config, pipeline, and presentation lives in `sdivi-cli`. Library crates never reach for stdout, env vars, or FS state beyond what's passed to them.
- **Concrete types over traits.** `FeatureRecord`, `Snapshot`, `BoundarySpec`, `PatternCatalog`, `DependencyGraph`, `LeidenPartition`, `DivergenceSummary`, and the `*Input` family in `sdivi-core::input` are concrete `serde` structs. The only trait extension point is `LanguageAdapter`.
- **Ownership-enforced memory discipline.** Tree-sitter CSTs are dropped per file inside the parsing API. Memory stays proportional to the largest single source file, not the codebase total.
- **Determinism by construction.** `BTreeMap` over `HashMap` everywhere output ordering matters. `StdRng` seeded from `Config::random_seed`. Pattern fingerprints via `blake3` keyed hash. `normalize_and_hash` is exposed in `sdivi-core` so foreign extractors produce the same canonical hashes as the Rust pipeline.
- **Pure functions for derived data.** `compute_delta`, `compute_thresholds_check`, `compute_pattern_metrics`, `detect_boundaries`, `infer_boundaries`, `compute_coupling_topology`, `compute_boundary_violations`, `compute_trend` are all referentially transparent. They live in `sdivi-core` and are callable from CLI, Rust embedders, and WASM.
- **Snapshot schema clean break (KDD-1).** `snapshot_version: "1.0"`. sdivi-rust does not read sdi-py snapshots.
- **Native Leiden, no FFI (KDD-2).** Native Rust port; verified against `leidenalg` on partition quality, not bit identity. Cluster assignments exposed as `BTreeMap<NodeId, ClusterId>`; consecutive-snapshot stability score computed against caller-supplied prior partitions.

### Anti-Patterns This Project Avoids

- **No ML/LLM calls in the analysis pipeline.** sdivi-rust is a measurement instrument; determinism is non-negotiable.
- **No network calls during analysis.** No telemetry, no update checks, no remote lookups. Snapshots must be producible on an airgapped machine.
- **No opinions about code quality.** Pattern entropy is a measurement, not a judgment. Threshold breaches are reported as "exceeded," never as "violations" or "problems."
- **No automatic alert suppression.** Teams declare migration intent via per-category threshold overrides with explicit `expires` dates. After expiry, defaults resume without manual intervention.
- **No interactive TUI or daemon mode.** CLI invocation only. Run, produce output, exit.
- **No `unsafe` in `sdivi-core` or language adapters.** Any future need for `unsafe` lives in a dedicated crate behind a feature flag with a per-block `SAFETY:` comment. Bindings crates may use `unsafe` only as required by the binding macro.
- **No `panic!` for recoverable errors.** `panic!` is reserved for "this should be impossible" invariants. Recoverable errors return `Result<T, E>`.
- **No hidden global state.** No global mutable config, no lazy_static analysis caches with shared write access, no thread-local hidden context.
- **No automatic drift-vs-evolution classification (KD1).** The tool measures divergence from declared intent only.
- **No FFI to the C++ Leiden.** Determinism story requires a native port.

### Data Flow

Two entry points: the orchestration path (`sdivi-pipeline`, used by CLI and FS-based embedders) and the pure-compute path (`sdivi-core`, used by WASM / the consumer app and any embedder that already has its own extractors).

```
                    ── orchestration path (sdivi-pipeline) ──

config.toml + boundaries.yaml + repo path
       │
       ▼
[Config::load_or_default] ──► Config (precedence resolved)
       │
       ▼
[sdivi_pipeline::Pipeline::new(&Config)]
       │
       ▼
Stage 1: parsing       walkdir + ignore + tree-sitter ──► Iterator<FeatureRecord>
       │                            (rayon-parallel; CST dropped per-file)
       ▼
Stage 2: graph         resolve imports, build petgraph ──► DependencyGraph
       │
       ▼
Stage 3: detection     native Leiden(seed, gamma)      ──► LeidenPartition
       │                            (warm-start from .sdivi/cache/partition.json — I/O in sdivi-pipeline)
       ▼
Stage 4: patterns      tree-sitter queries + blake3    ──► PatternCatalog
       │
       ▼
Stage 5: snapshot      sdivi_core::assemble_snapshot(...)──► Snapshot {snapshot_version: "1.0"}
       │                            (atomic tempfile + rename to .sdivi/snapshots/ — I/O in sdivi-pipeline)
       ▼
sdivi_core::compute_delta(prev, curr) ──► DivergenceSummary (null per-dim when no prev)
       │
       ▼
sdivi-cli formats text/JSON ──► stdout    (logs/progress ──► stderr)


                    ── pure-compute path (sdivi-core / WASM) ──

caller-supplied:  DependencyGraphInput, PatternInstanceInput[], LeidenConfigInput,
                  BoundarySpecInput, ThresholdsInput, PriorPartition[]
       │
       ▼
sdivi_core::detect_boundaries(graph, cfg, prior) ──► cluster_assignments,
                                                   stability_score, modularity
sdivi_core::compute_coupling_topology(graph)     ──► CouplingTopologyResult
sdivi_core::compute_pattern_metrics(patterns)    ──► entropy, convention_drift
sdivi_core::compute_boundary_violations(...)     ──► BoundaryViolationResult
sdivi_core::compute_thresholds_check(...)        ──► ThresholdCheckResult (exit-10 logic)
sdivi_core::normalize_and_hash(kind, children)   ──► canonical blake3 fingerprint
       │
       ▼
caller assembles its own report (the consumer app, agent runtime, etc.)
```

### Module Boundaries and Dependency Rules

- `sdivi-cli` is the composition root for the binary; depends on `sdivi-pipeline` (orchestration) and `sdivi-core` (shared types + `compute_thresholds_check` for exit-10 logic).
- `sdivi-pipeline` depends on `sdivi-parsing`, `sdivi-graph`, `sdivi-detection`, `sdivi-patterns`, `sdivi-snapshot`, `sdivi-config`, and `sdivi-core`. It owns the `Pipeline::snapshot(&Path)` API and all FS/clock-touching code (partition cache, snapshot atomic writes, retention enforcement).
- `sdivi-core` is the pure-compute facade: **no I/O, no clock, no tree-sitter, WASM-compatible**. Depends on `sdivi-graph`, `sdivi-detection`, `sdivi-patterns`, `sdivi-snapshot`, `sdivi-config` — each declared with `default-features = false` to disable the `pipeline-records` feature. Exposes `compute_*` functions and the `*Input` struct family. **No module imports from `sdivi-cli` or `sdivi-pipeline`.**
- `sdivi-parsing` depends on `tree-sitter`, language grammars, `walkdir`, `ignore`, `rayon`, and `sdivi-config`. Used only by `sdivi-pipeline` and the language adapter crates.
- `sdivi-graph`, `sdivi-detection`, `sdivi-patterns`, `sdivi-snapshot` each have a default Cargo feature `pipeline-records` that pulls `sdivi-parsing` and exposes the `*_from_records` paths taking `&[FeatureRecord]`. With the feature disabled (sdivi-core's WASM build), only the `*_from_input` paths taking `&DependencyGraphInput` / `&[PatternInstanceInput]` compile.
- `sdivi-graph` depends on `petgraph` and (feature-gated) `sdivi-parsing`.
- `sdivi-detection` depends on `sdivi-graph` output types only (no `sdivi-parsing` even gated).
- `sdivi-patterns` depends on (feature-gated) `sdivi-parsing`. **It must NOT depend on `sdivi-graph` or `sdivi-detection`.**
- `sdivi-snapshot` is the pure assembly + delta crate: depends on `sdivi-graph`, `sdivi-detection`, `sdivi-patterns`, and `sdivi-config`. Snapshot I/O (atomic write, retention) lives in `sdivi-pipeline`, not here.
- `sdivi-config` is a leaf crate; depended on by all.
- `sdivi-wasm` (`bindings/sdivi-wasm`) depends on `sdivi-core` only, plus `wasm-bindgen` / `tsify` / `serde-wasm-bindgen`. **No `sdivi-pipeline`, no `sdivi-parsing` in the dep tree.**
- Language adapter crates (`sdivi-lang-*`) depend only on `sdivi-parsing` and `tree-sitter` grammars.
- No cycles between crates. CI fails on a cycle introduced by `cargo metadata` graph inspection. CI also runs `cargo tree -p sdivi-core --target wasm32-unknown-unknown --no-default-features` and asserts zero entries for `tree-sitter*`, `walkdir`, `ignore`, `rayon`, `tempfile`.

## Repository Layout

```
sdivi-rust/
├── Cargo.toml, Cargo.lock, rust-toolchain.toml, rustfmt.toml, clippy.toml, deny.toml
├── README.md, CHANGELOG.md, MIGRATION_NOTES.md, DRIFT_LOG.md, LICENSE, NOTICE
├── .github/workflows/   ci.yml · release.yml · audit.yml · verify-leiden.yml · wasm.yml
├── .tekhton/DESIGN.md
├── crates/
│   ├── sdivi-core/        # pure-compute facade, WASM-compatible (KDD-12)
│   ├── sdivi-pipeline/    # orchestration: FS, clock, atomic writes; Pipeline::{new, snapshot}
│   ├── sdivi-cli/         # composition root; anyhow allowed here only
│   ├── sdivi-parsing/     # walkdir+ignore+rayon+tree-sitter; LanguageAdapter trait; CST drop invariant
│   ├── sdivi-graph/       # feature `pipeline-records` (default ON; OFF in sdivi-core's WASM build)
│   ├── sdivi-detection/   # feature `pipeline-records`; native Leiden + LeidenPartition
│   ├── sdivi-patterns/    # feature `pipeline-records`; fingerprint.rs (PUBLIC) re-exposed via sdivi-core
│   ├── sdivi-snapshot/    # pure: snapshot · delta · trend · boundary_inference
│   ├── sdivi-config/      # feature `loader` (default ON); types-only when off
│   └── sdivi-lang-{rust,python,typescript,javascript,go,java}/
├── bindings/
│   ├── sdivi-wasm/        # @geoffgodwin/sdivi-wasm; depends on sdivi-core only
│   ├── sdi-py/          # post-MVP / v1: PyO3 wheel
│   └── sdivi-node/        # post-MVP / v1: napi-rs prebuilt
├── examples/            # embed_pipeline.rs · embed_compute.rs · custom_config.rs · binding_node.ts
├── tests/               # cross-crate: full_pipeline · snapshot_diff_trend · boundary_lifecycle + fixtures/
├── benches/             # criterion, gated behind `bench` feature
└── docs/                # cli-integration · library-embedding · migrating-from-sdi-py · determinism
```

## Key Design Decisions

### KDD-1: Snapshot schema is a clean break from sdi-py

**Decision:** sdivi-rust ships `snapshot_version: "1.0"` and refuses to read sdi-py snapshot JSON; `.sdivi/cache/*` is also a clean break. Trend continuity for migrators is explicitly accepted as lost. `.sdivi/config.toml` and `.sdivi/boundaries.yaml` remain read-compatible — those are user-edited and worth migrating; snapshots are tool-generated and trivially regeneratable.

### KDD-2: Native Leiden port, no FFI to C++

**Decision (ratified M05):** Native Rust port of Leiden (Traag et al. 2019, Modularity + CPM) in `sdivi-detection`. Verification is partition quality (modularity within 1%, community count within ±10%) — not bit-identity. FFI to C++ `leidenalg` rejected: non-Rust toolchain, complicates single-binary distribution, breaks determinism across platforms.

### KDD-3: Library surface is canonical

**Decision:** Every CLI feature is a library feature first. `sdivi-cli` cannot add analysis code paths unreachable through `sdivi-core` or `sdivi-pipeline`. SemVer commitment begins at `0.1.0`; adding `pub` is deliberate, removing `pub` is breaking. See KDD-12 for the layered structure.

### KDD-4: Tree-sitter grammars linked at compile time

**Decision:** Each grammar is a build dependency gated by `lang-<name>` Cargo feature. Default feature set matches sdi-py's supported languages. Compile-time over runtime dynamic loading: simpler, matches ecosystem norms, lets binary-size-sensitive consumers strip languages they don't need.

### KDD-5: `petgraph` is the default — no CSR view

**Decision (ratified M05):** Use `petgraph::Graph` everywhere in `sdivi-graph`. The Leiden algorithm builds its own internal `Vec<Vec<usize>>` adjacency list at the start of each run; a separate CSR module would duplicate that conversion without benefit. Revisit only if a measured bottleneck demands it.

### KDD-6: YAML write — accept comment loss for MVP

**Decision:** `serde_yaml` for read; programmatic writes via `sdivi boundaries ratify` regress comment preservation. Comment-preserving alternatives (immature crates or hand-written emitter) rejected for MVP scope. Revisit only on user complaint.

### KDD-7: MSRV is "stable latest minus 2"

**Decision:** Pinned in `rust-toolchain.toml`; verified in CI matrix. Generous enough for distros, conservative enough to use modern features. Bump deliberately, not opportunistically.

### KDD-8: License is Apache 2.0

**Decision (ratified 2026-04-28):** Apache 2.0 across the workspace. Permissive enough for enterprise compliance, explicit contributor patent grant (which MIT lacks), `NOTICE` file preserves attribution. Every published crate's `Cargo.toml` sets `license = "Apache-2.0"`.

### KDD-9: Deltas are `null` on first snapshot, not zero

**Decision:** `compute_delta(prev: &Snapshot, curr: &Snapshot)` requires both arguments. The "first snapshot" path returns a `DivergenceSummary` with all per-dimension fields `null`. Identical consecutive snapshots yield `0`, distinguishing "no comparison possible" from "no change." Mixing the two is a semantic bug masquerading as a numeric one.

### KDD-10: `BTreeMap` over `IndexMap` for catalogs

**Decision:** `BTreeMap` everywhere output ordering matters — orders by key without relying on insertion order. Revisit only if profiling shows comparison cost dominates a hot path.

### KDD-11: Bindings live in-repo until they earn their own repos

**Decision:** `bindings/sdivi-wasm`, `bindings/sdi-py`, and `bindings/sdivi-node` ship in this workspace. `sdivi-wasm` is in v0; the other two remain post-MVP / v1 era. Cross-repo CI complexity outweighs the workspace benefit only after non-trivial consumer-side surface area.

### KDD-12: Two-layer library shape — `sdivi-core` (pure compute) + `sdivi-pipeline` (orchestration)

**Decision (ratified 2026-04-29):**
- **`sdivi-core`** — pure-compute facade. No I/O, no clock, no tree-sitter, WASM-compatible. Public surface: `compute_*` functions over `*Input` serde structs (`DependencyGraphInput`, `PatternInstanceInput`, `LeidenConfigInput`, `BoundarySpecInput`, `ThresholdsInput`, `PriorPartition`, `NormalizeNode`); `compute_delta`, `assemble_snapshot`, `compute_trend`, `infer_boundaries` re-exported from `sdivi-snapshot`; `normalize_and_hash` for foreign extractors.
- **`sdivi-pipeline`** — orchestration. Owns `Pipeline::snapshot(&Path)` + warm-start cache I/O + atomic snapshot writes. Depends on `sdivi-parsing`, `sdivi-graph`, `sdivi-detection`, `sdivi-patterns`, `sdivi-snapshot`, `sdivi-config`, and `sdivi-core`. CLI consumers and FS-based Rust embedders use this.

`sdivi-graph`, `sdivi-detection`, `sdivi-patterns`, `sdivi-snapshot` carry a default Cargo feature `pipeline-records` that pulls `sdivi-parsing` and exposes `*_from_records` paths. `sdivi-core` declares them with `default-features = false` so the WASM build pulls only the pure-compute paths taking `*Input` structs.

### KDD-13: WASM is in v0

**Decision (ratified 2026-04-29):** `bindings/sdivi-wasm` ships in v0. `wasm-bindgen` exports of every `sdivi-core::compute_*` function plus `normalize_and_hash`; `tsify`-derived `.d.ts` for strict-TS consumers. Published as `@geoffgodwin/sdivi-wasm@0.1.0` to npm in the same release workflow as the crates.io publish, behind the same manual approval gate. PyO3/napi-rs bindings remain post-MVP / v1 era.

### Unresolved Open Questions

- **Crate name reservation on crates.io:** verify `sdivi`, `sdivi-core`, `sdivi-cli`, `sdivi-pipeline` are available before M13. Fallback: prefix with `sdivi-rs-` and document.
- **FMA bit determinism across platforms:** documented in `docs/determinism.md` (M11). Aggregate equality only across platforms; revisit via build flag if a real adopter needs bit-identity.

## Config Architecture

Config is loaded via TOML files plus environment variables, resolved through a 5-level precedence chain. The schema is **read-compatible with sdi-py** (KD13) — sdi-py users can drop in their existing `.sdivi/config.toml` unchanged. New `sdivi-rust`-only sections (`[determinism]`, `[bindings]`) are additive.

### Loading Strategy

`Config::load_or_default(repo_root)` walks the precedence order and returns a fully-resolved `Config`. `Config::default()` returns built-in defaults. All keys are optional. Malformed TOML returns `ConfigError::Parse`; out-of-range values return `ConfigError::InvalidValue { key, message }`. Unknown keys produce a deprecation warning to stderr but never error (rule 12 from sdi-py: keys are reserved forever once introduced).

### Precedence (Highest to Lowest)

1. Function arguments (library) / CLI flags (binary)
2. Environment variables: `SDIVI_LOG_LEVEL`, `SDIVI_WORKERS`, `SDIVI_CONFIG_PATH`, `SDIVI_SNAPSHOT_DIR`, `NO_COLOR`
3. Project-local `.sdivi/config.toml`
4. Global `$XDG_CONFIG_HOME/sdivi/config.toml` (fallback `~/.config/sdivi/config.toml`)
5. Built-in defaults compiled into `sdivi-core`

### Default Configuration Summary

`Config::default()` in `sdivi-config/src/config.rs` is the source of truth. Sections and the load-bearing defaults:

| Section | Key defaults |
|---|---|
| `[core]` | `languages = "auto"`; standard exclude globs (vendor, node_modules, __pycache__, dist, build, target, .git); `random_seed = 42` |
| `[snapshots]` | `dir = ".sdivi/snapshots"`, `retention = 100` (`0` = unlimited) |
| `[boundaries]` | `spec_file = ".sdivi/boundaries.yaml"`, `leiden_gamma = 1.0`, `stability_threshold = 3`, `weighted_edges = false` |
| `[patterns]` | `categories = "auto"`, `min_pattern_nodes = 5`, `scope_exclude = []` (excludes from catalog only — files remain in graph) |
| `[thresholds]` | `pattern_entropy_rate = 2.0`, `convention_drift_rate = 3.0`, `coupling_delta_rate = 0.15`, `boundary_violation_rate = 2.0` |
| `[change_coupling]` | `min_frequency = 0.6`, `history_depth = 500` |
| `[output]` | `format = "text" \| "json"`, `color = "auto" \| "always" \| "never"` |
| `[determinism]` | `enforce_btree_order = true` (sdivi-rust-only; reserved for FMA toggles) |
| `[bindings]` | reserved for future binding-specific knobs |

### Per-Category Threshold Overrides

`expires` is **mandatory**. Missing it returns `ConfigError::InvalidValue` (CLI exit 2). After expiry the override is silently ignored and defaults resume — no manual reset.

```toml
[thresholds.overrides.error_handling]
pattern_entropy_rate = 5.0
expires = "2026-09-30"
reason = "Migrating to ? operator from `match Err(_)` chains"
```

### Merge Strategy

Section-by-section override, later wins. Within a section, key-by-key override. Lists in `core.exclude` and `patterns.scope_exclude` are **replaced**, not merged. Threshold override sub-tables merge per-category: each `[thresholds.overrides.<cat>]` block is added; an existing category override is replaced wholesale.

### Required vs Optional

**All keys are optional.** Missing keys fall through to defaults. The single mandatory field is `expires` *within* a `[thresholds.overrides.<cat>]` block — only required if that block exists at all. There is no top-level required key.

### CLI ↔ Config Mapping

| Flag             | Config key / effect                         |
|------------------|---------------------------------------------|
| `--format json`  | `output.format = "json"`                    |
| `--no-color`     | `output.color = "never"` (also `NO_COLOR=1`)|
| `--workers N`    | Effective parallelism (no config key)       |
| `--seed N`       | `core.random_seed = N`                      |

### Runtime Mutability

`Config` is consumed at `Pipeline::new`. The pipeline does not mutate config during a snapshot run. Per-call overrides build a new `Config`. There is no global mutable config in `sdivi-core`.

## Non-Negotiable Rules

1. **`unsafe` is forbidden in `sdivi-core` and language adapter crates.** Bindings crates (`sdi-py`, `sdivi-node`) may use `unsafe` only as required by the binding macro. Any other `unsafe` lives in a dedicated crate behind a feature flag with a per-block `// SAFETY:` comment justifying the invariant.
2. **No network calls anywhere in the analysis pipeline.** No telemetry, no update checks, no remote lookups. A snapshot must be producible on an airgapped machine. CI tests must not require network.
3. **No ML/LLM calls in the pipeline.** Determinism is the contract. A measurement instrument cannot depend on a stochastic model.
4. **Tree-sitter CSTs are dropped before the parsing function returns.** The `sdivi-parsing` API consumes file content + grammar and returns a `FeatureRecord`. No type containing a `Tree` may escape `parse_file`. Memory usage stays proportional to the largest single file, not the codebase total.
5. **`BTreeMap` is the default ordered map.** `HashMap` is allowed only when iteration order does not influence output. Pattern catalogs, snapshot fields, and serde-emitted maps use `BTreeMap`.
6. **All RNG is `StdRng` seeded explicitly from `Config::random_seed`.** Default seed is `42`. No `thread_rng`, no `SystemTime`-based seeding, no implicit RNG.
7. **Pattern fingerprints use `blake3` with a fixed key constant.** The key constant is defined once in `sdivi-patterns::fingerprint` and never changes within a `snapshot_version`.
8. **Logs, progress bars, and warnings go to stderr. Snapshot JSON, summaries, and table output go to stdout.** `sdivi show --format json | jq '.'` must work without contamination. CI test `tests/stdout_stderr_split.rs` is non-negotiable.
9. **Exit codes are public API: `0`, `1`, `2`, `3`, `10`.** Code `10` is exclusively `sdivi check`. Adding or repurposing an exit code is a breaking change requiring a major version bump.
10. **`.sdivi/config.toml` and `.sdivi/boundaries.yaml` are read-compatible with sdi-py.** New config keys are additive. Existing key semantics may not change. Removed keys are reserved forever.
11. **`snapshot_version` is `"1.0"` for all sdivi-rust output.** sdivi-rust does not read sdi-py snapshots. Reading an incompatible `snapshot_version` produces a warning and baseline treatment (no delta), never a crash.
12. **Per-category threshold overrides require an `expires` field.** Missing `expires` is a config error (exit 2). After expiry the override is silently ignored — no manual reset, no retention.
13. **Snapshot writes are atomic.** Write to a tempfile in the target snapshot directory, then rename. A killed process must never leave a half-written `.json` file. Retention is enforced synchronously after each successful write.
14. **First-snapshot deltas are `null`, not zero.** `null` means "no prior snapshot to compare." `0` means "snapshots compared and no change observed." These are different and observable in the CI gate.
15. **Missing tree-sitter grammars are warnings unless all detected languages lack grammars.** A single missing grammar logs to stderr and skips those files. Only when *all* detected languages lack grammars does `sdivi snapshot` exit with code 3.
16. **Missing `BoundarySpec` is normal operation.** All metrics except intent divergence are computed; no warning is emitted. Intent divergence is simply absent from the snapshot.
17. **`sdivi-cli` cannot add code paths unreachable through `sdivi-core`.** Every CLI feature has a library entry point. The CLI is a thin presentation layer.
18. **Public API stability begins at `0.1.0`.** Adding a `pub` item is deliberate; removing or renaming a `pub` item is a breaking change. Internal-only items live in `pub(crate)` modules.
19. **`#![deny(missing_docs)]` is enabled on `sdivi-core`.** Every public item has at least one rustdoc comment with an `# Examples` block where it is meaningful. Doc tests run in CI.
20. **`cargo clippy -- -D warnings` and `cargo fmt --check` are part of CI.** No `#[allow(...)]` on public items without an inline justification comment.

## Implementation Milestones

Managed as individual files in `.claude/milestones/`; see `MANIFEST.cfg` for ordering and status.
M01–M11 done. Remaining: **M12** (sdivi-wasm + consumer-app integration) and **M13** (release pipeline). M13 depends on M12.

## Code Conventions

- **Crate names:** `kebab-case`, `sdivi-` prefix (`sdivi-core`, `sdivi-cli`, `sdivi-lang-python`).
- **Modules:** `snake_case` (`parsing`, `snapshot`).
- **Types and traits:** `PascalCase` (`PatternFingerprint`, `BoundarySpec`, `LanguageAdapter`).
- **Functions, methods, fields, locals:** `snake_case` (`parse_repository`, `random_seed`).
- **Constants:** `SCREAMING_SNAKE_CASE` (`DEFAULT_GAMMA`, `MAX_RETENTION`, `EXIT_THRESHOLD_EXCEEDED`).
- **Generics:** single-letter when unambiguous (`T`, `E`); descriptive when multiple parameters in scope (`Lang`, `Adapter`).
- **Lifetimes:** `'a` by default; descriptive when more than one in scope (`'src`, `'cfg`).
- **File ceiling:** 500 lines guideline. Above that, split by sub-concern.
- **Tests:** unit tests co-located via `#[cfg(test)] mod tests`; integration tests in `tests/<crate-or-feature>.rs`.
- **Imports / `use` ordering:** rustfmt default — no project-specific grouping rules.
- **Error handling pattern:**
  - Library crates use `thiserror` to derive named variants with structured fields (`#[error("missing expires on {category}")] MissingExpires { category: String }`).
  - `sdivi-cli` uses `anyhow::Result` at the binary boundary in `main.rs` only.
  - `panic!` is reserved for "this should be impossible" invariant violations.
  - `Result<T, E>` is the default return style for any function that can fail.
  - All error variants carry context (file path, line number, key name) so callers can surface them meaningfully.
- **Public API discipline:**
  - Adding `pub` is deliberate; removing or renaming `pub` is a breaking change.
  - Internal-only items live in `pub(crate)` modules.
  - Enums that may grow new variants over time use `#[non_exhaustive]` (`BoundaryViolation`, `PatternInstance`).
  - `ExitCode` is closed (no `#[non_exhaustive]`) because the contract is fixed.
- **Doc discipline:**
  - `#![deny(missing_docs)]` on `sdivi-core`.
  - Every public item has a doc comment; an `# Examples` block where meaningful.
  - Doc tests run in CI; broken examples fail the build.
  - **Doc comment placement when inserting items.** A `///` block attaches to the *next* item declaration. When inserting a new `pub fn`/`struct`/`const` immediately before an existing documented item, verify that a non-`///` line (a blank line is sufficient) separates the two doc blocks — otherwise the existing item's doc block silently re-attaches to your new item, and the original item is left undocumented. This is a correctness bug, not a style issue: it has recurred across multiple milestone runs and `#![deny(missing_docs)]` will catch it on `sdivi-core` but not on the inner crates.
- **Lint discipline:**
  - `cargo clippy -- -D warnings` and `cargo fmt --check` are CI gates.
  - `#[allow(...)]` on public items requires an inline justification comment.
- **Git workflow:**
  - Branches: `feat/<topic>`, `fix/<topic>`, `docs/<topic>`, `chore/<topic>`.
  - Commits: conventional-style summary lines (`feat(sdivi-detection): native Leiden CPM quality`); imperative mood; no trailing period.
  - PRs reference the milestone they belong to in the title (`[M5] Leiden modularity quality function`).
  - Squash-merge to `main`. Tags on `main` only.
  - `CHANGELOG.md` updated by the PR author for any user-visible change.

## Critical System Rules

1. A `sdivi_pipeline::Pipeline::snapshot` call against the same repo state with the same `Config` (including the same `random_seed`) produces a **bit-identical** `Snapshot` JSON. Any divergence is a bug.
2. The parsing stage never holds two `tree_sitter::Tree` values in scope simultaneously across the same execution unit — the per-file `parse_file` API drops its `Tree` before returning.
3. `sdivi_core::compute_delta(prev, curr)` is referentially transparent: same inputs → same `DivergenceSummary`. It performs no I/O, reads no globals, and uses no clock. The same applies to every other `sdivi_core::compute_*` function.
4. `sdivi check` is the only command that exits with code `10`. Every other command's success path must exit `0`. Exit-10 logic delegates to `sdivi_core::compute_thresholds_check`.
5. A first-snapshot `DivergenceSummary` has `null` per-dimension fields. `0` is reserved for "compared and unchanged."
6. Threshold overrides without `expires` are a `ConfigError::MissingExpiresOnOverride { category }` with exit `2`. After expiry, the override is silently ignored — defaults resume.
7. `sdivi snapshot` exits `3` only if **all** detected languages lack tree-sitter grammars. A single missing grammar logs to stderr and skips files.
8. A missing `.sdivi/boundaries.yaml` is **normal operation**. No warning is emitted. Intent divergence fields are simply absent from the snapshot.
9. Snapshot files are written atomically (tempfile in target dir + rename). A killed process never leaves a half-written `.json` in `.sdivi/snapshots/`. The atomic write lives in `sdivi-pipeline::store`, not `sdivi-core` (Rule 22).
10. Retention is enforced **synchronously after** the rename. A failed write does not remove an old snapshot.
11. Logs, progress, warnings → **stderr**. Snapshot JSON, summaries, table output → **stdout**. `sdivi show --format json | jq '.'` must always work.
12. `Config` is consumed at `sdivi_pipeline::Pipeline::new`. The pipeline mutates no config field during a run. There is no global mutable config.
13. The pipeline performs **zero network calls**. Tests assert this by running with network disabled when supported by the CI runner.
14. `sdivi-patterns` does not import or depend on `sdivi-graph` or `sdivi-detection`. Violation is a `cargo metadata` graph cycle and must be CI-blocked.
15. `sdivi-cli` adds no analysis code paths unreachable through `sdivi-core` or `sdivi-pipeline`. Every CLI feature is callable from the library; pure-compute features are callable from `sdivi-core` directly (and therefore from WASM / the consumer app).
16. `snapshot_version` is the literal string `"1.0"` for all sdivi-rust output. Bumping it is a breaking change requiring a major version bump and a `MIGRATION_NOTES.md` entry.
17. Reading a `Snapshot` JSON with an incompatible `snapshot_version` produces a stderr warning and baseline treatment (no delta) — never a crash.
18. RNG is `StdRng` seeded from `Config::random_seed` (or `LeidenConfigInput::seed`). No `thread_rng`, no `SystemTime`-derived seeds, no implicit RNG anywhere in the analysis pipeline.
19. Pattern fingerprints use `blake3` with a single fixed key constant. The constant lives in `sdivi-patterns::fingerprint` and is re-exposed via `sdivi_core::normalize_and_hash` for foreign extractors. Changing the constant invalidates all existing snapshot fingerprints and requires a snapshot version bump.
20. Adding a new variant to `ExitCode` is a breaking change. Reusing or repurposing an existing exit code is a breaking change.
21. **`sdivi-core` is WASM-compatible.** No `std::fs::*`, no `std::time::SystemTime`, no `walkdir`, no `ignore`, no `rayon`, no `tree-sitter` in its dependency tree. CI verifies via `cargo build -p sdivi-core --target wasm32-unknown-unknown --no-default-features` and `cargo tree -p sdivi-core --target wasm32-unknown-unknown --no-default-features` (asserting zero entries for the forbidden crates). Any code path that needs the clock takes the time as input (e.g. `compute_thresholds_check` takes `today: NaiveDate`).
22. **All FS, clock, and atomic-write I/O lives in `sdivi-pipeline` or `sdivi-cli`, never in `sdivi-core` or the compute crates with the `pipeline-records` feature disabled.** Pure (de)serialization stays in the originating compute crate; the FS calls live in `sdivi-pipeline`.
23. **`normalize_and_hash` produces the same `blake3` digest in WASM as in native sdivi-core for the same `NormalizeNode` input.** Cross-platform determinism is asserted in CI.

## What Not to Build Yet

- **GitHub Actions reusable action** — post-MVP polish. Document manual `cargo install sdivi && sdivi check` for now.
- **PyO3 / napi-rs bindings (`sdi-py`, `sdivi-node`)** — post-MVP / v1 era. Revisit when a concrete consumer appears.
- **IDE / editor plugin** — requires a stable API and snapshot schema. Post-1.0.
- **SaaS dashboard or web UI** — sdivi-rust is a measurement instrument. Output is JSON; existing dashboards consume it.
- **Auto-remediation / gardener agent** — sdivi-rust measures drift; it never fixes it. Separate project.
- **Plugin system for custom analyzers** — built-in pattern categories only at MVP. Extensibility design after real user feedback.
- **Cross-language dependency inference** — v0 tracks only explicit in-language imports.
- **Historical backfill UX** — `sdivi snapshot --commit REF` works for individual commits. Batch backfill is unsupported; users script it.
- **Real-time / watch mode** — no file watcher daemon. CLI invocation on merge events is the intended cadence.
- **Automatic drift-vs-evolution classification** — humans declare migration intent via threshold overrides.
- **Stdin input for `sdivi diff`** — deferred.
- **`sdivi config` subcommand** — edit `.sdivi/config.toml` directly.
- **Comment-preserving YAML write** — accept comment loss for MVP (KDD-6). Revisit on user complaint.
- **Importing sdi-py snapshots** — clean break (KDD-1). Trend continuity for migrators is acceptably lost.
- **Bit-identical snapshot output across platforms** — aggregate equality only; revisit via build flag if a real adopter needs it.
- **Bindings split into separate repos** — in-repo until non-trivial cross-repo CI complexity earns the split (KDD-11).

## Testing Strategy

### Frameworks and Tools

- `cargo test` is the entry point for everything default-runnable.
- `proptest` for property-based determinism tests.
- `criterion` for benchmarks, gated behind feature `bench`, run on release tags only.
- `assert_cmd` + `predicates` for CLI integration testing.
- `tempfile` for filesystem fixtures.
- Python + `leidenalg` for the `verify-leiden` cross-check suite, gated behind feature `verify-leiden`.

### Test Tiers

- **Unit tests** in each crate via `#[cfg(test)] mod tests` blocks or per-crate `tests/` files. Coverage targets:
  - 80%+ for `sdivi-core`, `sdivi-parsing`, `sdivi-graph`, `sdivi-detection`, `sdivi-patterns`, `sdivi-snapshot`
  - 60%+ for `sdivi-cli` (rest covered by integration tests)
- **Per-crate integration tests** in each crate's `tests/` directory exercising real tree-sitter parsing, real graphs, real fixture repos.
- **Cross-crate integration tests** live in the `tests/` directory of the most structurally appropriate member crate, reaching workspace fixtures via `concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/")`. The top-level `tests/` directory holds layout placeholders and the shared `fixtures/` tree until a root `[package]` crate is introduced; Cargo cannot compile files in `tests/` at the workspace root without one. When a root package is added in a future milestone, migrate the cross-crate suites (full pipeline, snapshot/diff/trend lifecycles, boundary lifecycle, multi-language import extraction) there.
- **Doc tests** via `cargo test --doc`. Every public function with an `# Examples` block has a runnable doc test. Broken examples fail CI.
- **Property tests** in `crates/<crate>/tests/proptest.rs` files: `prop_test_pipeline_deterministic`, `prop_test_delta_pure`, `prop_test_leiden_seeded`. `proptest-regressions/` directories committed.
- **KDD-2 verification suite** in `crates/sdivi-detection/tests/leiden_quality.rs`, gated `#[cfg(feature = "verify-leiden")]`. Pass criteria: modularity within 1%, community count within ±10%. **Not** bit-identity.
- **CLI exit-code tests** in `crates/sdivi-cli/tests/exit_codes.rs` exhaustively covering 0/1/2/3/10.
- **Stdout/stderr discipline tests** in `crates/sdivi-cli/tests/stdout_stderr_split.rs`.
- **Atomic-write tests** in `crates/sdivi-snapshot/tests/atomic_write.rs` simulating mid-write panic.
- **Memory-invariant test** in `crates/sdivi-parsing/tests/memory_invariant.rs` asserting the CST-drop ownership rule.
- **Benchmarks** under `benches/` in each crate; gated `#[cfg(feature = "bench")]`; tracked in `CHANGELOG.md` per release.

### Test Fixtures

Under `tests/fixtures/`:

- `simple-rust/` — small Cargo crate with known imports
- `simple-python/`, `simple-typescript/`, `simple-javascript/`, `simple-go/`, `simple-java/` — per-language minimal fixtures
- `multi-language/` — Python + TypeScript exercise
- `high-entropy/` — deliberate pattern variance
- `evolving/` — git repo with progressive drift, built by setup script under `target/test-fixtures` before tests run
- `leiden-graphs/{small,medium,large}/` — adjacency lists + reference modularities for KDD-2 verification

### Patterns

- Use **factory functions** for test data, not on-disk fixtures, where the data is small and synthetic.
- Use **on-disk fixtures** for repository-shaped scenarios (parsing, graph, full pipeline).
- **Mock no internals.** No mock for `Pipeline`, no mock for `LanguageAdapter`. Real types, real fixtures.
- **Mock no network.** There is no network code to mock — Rule 13 forbids it.
- **Use real filesystem** via `tempfile` for any test that touches `.sdivi/`.

### What We Do NOT Test

- Real network access (Rule 13).
- Cross-version migration of sdi-py snapshot JSON (KDD-1 clean break).
- Bit-identity of snapshot output across platforms (aggregate equality only — see `docs/determinism.md`).

### Commands

- Default: `cargo test --workspace`
- With Leiden verification: `cargo test --workspace --features verify-leiden` (requires Python + leidenalg)
- Doc tests only: `cargo test --doc --workspace`
- Single crate: `cargo test -p sdivi-detection`
- Benchmarks: `cargo bench --features bench`
- Coverage (CI nightly): `cargo llvm-cov --workspace --html`

## Development Environment

### Prerequisites

- Rust toolchain matching `rust-toolchain.toml` (MSRV: stable latest minus 2). Install via `rustup`.
- `git`
- C compiler (cc on Linux/macOS, MSVC build tools on Windows) — required by tree-sitter grammar build dependencies.
- For the `verify-leiden` feature only: Python ≥ 3.9 and `pip install leidenalg igraph`.
- `cargo audit` for the audit job: `cargo install cargo-audit` (CI installs automatically; local dev optional).

### Setup

```bash
git clone https://github.com/<org>/sdivi-rust.git
cd sdivi-rust
rustup show              # confirms rust-toolchain.toml pin
cargo build --workspace  # downloads + compiles all crates
cargo test --workspace   # runs default test suite
```

### Build Commands

- `cargo build --workspace` — debug build of every crate.
- `cargo build --release` — produces optimized `target/release/sdivi`.
- `cargo build -p sdivi-core` — library-only build for embedding consumers.
- `cargo build --no-default-features --features lang-python` — minimal binary supporting only Python.
- `cargo doc --workspace --no-deps --open` — build and view rustdoc locally.

### Run Commands

- `./target/release/sdivi --help` — CLI help.
- `./target/release/sdivi init` — initialize a `.sdivi/` directory in CWD.
- `./target/release/sdivi snapshot` — capture a snapshot.
- `./target/release/sdivi check` — run the threshold gate; exit 10 on breach.
- `cargo run --example embed_pipeline -- <repo-path>` — run an embedder example.
- `cargo run -p sdivi-cli -- show --format json | jq '.'` — inspect latest snapshot.

### Environment Variables

- `SDIVI_LOG_LEVEL=debug` — `tracing` level.
- `SDIVI_WORKERS=4` — parallel parsing workers.
- `SDIVI_CONFIG_PATH=/abs/path/config.toml` — override config search path.
- `SDIVI_SNAPSHOT_DIR=.sdivi/snapshots` — override snapshot directory.
- `NO_COLOR=1` — disable ANSI color in output (also `--no-color`).
- For development only: `RUST_BACKTRACE=1` for panic traces.

### IDE / Editor Recommendations

- **VS Code** with `rust-analyzer` extension. Enable `clippy` on save: `"rust-analyzer.checkOnSave.command": "clippy"`.
- **JetBrains RustRover** also supported; no project-specific settings.
- The repo ships no `.vscode/` or `.idea/` directory — editor config is per-developer.

## Documentation Responsibilities

### Documentation Sources

- **`README.md`** at repo root — quick start, install paths, one-paragraph SDIVI overview, links. Under 200 lines.
- **`CHANGELOG.md`** — hand-maintained, conventional sections (Added / Changed / Deprecated / Removed / Fixed / Security). One entry per release tag.
- **`MIGRATION_NOTES.md`** — entries for breaking changes between 0.x → 0.(x+1) bumps and post-1.0 majors.
- **`DRIFT_LOG.md`** — vendoring decisions, `[patch.crates-io]` justifications, design drift between `.tekhton/DESIGN.md` and reality.
- **rustdoc on `sdivi-core`** — canonical API reference, published to docs.rs on `cargo publish`. `#![deny(missing_docs)]` enforced. Every public item has a doc comment with an `# Examples` block where meaningful.
- **`docs/cli-integration.md`** — manual CI integration recipe, GitHub Actions snippet, exit-code reference.
- **`docs/library-embedding.md`** — embedding `sdivi-core` in a Rust agent runtime; minimal viable consumer; common pitfalls.
- **`docs/migrating-from-sdi-py.md`** — what carries vs what changes vs explicit non-goals.
- **`docs/determinism.md`** — `BTreeMap` discipline, seed contract, FMA notes.
- **`examples/` directory** — runnable consumer snippets, not a published crate.

### Ownership

- **Feature author** owns doc updates for the surface they introduce — rustdoc on new public items, `docs/*.md` updates if the feature changes embedder-facing behavior, `CHANGELOG.md` entry.
- **The Tekhton milestone author** owns the relevant section of `MIGRATION_NOTES.md` if the milestone introduces a breaking change.
- **The release manager** owns the `CHANGELOG.md` finalization at tag time.
- **Auto-generated:** rustdoc HTML on docs.rs (publishes on `cargo publish`). Nothing else is auto-generated.

### Update Cadence

- **Per feature** (in the same PR): rustdoc on new public items, `docs/*.md` if behavior changes for embedders, `CHANGELOG.md` entry under the next-release section.
- **Per milestone** (Tekhton checkpoint): `docs/migrating-from-sdi-py.md` if the milestone changes anything visible to migrators.
- **Per release** (tag): `CHANGELOG.md` finalized into a versioned section, `MIGRATION_NOTES.md` extended if breaking, binary size noted.

### Public Surface

The "public surface" requiring docs:

- Every `pub` item in `sdivi-core` and the language adapter crates (rustdoc, doc test where meaningful).
- Every `sdivi` CLI command, flag, and exit code.
- Every `.sdivi/config.toml` key (both `sdi-py`-shared and sdivi-rust-only).
- The `Snapshot` JSON schema (versioned via `snapshot_version`).
- The `BoundarySpec` YAML schema.
- The `ExitCode` enum.
- Bindings (post-MVP): every `sdivi` Python and Node entry point.

### Doc Freshness Policy

**Strict (block).** Specifically:

- `cargo doc --workspace --no-deps` must produce zero warnings.
- `cargo test --doc --workspace` must pass — broken doc-test examples fail CI.
- `#![deny(missing_docs)]` blocks compilation if a public item lacks a doc comment.
- Adding a `pub` item without a doc test is a CI failure on `sdivi-core`.
- A breaking change without a `MIGRATION_NOTES.md` entry blocks the release workflow's manual approval gate.
- The audit cron blocks the release workflow if a yanked dep is in use.
<!-- tekhton-managed -->
